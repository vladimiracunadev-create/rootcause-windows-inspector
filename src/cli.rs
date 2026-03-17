//! Interfaz de línea de comandos para RootCause Windows Inspector.
//!
//! Permite operar el motor de diagnóstico sin GUI: útil para scripts,
//! automatización y despliegue corporativo.
//!
//! # Uso básico
//!
//! ```text
//! rootcause --help
//! rootcause status
//! rootcause snapshot > captura.json
//! rootcause history 5
//! rootcause wpr start --note "Disco al 100%"
//! rootcause kill 1234
//! ```

use crate::meta;
use crate::services::inspector::InspectorService;

// ── Punto de entrada CLI ───────────────────────────────────────────────────────

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
        "status" => cmd_status(),
        "snapshot" => cmd_snapshot(),
        "history" => {
            let n = args
                .get(1)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10);
            cmd_history(n)
        }
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
        other => {
            eprintln!(
                "Comando desconocido: '{other}'\nUsa  rootcause --help  para ver todas las opciones."
            );
            1
        }
    }
}

// ── Help ───────────────────────────────────────────────────────────────────────

fn print_help() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════╗
║  {name:<52} ║
║  v{version:<51} ║
║  {author:<52} ║
╚══════════════════════════════════════════════════════════╝

MODO GUI (por defecto):
  rootcause                         Abre la interfaz gráfica
  rootcause --gui                   Abre la interfaz gráfica (explícito)

INFORMACIÓN:
  rootcause --help                  Esta ayuda
  rootcause --version               Versión del software

DIAGNÓSTICO DEL SISTEMA:
  rootcause status                  Estado del sistema en texto plano
  rootcause snapshot                Captura completa en JSON (stdout)
  rootcause history [N]             Últimas N capturas del historial
                                    SQLite (por defecto: 10)
  rootcause export                  Exportar última captura a archivo JSON

MODO DE PRECISIÓN WPR/ETW:
  rootcause wpr start [--note NOTA] Iniciar captura WPR
  rootcause wpr stop  [--note NOTA] Detener y guardar ETL
  rootcause wpr cancel              Cancelar captura activa
  rootcause wpr analyze             Resumir el último ETL capturado
  Nota: requiere WPR instalado (Windows Performance Toolkit)

INTERVENCIÓN CONTROLADA (requiere administrador):
  rootcause kill <PID>              Finalizar proceso por PID
  rootcause block-ip <IP>           Bloquear IP remota via firewall
  rootcause stop-service <nombre>   Detener servicio por nombre
  Servicios permitidos: bits, dosvc, sysmain, wuauserv

EJEMPLOS:
  rootcause status
  rootcause snapshot > captura.json
  rootcause history 5
  rootcause wpr start --note "Disco al 100% durante actualizacion"
  rootcause wpr stop  --note "Disco al 100% durante actualizacion"
  rootcause wpr analyze
  rootcause kill 1234
  rootcause block-ip 185.220.101.45
  rootcause stop-service bits

REPOSITORIO:
  {github}
"#,
        name = meta::DISPLAY_NAME,
        version = meta::VERSION,
        author = meta::AUTHOR,
        github = meta::GITHUB,
    );
}

// ── Inicializar servicio ───────────────────────────────────────────────────────

fn init_inspector() -> Result<InspectorService, i32> {
    InspectorService::new().map_err(|e| {
        eprintln!("Error al inicializar el motor de inspección: {e}");
        1
    })
}

// ── Comandos ───────────────────────────────────────────────────────────────────

fn cmd_status() -> i32 {
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.collect_snapshot() {
        Ok(snap) => {
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
            println!("│  I/O W     : {:.1} MB/intervalo", ov.io_write_mb_delta);
            println!("│  Temp      : {:.1} MB total", ov.temp_total_mb);
            if !snap.alerts.is_empty() {
                println!("├─────────────────────────────────────────────┤");
                println!("│  Alertas   : {}", snap.alerts.len());
                for a in snap.alerts.iter().take(5) {
                    println!("│    [{:?}] {}", a.severity, a.title);
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

fn cmd_snapshot() -> i32 {
    let mut insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    match insp.collect_snapshot() {
        Ok(snap) => match serde_json::to_string_pretty(&snap) {
            Ok(json) => {
                println!("{json}");
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

fn cmd_history(n: usize) -> i32 {
    let insp = match init_inspector() {
        Ok(i) => i,
        Err(c) => return c,
    };
    let rows = insp.load_history(n);
    if rows.is_empty() {
        println!("Sin historial disponible.");
        println!("Ejecuta la app al menos una vez para generar registros.");
        return 0;
    }
    println!(
        "{:<20}  {:>6}  {:>8}  {:>9}  {:>5}  {}",
        "Fecha/Hora", "CPU%", "RAM GB", "I/O W MB", "Alrt", "Proceso dominante"
    );
    println!("{}", "─".repeat(80));
    for row in &rows {
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
        eprintln!("Ejemplo: rootcause wpr start --note \"Disco al 100%\"");
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

// ── Utilidades ─────────────────────────────────────────────────────────────────

/// Extrae el valor de `--note <texto>` de los argumentos.
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
