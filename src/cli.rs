//! Interfaz de línea de comandos para RootCause Windows Inspector.

use crate::meta;
use crate::models::{AiIncidentAdvice, IncidentSummary, SnapshotRow, SystemSnapshot};
use crate::services::{inspector::InspectorService, windows};
use serde::Serialize;
use std::fs;

/// Ejecuta el modo CLI y devuelve el código de salida del proceso.
pub fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        print_help();
        return 0;
    }
    match args[0].as_str() {
        "--help" | "-h" | "help" => {
            print_help();
            0
        }
        "--version" | "-V" | "version" => {
            println!("{} v{}", meta::DISPLAY_NAME, meta::VERSION);
            0
        }
        "status" => cmd_status(&args[1..]),
        "snapshot" => cmd_snapshot(&args[1..]),
        "history" => cmd_history(&args[1..]),
        "incidents" => cmd_incidents(&args[1..]),
        "export" => cmd_export(),
        "wpr" => cmd_wpr(&args[1..]),
        "kill" => {
            let pid = args.get(1).and_then(|s| s.parse::<u32>().ok());
            cmd_kill(pid)
        }
        "block-ip" => {
            let ip = args.get(1).map(|s| s.as_str());
            cmd_block_ip(ip)
        }
        "stop-service" => {
            let name = args.get(1).map(|s| s.as_str());
            cmd_stop_service(name)
        }
        "ai" => cmd_ai(&args[1..]),
        "config" => cmd_config(&args[1..]),
        "autostart" => cmd_autostart(&args[1..]),
        other => {
            eprintln!(
                "Comando desconocido: '{other}'\nUsa  rootcause --help  para ver todas las opciones."
            );
            1
        }
    }
}

fn print_help() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════════╗
║  {name:<58}║
║  v{version:<57}║
║  {author:<58}║
╚══════════════════════════════════════════════════════════════════╝

MODO GUI (por defecto):
  rootcause                               Abre la interfaz gráfica
  rootcause --gui                         Abre la interfaz gráfica (explícito)

INFORMACIÓN:
  rootcause --help                        Esta ayuda
  rootcause --version                     Versión del software

DIAGNÓSTICO DEL SISTEMA:
  rootcause status [--json]               Estado del sistema
  rootcause snapshot [--output PATH]      Captura completa en JSON
  rootcause history [N] [--json]          Últimas N capturas del historial
  rootcause incidents [N] [--json]        Últimos incidentes persistidos
  rootcause export                        Exportar última captura a archivo JSON

MODO DE PRECISIÓN WPR/ETW:
  rootcause wpr start [--note NOTA]       Iniciar captura WPR
  rootcause wpr stop  [--note NOTA]       Detener y guardar ETL
  rootcause wpr cancel                    Cancelar captura activa
  rootcause wpr analyze                   Resumir el último ETL capturado

AUTOSTART Y PERSISTENCIA:
  rootcause autostart [--json]            Entradas Registro Run + Startup + Tareas programadas

CONFIGURACIÓN E IA OPCIONAL:
  rootcause config show [--json]          Ver ruta y configuración efectiva
  rootcause config init                   Crear config JSON base si no existe
  rootcause ai explain-latest [--json]    Enriquecer último incidente con IA

INTERVENCIÓN CONTROLADA (requiere administrador):
  rootcause kill <PID>                    Finalizar proceso por PID
  rootcause block-ip <IP>                 Bloquear IP remota via firewall
  rootcause stop-service <nombre>         Detener servicio por nombre
  Servicios permitidos: bits, dosvc, sysmain, wuauserv

REPOSITORIO:
  {github}
"#,
        name = meta::DISPLAY_NAME,
        version = meta::VERSION,
        author = meta::AUTHOR,
        github = meta::GITHUB,
    );
}

fn init_inspector() -> Result<InspectorService, i32> {
    InspectorService::new().map_err(|e| {
        eprintln!("Error al inicializar el motor de inspección: {e}");
        1
    })
}

fn cmd_status(args: &[String]) -> i32 {
    let json_mode = has_flag(args, "--json");
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.collect_snapshot() {
        Ok(snap) => {
            if json_mode {
                let payload =
                    StatusJson::from_snapshot(&snap, &insp.config_path().display().to_string());
                return print_json(&payload);
            }

            let ov = &snap.overview;
            let sev = format!("{:?}", ov.primary_severity).to_uppercase();
            println!("┌─────────────────────────────────────────────┐");
            println!(
                "│  {name}  v{ver}",
                name = meta::DISPLAY_NAME,
                ver = meta::VERSION
            );
            println!("├─────────────────────────────────────────────┤");
            println!("│  Estado    : {sev}");
            println!("│  Causa     : {}", ov.primary_reason);
            println!("│  CPU       : {:.1}%", ov.cpu_usage_percent);
            println!(
                "│  RAM       : {:.1} / {:.1} GB",
                ov.memory_used_gb, ov.memory_total_gb
            );
            println!(
                "│  I/O       : {:.1} MB/intervalo",
                ov.io_read_mb_delta + ov.io_write_mb_delta
            );
            println!("│  Temp      : {:.1} MB total", ov.temp_total_mb);
            println!("â”‚  Anomalias : {}", snap.anomalies.len());
            if let Some(incident) = snap.incident.as_ref() {
                if let Some(risk) = incident.risk_level {
                    println!(
                        "â”‚  Riesgo    : {} ({})",
                        risk.label(),
                        incident.risk_score
                    );
                }
                if !incident.root_cause_hypothesis.is_empty() {
                    println!("â”‚  Hipotesis : {}", incident.root_cause_hypothesis);
                }
            }
            if !snap.alerts.is_empty() {
                println!("├─────────────────────────────────────────────┤");
                println!("│  Alertas   : {}", snap.alerts.len());
                for alert in snap.alerts.iter().take(5) {
                    println!("│    [{:?}] {}", alert.severity, alert.title);
                }
            }
            println!("└─────────────────────────────────────────────┘");
            0
        }
        Err(e) => {
            eprintln!("Error al capturar estado: {e}");
            1
        }
    }
}

fn cmd_snapshot(args: &[String]) -> i32 {
    let output_path = option_value(args, "--output");
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.collect_snapshot() {
        Ok(snap) => match serde_json::to_string_pretty(&snap) {
            Ok(json) => {
                if let Some(path) = output_path {
                    match fs::write(path, &json) {
                        Ok(_) => println!("Snapshot exportado en {path}"),
                        Err(error) => {
                            eprintln!("No se pudo escribir {path}: {error}");
                            return 1;
                        }
                    }
                } else {
                    println!("{json}");
                }
                0
            }
            Err(e) => {
                eprintln!("Error al serializar snapshot: {e}");
                1
            }
        },
        Err(e) => {
            eprintln!("Error al capturar snapshot: {e}");
            1
        }
    }
}

fn cmd_history(args: &[String]) -> i32 {
    let json_mode = has_flag(args, "--json");
    let n = args
        .iter()
        .find_map(|value| value.parse::<usize>().ok())
        .unwrap_or(10);
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    let rows = insp.load_history(n);
    if json_mode {
        return print_json(&rows);
    }
    if rows.is_empty() {
        println!("Sin historial disponible.");
        println!("Ejecuta la app al menos una vez para generar registros.");
        return 0;
    }
    println!(
        "{:<20}  {:>6}  {:>8}  {:>9}  {:>5}  Proceso dominante",
        "Fecha/Hora", "CPU%", "RAM GB", "I/O W MB", "Alrt"
    );
    println!("{}", "─".repeat(80));
    for row in &rows {
        print_history_row(row);
    }
    0
}

fn cmd_incidents(args: &[String]) -> i32 {
    let json_mode = has_flag(args, "--json");
    let n = args
        .iter()
        .find_map(|value| value.parse::<usize>().ok())
        .unwrap_or(10);
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    let incidents = insp.load_incidents(n);
    if json_mode {
        return print_json(&incidents);
    }
    if incidents.is_empty() {
        println!("Sin incidentes persistidos.");
        println!("Cuando RootCause detecte Warning/Critical los guardará aquí.");
        return 0;
    }
    for incident in &incidents {
        print_incident(incident);
    }
    0
}

fn cmd_export() -> i32 {
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.collect_snapshot() {
        Ok(snap) => match insp.export_snapshot(&snap) {
            Ok(path) => {
                println!("Exportado → {path}");
                if let Ok(bp) = insp.export_history_backup() {
                    println!("Historial backup → {bp}");
                }
                0
            }
            Err(e) => {
                eprintln!("Error al exportar: {e}");
                1
            }
        },
        Err(e) => {
            eprintln!("Error al capturar para exportar: {e}");
            1
        }
    }
}

fn cmd_wpr(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Subcomando WPR requerido: start | stop | cancel | analyze");
        return 1;
    }
    let note = extract_note(args);
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    let result = match args[0].as_str() {
        "start" => insp.start_precision_capture(&note),
        "stop" => insp.stop_precision_capture(&note),
        "cancel" => insp.cancel_precision_capture(),
        "analyze" => insp.analyze_last_precision_trace(),
        other => {
            eprintln!(
                "Subcomando WPR desconocido: '{other}'\nOpciones: start | stop | cancel | analyze"
            );
            return 1;
        }
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(e) => {
            eprintln!("Error WPR: {e}");
            1
        }
    }
}

fn cmd_kill(pid: Option<u32>) -> i32 {
    let Some(pid) = pid else {
        eprintln!("PID requerido.  Ejemplo: rootcause kill 1234");
        return 1;
    };
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.terminate_process(pid) {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(e) => {
            eprintln!("Error al finalizar PID {pid}: {e}");
            1
        }
    }
}

fn cmd_block_ip(ip: Option<&str>) -> i32 {
    let Some(ip) = ip else {
        eprintln!("Dirección IP requerida.  Ejemplo: rootcause block-ip 185.220.101.45");
        return 1;
    };
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.block_remote_ip(ip) {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(e) => {
            eprintln!("Error al bloquear {ip}: {e}");
            1
        }
    }
}

fn cmd_stop_service(name: Option<&str>) -> i32 {
    let Some(name) = name else {
        eprintln!("Nombre de servicio requerido.  Ejemplo: rootcause stop-service bits");
        eprintln!("Servicios permitidos: bits, dosvc, sysmain, wuauserv");
        return 1;
    };
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.stop_service(name) {
        Ok(msg) => {
            println!("{msg}");
            0
        }
        Err(e) => {
            eprintln!("Error al detener '{name}': {e}");
            1
        }
    }
}

fn cmd_ai(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Subcomando IA requerido: explain-latest");
        return 1;
    }
    let json_mode = has_flag(args, "--json");
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match args[0].as_str() {
        "explain-latest" => match insp.explain_latest_incident_with_ai() {
            Ok(advice) => {
                if json_mode {
                    print_json(&advice)
                } else {
                    print_ai_advice(&advice);
                    0
                }
            }
            Err(error) => {
                eprintln!("Error IA: {error}");
                1
            }
        },
        other => {
            eprintln!("Subcomando IA desconocido: {other}");
            1
        }
    }
}

fn cmd_config(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("Subcomando config requerido: show | init");
        return 1;
    }
    let json_mode = has_flag(args, "--json");
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match args[0].as_str() {
        "show" => {
            let view = ConfigView {
                path: insp.config_path().display().to_string(),
                config: insp.config().clone(),
            };
            if json_mode {
                print_json(&view)
            } else {
                println!("Config path : {}", view.path);
                println!(
                    "IA opcional : {}",
                    if view.config.ai.enabled {
                        "habilitada"
                    } else {
                        "deshabilitada"
                    }
                );
                println!(
                    "Acciones man : {}",
                    if view.config.remediation.manual_actions_enabled {
                        "habilitadas"
                    } else {
                        "deshabilitadas"
                    }
                );
                println!(
                    "Umbrales proc: CPU {:.0}/{:.0}% | RAM {:.0}/{:.0} MB | I/O {:.0}/{:.0} MB",
                    view.config.thresholds.process.cpu_warning_percent,
                    view.config.thresholds.process.cpu_critical_percent,
                    view.config.thresholds.process.memory_warning_mb,
                    view.config.thresholds.process.memory_critical_mb,
                    view.config.thresholds.process.io_write_warning_mb,
                    view.config.thresholds.process.io_write_critical_mb
                );
                println!(
                    "Anomalias V1: CPU {:.0}% x{} | RAM +{:.0} MB x{} | Escritura {:.0} MB x{}",
                    view.config.anomaly.cpu_sustained_percent,
                    view.config.anomaly.cpu_sustained_samples,
                    view.config.anomaly.memory_growth_mb,
                    view.config.anomaly.memory_growth_samples,
                    view.config.anomaly.aggressive_write_mb,
                    view.config.anomaly.aggressive_write_samples
                );
                println!(
                    "Red/persist.: pub {} | local {} | respawn {} en {}s | persistencia {}",
                    view.config.anomaly.public_destination_count,
                    view.config.anomaly.local_scan_destination_count,
                    view.config.anomaly.respawn_count,
                    view.config.anomaly.respawn_window_secs,
                    if view.config.anomaly.watch_persistence {
                        "observada"
                    } else {
                        "desactivada"
                    }
                );
                println!(
                    "Resiliencia : heartbeat {}s | stale {}s | ventana {}s | max reinicios {} | config {}",
                    view.config.resilience.heartbeat_interval_secs,
                    view.config.resilience.stale_after_secs,
                    view.config.resilience.restart_window_secs,
                    view.config.resilience.max_restarts_in_window,
                    if view.config.resilience.watch_config_integrity {
                        "vigilada"
                    } else {
                        "sin vigilancia"
                    }
                );
                0
            }
        }
        "init" => match insp.write_default_config_if_missing() {
            Ok(path) => {
                println!("Configuración base disponible en {path}");
                0
            }
            Err(error) => {
                eprintln!("No se pudo inicializar la configuración: {error}");
                1
            }
        },
        other => {
            eprintln!("Subcomando config desconocido: {other}");
            1
        }
    }
}

fn cmd_autostart(args: &[String]) -> i32 {
    let json_mode = has_flag(args, "--json");
    match windows::persistence_entries() {
        Ok(entries) => {
            if json_mode {
                return print_json(&entries);
            }
            if entries.is_empty() {
                println!("No se encontraron entradas de autostart.");
                return 0;
            }
            println!("{:<32}  {:<26}  {}", "Nombre", "Tipo", "Comando / Ruta");
            println!("{}", "─".repeat(100));
            for e in &entries {
                let kind_short = if e.entry_kind.contains("RunOnce") {
                    "RunOnce"
                } else if e.entry_kind.contains("HKLM") {
                    "Registro (Sistema)"
                } else if e.entry_kind.contains("HKCU") {
                    "Registro (Usuario)"
                } else if e.entry_kind.contains("Scheduled") {
                    "Tarea programada"
                } else if e.entry_kind.contains("All Users") {
                    "Startup (Todos)"
                } else {
                    "Startup (Usuario)"
                };
                let disk = if e.exists_on_disk { "✓" } else { "✗" };
                let name_col: String = e.name.chars().take(31).collect();
                let cmd_col: String = e.command.chars().take(55).collect();
                println!("{:<32}  {:<26}  {} {}", name_col, kind_short, disk, cmd_col);
            }
            println!();
            let missing = entries.iter().filter(|e| !e.exists_on_disk).count();
            println!(
                "{} entrada(s) total — {} sin archivo en disco",
                entries.len(),
                missing
            );
            0
        }
        Err(e) => {
            eprintln!("Error al leer entradas de autostart: {e}");
            1
        }
    }
}

fn print_history_row(row: &SnapshotRow) {
    let ts: String = row.collected_at.chars().take(19).collect();
    let flag = if row.has_critical { "⚠" } else { " " };
    println!(
        "{:<20}  {:>5.1}%  {:>6.1} GB  {:>7.1} MB  {:>3}{} {}",
        ts,
        row.cpu_usage,
        row.memory_used_gb,
        row.io_write_mb_delta,
        row.alerts_count,
        flag,
        row.dominant_process,
    );
}

fn print_incident(incident: &IncidentSummary) {
    println!(
        "[{:?}] {}  ({})",
        incident.severity, incident.title, incident.collected_at
    );
    println!("  Tipo      : {}", incident.kind);
    println!("  Resumen   : {}", incident.summary);
    if let Some(risk) = incident.risk_level {
        println!("  Riesgo    : {} ({})", risk.label(), incident.risk_score);
    }
    if !incident.root_cause_hypothesis.is_empty() {
        println!("  Hipotesis : {}", incident.root_cause_hypothesis);
    }
    if incident.anomaly_count > 0 {
        println!("  Anomalias : {}", incident.anomaly_count);
    }
    if !incident.anomaly_types.is_empty() {
        println!("  Tipos     : {}", incident.anomaly_types.join(" | "));
    }
    if !incident.probable_causes.is_empty() {
        println!("  Causas    : {}", incident.probable_causes.join(" | "));
    }
    if !incident.recommended_actions.is_empty() {
        println!("  Acciones  : {}", incident.recommended_actions.join(" | "));
    }
    if let Some(event) = incident.anomaly_events.first() {
        println!("  Evidencia : {}", event.summary);
    }
    if let Some(ai) = incident.ai_advice.as_ref() {
        println!("  IA        : {} ({})", ai.summary, ai.confidence);
    }
    println!();
}

fn print_ai_advice(advice: &AiIncidentAdvice) {
    println!("Resumen     : {}", advice.summary);
    println!("Confianza   : {}", advice.confidence);
    println!("Proveedor   : {}", advice.provider);
    println!("Modelo      : {}", advice.model);
    if !advice.probable_causes.is_empty() {
        println!("Causas      : {}", advice.probable_causes.join(" | "));
    }
    if !advice.suggested_actions.is_empty() {
        println!("Acciones    : {}", advice.suggested_actions.join(" | "));
    }
    if !advice.warnings.is_empty() {
        println!("Advertencias: {}", advice.warnings.join(" | "));
    }
}

fn extract_note(args: &[String]) -> String {
    let mut i = 0;
    while i + 1 < args.len() {
        if args[i] == "--note" {
            return args[i + 1].clone();
        }
        i += 1;
    }
    "Captura desde CLI".to_owned()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn option_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].as_str())
}

fn print_json<T: Serialize>(value: &T) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("Error al serializar JSON: {error}");
            1
        }
    }
}

#[derive(Serialize)]
struct StatusJson {
    severity: String,
    agent_status: String,
    agent_summary: String,
    last_heartbeat_at: String,
    config_integrity_changed: bool,
    unexpected_shutdown_detected: bool,
    watchdog_backoff_active: bool,
    cpu_percent: f32,
    ram_percent: f32,
    ram_used_gb: f32,
    ram_total_gb: f32,
    io_mbps: f32,
    io_write_mb: f32,
    network_rx_mb: f32,
    network_tx_mb: f32,
    alert_count: usize,
    anomaly_count: usize,
    highest_risk_level: Option<String>,
    root_cause_hypothesis: String,
    primary_reason: String,
    dominant_process: String,
    config_path: String,
    incident_available: bool,
}

impl StatusJson {
    fn from_snapshot(snapshot: &SystemSnapshot, config_path: &str) -> Self {
        let ov = &snapshot.overview;
        let ram_percent = if ov.memory_total_gb > 0.0 {
            ov.memory_used_gb / ov.memory_total_gb * 100.0
        } else {
            0.0
        };
        Self {
            severity: format!("{:?}", ov.primary_severity),
            agent_status: snapshot.agent_health.status.label().to_owned(),
            agent_summary: snapshot.agent_health.summary.clone(),
            last_heartbeat_at: snapshot.agent_health.last_heartbeat_at.clone(),
            config_integrity_changed: snapshot.agent_health.config_changed,
            unexpected_shutdown_detected: snapshot.agent_health.unexpected_shutdown_detected,
            watchdog_backoff_active: snapshot.agent_health.watchdog_backoff_active,
            cpu_percent: ov.cpu_usage_percent,
            ram_percent,
            ram_used_gb: ov.memory_used_gb,
            ram_total_gb: ov.memory_total_gb,
            io_mbps: ov.io_read_mb_delta + ov.io_write_mb_delta,
            io_write_mb: ov.io_write_mb_delta,
            network_rx_mb: ov.network_rx_mb_delta,
            network_tx_mb: ov.network_tx_mb_delta,
            alert_count: snapshot.alerts.len(),
            anomaly_count: snapshot.anomalies.len(),
            highest_risk_level: snapshot
                .incident
                .as_ref()
                .and_then(|incident| incident.risk_level.map(|risk| risk.label().to_owned())),
            root_cause_hypothesis: snapshot
                .incident
                .as_ref()
                .map(|incident| incident.root_cause_hypothesis.clone())
                .unwrap_or_default(),
            primary_reason: ov.primary_reason.clone(),
            dominant_process: snapshot
                .processes
                .first()
                .map(|process| format!("{} ({})", process.name, process.pid))
                .unwrap_or_else(|| "Sin datos".to_owned()),
            config_path: config_path.to_owned(),
            incident_available: snapshot.incident.is_some(),
        }
    }
}

#[derive(Serialize)]
struct ConfigView {
    path: String,
    config: crate::config::RootCauseConfig,
}
