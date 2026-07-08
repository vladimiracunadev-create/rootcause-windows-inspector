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
mod i18n;
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
/// Construye el icono de la aplicación: un radar de círculos concéntricos en el
/// azul de marca (#1f6feb) sobre fondo oscuro (#0d1117) — la misma identidad del
/// `.ico` incrustado por `build.rs` y del favicon de la web. Se dibuja a mano
/// para no depender de decodificadores externos.
fn rootcause_icon() -> egui::IconData {
    let size: u32 = 64;
    let mut rgba = vec![0_u8; (size * size * 4) as usize];

    let center = (size as f32 - 1.0) / 2.0;
    let ring_w = 3.0_f32;
    let r_outer = 26.0_f32;
    let r_inner = 13.0_f32;
    let r_dot = 4.0_f32;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let on_ring = (dist - r_outer).abs() < ring_w || (dist - r_inner).abs() < ring_w;
            let on_dot = dist < r_dot;

            let (r, g, b) = if on_ring || on_dot {
                (31, 111, 235) // azul de marca #1f6feb
            } else {
                (13, 17, 23) // fondo #0d1117
            };

            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}
