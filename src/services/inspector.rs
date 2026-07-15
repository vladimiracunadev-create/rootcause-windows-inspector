//! Servicio principal de inspección.
//!
//! Orquesta la captura de métricas, aplica reglas, persiste evidencia y expone
//! acciones seguras para GUI y CLI.

use crate::config::{ConfigManager, RootCauseConfig};
use crate::models::{
    AiIncidentAdvice, Alert, AnomalyEvent, AuditRecord, HardwareInfo, IncidentSummary, NetworkScan,
    PersistenceChange, PersistenceEntry, PrecisionStatus, ProcessInsight, Severity, SnapshotRow,
    SystemOverview, SystemSnapshot, TempCleanResult, TraceAnalysisSummary, WatchedItem,
};
use crate::services::{
    ai::AiAdvisor,
    anomaly::{AnomalyTracker, DetectionInput, persistence_change_event},
    baseline::{self, SurfaceSpec},
    etl, netscan, network,
    persistence::{PersistenceStore, persistence_entry_key},
    resilience::ResilienceMonitor,
    rules, temp_scan, windows,
};

/// Superficie vigilada: servicios de Windows (motor genérico de baseline).
const SERVICE_SURFACE: SurfaceSpec = SurfaceSpec {
    id: "service",
    title_added: "Servicio nuevo detectado",
    title_modified: "Servicio modificado",
    title_removed: "Servicio eliminado",
    summary_noun: "El servicio",
};

/// Superficie vigilada: dispositivos de la red local ("red conocida"). Reutiliza
/// el motor genérico de baseline; la clave estable de cada equipo es su MAC.
const NETWORK_SURFACE_ID: &str = "network-device";
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{Networks, Pid, System, get_current_pid};

const APP_NAME: &str = "RootCauseInspector";

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
    config: RootCauseConfig,
    config_path: PathBuf,
    config_warning: Option<String>,
    resilience_monitor: ResilienceMonitor,
    anomaly_tracker: AnomalyTracker,
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
        let store = PersistenceStore::new(APP_NAME)?;
        let (config_manager, config_warning) = ConfigManager::load_or_default(APP_NAME);
        let resilience_monitor = ResilienceMonitor::new(
            APP_NAME,
            config_manager.path(),
            &config_manager.config().resilience,
        )?;

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

        let service = Self {
            system,
            networks,
            process_baselines: HashMap::new(),
            protected_names,
            stoppable_services,
            own_pid,
            store,
            config: config_manager.config().clone(),
            config_path: config_manager.path().to_path_buf(),
            config_warning,
            resilience_monitor,
            anomaly_tracker: AnomalyTracker::default(),
            precision_traces_dir,
            precision_analysis_dir,
            precision_last_trace_path,
            precision_last_analysis_path,
        };

        for record in service.resilience_monitor.startup_audits() {
            let _ = service.store.record_audit(&record);
        }

        Ok(service)
    }

    /// Devuelve una frase rápida desde el historial persistido.
    pub fn latest_history_line(&self) -> String {
        self.store
            .latest_summary_line()
            .ok()
            .flatten()
            .unwrap_or_else(|| format!("Historial listo en {}", self.store.db_path().display()))
    }

    /// Ruta de la configuración operativa actual.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Configuración actual cargada por el motor.
    pub fn config(&self) -> &RootCauseConfig {
        &self.config
    }

    pub fn write_default_config_if_missing(&self) -> Result<String> {
        let path = ConfigManager::write_default_if_missing(APP_NAME)?;
        Ok(path.display().to_string())
    }

    /// Persiste `config` en disco y actualiza el estado interno del motor.
    pub fn save_config(&mut self, config: &RootCauseConfig) -> Result<()> {
        ConfigManager::save_to_path(&self.config_path, config)?;
        self.config = config.clone();
        Ok(())
    }

    /// Carga las últimas N filas del historial SQLite.
    pub fn load_history(&self, limit: usize) -> Vec<SnapshotRow> {
        self.store.load_recent(limit).unwrap_or_default()
    }

    /// Carga los últimos N incidentes persistidos.
    pub fn load_incidents(&self, limit: usize) -> Vec<IncidentSummary> {
        self.store.load_recent_incidents(limit).unwrap_or_default()
    }

    pub fn latest_incident(&self) -> Option<IncidentSummary> {
        self.store.latest_incident().ok().flatten()
    }

    /// Recopila información estática del hardware del equipo.
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
        let heartbeat_audits = self.resilience_monitor.heartbeat().unwrap_or_default();
        for record in &heartbeat_audits {
            let _ = self.store.record_audit(record);
        }

        self.system.refresh_all();
        self.networks.refresh(true);

        let collected_at = Utc::now();
        let mut process_names = HashMap::new();
        let mut process_paths = HashMap::new();
        let mut active_pids = HashSet::new();
        let mut processes = Vec::new();
        let mut total_read_delta_bytes = 0_u64;
        let mut total_write_delta_bytes = 0_u64;

        for process in self.system.processes().values() {
            let pid = process.pid().as_u32();
            active_pids.insert(pid);

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

            let write_delta_mb = bytes_to_mb(write_delta);
            let (severity, score, reasons, category) = rules::classify_process(
                &name,
                &exe_path,
                cpu_percent,
                memory_mb,
                write_delta_mb,
                &self.config.thresholds.process,
            );
            let can_terminate = self.can_terminate_process(pid, &name, &exe_path);

            processes.push(ProcessInsight {
                pid,
                name,
                exe_path,
                parent_pid,
                cpu_percent,
                memory_mb,
                io_read_mb_delta: bytes_to_mb(read_delta),
                io_write_mb_delta: write_delta_mb,
                status: format!("{:?}", process.status()),
                category,
                severity,
                score,
                can_terminate,
                reasons,
                command_line: None,
            });
        }

        self.process_baselines
            .retain(|pid, _| active_pids.contains(pid));

        processes.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.io_write_mb_delta.total_cmp(&a.io_write_mb_delta))
                .then_with(|| b.memory_mb.total_cmp(&a.memory_mb))
                .then_with(|| b.cpu_percent.total_cmp(&a.cpu_percent))
        });

        let cmdline_pids: Vec<u32> = processes
            .iter()
            .filter(|process| {
                let lower_name = process.name.to_ascii_lowercase();
                let lower_path = process.exe_path.to_ascii_lowercase();
                matches!(process.severity, Severity::Critical)
                    || process.io_write_mb_delta > 20.0
                    || process.cpu_percent >= self.config.anomaly.cpu_sustained_percent * 0.7
                    || lower_path.contains("\\temp\\")
                    || lower_path.contains("\\downloads\\")
                    || [
                        "powershell",
                        "cmd.exe",
                        "wscript",
                        "cscript",
                        "mshta",
                        "python",
                    ]
                    .iter()
                    .any(|item| lower_name.contains(item))
            })
            .take(12)
            .map(|p| p.pid)
            .collect();
        if !cmdline_pids.is_empty() {
            let cmdlines = windows::batch_process_cmdlines(&cmdline_pids);
            for process in &mut processes {
                if let Some(cmdline) = cmdlines.get(&process.pid) {
                    process.command_line = Some(cmdline.clone());
                }
            }
        }

        let connections = match windows::netstat() {
            Ok(output) => network::parse_netstat_output(&output, &process_names, &process_paths),
            Err(_) => Vec::new(),
        };
        // Red local (escaneo pasivo): lee la tabla de vecinos y la coteja con la
        // baseline de "red conocida". Los dispositivos nuevos generan alertas
        // `unknown-device`. El barrido activo se hace solo bajo demanda (GUI/CLI).
        let mut network = netscan::scan_from_json(
            &windows::network_scan_raw(false).unwrap_or_default(),
            false,
            &collected_at.to_rfc3339(),
        );
        let network_change_events = self.detect_network_changes(collected_at, &mut network);

        let temp = temp_scan::scan_temp_overview(&self.config.thresholds.temp).unwrap_or_default();
        let events = windows::recent_system_events(18).unwrap_or_default();
        let services = windows::relevant_services().unwrap_or_default();
        let mut persistence_entries = windows::persistence_entries().unwrap_or_default();
        // Compara contra la baseline conocida: anota NUEVA/MODIFICADA/ELIMINADA,
        // agrega entradas sintéticas eliminadas y devuelve eventos por cada cambio.
        let persistence_change_events =
            self.detect_persistence_changes(collected_at, &mut persistence_entries);
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

        let mut anomalies = self.anomaly_tracker.analyze(DetectionInput {
            collected_at,
            processes: &processes,
            connections: &connections,
            services: &services,
            persistence_entries: &persistence_entries,
            config: &self.config.anomaly,
        });
        // Cambios en servicios vigilados vs baseline (superficie genérica).
        let service_change_events = if self.config.anomaly.watch_service_changes {
            let mut service_items = windows::services_baseline_items().unwrap_or_default();
            self.detect_service_changes(collected_at, &mut service_items)
        } else {
            Vec::new()
        };

        // Añade los cambios de autoarranque y de servicios, y re-ordena por
        // severidad/score para que un cambio de alta severidad no quede fuera
        // del recorte de alertas.
        let mut change_events = persistence_change_events;
        change_events.extend(service_change_events);
        change_events.extend(network_change_events);
        if !change_events.is_empty() {
            anomalies.extend(change_events);
            anomalies.sort_by(|left, right| {
                right
                    .severity
                    .cmp(&left.severity)
                    .then_with(|| right.score.cmp(&left.score))
                    .then_with(|| left.kind.cmp(&right.kind))
            });
        }

        let mut alerts = rules::build_alerts(
            rules::AlertBuildInputs {
                processes: &processes,
                connections: &connections,
                temp_entries: &temp.top_entries,
                services: &services,
                anomalies: &anomalies,
                precision: &precision,
            },
            &mut overview,
            self.config.alerting.max_alerts,
        );

        if let Some(warning) = self.config_warning.as_ref() {
            alerts.push(Alert {
                severity: Severity::Warning,
                title: "Configuración con fallback".to_owned(),
                detail: warning.clone(),
                pid: None,
                path: Some(self.config_path.display().to_string()),
                hint: "Corrige el JSON o genera un archivo limpio con `rootcause config init`"
                    .to_owned(),
            });
        }

        let mut snapshot = SystemSnapshot {
            collected_at,
            overview,
            alerts,
            agent_health: self.resilience_monitor.health().clone(),
            processes,
            temp,
            connections,
            network: Some(network),
            events,
            services,
            persistence_entries,
            anomalies,
            incident: None,
            precision,
            trace_analysis,
        };

        apply_agent_health_to_snapshot(&mut snapshot, self.config.alerting.max_alerts);
        apply_trace_analysis_to_snapshot(&mut snapshot, self.config.alerting.max_alerts);

        if let Some(incident) = rules::derive_incident(&snapshot) {
            snapshot.incident = Some(incident.clone());
            let _ = self
                .store
                .persist_incident(&incident, self.config.collection.incident_limit);
        }

        if let Err(error) = self
            .store
            .persist_snapshot(&snapshot, self.config.collection.history_limit)
        {
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
        let result = windows::start_wpr_general_profile(&self.precision_traces_dir, problem_hint)
            .map(|message| {
                format!(
                    "{} | Carpeta de trabajo: {}",
                    message,
                    self.precision_traces_dir.display()
                )
            });
        self.audit_action(
            "precision-start",
            problem_hint,
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
    }

    /// Detiene la captura WPR y guarda el ETL en la carpeta de trazas.
    pub fn stop_precision_capture(&mut self, problem_description: &str) -> Result<String> {
        let filename = format!(
            "rootcause-precision-{}.etl",
            Utc::now().format("%Y%m%d-%H%M%S")
        );
        let output_path = self.precision_traces_dir.join(filename);
        let result = windows::stop_wpr_capture(&output_path, problem_description).map(|message| {
            self.precision_last_trace_path = Some(output_path.clone());
            self.precision_last_analysis_path = None;
            format!("{} | ETL: {}", message, output_path.display())
        });
        self.audit_action(
            "precision-stop",
            &output_path.display().to_string(),
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
    }

    /// Cancela la captura WPR actual.
    pub fn cancel_precision_capture(&mut self) -> Result<String> {
        let result = windows::cancel_wpr_capture();
        self.audit_action(
            "precision-cancel",
            "wpr",
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
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
        let message = format!("{} | {}", export_message, analysis.headline);
        self.audit_action(
            "precision-analyze",
            &etl_path.display().to_string(),
            Some(&message),
            None,
        );
        Ok(message)
    }

    /// Ejecuta el adaptador IA opcional sobre el incidente más reciente.
    pub fn explain_latest_incident_with_ai(&self) -> Result<AiIncidentAdvice> {
        let incident = self
            .latest_incident()
            .ok_or_else(|| anyhow!("No hay incidentes persistidos para enriquecer"))?;
        let advisor = AiAdvisor::new(self.config.ai.clone());
        let result = advisor.summarize_incident(&incident);

        match &result {
            Ok(advice) => {
                let _ = self.store.update_incident_ai(&incident.incident_id, advice);
                self.audit_action(
                    "ai-explain-latest",
                    &incident.incident_id,
                    Some(&advice.summary),
                    None,
                );
            }
            Err(error) => {
                self.audit_action(
                    "ai-explain-latest",
                    &incident.incident_id,
                    None,
                    Some(error),
                );
            }
        }

        result
    }

    /// Exporta el historial a JSON junto al SQLite.
    pub fn export_history_backup(&self) -> Result<String> {
        let path = self
            .store
            .export_history_backup(self.config.collection.history_limit)?;
        Ok(path.display().to_string())
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
        if !self.config.remediation.manual_actions_enabled {
            let error = anyhow!("Las acciones manuales están desactivadas por configuración");
            self.audit_action("terminate-process", &pid.to_string(), None, Some(&error));
            return Err(error);
        }

        let result = (|| {
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
        })();

        self.audit_action(
            "terminate-process",
            &pid.to_string(),
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
    }

    /// Bloquea una IP pública vía firewall de Windows.
    pub fn block_remote_ip(&self, ip_or_endpoint: &str) -> Result<String> {
        if !self.config.remediation.manual_actions_enabled {
            let error = anyhow!("Las acciones manuales están desactivadas por configuración");
            self.audit_action("block-ip", ip_or_endpoint, None, Some(&error));
            return Err(error);
        }

        let ip = network::extract_ip(ip_or_endpoint).unwrap_or_else(|| ip_or_endpoint.to_owned());
        let result = windows::block_remote_ip(&ip);
        self.audit_action(
            "block-ip",
            &ip,
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
    }

    /// Detiene temporalmente un servicio permitido.
    pub fn stop_service(&self, service_name: &str) -> Result<String> {
        if !self.config.remediation.manual_actions_enabled {
            let error = anyhow!("Las acciones manuales están desactivadas por configuración");
            self.audit_action("stop-service", service_name, None, Some(&error));
            return Err(error);
        }

        let lowered = service_name.trim().to_ascii_lowercase();
        let result = if !self.stoppable_services.contains(&lowered) {
            Err(anyhow!(
                "El servicio {service_name} no está permitido para detención rápida desde la UI"
            ))
        } else {
            windows::stop_service(&lowered)
        };
        self.audit_action(
            "stop-service",
            &lowered,
            result.as_ref().ok().map(|s| s.as_str()),
            result.as_ref().err(),
        );
        result
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

    /// Compara `entries` contra la baseline conocida y los anota con su estado
    /// de cambio. Si la baseline está vacía (primera ejecución), la siembra con
    /// el estado actual y no marca nada como cambio (primera foto = estado bueno).
    /// Devuelve `true` si había una baseline previa contra la que comparar.
    fn diff_persistence_baseline(&self, entries: &mut Vec<PersistenceEntry>) -> bool {
        let baseline = match self.store.load_persistence_baseline() {
            Ok(baseline) => baseline,
            Err(_) => return false,
        };

        if baseline.is_empty() {
            // Primera foto: aceptar todo como baseline "buena conocida".
            let _ = self.store.replace_persistence_baseline(entries);
            return false;
        }

        let mut current_keys = HashSet::new();
        for entry in entries.iter_mut() {
            let key = persistence_entry_key(entry);
            current_keys.insert(key.clone());
            entry.change_status = match baseline.get(&key) {
                None => PersistenceChange::Added,
                Some(base) if base.command != entry.command => PersistenceChange::Modified,
                Some(_) => PersistenceChange::Unchanged,
            };
        }

        // Entradas que estaban en la baseline y ya no aparecen: sintéticas eliminadas.
        for (key, base) in &baseline {
            if !current_keys.contains(key) {
                let mut removed = base.clone();
                removed.change_status = PersistenceChange::Removed;
                removed.note = "Estaba en la baseline y ya no aparece.".to_owned();
                entries.push(removed);
            }
        }

        true
    }

    /// Ejecuta la comparación con la baseline y genera un evento por cada cambio.
    fn detect_persistence_changes(
        &self,
        collected_at: DateTime<Utc>,
        entries: &mut Vec<PersistenceEntry>,
    ) -> Vec<AnomalyEvent> {
        let had_baseline = self.diff_persistence_baseline(entries);
        if !had_baseline || !self.config.anomaly.watch_persistence {
            return Vec::new();
        }
        entries
            .iter()
            .filter(|entry| entry.change_status.is_change())
            .filter_map(|entry| persistence_change_event(collected_at, entry))
            .collect()
    }

    /// Reconoce el estado actual de autoarranque como la nueva baseline "buena".
    /// A partir de aquí, los cambios previos dejan de reportarse.
    pub fn accept_persistence_baseline(&self) -> Result<usize> {
        let entries = windows::persistence_entries().unwrap_or_default();
        let count = entries.len();
        self.store.replace_persistence_baseline(&entries)?;
        self.audit_action(
            "accept-persistence-baseline",
            &format!("{count} entradas"),
            Some("Baseline de autoarranque actualizada"),
            None,
        );
        Ok(count)
    }

    /// Lista las entradas de autoarranque anotadas con su estado de cambio
    /// respecto a la baseline conocida. Uso principal: CLI.
    pub fn autostart_entries_with_changes(&self) -> Vec<PersistenceEntry> {
        let mut entries = windows::persistence_entries().unwrap_or_default();
        self.diff_persistence_baseline(&mut entries);
        entries
    }

    /// Compara los servicios actuales contra la baseline y genera un evento por
    /// cada cambio (nuevo/modificado/eliminado). Anota `items` in situ.
    fn detect_service_changes(
        &self,
        collected_at: DateTime<Utc>,
        items: &mut Vec<WatchedItem>,
    ) -> Vec<AnomalyEvent> {
        let had_baseline = baseline::diff_surface(&self.store, SERVICE_SURFACE.id, items);
        if !had_baseline {
            return Vec::new();
        }
        items
            .iter()
            .filter(|item| item.change_status.is_change())
            .filter_map(|item| baseline::surface_change_event(collected_at, &SERVICE_SURFACE, item))
            .collect()
    }

    /// Reconoce el estado actual de los servicios como la nueva baseline "buena".
    pub fn accept_service_baseline(&self) -> Result<usize> {
        let items = windows::services_baseline_items().unwrap_or_default();
        let count = items.len();
        self.store.replace_baseline(SERVICE_SURFACE.id, &items)?;
        self.audit_action(
            "accept-service-baseline",
            &format!("{count} servicios"),
            Some("Baseline de servicios actualizada"),
            None,
        );
        Ok(count)
    }

    /// Lista los servicios anotados con su estado de cambio vs baseline. CLI.
    pub fn service_entries_with_changes(&self) -> Vec<WatchedItem> {
        let mut items = windows::services_baseline_items().unwrap_or_default();
        baseline::diff_surface(&self.store, SERVICE_SURFACE.id, &mut items);
        items
    }

    /// Explora la red local y devuelve el escaneo ya cotejado contra la baseline
    /// de "red conocida". `deep = true` hace barrido activo de descubrimiento y
    /// resuelve nombres (más lento). No emite alertas: la protección permanente
    /// vive en `collect_snapshot`; esto es para vista y CLI bajo demanda.
    pub fn scan_network(&self, deep: bool) -> NetworkScan {
        let raw = windows::network_scan_raw(deep).unwrap_or_default();
        let mut scan = netscan::scan_from_json(&raw, deep, &Utc::now().to_rfc3339());
        self.annotate_network_changes(&mut scan);
        scan
    }

    /// Cruza los dispositivos del escaneo contra la baseline conocida: anota el
    /// estado de cambio en cada uno, añade filas sintéticas para los que ya no
    /// responden y re-clasifica severidad/motivo. Si no hay baseline (primera
    /// vez), la siembra en silencio. Devuelve `true` si había baseline previa.
    fn annotate_network_changes(&self, scan: &mut NetworkScan) -> bool {
        let mut items = netscan::device_watch_items(&scan.devices);
        let had_baseline = baseline::diff_surface(&self.store, NETWORK_SURFACE_ID, &mut items);

        let mut status_by_key: HashMap<String, PersistenceChange> = HashMap::new();
        for item in &items {
            status_by_key.insert(item.key.clone(), item.change_status);
        }
        for device in scan.devices.iter_mut() {
            if let Some(status) = status_by_key.get(&netscan::device_key(device)) {
                device.change_status = *status;
            }
            netscan::classify_device(device);
        }

        let present: HashSet<String> = scan.devices.iter().map(netscan::device_key).collect();
        for item in items
            .iter()
            .filter(|item| item.change_status == PersistenceChange::Removed)
        {
            if !present.contains(&item.key) {
                scan.devices.push(netscan::device_from_watch_item(item));
            }
        }

        scan.total_devices = scan.devices.iter().filter(|device| !device.is_self).count();
        scan.new_devices = scan
            .devices
            .iter()
            .filter(|device| device.change_status == PersistenceChange::Added)
            .count();
        had_baseline
    }

    /// Anota el escaneo con los cambios vs baseline y genera un evento por cada
    /// dispositivo nuevo/desconocido (gated por configuración de anomalías).
    fn detect_network_changes(
        &self,
        collected_at: DateTime<Utc>,
        scan: &mut NetworkScan,
    ) -> Vec<AnomalyEvent> {
        let had_baseline = self.annotate_network_changes(scan);
        if !had_baseline
            || !self.config.anomaly.enabled
            || !self.config.anomaly.watch_network_devices
        {
            return Vec::new();
        }
        scan.devices
            .iter()
            .filter_map(|device| netscan::new_device_event(collected_at, device))
            .collect()
    }

    /// Reconoce el estado actual de la red como la nueva baseline "conocida". A
    /// partir de aquí, los dispositivos presentes dejan de marcarse como nuevos.
    pub fn accept_network_baseline(&self) -> Result<usize> {
        let scan = self.scan_network(false);
        let items = netscan::device_watch_items(&scan.devices);
        let count = items.len();
        self.store.replace_baseline(NETWORK_SURFACE_ID, &items)?;
        self.audit_action(
            "accept-network-baseline",
            &format!("{count} dispositivos"),
            Some("Baseline de red conocida actualizada"),
            None,
        );
        Ok(count)
    }

    /// Limpia la carpeta `%TEMP%` del usuario: borra lo no usado y con más de 24h
    /// de antigüedad, saltando lo bloqueado. `dry_run` simula sin borrar. Solo
    /// toca `%TEMP%` (nunca el sistema ni SoftwareDistribution).
    pub fn clean_temp(&self, dry_run: bool) -> TempCleanResult {
        let result = temp_scan::clean_user_temp(24, dry_run);
        if !dry_run {
            let target = format!("{} borrados", result.deleted_count);
            let detail = format!(
                "Limpieza %TEMP% (>24h, no en uso): {:.1} MB liberados, {} en uso saltados",
                result.freed_mb, result.skipped_in_use
            );
            self.audit_action("clean-temp", &target, Some(detail.as_str()), None);
        }
        result
    }

    fn audit_action(
        &self,
        action: &str,
        target: &str,
        success_message: Option<&str>,
        error: Option<&anyhow::Error>,
    ) {
        let record = AuditRecord {
            occurred_at: Utc::now().to_rfc3339(),
            action: action.to_owned(),
            target: target.to_owned(),
            success: error.is_none(),
            detail: success_message
                .map(str::to_owned)
                .or_else(|| error.map(ToString::to_string))
                .unwrap_or_default(),
        };
        let _ = self.store.record_audit(&record);
    }
}

impl Drop for InspectorService {
    fn drop(&mut self) {
        if let Ok(record) = self.resilience_monitor.shutdown() {
            let _ = self.store.record_audit(&record);
        }
    }
}

fn apply_trace_analysis_to_snapshot(snapshot: &mut SystemSnapshot, max_alerts: usize) {
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
    snapshot.alerts.truncate(max_alerts);
}

fn apply_agent_health_to_snapshot(snapshot: &mut SystemSnapshot, max_alerts: usize) {
    let health = &snapshot.agent_health;
    use crate::models::AgentStatus;

    match health.status {
        AgentStatus::Healthy => {
            snapshot.alerts.push(Alert {
                severity: Severity::Healthy,
                title: "Salud del agente estable".to_owned(),
                detail: health.summary.clone(),
                pid: None,
                path: None,
                hint: "Heartbeat local e integridad básica de configuración activos.".to_owned(),
            });
        }
        AgentStatus::Recovered => {
            if snapshot.overview.primary_severity < Severity::Warning {
                snapshot.overview.primary_severity = Severity::Warning;
                snapshot.overview.primary_reason = health.summary.clone();
            }
            snapshot.alerts.insert(
                0,
                Alert {
                    severity: Severity::Warning,
                    title: "Agente recuperado tras cierre abrupto".to_owned(),
                    detail: health.summary.clone(),
                    pid: None,
                    path: None,
                    hint: "Revisa incidentes y auditoría si la detención no fue esperada."
                        .to_owned(),
                },
            );
        }
        AgentStatus::Degraded => {
            if snapshot.overview.primary_severity < Severity::Warning {
                snapshot.overview.primary_severity = Severity::Warning;
                snapshot.overview.primary_reason = health.summary.clone();
            }
            snapshot.alerts.insert(
                0,
                Alert {
                    severity: Severity::Warning,
                    title: "Resiliencia del agente requiere revisión".to_owned(),
                    detail: health.summary.clone(),
                    pid: None,
                    path: None,
                    hint: "Valida cambios de configuración y evita reinicios repetidos hasta confirmar estabilidad."
                        .to_owned(),
                },
            );
        }
    }

    snapshot.alerts.truncate(max_alerts);
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
        let config = RootCauseConfig::default();
        let (severity, score, reasons, category) = rules::classify_process(
            "weird-updater.exe",
            r"C:\Users\vbav\AppData\Local\Temp\weird-updater.exe",
            72.0,
            1800.0,
            350.0,
            &config.thresholds.process,
        );
        assert_eq!(severity, Severity::Critical);
        assert!(score > 55);
        assert!(reasons.iter().any(|reason| reason.contains("temporal")));
        assert_eq!(category, "Temporal / instalador");
    }
}
