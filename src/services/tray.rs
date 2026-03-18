//! Icono de bandeja del sistema (System Tray) — esqueleto de arquitectura.
//!
//! ## Estado actual
//! Este módulo es un esqueleto documentado. La implementación real requiere
//! activar la feature `tray` y añadir la dependencia `tray-icon = "0.14"` en
//! Cargo.toml. Se dejó como esqueleto para no aumentar el tamaño del binario
//! hasta que el feature esté listo para producción.
//!
//! ## Activación futura
//! ```toml
//! # Cargo.toml
//! [features]
//! tray = ["dep:tray-icon"]
//!
//! [dependencies]
//! tray-icon = { version = "0.14", optional = true }
//! ```
//!
//! ## Diseño de la bandeja
//! El icono de bandeja funciona en segundo plano con un loop de eventos propio
//! y expone un menú contextual con las acciones más usadas:
//!
//! ```
//! [RootCause — Normal]
//!   ├─ Abrir panel …
//!   ├─ Actualizar ahora
//!   ├─ Exportar snapshot
//!   ├─ ─────────────────
//!   ├─ Inicio automático  [✓]
//!   └─ Salir
//! ```
//!
//! El color del icono cambia según la severidad:
//! * Verde  → Normal / Low
//! * Amarillo → Medium
//! * Rojo   → High / Critical (+ notificación toast)

/// Configuración del icono de bandeja.
#[allow(dead_code)]
pub struct TrayConfig {
    /// Intervalo de actualización en segundos (mínimo 5).
    pub refresh_interval_secs: u64,
    /// Mostrar notificación toast al alcanzar estado Critical.
    pub alert_on_critical: bool,
}

impl Default for TrayConfig {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 30,
            alert_on_critical: true,
        }
    }
}

/// Acción solicitada desde el menú contextual de la bandeja.
#[allow(dead_code)]
pub enum TrayAction {
    OpenPanel,
    RefreshNow,
    ExportSnapshot,
    ToggleAutostart,
    Quit,
}

/// Lanza el icono de bandeja en un hilo dedicado.
///
/// # Errores
/// Devuelve `Err` si el sistema operativo rechaza la creación del icono
/// (p. ej. entorno sin escritorio, Windows Server Core sin GUI).
///
/// # Notas de implementación (pendiente)
/// ```rust,ignore
/// use tray_icon::{TrayIcon, TrayIconBuilder, menu::{Menu, MenuItem}};
///
/// pub fn spawn(config: TrayConfig) -> anyhow::Result<()> {
///     let menu = Menu::new();
///     menu.append(&MenuItem::new("Abrir panel", true, None))?;
///     menu.append(&MenuItem::new("Actualizar ahora", true, None))?;
///     menu.append(&MenuItem::new("Exportar snapshot", true, None))?;
///     menu.append(&PredefinedMenuItem::separator())?;
///     menu.append(&MenuItem::new("Salir", true, None))?;
///
///     let _tray = TrayIconBuilder::new()
///         .with_menu(Box::new(menu))
///         .with_tooltip("RootCause Windows Inspector")
///         .with_icon(load_icon_bytes(include_bytes!("../../icons/rootcause.png")))
///         .build()?;
///
///     // Event loop — debe correr en el hilo principal en Windows.
///     event_loop.run(move |event, _, control_flow| {
///         *control_flow = ControlFlow::Wait;
///         if let TrayIconEvent::MenuItemClicked(id) = event { … }
///     });
/// }
/// ```
#[allow(dead_code)]
pub fn spawn(_config: TrayConfig) -> anyhow::Result<()> {
    anyhow::bail!(
        "El icono de bandeja requiere activar la feature `tray` y la dependencia `tray-icon`. \
         Ver documentación del módulo para instrucciones de activación."
    )
}
