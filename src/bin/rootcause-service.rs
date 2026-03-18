//! Windows Service — esqueleto de arquitectura para RootCause.
//!
//! ## Estado actual
//! Esqueleto documentado. La implementación completa requiere la crate
//! `windows-service = "0.6"`. Se dejó como placeholder para no romper la
//! compilación actual mientras se define el alcance del servicio.
//!
//! ## Activación futura
//! ```toml
//! # Cargo.toml
//! [[bin]]
//! name = "rootcause-service"
//! path = "src/bin/rootcause-service.rs"
//!
//! [features]
//! service = ["dep:windows-service"]
//!
//! [dependencies]
//! windows-service = { version = "0.6", optional = true }
//! ```
//!
//! ## Responsabilidades del servicio
//! * Capturar snapshots en background (cada N segundos, configurable en registro).
//! * Escribir en la base SQLite compartida con la GUI/CLI.
//! * Exponer un named pipe `\\.\pipe\rootcause` para que GUI y CLI lean el
//!   último estado sin lanzar su propio inspector.
//! * Enviar notificaciones toast al usuario activo cuando la severidad sube a
//!   Critical (via `windows::UI::Notifications`).
//!
//! ## Ciclo de vida del servicio
//! ```
//! SCM start → service_main() → loop { capture → store → notify? → sleep }
//!                                ↑
//!                          service_control_handler() recibe STOP/PAUSE/CONTINUE
//! ```
//!
//! ## Instalación (cuando esté implementado)
//! ```powershell
//! # Registrar
//! sc.exe create RootCause binPath= "C:\Program Files\RootCause\rootcause-service.exe"
//!   start= auto DisplayName= "RootCause Background Monitor"
//!
//! # Iniciar
//! sc.exe start RootCause
//!
//! # Detener y eliminar
//! sc.exe stop RootCause
//! sc.exe delete RootCause
//! ```

fn main() {
    eprintln!(
        "rootcause-service: este binario es un esqueleto de arquitectura.\n\
         La implementación del Windows Service requiere activar la feature `service`\n\
         y añadir la dependencia `windows-service = \"0.6\"` en Cargo.toml.\n\
         Ver src/bin/rootcause-service.rs para instrucciones detalladas."
    );
    std::process::exit(1);
}

// ── Implementación futura (windows-service) ──────────────────────────────────
//
// use windows_service::{
//     define_windows_service,
//     service::{
//         ServiceControl, ServiceControlAccept, ServiceExitCode,
//         ServiceState, ServiceStatus, ServiceType,
//     },
//     service_control_handler::{self, ServiceControlHandlerResult},
//     service_dispatcher,
// };
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use std::time::Duration;
//
// const SERVICE_NAME: &str = "RootCause";
//
// define_windows_service!(ffi_service_main, service_main);
//
// fn main() -> windows_service::Result<()> {
//     service_dispatcher::start(SERVICE_NAME, ffi_service_main)
// }
//
// fn service_main(_args: Vec<std::ffi::OsString>) {
//     let running = Arc::new(AtomicBool::new(true));
//     let running_clone = running.clone();
//
//     let status_handle = service_control_handler::register(
//         SERVICE_NAME,
//         move |ctrl| match ctrl {
//             ServiceControl::Stop => {
//                 running_clone.store(false, Ordering::SeqCst);
//                 ServiceControlHandlerResult::NoError
//             }
//             ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
//             _ => ServiceControlHandlerResult::NotImplemented,
//         },
//     ).unwrap();
//
//     status_handle.set_service_status(ServiceStatus {
//         service_type: ServiceType::OWN_PROCESS,
//         current_state: ServiceState::Running,
//         controls_accepted: ServiceControlAccept::STOP,
//         exit_code: ServiceExitCode::Win32(0),
//         checkpoint: 0,
//         wait_hint: Duration::default(),
//         process_id: None,
//     }).unwrap();
//
//     // Main loop
//     while running.load(Ordering::SeqCst) {
//         // capture_and_store();
//         std::thread::sleep(Duration::from_secs(30));
//     }
//
//     status_handle.set_service_status(ServiceStatus {
//         current_state: ServiceState::Stopped,
//         ..Default::default()
//     }).unwrap();
// }
