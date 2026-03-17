//! Servicio principal de inspección.
//!
//! Orquesta la captura de métricas, aplica heurísticas y entrega una
//! instantánea lista para la UI. La idea es que el frontend no piense en cómo
//! obtener datos, solo en cómo mostrarlos con claridad.

use crate::models::{
    Alert, ConnectionInsight, HardwareInfo, PrecisionStatus, ProcessInsight, ServiceState,
    Severity, SnapshotRow, SystemOverview, SystemSnapshot, TempEntry, TraceAnalysisSummary,
};
use crate::services::{etl, network, persistence::PersistenceStore, temp_scan, windows};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Networks, Pid, System, get_current_pid};

/// Estado incremental necesario para calcular deltas entre muestreos.
#[derive(Default)]
struct ProcessIoBaseline {
    read_total_bytes: u64,
    write_total_bytes: u64,
}

/// Motor principal del software.
pub struct InspectorService {
    system: System,
    networks: Networks,
    process_baselines: HashMap<u32, ProcessIoBaseline>,
    protected_names: HashSet<String>,
    stoppable_services: HashSet<String>,
    own_pid: u32,
    store: PersistenceStore,
    precision_traces_dir: PathBuf,
    precision_analysis_dir: PathBuf,
    precision_last_trace_path: Option<PathBuf>,
    precision_last_analysis_path: Option<PathBuf>,
}

impl InspectorService {
    /// Inicializa recursos persistentes y el estado de monitoreo.
    pub fn new() -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let protected_names = [
            "system",
            "registry",
            "smss.exe",
            "csrss.exe",
            "wininit.exe",
            "services.exe",
            "lsass.exe",
            "winlogon.exe",
            "svchost.exe",
            "fontdrvhost.exe",
            "dwm.exe",
            "memory compression",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();

        let stoppable_services = ["bits", "dosvc", "sysmain", "wuauserv"]
            .into_iter()
            .map(str::to_owned)
            .collect();

        let own_pid = get_current_pid()
            .ok()
            .map(|pid| pid.as_u32())
            .unwrap_or_default();
        let store = PersistenceStore::new("RootCauseInspector")?;

        let base_precision_dir = dirs::document_dir()
            .or_else(dirs::download_dir)
            .or_else(dirs::data_local_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("RootCause")
            .join("traces");
        let precision_traces_dir = base_precision_dir.clone();
        let precision_analysis_dir = base_precision_dir.join("analysis");
        fs::create_dir_all(&precision_traces_dir)
            .with_context(|| format!("No se pudo crear {}", precision_traces_dir.display()))?;
        fs::create_dir_all(&precision_analysis_dir)
            .with_context(|| format!("No se pudo crear {}", precision_analysis_dir.display()))?;

        let precision_last_trace_path = latest_matching_file(&precision_traces_dir, |path| {
            path.extension()
                .and_then(|v| v.to_str())
                .map(|v| v.eq_ignore_ascii_case("etl"))
                .unwrap_or(false)
        });
        let precision_last_analysis_path =
            latest_matching_file_recursive(&precision_analysis_dir, |path| {
                path.file_name()
                    .and_then(|v| v.to_str())
                    .map(|v| v.eq_ignore_ascii_case("trace-analysis.json"))
                    .unwrap_or(false)
            });

        Ok(Self {
            system,
            networks,
            process_baselines: HashMap::new(),
            protected_names,
            stoppable_services,
            own_pid,
            store,
            precision_traces_dir,
            precision_analysis_dir,
            precision_last_trace_path,
            precision_last_analysis_path,
        })
    }

    /// Devuelve una frase rápida desde el historial persistido.
    pub fn latest_history_line(&self) -> String {
        self.store
            .latest_summary_line()
            .ok()
            .flatten()
            .unwrap_or_else(|| format!("Historial listo en {}", self.store.db_path().display()))
    }

    /// Carga las últimas N filas del historial SQLite para la pestaña Historial.
    pub fn load_history(&self, limit: usize) -> Vec<SnapshotRow> {
        self.store.load_recent(limit).unwrap_or_default()
    }

    /// Recopila información estática del hardware del equipo (se llama una sola vez al iniciar).
    pub fn get_hardware_info(&self) -> HardwareInfo {
        let cpu = self.system.cpus().first();
        HardwareInfo {
            os_name: sysinfo::System::name().unwrap_or_else(|| "Windows".to_owned()),
            os_version: sysinfo::System::long_os_version().unwrap_or_default(),
            host_name: sysinfo::System::host_name().unwrap_or_default(),
            cpu_brand: cpu.map(|c| c.brand().to_owned()).unwrap_or_default(),
            cpu_cores: self.system.cpus().len(),
            cpu_freq_mhz: cpu.map(|c| c.frequency()).unwrap_or(0),
            total_ram_gb: self.system.total_memory() as f32 / 1_073_741_824.0,
            architecture: std::env::consts::ARCH.to_owned(),
        }
    }

    /// Captura una instantánea completa.
    pub fn collect_snapshot(&mut self) -> Result<SystemSnapshot> {
        self.system.refresh_all();
        self.networks.refresh(true);

        let collected_at = Utc::now();
        let mut process_names = HashMap::new();
        let mut process_paths = HashMap::new();
        let mut processes = Vec::new();
        let mut total_read_delta_bytes = 0_u64;
        let mut total_write_delta_bytes = 0_u64;

        for process in self.system.processes().values() {
            let pid = process.pid().as_u32();
            let name = os_str_to_string(process.name());
            let exe_path = process
                .exe()
                .map(|path| path.display().to_string())
                .unwrap_or_default();
            let parent_pid = process.parent().map(Pid::as_u32);
            let memory_mb = bytes_to_mb(process.memory());
            let cpu_percent = process.cpu_usage();
            let disk_usage = process.disk_usage();

            let baseline = self.process_baselines.entry(pid).or_default();
            let read_delta = disk_usage
                .total_read_bytes
                .saturating_sub(baseline.read_total_bytes);
            let write_delta = disk_usage
                .total_written_bytes
                .saturating_sub(baseline.write_total_bytes);
            baseline.read_total_bytes = disk_usage.total_read_bytes;
            baseline.write_total_bytes = disk_usage.total_written_bytes;

            total_read_delta_bytes = total_read_delta_bytes.saturating_add(read_delta);
            total_write_delta_bytes = total_write_delta_bytes.saturating_add(write_delta);

            process_names.insert(pid, name.clone());
            process_paths.insert(pid, exe_path.clone());

            let (severity, score, reasons, category) =
                classify_process(&name, &exe_path, cpu_percent, memory_mb, write_delta);
            let can_terminate = self.can_terminate_process(pid, &name, &exe_path);

            processes.push(ProcessInsight {
                pid,
                name,
                exe_path,
                parent_pid,
                cpu_percent,
                memory_mb,
                io_read_mb_delta: bytes_to_mb(read_delta),
                io_write_mb_delta: bytes_to_mb(write_delta),
                status: format!("{:?}", process.status()),
                category,
                severity,
                score,
                can_terminate,
                reasons,
                command_line: None,
            });
        }

        processes.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.io_write_mb_delta.total_cmp(&a.io_write_mb_delta))
                .then_with(|| b.memory_mb.total_cmp(&a.memory_mb))
                .then_with(|| b.cpu_percent.total_cmp(&a.cpu_percent))
        });

        // Obtener cmdline para los primeros procesos Críticos o de alto I/O (máx 6).
        let critical_pids: Vec<u32> = processes
            .iter()
            .filter(|p| matches!(p.severity, Severity::Critical) || p.io_write_mb_delta > 20.0)
            .take(6)
            .map(|p| p.pid)
            .collect();
        if !critical_pids.is_empty() {
            let cmdlines = windows::batch_process_cmdlines(&critical_pids);
            for p in processes.iter_mut() {
                if let Some(cmdline) = cmdlines.get(&p.pid) {
                    p.command_line = Some(cmdline.clone());
                }
            }
        }

        let connections = match windows::netstat() {
            Ok(output) => network::parse_netstat_output(&output, &process_names, &process_paths),
            Err(_) => Vec::new(),
        };
        let temp = temp_scan::scan_temp_overview().unwrap_or_default();
        let events = windows::recent_system_events(18).unwrap_or_default();
        let services = windows::relevant_services().unwrap_or_default();
        let trace_analysis = self.load_last_trace_analysis().ok().flatten();
        let precision = self.precision_status();

        let network_rx_mb_delta = self
            .networks
            .list()
            .values()
            .map(|data| bytes_to_mb(data.received()))
            .sum::<f32>();
        let network_tx_mb_delta = self
            .networks
            .list()
            .values()
            .map(|data| bytes_to_mb(data.transmitted()))
            .sum::<f32>();

        let mut overview = SystemOverview {
            cpu_usage_percent: self.system.global_cpu_usage(),
            memory_used_gb: bytes_to_gb(self.system.used_memory()),
            memory_total_gb: bytes_to_gb(self.system.total_memory()),
            network_rx_mb_delta,
            network_tx_mb_delta,
            io_read_mb_delta: bytes_to_mb(total_read_delta_bytes),
            io_write_mb_delta: bytes_to_mb(total_write_delta_bytes),
            temp_total_mb: temp.total_mb,
            primary_severity: Severity::Healthy,
            primary_reason: "Sin señales fuertes en esta muestra".to_owned(),
        };

        let alerts = build_alerts(
            &processes,
            &connections,
            &temp.top_entries,
            &services,
            &precision,
            &mut overview,
        );

        let mut snapshot = SystemSnapshot {
            collected_at,
            overview,
            alerts,
            processes,
            temp,
            connections,
            events,
            services,
            precision,
            trace_analysis,
        };

        apply_trace_analysis_to_snapshot(&mut snapshot);

        if let Err(error) = self.store.persist_snapshot(&snapshot) {
            snapshot.alerts.push(Alert {
                severity: Severity::Warning,
                title: "Persistencia con advertencia".to_owned(),
                detail: format!("No se pudo guardar el historial SQLite: {error}"),
                pid: None,
                path: None,
                hint: "La app sigue funcionando; solo se pierde este punto del historial"
                    .to_owned(),
            });
        }

        Ok(snapshot)
    }

    /// Devuelve el estado del modo de precisión.
    pub fn precision_status(&self) -> PrecisionStatus {
        let wpr_available = windows::wpr_available();
        let wpa_available = windows::wpa_available();
        let tracerpt_available = windows::tracerpt_available();
        let traces_directory = self.precision_traces_dir.display().to_string();
        let last_trace_path = self
            .precision_last_trace_path
            .as_ref()
            .map(|path| path.display().to_string());
        let last_analysis_path = self
            .precision_last_analysis_path
            .as_ref()
            .map(|path| path.display().to_string());

        if !wpr_available {
            return PrecisionStatus {
                wpr_available,
                wpa_available,
                tracerpt_available,
                is_recording: false,
                recommended_profile: "GeneralProfile -filemode".to_owned(),
                traces_directory,
                last_trace_path,
                last_analysis_path,
                analyzer_label: if tracerpt_available {
                    "tracerpt".to_owned()
                } else {
                    "sin motor ETL".to_owned()
                },
                status_detail: "WPR no está instalado o no está en PATH".to_owned(),
                guidance: "Instala Windows Performance Toolkit para activar captura ETW desde la app. El resumen ETL automático usa tracerpt cuando está disponible.".to_owned(),
            };
        }

        let status_detail = windows::wpr_status()
            .unwrap_or_else(|error| format!("No se pudo consultar WPR: {error}"));
        let is_recording = windows::wpr_is_recording().unwrap_or(false);

        PrecisionStatus {
            wpr_available,
            wpa_available,
            tracerpt_available,
            is_recording,
            recommended_profile: "GeneralProfile -filemode".to_owned(),
            traces_directory,
            last_trace_path,
            last_analysis_path,
            analyzer_label: if tracerpt_available {
                "tracerpt + heurísticas RootCause".to_owned()
            } else {
                "sin análisis local ETL".to_owned()
            },
            status_detail,
            guidance: if is_recording {
                "Reproduce el problema real y detén la captura apenas aparezca la lentitud para evitar ETL enormes.".to_owned()
            } else if tracerpt_available {
                "Puedes iniciar captura, detenerla y luego resumir el ETL desde la propia interfaz para obtener una primera lectura.".to_owned()
            } else {
                "Puedes capturar ETL con WPR, pero para resumirlo dentro de la app conviene disponer también de tracerpt/WPT.".to_owned()
            },
        }
    }

    /// Inicia una captura WPR desde la propia aplicación.
    pub fn start_precision_capture(&mut self, problem_hint: &str) -> Result<String> {
        let message = windows::start_wpr_general_profile(&self.precision_traces_dir, problem_hint)?;
        Ok(format!(
            "{} | Carpeta de trabajo: {}",
            message,
            self.precision_traces_dir.display()
        ))
    }

    /// Detiene la captura WPR y guarda el ETL en la carpeta de trazas.
    pub fn stop_precision_capture(&mut self, problem_description: &str) -> Result<String> {
        let filename = format!(
            "rootcause-precision-{}.etl",
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        let output_path = self.precision_traces_dir.join(filename);
        let message = windows::stop_wpr_capture(&output_path, problem_description)?;
        self.precision_last_trace_path = Some(output_path.clone());
        self.precision_last_analysis_path = None;
        Ok(format!("{} | ETL: {}", message, output_path.display()))
    }

    /// Cancela la captura WPR actual.
    pub fn cancel_precision_capture(&mut self) -> Result<String> {
        windows::cancel_wpr_capture()
    }

    /// Resume el último ETL disponible usando tracerpt y heurísticas locales.
    pub fn analyze_last_precision_trace(&mut self) -> Result<String> {
        let etl_path = self
            .precision_last_trace_path
            .clone()
            .or_else(|| {
                latest_matching_file(&self.precision_traces_dir, |path| {
                    path.extension()
                        .and_then(|v| v.to_str())
                        .map(|v| v.eq_ignore_ascii_case("etl"))
                        .unwrap_or(false)
                })
            })
            .ok_or_else(|| anyhow!("No hay ETL conocido para analizar"))?;

        let (xml_path, summary_path, json_path) =
            etl::analysis_layout(&self.precision_analysis_dir, &etl_path);
        let export_message =
            windows::export_etl_with_tracerpt(&etl_path, &xml_path, &summary_path)?;
        let output_dir = json_path
            .parent()
            .unwrap_or(self.precision_analysis_dir.as_path());
        let analysis =
            etl::summarize_exported_etl(&etl_path, &xml_path, &summary_path, output_dir)?;
        self.precision_last_analysis_path = Some(json_path);
        Ok(format!("{} | {}", export_message, analysis.headline))
    }

    /// Exporta una instantánea a JSON en Descargas o Documentos.
    pub fn export_snapshot(&self, snapshot: &SystemSnapshot) -> Result<String> {
        let path = self.store.export_path();
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, json)
            .with_context(|| format!("No se pudo escribir {}", path.display()))?;
        Ok(path.display().to_string())
    }

    /// Finaliza un proceso si la política local lo permite.
    pub fn terminate_process(&self, pid: u32) -> Result<String> {
        if pid == self.own_pid {
            return Err(anyhow!("La aplicación no se permite finalizar a sí misma"));
        }
        let process = self
            .system
            .process(Pid::from(pid as usize))
            .ok_or_else(|| anyhow!("El proceso ya no existe"))?;
        let name = os_str_to_string(process.name());
        let exe_path = process
            .exe()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        if !self.can_terminate_process(pid, &name, &exe_path) {
            return Err(anyhow!("Proceso protegido por política local"));
        }
        windows::terminate_process(pid)
    }

    /// Bloquea una IP pública vía firewall de Windows.
    pub fn block_remote_ip(&self, ip_or_endpoint: &str) -> Result<String> {
        let ip = network::extract_ip(ip_or_endpoint).unwrap_or_else(|| ip_or_endpoint.to_owned());
        windows::block_remote_ip(&ip)
    }

    /// Detiene temporalmente un servicio permitido.
    pub fn stop_service(&self, service_name: &str) -> Result<String> {
        let lowered = service_name.trim().to_ascii_lowercase();
        if !self.stoppable_services.contains(&lowered) {
            return Err(anyhow!(
                "El servicio {service_name} no está permitido para detención rápida desde la UI"
            ));
        }
        windows::stop_service(&lowered)
    }

    fn can_terminate_process(&self, pid: u32, name: &str, exe_path: &str) -> bool {
        if pid == 0 || pid == 4 || pid == self.own_pid {
            return false;
        }
        let lower_name = name.to_ascii_lowercase();
        if self.protected_names.contains(&lower_name) {
            return false;
        }
        let lower_path = exe_path.to_ascii_lowercase();
        if lower_path.starts_with(r"c:\windows\system32") && lower_name.contains("svchost") {
            return false;
        }
        true
    }

    fn load_last_trace_analysis(&mut self) -> Result<Option<TraceAnalysisSummary>> {
        if self.precision_last_analysis_path.is_none() {
            self.precision_last_analysis_path =
                latest_matching_file_recursive(&self.precision_analysis_dir, |path| {
                    path.file_name()
                        .and_then(|v| v.to_str())
                        .map(|v| v.eq_ignore_ascii_case("trace-analysis.json"))
                        .unwrap_or(false)
                });
        }
        let Some(path) = self.precision_last_analysis_path.as_ref() else {
            return Ok(None);
        };
        let text = fs::read_to_string(path)
            .with_context(|| format!("No se pudo leer {}", path.display()))?;
        let analysis = serde_json::from_str::<TraceAnalysisSummary>(&text)?;
        Ok(Some(analysis))
    }
}

fn apply_trace_analysis_to_snapshot(snapshot: &mut SystemSnapshot) {
    let Some(analysis) = snapshot.trace_analysis.as_ref() else {
        return;
    };
    let Some(finding) = analysis.findings.first() else {
        return;
    };

    if finding.severity > snapshot.overview.primary_severity {
        snapshot.overview.primary_severity = finding.severity;
        snapshot.overview.primary_reason = format!("Traza ETL: {}", analysis.headline);
    }

    snapshot.alerts.insert(
        0,
        Alert {
            severity: finding.severity,
            title: format!("Resultado de ETL: {}", finding.title),
            detail: finding.detail.clone(),
            pid: None,
            path: Some(analysis.etl_path.clone()),
            hint: format!("Evidencia: {}", finding.evidence),
        },
    );
    snapshot.alerts.truncate(8);
}

/// Determina severidad y explicación para cada proceso.
pub fn classify_process(
    name: &str,
    exe_path: &str,
    cpu_percent: f32,
    memory_mb: f32,
    write_delta_bytes: u64,
) -> (Severity, u8, Vec<String>, String) {
    let mut score = 0_u8;
    let mut reasons = Vec::new();
    let lower_name = name.to_ascii_lowercase();
    let lower_path = exe_path.to_ascii_lowercase();
    let write_mb = bytes_to_mb(write_delta_bytes);

    if cpu_percent >= 65.0 {
        score = score.saturating_add(35);
        reasons.push(format!("CPU alto ({cpu_percent:.1}%)"));
    } else if cpu_percent >= 30.0 {
        score = score.saturating_add(18);
        reasons.push(format!("CPU sostenido ({cpu_percent:.1}%)"));
    }

    if memory_mb >= 2_500.0 {
        score = score.saturating_add(28);
        reasons.push(format!("Memoria elevada ({memory_mb:.0} MB)"));
    } else if memory_mb >= 1_000.0 {
        score = score.saturating_add(14);
        reasons.push(format!("Memoria moderada-alta ({memory_mb:.0} MB)"));
    }

    if write_mb >= 200.0 {
        score = score.saturating_add(40);
        reasons.push(format!(
            "Escritura intensa ({write_mb:.1} MB en el intervalo)"
        ));
    } else if write_mb >= 40.0 {
        score = score.saturating_add(20);
        reasons.push(format!("Escritura perceptible ({write_mb:.1} MB)"));
    }

    if lower_path.contains("\\temp\\") || lower_path.contains("\\appdata\\local\\temp\\") {
        score = score.saturating_add(24);
        reasons.push("Ejecutable lanzado desde carpeta temporal".to_owned());
    }

    if [
        "update",
        "installer",
        "setup",
        "msiexec",
        "trustedinstaller",
        "dism",
    ]
    .iter()
    .any(|needle| lower_name.contains(needle))
    {
        score = score.saturating_add(12);
        reasons.push("Patrón de actualización/instalación detectado".to_owned());
    }

    let severity = match score {
        0..=24 => Severity::Healthy,
        25..=54 => Severity::Warning,
        _ => Severity::Critical,
    };

    if reasons.is_empty() {
        reasons.push("Sin presión relevante en esta muestra".to_owned());
    }

    let category = if lower_path.contains("\\temp\\") {
        "Temporal / instalador".to_owned()
    } else if lower_path.contains("\\windows\\softwaredistribution")
        || lower_name.contains("update")
    {
        "Actualización / mantenimiento".to_owned()
    } else if lower_path.starts_with(r"c:\windows") {
        "Sistema operativo".to_owned()
    } else {
        "Aplicación de usuario".to_owned()
    };

    (severity, score, reasons, category)
}

fn build_alerts(
    processes: &[ProcessInsight],
    connections: &[ConnectionInsight],
    temp_entries: &[TempEntry],
    services: &[ServiceState],
    precision: &PrecisionStatus,
    overview: &mut SystemOverview,
) -> Vec<Alert> {
    let mut alerts = Vec::new();

    if let Some(process) = processes
        .iter()
        .find(|process| process.severity == Severity::Critical)
    {
        overview.primary_severity = Severity::Critical;
        overview.primary_reason = format!(
            "Proceso crítico detectado: {} (PID {})",
            process.name, process.pid
        );
        alerts.push(Alert {
            severity: Severity::Critical,
            title: "Proceso dominante con presión alta".to_owned(),
            detail: format!(
                "{} usa {:.1}% CPU, {:.0} MB RAM y escribe {:.1} MB por intervalo.",
                process.name, process.cpu_percent, process.memory_mb, process.io_write_mb_delta
            ),
            pid: Some(process.pid),
            path: Some(process.exe_path.clone()),
            hint: "Revísalo primero; si confirmas que no es esencial, puedes finalizarlo"
                .to_owned(),
        });
    }

    if let Some(connection) = connections
        .iter()
        .find(|item| item.severity == Severity::Critical)
    {
        if overview.primary_severity < Severity::Critical {
            overview.primary_severity = Severity::Critical;
            overview.primary_reason = format!(
                "Conexión pública sospechosa: {} -> {}",
                connection.process_name, connection.remote_address
            );
        }
        alerts.push(Alert {
            severity: Severity::Critical,
            title: "Conexión remota a revisar".to_owned(),
            detail: format!(
                "{} (PID {}) mantiene conexión con {}.",
                connection.process_name, connection.pid, connection.remote_address
            ),
            pid: Some(connection.pid),
            path: Some(connection.exe_path.clone()),
            hint: "Valida la ruta del ejecutable y bloquea la IP si no corresponde".to_owned(),
        });
    }

    if let Some(temp_entry) = temp_entries
        .iter()
        .find(|entry| entry.severity == Severity::Critical)
    {
        if overview.primary_severity < Severity::Warning {
            overview.primary_severity = Severity::Warning;
            overview.primary_reason = format!("Crecimiento temporal alto en {}", temp_entry.path);
        }
        alerts.push(Alert {
            severity: temp_entry.severity,
            title: "Acumulación temporal relevante".to_owned(),
            detail: format!(
                "{} pesa {:.1} MB y contiene {} archivos.",
                temp_entry.path, temp_entry.size_mb, temp_entry.file_count
            ),
            pid: None,
            path: Some(temp_entry.path.clone()),
            hint: temp_entry.note.clone(),
        });
    }

    for service in services {
        let lower_name = service.name.to_ascii_lowercase();
        if (lower_name == "wuauserv" || lower_name == "bits" || lower_name == "dosvc")
            && service.status.eq_ignore_ascii_case("Running")
        {
            alerts.push(Alert {
                severity: Severity::Warning,
                title: "Servicio de actualización activo".to_owned(),
                detail: format!(
                    "{} está {} y puede explicar actividad en segundo plano.",
                    service.display_name, service.status
                ),
                pid: None,
                path: None,
                hint: "Correlaciónalo con SoftwareDistribution, Delivery Optimization y procesos de instalación/descarga".to_owned(),
            });
        }
    }

    if !precision.wpr_available {
        alerts.push(Alert {
            severity: Severity::Healthy,
            title: "Modo de precisión disponible bajo instalación adicional".to_owned(),
            detail: "La app puede operar sin WPR, pero para correlación fina conviene instalar Windows Performance Toolkit.".to_owned(),
            pid: None,
            path: None,
            hint: "Úsalo cuando necesites saber con más exactitud qué archivo o actividad disparó la lentitud.".to_owned(),
        });
    }

    if alerts.is_empty() {
        overview.primary_severity = Severity::Healthy;
        overview.primary_reason = "No se observaron conflictos claros en esta muestra".to_owned();
        alerts.push(Alert {
            severity: Severity::Healthy,
            title: "Estado estable".to_owned(),
            detail: "No aparecieron procesos o conexiones anómalas dominantes en esta captura."
                .to_owned(),
            pid: None,
            path: None,
            hint: "Mantén el monitoreo unos minutos cuando aparezca la lentitud real".to_owned(),
        });
    }

    alerts
}

fn latest_matching_file<F>(dir: &Path, predicate: F) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && predicate(path))
        .filter_map(|path| {
            let modified = path.metadata().ok()?.modified().ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn latest_matching_file_recursive<F>(dir: &Path, predicate: F) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .map(|entry| entry.into_path())
        .filter(|path| path.is_file() && predicate(path))
        .filter_map(|path| {
            let modified = path.metadata().ok()?.modified().ok()?;
            Some((modified, path))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn os_str_to_string(value: &OsStr) -> String {
    value.to_string_lossy().into_owned()
}

fn bytes_to_mb(bytes: u64) -> f32 {
    bytes as f32 / (1024.0 * 1024.0)
}

fn bytes_to_gb(bytes: u64) -> f32 {
    bytes as f32 / (1024.0 * 1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proceso_temporal_con_escritura_fuerte_es_critico() {
        let (severity, score, reasons, category) = classify_process(
            "weird-updater.exe",
            r"C:\Users\vbav\AppData\Local\Temp\weird-updater.exe",
            72.0,
            1800.0,
            350 * 1024 * 1024,
        );
        assert_eq!(severity, Severity::Critical);
        assert!(score > 55);
        assert!(reasons.iter().any(|reason| reason.contains("temporal")));
        assert_eq!(category, "Temporal / instalador");
    }
}
