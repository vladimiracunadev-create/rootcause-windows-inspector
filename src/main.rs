//! Punto de entrada de la aplicación.
//!
//! Soporta dos modos de operación y dos ediciones de compilación:
//!
//! ## Modos de operación
//! * **GUI** (por defecto): `rootcause` o `rootcause --gui`
//! * **CLI**: `rootcause <comando>` — útil para scripts y automatización.
//!
//! ## Ediciones de compilación
//! * **Completa** (feature `gui`, por defecto): incluye egui + interfaz gráfica (~18 MB)
//! * **CLI-only** (`--no-default-features`): solo consola, sin egui (~4 MB)
//!
//! El modo CLI se despacha a `cli::run()` sin inicializar ningún contexto
//! gráfico, por lo que funciona en sesiones de consola sin pantalla y en
//! Windows Server Core.

#[cfg(feature = "gui")]
mod app;
mod cli;
mod config;
mod meta;
mod models;
mod services;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Si hay argumentos y el primero no es --gui, despachar al modo CLI.
    if args.len() > 1 && args[1] != "--gui" {
        std::process::exit(cli::run(&args[1..]));
    }

    // Modo GUI — solo disponible en la edición completa.
    #[cfg(feature = "gui")]
    {
        if let Err(e) = launch_gui() {
            eprintln!("Error al iniciar la interfaz gráfica: {e}");
            std::process::exit(1);
        }
    }

    // Edición CLI-only: si no hay argumentos, mostrar ayuda.
    #[cfg(not(feature = "gui"))]
    {
        std::process::exit(cli::run(&["--help".to_owned()]));
    }
}

#[cfg(feature = "gui")]
fn launch_gui() -> eframe::Result<()> {
    use app::RootCauseApp;
    use eframe::egui;

    // Dimensionar la ventana al área de trabajo real del monitor (pantalla menos la
    // barra de tareas). El flag `maximized` de eframe 0.27 no se honra de forma
    // fiable en este backend, pero `with_inner_size` sí; y el tamaño interno que
    // fija egui está en PUNTOS lógicos. Por eso consultamos el área de trabajo y la
    // convertimos a puntos con el DPI del sistema. Si la consulta falla, se usa un
    // tamaño de respaldo conservador que cabe incluso en portátiles 1366x768.
    let (win_w, win_h) = work_area_points().unwrap_or((1200.0, 700.0));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("RootCause — Windows Inspector")
            .with_icon(rootcause_icon())
            .with_position([0.0, 0.0])
            .with_inner_size([win_w, win_h])
            // Mínimo bajo para que quepa incluso en portátiles pequeños.
            .with_min_inner_size([760.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RootCause — Windows Inspector",
        native_options,
        Box::new(|cc| Box::new(RootCauseApp::new(cc))),
    )
}

/// Área de trabajo del monitor primario (pantalla menos la barra de tareas), en
/// PUNTOS lógicos listos para `with_inner_size`, dejando un margen para la barra
/// de título y los bordes.
///
/// Consulta `SPI_GETWORKAREA` (píxeles) y divide por la escala del sistema
/// (`GetDpiForSystem`/96). Ambas llamadas son coherentes entre sí respecto al
/// estado de conciencia de DPI del proceso, así que el cociente da puntos válidos
/// tanto a 100% como a 150%/200%. Devuelve `None` si la API falla.
#[cfg(all(feature = "gui", windows))]
fn work_area_points() -> Option<(f32, f32)> {
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    const SPI_GETWORKAREA: u32 = 0x0030;

    unsafe extern "system" {
        fn SystemParametersInfoW(
            action: u32,
            ui_param: u32,
            pv_param: *mut core::ffi::c_void,
            win_ini: u32,
        ) -> i32;
        fn GetDpiForSystem() -> u32;
    }

    let mut rect = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok =
        unsafe { SystemParametersInfoW(SPI_GETWORKAREA, 0, (&mut rect as *mut Rect).cast(), 0) };
    if ok == 0 {
        return None;
    }

    let dpi = unsafe { GetDpiForSystem() }.max(96);
    let scale = dpi as f32 / 96.0;
    let work_w = (rect.right - rect.left) as f32 / scale;
    let work_h = (rect.bottom - rect.top) as f32 / scale;

    // Restar el grosor de la barra de título y los bordes (aprox., en puntos) para
    // que la ventana ENTERA quepa en el área de trabajo con la esquina en (0,0).
    let inner_w = (work_w - 16.0).clamp(760.0, 3200.0);
    let inner_h = (work_h - 48.0).clamp(560.0, 2000.0);

    if inner_w > 200.0 && inner_h > 200.0 {
        Some((inner_w, inner_h))
    } else {
        None
    }
}

/// Respaldo para plataformas no-Windows (no aplica en producción, mantiene la
/// compilación cruzada sana).
#[cfg(all(feature = "gui", not(windows)))]
fn work_area_points() -> Option<(f32, f32)> {
    None
}

#[cfg(feature = "gui")]
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

#[cfg(feature = "gui")]
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
