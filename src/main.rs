//! Punto de entrada de la aplicación.
//!
//! Este binario crea una ventana de escritorio ligera usando `eframe/egui`.
//! Toda la lógica de diagnóstico vive en módulos separados para que la
//! mantención futura sea simple y trazable.

mod app;
mod models;
mod services;

use app::RootCauseApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RootCause — Windows Inspector")
            .with_icon(rootcause_icon())
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1100.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RootCause — Windows Inspector",
        native_options,
        Box::new(|cc| Box::new(RootCauseApp::new(cc))),
    )
}

/// Construye un icono simple `RC` sin depender de decodificadores externos.
///
/// Esto asegura una marca mínima visible incluso antes de integrar recursos
/// más elaborados. En Windows, el recurso `.ico` también se incrusta mediante
/// `build.rs` para que el ejecutable, los accesos directos y el instalador
/// puedan reutilizar la misma identidad visual.
fn rootcause_icon() -> egui::IconData {
    let width: u32 = 64;
    let height: u32 = 64;
    let mut rgba = vec![0_u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let border = x < 2 || y < 2 || x >= width - 2 || y >= height - 2;
            let diagonal_accent = x + y < 18 || x + y > 108;

            let (r, g, b, a) = if border {
                (155, 210, 255, 255)
            } else if diagonal_accent && x > width / 2 {
                (38, 150, 255, 255)
            } else {
                (24, 88, 201, 255)
            };

            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = a;
        }
    }

    // Letras RC en estilo de bloques simples para evitar dependencias de fuentes.
    draw_rect(&mut rgba, width, 10, 14, 8, 36, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 10, 14, 18, 8, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 10, 28, 18, 8, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 22, 24, 10, 8, [255, 255, 255, 255]);

    draw_rect(&mut rgba, width, 34, 14, 8, 36, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 34, 14, 18, 8, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 34, 42, 18, 8, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 46, 24, 8, 8, [255, 255, 255, 255]);
    draw_rect(&mut rgba, width, 46, 34, 8, 8, [255, 255, 255, 255]);

    egui::IconData {
        rgba,
        width,
        height,
    }
}

fn draw_rect(rgba: &mut [u8], width: u32, x0: u32, y0: u32, w: u32, h: u32, color: [u8; 4]) {
    for y in y0..(y0 + h) {
        for x in x0..(x0 + w) {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = color[0];
            rgba[idx + 1] = color[1];
            rgba[idx + 2] = color[2];
            rgba[idx + 3] = color[3];
        }
    }
}
