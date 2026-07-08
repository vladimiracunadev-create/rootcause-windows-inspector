//! Icono de bandeja del sistema (System Tray).
//!
//! Implementación real, activada por la edición GUI (feature `gui`, que arrastra
//! la crate `tray-icon`). En la edición CLI-only este módulo queda vacío.
//!
//! ## Diseño
//! El icono vive en el hilo del event-loop de eframe/winit (se crea en
//! `RootCauseApp::new`, que corre en ese hilo). winit bombea los mensajes de
//! Windows, así que los clics del menú llegan al canal global de `tray-icon`,
//! que la app drena cada frame con [`Tray::poll`].
//!
//! El icono es un punto de color según la salud global:
//! * Verde  → Saludable
//! * Ámbar  → Advertencia
//! * Rojo   → Crítico
//!
//! El tooltip muestra el veredicto actual. El menú contextual expone las acciones
//! más usadas. Cerrar a la bandeja (mantener el proceso vivo al cerrar la ventana)
//! queda para una iteración futura porque depende de APIs de viewport de egui que
//! conviene validar aparte.
#![cfg(feature = "gui")]

use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Acción solicitada desde el menú contextual de la bandeja.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    /// Mostrar y enfocar la ventana principal.
    Show,
    /// Forzar una captura ahora.
    Refresh,
    /// Exportar la última captura a JSON.
    Export,
    /// Salir de la aplicación.
    Quit,
}

/// Icono de bandeja activo. Mantener vivo mientras la app corre; al soltarse, el
/// icono desaparece de la bandeja.
pub struct Tray {
    icon: TrayIcon,
    id_show: MenuId,
    id_refresh: MenuId,
    id_export: MenuId,
    id_quit: MenuId,
    last_level: Option<u8>,
    last_label: String,
}

impl Tray {
    /// Crea el icono de bandeja. Devuelve `None` si el sistema lo rechaza (p. ej.
    /// sesión sin escritorio); la app debe seguir funcionando sin bandeja.
    pub fn new() -> Option<Self> {
        let show = MenuItem::new("Mostrar ventana", true, None);
        let refresh = MenuItem::new("Actualizar ahora", true, None);
        let export = MenuItem::new("Exportar snapshot", true, None);
        let quit = MenuItem::new("Salir", true, None);

        let menu = Menu::new();
        menu.append(&show).ok()?;
        menu.append(&refresh).ok()?;
        menu.append(&export).ok()?;
        menu.append(&PredefinedMenuItem::separator()).ok()?;
        menu.append(&quit).ok()?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("RootCause Windows Inspector")
            .with_icon(build_icon(level_color(0))?)
            .build()
            .ok()?;

        Some(Self {
            id_show: show.id().clone(),
            id_refresh: refresh.id().clone(),
            id_export: export.id().clone(),
            id_quit: quit.id().clone(),
            icon,
            last_level: None,
            last_label: String::new(),
        })
    }

    /// Drena los eventos de menú acumulados y devuelve la última acción pedida.
    pub fn poll(&self) -> Option<TrayAction> {
        let mut action = None;
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let next = if event.id == self.id_show {
                Some(TrayAction::Show)
            } else if event.id == self.id_refresh {
                Some(TrayAction::Refresh)
            } else if event.id == self.id_export {
                Some(TrayAction::Export)
            } else if event.id == self.id_quit {
                Some(TrayAction::Quit)
            } else {
                None
            };
            if next.is_some() {
                action = next;
            }
        }
        action
    }

    /// Ajusta el color del icono (por nivel de salud: 0=verde, 1=ámbar, 2=rojo) y
    /// el tooltip (por etiqueta de veredicto). Solo toca el SO cuando algo cambió.
    pub fn set_state(&mut self, level: u8, label: &str) {
        if self.last_level != Some(level) {
            self.last_level = Some(level);
            if let Some(icon) = build_icon(level_color(level)) {
                let _ = self.icon.set_icon(Some(icon));
            }
        }
        if self.last_label != label {
            self.last_label = label.to_owned();
            let _ = self.icon.set_tooltip(Some(format!("RootCause — {label}")));
        }
    }
}

/// Color RGB del punto según el nivel de salud.
fn level_color(level: u8) -> [u8; 3] {
    match level {
        0 => [63, 185, 80],  // verde — saludable
        1 => [210, 153, 34], // ámbar — advertencia
        _ => [248, 81, 73],  // rojo — crítico
    }
}

/// Construye un icono de 32×32 con un punto relleno del color dado sobre fondo
/// transparente. Devuelve `None` si `tray-icon` rechaza el buffer.
fn build_icon(rgb: [u8; 3]) -> Option<Icon> {
    let size: u32 = 32;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];
    let center = (size as f32 - 1.0) / 2.0;
    let radius = size as f32 / 2.0 - 1.5;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            if dx * dx + dy * dy <= radius * radius {
                let idx = ((y * size + x) * 4) as usize;
                rgba[idx] = rgb[0];
                rgba[idx + 1] = rgb[1];
                rgba[idx + 2] = rgb[2];
                rgba[idx + 3] = 255;
            }
        }
    }
    Icon::from_rgba(rgba, size, size).ok()
}
