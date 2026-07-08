//! Capa de interfaz — diseño con tabs, estilo PC Manager dark.
//!
//! Estructura: barra superior con logo + controles, barra de tabs horizontal,
//! cada tab dibuja su contenido con tablas, progress bars y tooltips para
//! nombres o rutas largas. Sin scroll horizontal.

use crate::config::RootCauseConfig;
use crate::i18n::{self, Lang, tr};
use crate::meta;
use crate::models::{
    AgentHealth, AgentStatus, AnomalyEvent, HardwareInfo, PersistenceChange, PersistenceEntry,
    ProcessInsight, RiskLevel, ServiceState, Severity, SnapshotRow, SystemSnapshot,
    TraceAnalysisSummary, TracePathSummary, TraceProcessSummary,
};
use crate::services::docker::{self, DockerScan};
use crate::services::inspector::InspectorService;
use crate::services::tray::{Tray, TrayAction};
use crate::services::windows;
use eframe::egui::{self, Color32, FontId, Margin, RichText, Rounding, Sense, Stroke, Vec2};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ── Muestra para sparklines ─────────────────────────────────────────────────────

struct MetricSample {
    cpu: f32,
    ram_pct: f32,
    io_write: f32,
}

// ── Paleta ─────────────────────────────────────────────────────────────────────

const BG_APP: Color32 = Color32::from_rgb(13, 17, 23);
const BG_PANEL: Color32 = Color32::from_rgb(22, 27, 34);
const BG_CARD: Color32 = Color32::from_rgb(30, 37, 46);
const BG_ROW_ALT: Color32 = Color32::from_rgb(18, 23, 30);
const BORDER: Color32 = Color32::from_rgb(48, 54, 61);
const ACCENT: Color32 = Color32::from_rgb(31, 111, 235);

const TEXT_PRI: Color32 = Color32::from_rgb(230, 237, 243);
const TEXT_SEC: Color32 = Color32::from_rgb(139, 148, 158);
const TEXT_MUT: Color32 = Color32::from_rgb(72, 80, 92);

const C_OK_FG: Color32 = Color32::from_rgb(63, 185, 80);
const C_OK_BG: Color32 = Color32::from_rgb(13, 43, 26);
const C_WN_FG: Color32 = Color32::from_rgb(210, 153, 34);
const C_WN_BG: Color32 = Color32::from_rgb(43, 29, 14);
const C_CR_FG: Color32 = Color32::from_rgb(248, 81, 73);
const C_CR_BG: Color32 = Color32::from_rgb(43, 14, 14);
const C_BL_FG: Color32 = Color32::from_rgb(88, 166, 255);
const C_BL_BG: Color32 = Color32::from_rgb(14, 34, 68);

// ── Servicios que el usuario puede detener desde la UI ─────────────────────────

const STOPPABLE_SERVICES: &[&str] = &["wuauserv", "bits", "dosvc", "sysmain"];

// ── Anchos de columna para tablas ──────────────────────────────────────────────

const W_NAME: f32 = 180.0;
const W_PID: f32 = 58.0;
const W_PCT: f32 = 46.0;
const W_BAR: f32 = 80.0;
const W_MB: f32 = 62.0;
const W_SCORE: f32 = 50.0;
const W_ACTION: f32 = 76.0;
const W_PROTO: f32 = 54.0;
const W_ADDR: f32 = 160.0;
const W_STATE: f32 = 74.0;

// ── Tabs ───────────────────────────────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy, Default)]
enum Tab {
    #[default]
    Overview,
    Processes,
    Connections,
    TempFiles,
    Precision,
    Services,
    Autostart,
    History,
    Config,
    Manual,
    About,
}

impl Tab {
    // Iconos: emoji estándar (cubiertos por la fuente NotoEmoji que egui empaqueta
    // por defecto) para garantizar que SIEMPRE rendericen. Los glifos geométricos
    // anteriores (◈ ▤ ◧ ◫) no estaban en la fuente y salían como "□" (tofu).
    //
    // Cada entrada lleva la etiqueta en español y en inglés; el idioma activo se
    // resuelve en el momento de dibujar con `tr`.
    const ALL: &'static [(Tab, &'static str, &'static str, &'static str)] = &[
        (Tab::Overview, "📊", "Resumen", "Overview"),
        (Tab::Processes, "⚙", "Procesos", "Processes"),
        (Tab::Connections, "🌐", "Conexiones", "Connections"),
        (Tab::TempFiles, "🗑", "Temporales", "Storage"),
        (Tab::Precision, "🎯", "ETW / WPR", "ETW / WPR"),
        (Tab::Services, "🔧", "Servicios", "Services"),
        (Tab::Autostart, "🚀", "Autostart", "Autostart"),
        (Tab::History, "🕒", "Historial", "History"),
        (Tab::Config, "⚙", "Configuración", "Settings"),
        (Tab::Manual, "📖", "Manual", "Manual"),
        (Tab::About, "ℹ", "Acerca", "About"),
    ];
}

// ── Acciones de precisión ──────────────────────────────────────────────────────

enum PrecisionAction {
    Start,
    Stop,
    Cancel,
    Analyze,
}

// ── Acciones de Docker (tab Temporales) ────────────────────────────────────────

/// Qué purga segura está en curso o pendiente de confirmar.
#[derive(Clone, Copy, PartialEq)]
enum DockerPruneKind {
    /// Imágenes colgantes (`<none>:<none>`).
    Images,
    /// Caché de build.
    Cache,
}

/// Acción solicitada por la UI de Docker en un frame.
enum DockerUiAction {
    /// (Re)escanear el uso de disco de Docker.
    Scan,
    /// Ejecutar una purga ya confirmada.
    Prune(DockerPruneKind),
}

// ── Estado ─────────────────────────────────────────────────────────────────────

pub struct RootCauseApp {
    inspector: Option<InspectorService>,
    snapshot: Option<SystemSnapshot>,
    last_refresh_at: Instant,
    refresh_interval_secs: u64,
    auto_refresh: bool,
    only_public_connections: bool,
    filter_text: String,
    precision_note: String,
    status_line: String,
    status_is_error: bool,
    active_tab: Tab,
    // Sparklines (punto 1)
    metric_history: VecDeque<MetricSample>,
    // Notificaciones toast (punto 7)
    notifications_enabled: bool,
    last_critical_notification: Instant,
    // Historial (puntos 3 y 8)
    history_rows: Vec<SnapshotRow>,
    history_last_load: Instant,
    history_compare_a: Option<usize>,
    history_compare_b: Option<usize>,
    history_filter: String,
    // Filtro por severidad en tab Procesos (punto 6)
    proc_severity_filter: Option<Severity>,
    // Información de hardware (recopilada una sola vez al iniciar)
    hardware_info: HardwareInfo,
    // Configuración operativa (snapshot al iniciar, para el panel Config)
    cached_config: RootCauseConfig,
    config_path: String,
    // Limpieza de %TEMP% (tab Temporales): confirmación de 2 pasos + resultado
    temp_clean_confirm: bool,
    temp_clean_result: Option<String>,
    // Docker (tab Temporales): último escaneo, confirmación de purga y resultado
    docker_scan: Option<DockerScan>,
    docker_prune_confirm: Option<DockerPruneKind>,
    docker_result: Option<String>,
    // Icono de bandeja del sistema (None si el SO lo rechaza o falla la creación)
    tray: Option<Tray>,
}

impl RootCauseApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        apply_theme(&cc.egui_ctx);
        let inspector = InspectorService::new();
        let mut app = Self {
            inspector: None,
            snapshot: None,
            last_refresh_at: Instant::now() - Duration::from_secs(60),
            refresh_interval_secs: 5,
            auto_refresh: true,
            only_public_connections: true,
            filter_text: String::new(),
            precision_note: "Disco lento durante actualización o proceso sospechoso".to_owned(),
            status_line: String::new(),
            status_is_error: false,
            active_tab: Tab::Overview,
            metric_history: VecDeque::with_capacity(62),
            notifications_enabled: true,
            last_critical_notification: Instant::now() - Duration::from_secs(300),
            history_rows: Vec::new(),
            history_last_load: Instant::now() - Duration::from_secs(300),
            history_compare_a: None,
            history_compare_b: None,
            history_filter: String::new(),
            proc_severity_filter: None,
            hardware_info: HardwareInfo::default(),
            cached_config: RootCauseConfig::default(),
            config_path: String::new(),
            temp_clean_confirm: false,
            temp_clean_result: None,
            docker_scan: None,
            docker_prune_confirm: None,
            docker_result: None,
            // El icono de bandeja se crea aquí, en el hilo del event-loop de winit
            // (necesario para que sus mensajes de Windows se bombeen). No fatal.
            tray: Tray::new(),
        };
        match inspector {
            Ok(svc) => {
                app.hardware_info = svc.get_hardware_info();
                app.status_line = svc.latest_history_line();
                app.refresh_interval_secs = svc.config().collection.refresh_interval_secs;
                app.notifications_enabled = svc.config().alerting.notify_on_critical;
                app.cached_config = svc.config().clone();
                // Aplicar el idioma guardado antes del primer frame.
                i18n::set_lang(app.cached_config.ui.language);
                app.config_path = svc.config_path().display().to_string();
                app.inspector = Some(svc);
            }
            Err(e) => {
                app.status_line = format!("Error inicializando el motor: {e}");
                app.status_is_error = true;
            }
        }
        app
    }

    fn refresh_now(&mut self) {
        let Some(insp) = self.inspector.as_mut() else {
            return;
        };
        match insp.collect_snapshot() {
            Ok(snap) => {
                self.status_line = format!(
                    "Captura {}  ·  {}",
                    snap.collected_at.format("%H:%M:%S"),
                    snap.overview.primary_reason
                );
                self.status_is_error = false;

                // Sparklines: acumular muestra (máx 60 puntos ≈ 5 min a 5 s)
                let ram_total = snap.overview.memory_total_gb.max(0.1);
                let sample = MetricSample {
                    cpu: snap.overview.cpu_usage_percent,
                    ram_pct: snap.overview.memory_used_gb / ram_total * 100.0,
                    io_write: snap.overview.io_write_mb_delta,
                };
                if self.metric_history.len() >= 60 {
                    self.metric_history.pop_front();
                }
                self.metric_history.push_back(sample);

                // Notificación toast si el equipo pasa a estado Crítico
                let notification_cooldown_secs = insp.config().alerting.notification_cooldown_secs;
                if self.notifications_enabled
                    && matches!(snap.overview.primary_severity, Severity::Critical)
                    && self.last_critical_notification.elapsed()
                        > Duration::from_secs(notification_cooldown_secs)
                {
                    windows::show_toast_notification(
                        "RootCause — Alerta Crítica",
                        &snap.overview.primary_reason,
                    );
                    self.last_critical_notification = Instant::now();
                }

                self.snapshot = Some(snap);
                self.last_refresh_at = Instant::now();
            }
            Err(e) => {
                self.status_line = format!("Error al capturar: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn load_history(&mut self) {
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        self.history_rows = insp.load_history(60);
        self.history_last_load = Instant::now();
    }

    fn export_snapshot(&mut self) {
        let Some(snap) = self.snapshot.as_ref() else {
            self.status_line = "Sin datos para exportar".into();
            return;
        };
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        match insp.export_snapshot(snap) {
            Ok(path) => {
                // También actualizar el backup JSON del historial como seguro de último recurso.
                let backup_note = insp
                    .export_history_backup()
                    .map(|p| format!("  ·  historial → {p}"))
                    .unwrap_or_default();
                self.status_line = format!("Exportado → {path}{backup_note}");
                self.status_is_error = false;
            }
            Err(e) => {
                self.status_line = format!("Error al exportar: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn start_precision_capture(&mut self) {
        let result = {
            let Some(i) = self.inspector.as_mut() else {
                return;
            };
            i.start_precision_capture(&self.precision_note)
        };
        self.apply_precision_result(result, "No se pudo iniciar WPR");
    }

    fn stop_precision_capture(&mut self) {
        let result = {
            let Some(i) = self.inspector.as_mut() else {
                return;
            };
            i.stop_precision_capture(&self.precision_note)
        };
        self.apply_precision_result(result, "No se pudo detener WPR");
    }

    fn cancel_precision_capture(&mut self) {
        let result = {
            let Some(i) = self.inspector.as_mut() else {
                return;
            };
            i.cancel_precision_capture()
        };
        self.apply_precision_result(result, "No se pudo cancelar WPR");
    }

    fn analyze_last_trace(&mut self) {
        let result = {
            let Some(i) = self.inspector.as_mut() else {
                return;
            };
            i.analyze_last_precision_trace()
        };
        self.apply_precision_result(result, "No se pudo resumir el ETL");
    }

    fn apply_precision_result(&mut self, result: anyhow::Result<String>, prefix: &str) {
        match result {
            Ok(msg) => {
                self.status_line = msg;
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(e) => {
                self.status_line = format!("{prefix}: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn terminate_process(&mut self, pid: u32) {
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        match insp.terminate_process(pid) {
            Ok(msg) => {
                self.status_line = format!("Proceso finalizado  ·  {msg}");
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(e) => {
                self.status_line = format!("No se pudo finalizar PID {pid}: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn block_remote_ip(&mut self, ip: &str) {
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        match insp.block_remote_ip(ip) {
            Ok(msg) => {
                self.status_line = msg;
                self.status_is_error = false;
            }
            Err(e) => {
                self.status_line = format!("No se pudo bloquear: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn stop_service(&mut self, name: &str) {
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        match insp.stop_service(name) {
            Ok(msg) => {
                self.status_line = msg;
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(e) => {
                self.status_line = format!("No se pudo detener {name}: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn accept_persistence_baseline(&mut self) {
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        match insp.accept_persistence_baseline() {
            Ok(count) => {
                self.status_line = format!(
                    "Baseline de autoarranque actualizada ({count} entradas). \
                     Los cambios previos ya no se marcarán."
                );
                self.status_is_error = false;
                // Fuerza un refresco para re-evaluar contra la nueva baseline.
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(e) => {
                self.status_line = format!("No se pudo aceptar la baseline: {e}");
                self.status_is_error = true;
            }
        }
    }

    fn execute_temp_clean(&mut self) {
        self.temp_clean_confirm = false;
        let Some(insp) = self.inspector.as_ref() else {
            return;
        };
        let r = insp.clean_temp(false);
        self.temp_clean_result = Some(format!(
            "Limpieza: {} borradas · {:.1} MB liberados · {} en uso (saltadas) · {} recientes (saltadas)",
            r.deleted_count, r.freed_mb, r.skipped_in_use, r.skipped_recent
        ));
        self.status_line = format!("%TEMP% limpiado: {:.1} MB liberados", r.freed_mb);
        self.status_is_error = false;
        // Forzar re-escaneo para que la tabla de temporales refleje el cambio.
        self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
    }

    /// Ejecuta una acción de Docker (escaneo o purga) de forma síncrona. Docker
    /// puede tardar 1–2 s; se hace bajo demanda (pulsando un botón), no en el
    /// bucle de refresco, así que el bloqueo puntual es aceptable.
    fn execute_docker_action(&mut self, action: DockerUiAction) {
        match action {
            DockerUiAction::Scan => {
                self.docker_result = None;
                self.docker_prune_confirm = None;
                self.docker_scan = Some(docker::scan());
            }
            DockerUiAction::Prune(kind) => {
                self.docker_prune_confirm = None;
                let outcome = match kind {
                    DockerPruneKind::Images => docker::prune_dangling_images(),
                    DockerPruneKind::Cache => docker::prune_build_cache(),
                };
                match outcome {
                    Ok(msg) => {
                        self.docker_result = Some(format!("✅  {msg}"));
                        self.status_line = format!("Docker: {msg}");
                        self.status_is_error = false;
                    }
                    Err(e) => {
                        self.docker_result = Some(format!("❌  {e}"));
                        self.status_line = format!("Docker: {e}");
                        self.status_is_error = true;
                    }
                }
                // Re-escanear para reflejar el espacio liberado.
                self.docker_scan = Some(docker::scan());
            }
        }
    }
}

// ── Loop principal ─────────────────────────────────────────────────────────────

impl eframe::App for RootCauseApp {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(BG_APP).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // El tamaño de la ventana se fija al crearla desde el área de trabajo real
        // del monitor (ver `work_area_points` en main.rs). No hace falta ajustar
        // nada en tiempo de ejecución.
        ctx.request_repaint_after(Duration::from_secs(1));

        // ── Atajos de teclado ──────────────────────────────────────────────────
        // Se recogen en variables locales primero para evitar conflictos de borrow.
        let mut should_refresh = false;
        let mut should_export = false;
        let mut tab_switch: Option<usize> = None;

        ctx.input(|i| {
            // F5 → Actualizar ahora
            if i.key_pressed(egui::Key::F5) {
                should_refresh = true;
            }
            // Ctrl+E → Exportar JSON
            if i.key_pressed(egui::Key::E) && i.modifiers.ctrl {
                should_export = true;
            }
            // Ctrl+1..9 → Cambiar de tab
            for (key, idx) in [
                (egui::Key::Num1, 0usize),
                (egui::Key::Num2, 1),
                (egui::Key::Num3, 2),
                (egui::Key::Num4, 3),
                (egui::Key::Num5, 4),
                (egui::Key::Num6, 5),
                (egui::Key::Num7, 6),
                (egui::Key::Num8, 7),
                (egui::Key::Num9, 8),
                (egui::Key::Num0, 9),
            ] {
                if i.key_pressed(key) && i.modifiers.ctrl {
                    tab_switch = Some(idx);
                }
            }
        });

        if should_refresh {
            self.refresh_now();
        }
        if should_export {
            self.export_snapshot();
        }
        if let Some(idx) = tab_switch
            && let Some(&(tab, _, _, _)) = Tab::ALL.get(idx)
        {
            self.active_tab = tab;
        }

        if self.snapshot.is_none()
            || (self.auto_refresh
                && self.last_refresh_at.elapsed()
                    >= Duration::from_secs(self.refresh_interval_secs))
        {
            self.refresh_now();
        }

        // Recargar historial cuando el tab está activo y los datos son viejos (> 30 s)
        if self.active_tab == Tab::History
            && self.history_last_load.elapsed() > Duration::from_secs(30)
        {
            self.load_history();
        }

        // ── Icono de bandeja: acciones del menú + color/tooltip por salud ──────
        // Se captura la acción antes de actuar para no chocar con el préstamo &mut.
        let tray_action = self.tray.as_ref().and_then(Tray::poll);
        match tray_action {
            Some(TrayAction::Show) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            Some(TrayAction::Refresh) => self.refresh_now(),
            Some(TrayAction::Export) => self.export_snapshot(),
            Some(TrayAction::Quit) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            None => {}
        }
        if self.tray.is_some() {
            // Nivel + etiqueta a partir de la salud global (mismo criterio que el
            // banner de veredicto del Resumen). Se calcula sin retener el préstamo
            // de `snapshot` para poder tomar `tray` como &mut después.
            let state = self.snapshot.as_ref().map(|snap| {
                let score = compute_health_score(snap);
                let (level, word) = if score >= 80 {
                    (0_u8, tr("Saludable", "Healthy"))
                } else if score >= 50 {
                    (1, tr("Advertencia", "Warning"))
                } else {
                    (2, tr("Crítico", "Critical"))
                };
                (level, format!("{word} · {score}/100"))
            });
            if let (Some((level, label)), Some(tray)) = (state, self.tray.as_mut()) {
                tray.set_state(level, &label);
            }
        }

        // Orden: barra lateral (izquierda, altura completa) primero, luego la
        // topbar sobre el contenido, la barra de estado abajo y el panel central.
        draw_sidebar(self, ctx);
        draw_topbar(self, ctx);
        draw_statusbar(self, ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG_APP)
                    .inner_margin(Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                // El tab Manual es contenido estático — no necesita snapshot.
                if self.active_tab == Tab::Manual {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, draw_tab_manual);
                    return;
                }
                // El tab Configuración no necesita snapshot — edita idioma y umbrales.
                if self.active_tab == Tab::Config {
                    let mut save_config = false;
                    let lang_before = self.cached_config.ui.language;
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            draw_tab_config(
                                ui,
                                &mut self.cached_config,
                                &self.config_path,
                                &mut save_config,
                            )
                        });
                    // Cambiar el idioma aplica al instante y persiste sin pulsar Guardar.
                    if self.cached_config.ui.language != lang_before {
                        i18n::set_lang(self.cached_config.ui.language);
                        save_config = true;
                    }
                    if save_config && let Some(svc) = self.inspector.as_mut() {
                        match svc.save_config(&self.cached_config) {
                            Ok(()) => {
                                self.status_line =
                                    tr("Configuración guardada correctamente.", "Settings saved.")
                                        .to_owned();
                                self.status_is_error = false;
                            }
                            Err(e) => {
                                self.status_line = format!(
                                    "{}: {e}",
                                    tr("Error al guardar config", "Failed to save settings")
                                );
                                self.status_is_error = true;
                            }
                        }
                    }
                    return;
                }
                // El tab Acerca no necesita snapshot — se muestra siempre.
                if self.active_tab == Tab::About {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            draw_tab_about(ui, &self.hardware_info, self.snapshot.as_ref())
                        });
                    return;
                }

                let Some(snapshot) = self.snapshot.clone() else {
                    loading_screen(ui);
                    return;
                };

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut precision_action: Option<PrecisionAction> = None;
                        let mut pid_to_kill: Option<u32> = None;
                        let mut ip_to_block: Option<String> = None;
                        let mut svc_to_stop: Option<String> = None;
                        let mut accept_baseline = false;
                        // Option<Option<Severity>>: outer=changed, inner=new value
                        let mut sev_filter_change: Option<Option<Severity>> = None;

                        match self.active_tab {
                            Tab::Overview => {
                                draw_tab_overview(
                                    ui,
                                    &snapshot,
                                    &self.metric_history,
                                    &self.hardware_info,
                                );
                            }
                            Tab::Processes => draw_tab_processes(
                                ui,
                                &snapshot,
                                &self.filter_text,
                                self.proc_severity_filter,
                                |pid| pid_to_kill = Some(pid),
                                |sev| sev_filter_change = Some(sev),
                            ),
                            Tab::Connections => draw_tab_connections(
                                ui,
                                &snapshot,
                                &self.filter_text,
                                &mut self.only_public_connections,
                                |ip| ip_to_block = Some(ip.to_owned()),
                            ),
                            Tab::TempFiles => {
                                let mut do_clean = false;
                                let mut docker_action: Option<DockerUiAction> = None;
                                draw_tab_temp(
                                    ui,
                                    &snapshot,
                                    &self.filter_text,
                                    &mut self.temp_clean_confirm,
                                    &self.temp_clean_result,
                                    &mut do_clean,
                                    &self.docker_scan,
                                    &mut self.docker_prune_confirm,
                                    &self.docker_result,
                                    &mut docker_action,
                                );
                                if do_clean {
                                    self.execute_temp_clean();
                                }
                                if let Some(action) = docker_action {
                                    self.execute_docker_action(action);
                                }
                            }
                            Tab::Precision => draw_tab_precision(
                                ui,
                                &snapshot,
                                &mut self.precision_note,
                                &mut precision_action,
                            ),
                            Tab::Services => draw_tab_services(ui, &snapshot, |svc| {
                                svc_to_stop = Some(svc.to_owned())
                            }),
                            Tab::Autostart => {
                                draw_tab_autostart(
                                    ui,
                                    &snapshot,
                                    &self.filter_text,
                                    &mut accept_baseline,
                                );
                            }
                            Tab::History => draw_tab_history(
                                ui,
                                &self.history_rows,
                                &mut self.history_filter,
                                &mut self.history_compare_a,
                                &mut self.history_compare_b,
                            ),
                            // Config, Manual y About se gestionan antes del guard de snapshot.
                            Tab::Config | Tab::Manual | Tab::About => {}
                        }

                        match precision_action {
                            Some(PrecisionAction::Start) => self.start_precision_capture(),
                            Some(PrecisionAction::Stop) => self.stop_precision_capture(),
                            Some(PrecisionAction::Cancel) => self.cancel_precision_capture(),
                            Some(PrecisionAction::Analyze) => self.analyze_last_trace(),
                            None => {}
                        }
                        if let Some(pid) = pid_to_kill {
                            self.terminate_process(pid);
                        }
                        if let Some(ip) = ip_to_block {
                            self.block_remote_ip(&ip);
                        }
                        if let Some(svc) = svc_to_stop {
                            self.stop_service(&svc);
                        }
                        if accept_baseline {
                            self.accept_persistence_baseline();
                        }
                        if let Some(new_sev) = sev_filter_change {
                            self.proc_severity_filter = new_sev;
                        }
                    });
            });
    }
}

// ── Header ─────────────────────────────────────────────────────────────────────

/// Devuelve (icono, etiqueta_es, etiqueta_en) de una pestaña buscándola en `Tab::ALL`.
fn tab_meta(tab: Tab) -> (&'static str, &'static str, &'static str) {
    Tab::ALL
        .iter()
        .find(|(t, _, _, _)| *t == tab)
        .map(|&(_, icon, es, en)| (icon, es, en))
        .unwrap_or(("", "", ""))
}

/// Fila de navegación de la barra lateral: icono + etiqueta, alineados a la
/// izquierda, con resaltado y barra de acento cuando está activa.
fn sidebar_item(ui: &mut egui::Ui, icon: &str, label: &str, active: bool) -> egui::Response {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::click());
    let bg = if active {
        BG_CARD
    } else if resp.hovered() {
        Color32::from_rgb(26, 33, 44)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        ui.painter().rect_filled(rect, Rounding::same(5.0), bg);
    }
    if active {
        let bar = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 1.0, rect.center().y - 9.0),
            Vec2::new(3.0, 18.0),
        );
        ui.painter().rect_filled(bar, Rounding::same(2.0), ACCENT);
    }
    let fg = if active { TEXT_PRI } else { TEXT_SEC };
    ui.painter().text(
        egui::pos2(rect.left() + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        icon,
        FontId::proportional(15.0),
        if active { C_BL_FG } else { TEXT_SEC },
    );
    ui.painter().text(
        egui::pos2(rect.left() + 42.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(13.5),
        fg,
    );
    resp
}

/// Encabezado de grupo dentro de la barra lateral.
fn sidebar_group(ui: &mut egui::Ui, label: &str) {
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.label(RichText::new(label).size(10.5).color(TEXT_MUT).strong());
    });
    ui.add_space(2.0);
}

/// Dibuja una fila de navegación y cambia de pestaña si se pulsa.
fn nav(ui: &mut egui::Ui, app: &mut RootCauseApp, tab: Tab) {
    let (icon, es, en) = tab_meta(tab);
    if sidebar_item(ui, icon, tr(es, en), app.active_tab == tab).clicked() {
        app.active_tab = tab;
    }
}

// ── Barra lateral de navegación (NavigationView estilo Windows 11) ──────────────

fn draw_sidebar(app: &mut RootCauseApp, ctx: &egui::Context) {
    egui::SidePanel::left("nav")
        .exact_width(232.0)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(10.0, 12.0)),
        )
        .show(ctx, |ui| {
            // Marca
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                draw_logo_icon(ui, 26.0);
                ui.add_space(9.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("RootCause")
                            .size(15.0)
                            .strong()
                            .color(TEXT_PRI),
                    );
                    ui.label(
                        RichText::new("Windows Inspector")
                            .size(10.5)
                            .color(TEXT_MUT),
                    );
                });
            });
            ui.add_space(14.0);

            // Navegación superior
            nav(ui, app, Tab::Overview);
            sidebar_group(ui, tr("ACTIVIDAD", "ACTIVITY"));
            nav(ui, app, Tab::Processes);
            nav(ui, app, Tab::Connections);
            sidebar_group(ui, tr("SISTEMA", "SYSTEM"));
            nav(ui, app, Tab::TempFiles);
            nav(ui, app, Tab::Services);
            nav(ui, app, Tab::Autostart);
            sidebar_group(ui, tr("ANÁLISIS", "ANALYSIS"));
            nav(ui, app, Tab::Precision);
            nav(ui, app, Tab::History);

            // Elementos inferiores anclados abajo
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                nav(ui, app, Tab::About);
                nav(ui, app, Tab::Manual);
                nav(ui, app, Tab::Config);
            });
        });
}

// ── Barra superior (título de la vista + controles) ─────────────────────────────

fn draw_topbar(app: &mut RootCauseApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .frame(
            egui::Frame::none()
                .fill(BG_APP)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(20.0, 12.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                // Título de la vista activa
                let (_, es, en) = tab_meta(app.active_tab);
                ui.label(
                    RichText::new(tr(es, en))
                        .size(18.0)
                        .strong()
                        .color(TEXT_PRI),
                );

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(10.0);

                if header_btn(ui, "🔄", "Actualizar").clicked() {
                    app.refresh_now();
                }
                if header_btn(ui, "💾", "Exportar JSON").clicked() {
                    app.export_snapshot();
                }

                ui.add_space(10.0);
                ui.checkbox(&mut app.auto_refresh, RichText::new("Auto").color(TEXT_SEC));
                ui.add(
                    egui::Slider::new(&mut app.refresh_interval_secs, 3..=30)
                        .text(RichText::new("s").color(TEXT_MUT))
                        .clamp_to_range(true),
                );

                ui.add_space(8.0);
                ui.checkbox(
                    &mut app.notifications_enabled,
                    RichText::new("🔔").color(TEXT_SEC),
                )
                .on_hover_text("Activar notificaciones toast cuando el estado sea Crítico");

                ui.add_space(8.0);
                draw_search_icon(ui, 14.0);
                ui.add_space(4.0);
                ui.add_sized(
                    [190.0, 26.0],
                    egui::TextEdit::singleline(&mut app.filter_text)
                        .hint_text(tr("Filtrar por nombre o ruta…", "Filter by name or path…"))
                        .text_color(TEXT_PRI),
                );

                if let Some(snap) = &app.snapshot {
                    let crit = snap
                        .alerts
                        .iter()
                        .filter(|a| matches!(a.severity, Severity::Critical))
                        .count();
                    let warn = snap
                        .alerts
                        .iter()
                        .filter(|a| matches!(a.severity, Severity::Warning))
                        .count();
                    ui.add_space(12.0);
                    if crit > 0 {
                        alert_badge(
                            ui,
                            &format!("{crit} crítica{}", if crit != 1 { "s" } else { "" }),
                            C_CR_FG,
                            C_CR_BG,
                        );
                    }
                    if warn > 0 {
                        alert_badge(
                            ui,
                            &format!("{warn} aviso{}", if warn != 1 { "s" } else { "" }),
                            C_WN_FG,
                            C_WN_BG,
                        );
                    }
                }
            });
        });
}

// ── Barra de estado ────────────────────────────────────────────────────────────

fn draw_statusbar(app: &RootCauseApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("statusbar")
        .frame(
            egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(16.0, 5.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (dot, txt) = if app.status_is_error {
                    (C_CR_FG, C_CR_FG)
                } else {
                    (C_OK_FG, TEXT_SEC)
                };
                ui.label(RichText::new("•").color(dot).size(14.0));
                ui.label(RichText::new(&app.status_line).size(11.5).color(txt));
            });
        });
}

// ── Tab: Resumen ───────────────────────────────────────────────────────────────

fn draw_tab_overview(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    history: &VecDeque<MetricSample>,
    hw: &HardwareInfo,
) {
    let ov = &snap.overview;
    let content_width = ui.available_width().max(320.0);
    let stacked_summary = content_width < 1120.0;
    let narrow_summary = content_width < 920.0;
    let full_width_card = (content_width - 6.0).max(220.0);
    let score_card_width = if stacked_summary {
        full_width_card
    } else {
        140.0
    };
    let metric_card_width = if stacked_summary {
        full_width_card
    } else {
        170.0
    };
    let anomaly_card_width = if stacked_summary {
        full_width_card
    } else {
        260.0
    };
    let process_card_width = if stacked_summary {
        full_width_card
    } else {
        220.0
    };
    let sparkline_width = if stacked_summary {
        full_width_card
    } else {
        200.0
    };
    let score = compute_health_score(snap);
    let (score_fg, score_bg, score_label) = if score >= 80 {
        (C_OK_FG, C_OK_BG, tr("Saludable", "Healthy"))
    } else if score >= 50 {
        (C_WN_FG, C_WN_BG, tr("Advertencia", "Warning"))
    } else {
        (C_CR_FG, C_CR_BG, tr("Crítico", "Critical"))
    };

    // ── Banner de veredicto (titular) ─────────────────────────────────────────
    // Un vistazo debe bastar para saber el estado global, al estilo de una tarjeta
    // héroe: aro de salud + titular grande + la causa dominante en una línea.
    let headline = if score >= 80 {
        tr("Tu PC está saludable", "Your PC is healthy")
    } else if score >= 50 {
        tr(
            "Hay señales que conviene revisar",
            "Some signals worth reviewing",
        )
    } else {
        tr("Atención: revísalo ahora", "Attention: review it now")
    };
    egui::Frame::none()
        .fill(score_bg)
        .stroke(Stroke::new(1.5, score_fg.linear_multiply(0.6)))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::symmetric(18.0, 16.0))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_max_width(full_width_card);
                ui.horizontal(|ui| {
                    draw_health_ring(ui, score as f32 / 100.0, score_fg, 48.0);
                    ui.add_space(14.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(headline).size(20.0).strong().color(score_fg));
                            ui.add_space(8.0);
                            pill(ui, score_label, score_fg, BG_CARD);
                        });
                        ui.add_space(4.0);
                        ui.add(
                            egui::Label::new(
                                RichText::new(&ov.primary_reason).size(12.5).color(TEXT_SEC),
                            )
                            .wrap(true),
                        );
                    });
                });
            });
        });
    ui.add_space(12.0);

    // ── Fila 1: score + cards de métricas ─────────────────────────────────────
    let ram_pct = ov.memory_used_gb / ov.memory_total_gb.max(0.1) * 100.0;
    let net = ov.network_rx_mb_delta + ov.network_tx_mb_delta;

    if stacked_summary {
        health_score_card(
            ui,
            score as f32 / 100.0,
            score_label,
            score_fg,
            score_bg,
            score_card_width,
        );
        ui.add_space(8.0);
        overview_card(
            ui,
            "CPU",
            &format!("{:.1}%", ov.cpu_usage_percent),
            "Uso global del procesador",
            ov.cpu_usage_percent / 100.0,
            severity_for_value(ov.cpu_usage_percent, 55.0, 80.0),
            metric_card_width,
        );
        ui.add_space(8.0);
        overview_card(
            ui,
            "RAM",
            &format!("{:.1} / {:.1} GB", ov.memory_used_gb, ov.memory_total_gb),
            &format!("{ram_pct:.0}% utilizado"),
            ram_pct / 100.0,
            severity_for_value(ram_pct, 70.0, 88.0),
            metric_card_width,
        );
        ui.add_space(8.0);
        overview_card(
            ui,
            "DISCO  I/O",
            &format!(
                "W {:.1}  R {:.1} MB",
                ov.io_write_mb_delta, ov.io_read_mb_delta
            ),
            "Suma de procesos en el intervalo",
            ov.io_write_mb_delta / 220.0,
            severity_for_value(ov.io_write_mb_delta, 80.0, 220.0),
            metric_card_width,
        );
        ui.add_space(8.0);
        overview_card(
            ui,
            "RED",
            &format!(
                "Rx {:.1}  Tx {:.1} MB",
                ov.network_rx_mb_delta, ov.network_tx_mb_delta
            ),
            "Actividad entre refrescos",
            net / 80.0,
            severity_for_value(net, 15.0, 80.0),
            metric_card_width,
        );
        ui.add_space(8.0);
        overview_card(
            ui,
            "TEMP",
            &format!("{:.0} MB", ov.temp_total_mb),
            "TEMP / cachés vigiladas",
            ov.temp_total_mb / 2000.0,
            severity_for_value(ov.temp_total_mb, 700.0, 2000.0),
            metric_card_width,
        );
    } else {
        ui.horizontal_wrapped(|ui| {
            health_score_card(
                ui,
                score as f32 / 100.0,
                score_label,
                score_fg,
                score_bg,
                score_card_width,
            );
            ui.add_space(4.0);
            overview_card(
                ui,
                "CPU",
                &format!("{:.1}%", ov.cpu_usage_percent),
                "Uso global del procesador",
                ov.cpu_usage_percent / 100.0,
                severity_for_value(ov.cpu_usage_percent, 55.0, 80.0),
                metric_card_width,
            );
            overview_card(
                ui,
                "RAM",
                &format!("{:.1} / {:.1} GB", ov.memory_used_gb, ov.memory_total_gb),
                &format!("{ram_pct:.0}% utilizado"),
                ram_pct / 100.0,
                severity_for_value(ram_pct, 70.0, 88.0),
                metric_card_width,
            );
            overview_card(
                ui,
                "DISCO  I/O",
                &format!(
                    "W {:.1}  R {:.1} MB",
                    ov.io_write_mb_delta, ov.io_read_mb_delta
                ),
                "Suma de procesos en el intervalo",
                ov.io_write_mb_delta / 220.0,
                severity_for_value(ov.io_write_mb_delta, 80.0, 220.0),
                metric_card_width,
            );
            overview_card(
                ui,
                "RED",
                &format!(
                    "Rx {:.1}  Tx {:.1} MB",
                    ov.network_rx_mb_delta, ov.network_tx_mb_delta
                ),
                "Actividad entre refrescos",
                net / 80.0,
                severity_for_value(net, 15.0, 80.0),
                metric_card_width,
            );
            overview_card(
                ui,
                "TEMP",
                &format!("{:.0} MB", ov.temp_total_mb),
                "TEMP / cachés vigiladas",
                ov.temp_total_mb / 2000.0,
                severity_for_value(ov.temp_total_mb, 700.0, 2000.0),
                metric_card_width,
            );
        });
    }

    // ── Alertas ───────────────────────────────────────────────────────────────
    if !snap.alerts.is_empty() {
        ui.add_space(18.0);
        section_header(ui, "▸  Dónde mirar primero");
        ui.add_space(8.0);
        for alert in snap.alerts.iter().take(6) {
            let fg = sev_fg(alert.severity);
            let bg = sev_bg(alert.severity);
            egui::Frame::none()
                .fill(bg)
                .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        draw_sev_icon(ui, alert.severity, 16.0);
                        ui.add_space(4.0);
                        ui.label(RichText::new(&alert.title).strong().color(fg).size(13.5));
                        if let Some(pid) = alert.pid {
                            pill(ui, &format!("PID {pid}"), TEXT_MUT, BG_CARD);
                        }
                    });
                    ui.add_space(2.0);
                    ui.label(RichText::new(&alert.detail).color(TEXT_SEC));
                    ui.label(
                        RichText::new(&alert.hint)
                            .italics()
                            .color(TEXT_MUT)
                            .size(11.5),
                    );
                    if let Some(path) = &alert.path {
                        ui.add(
                            egui::Label::new(
                                RichText::new(path).small().monospace().color(TEXT_MUT),
                            )
                            .wrap(true),
                        );
                    }
                });
            ui.add_space(4.0);
        }
    }

    // ── Top 3 procesos críticos (vista rápida) ─────────────────────────────────
    if let Some(incident) = snap.incident.as_ref() {
        ui.add_space(18.0);
        section_header(ui, "Riesgo y causa raiz");
        ui.add_space(8.0);
        let incident_sev = incident
            .risk_level
            .map(|risk| risk.to_severity())
            .unwrap_or(incident.severity);
        let fg = sev_fg(incident_sev);
        let bg = sev_bg(incident_sev);
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(14.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    draw_sev_icon(ui, incident_sev, 18.0);
                    ui.label(RichText::new(&incident.title).strong().color(fg).size(14.0));
                    if let Some(risk) = incident.risk_level {
                        alert_badge(ui, risk.label(), fg, BG_CARD);
                    }
                    if incident.risk_score > 0 {
                        pill(
                            ui,
                            &format!("Score {}", incident.risk_score),
                            TEXT_PRI,
                            BG_CARD,
                        );
                    }
                    if incident.anomaly_count > 0 {
                        pill(
                            ui,
                            &format!(
                                "{} anomalia{}",
                                incident.anomaly_count,
                                if incident.anomaly_count == 1 { "" } else { "s" }
                            ),
                            TEXT_MUT,
                            BG_CARD,
                        );
                    }
                });
                ui.add_space(6.0);
                ui.add(
                    egui::Label::new(RichText::new(&incident.summary).color(TEXT_SEC)).wrap(true),
                );
                if !incident.root_cause_hypothesis.is_empty() {
                    ui.add_space(4.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("Hipotesis: {}", incident.root_cause_hypothesis))
                                .color(TEXT_PRI)
                                .size(12.0),
                        )
                        .wrap(true),
                    );
                }
                if let Some(event) = incident.anomaly_events.first() {
                    ui.add_space(6.0);
                    if narrow_summary {
                        ui.vertical(|ui| {
                            if let Some(name) = event.process_name.as_ref() {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(format!(
                                            "Proceso: {}{}",
                                            name,
                                            event
                                                .pid
                                                .map(|pid| format!(" (PID {pid})"))
                                                .unwrap_or_default()
                                        ))
                                        .monospace()
                                        .color(TEXT_SEC),
                                    )
                                    .wrap(true),
                                );
                            }
                            if let Some(path) = event.exe_path.as_ref() {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(path).small().monospace().color(TEXT_MUT),
                                    )
                                    .wrap(true),
                                )
                                .on_hover_text(path);
                            }
                        });
                    } else {
                        ui.horizontal_wrapped(|ui| {
                            if let Some(name) = event.process_name.as_ref() {
                                ui.label(
                                    RichText::new(format!(
                                        "Proceso: {}{}",
                                        name,
                                        event
                                            .pid
                                            .map(|pid| format!(" (PID {pid})"))
                                            .unwrap_or_default()
                                    ))
                                    .monospace()
                                    .color(TEXT_SEC),
                                );
                            }
                            if let Some(path) = event.exe_path.as_ref() {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(trunc(path, 72))
                                            .small()
                                            .monospace()
                                            .color(TEXT_MUT),
                                    )
                                    .wrap(true),
                                )
                                .on_hover_text(path);
                            }
                        });
                    }
                }
                if !incident.recommended_actions.is_empty() {
                    ui.add_space(6.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "Sugerencia: {}",
                                incident.recommended_actions[0]
                            ))
                            .italics()
                            .color(TEXT_MUT),
                        )
                        .wrap(true),
                    );
                }
                if !incident.evidence.is_empty() {
                    ui.add_space(8.0);
                    for item in incident.evidence.iter().take(3) {
                        ui.add(
                            egui::Label::new(
                                RichText::new(format!(
                                    "{}: {}",
                                    item.label,
                                    trunc(&item.value, if narrow_summary { 120 } else { 80 })
                                ))
                                .small()
                                .color(TEXT_MUT),
                            )
                            .wrap(true),
                        );
                    }
                }
            });
    }

    if !snap.anomalies.is_empty() {
        ui.add_space(18.0);
        section_header(ui, "Anomalias destacadas");
        ui.add_space(8.0);
        if stacked_summary {
            for anomaly in snap.anomalies.iter().take(3) {
                anomaly_summary_card(ui, anomaly, anomaly_card_width);
                ui.add_space(8.0);
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                for anomaly in snap.anomalies.iter().take(3) {
                    anomaly_summary_card(ui, anomaly, anomaly_card_width);
                    ui.add_space(8.0);
                }
            });
        }
    }

    let top_procs: Vec<&ProcessInsight> = snap
        .processes
        .iter()
        .filter(|p| matches!(p.severity, Severity::Critical | Severity::Warning))
        .take(3)
        .collect();

    if !top_procs.is_empty() {
        ui.add_space(18.0);
        section_header(ui, "▸  Procesos que más impactan");
        ui.add_space(8.0);
        if stacked_summary {
            for p in top_procs {
                mini_process_card(ui, p, process_card_width);
                ui.add_space(8.0);
            }
        } else {
            ui.horizontal_wrapped(|ui| {
                for p in top_procs {
                    mini_process_card(ui, p, process_card_width);
                }
            });
        }
    }

    // ── Sparklines de tendencia ───────────────────────────────────────────────
    if history.len() >= 2 {
        ui.add_space(18.0);
        section_header(ui, "▸  Tendencia (últimas muestras)");
        ui.add_space(8.0);
        let cpu_vals: Vec<f32> = history.iter().map(|s| s.cpu).collect();
        let ram_vals: Vec<f32> = history.iter().map(|s| s.ram_pct).collect();
        let io_vals: Vec<f32> = history.iter().map(|s| s.io_write).collect();

        if stacked_summary {
            sparkline_card(ui, "CPU %", &cpu_vals, C_BL_FG, sparkline_width);
            ui.add_space(8.0);
            sparkline_card(ui, "RAM %", &ram_vals, C_WN_FG, sparkline_width);
            ui.add_space(8.0);
            sparkline_card(ui, "I/O Escrit. MB", &io_vals, C_CR_FG, sparkline_width);
        } else {
            ui.horizontal_wrapped(|ui| {
                sparkline_card(ui, "CPU %", &cpu_vals, C_BL_FG, sparkline_width);
                ui.add_space(8.0);
                sparkline_card(ui, "RAM %", &ram_vals, C_WN_FG, sparkline_width);
                ui.add_space(8.0);
                sparkline_card(ui, "I/O Escrit. MB", &io_vals, C_CR_FG, sparkline_width);
            });
        }
    }

    // ── Características del equipo ───────────────────────────────────────────
    if !hw.host_name.is_empty() || !hw.cpu_brand.is_empty() {
        ui.add_space(18.0);
        section_header(ui, "▸  Características del equipo");
        ui.add_space(8.0);
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(14.0))
            .show(ui, |ui| {
                ui.set_max_width(content_width.min(700.0));
                if narrow_summary {
                    hw_row(ui, "🖥  Equipo", &hw.host_name);
                    hw_row(ui, "💠  Sistema", &hw.os_name);
                    hw_row(ui, "📋  Versión OS", &hw.os_version);
                    hw_row(ui, "🏗  Arquitectura", &hw.architecture);
                    hw_row(
                        ui,
                        "⚙  CPU",
                        &format!("{}  ·  {} núcleos", hw.cpu_brand, hw.cpu_cores),
                    );
                    if hw.cpu_freq_mhz > 0 {
                        hw_row(
                            ui,
                            "⚡  Frecuencia",
                            &format!("{:.1} GHz", hw.cpu_freq_mhz as f32 / 1000.0),
                        );
                    }
                    hw_row(ui, "💾  RAM total", &format!("{:.1} GB", hw.total_ram_gb));
                } else {
                    ui.columns(2, |cols| {
                        let left = &mut cols[0];
                        hw_row(left, "🖥  Equipo", &hw.host_name);
                        hw_row(left, "💠  Sistema", &hw.os_name);
                        hw_row(left, "📋  Versión OS", &hw.os_version);
                        hw_row(left, "🏗  Arquitectura", &hw.architecture);

                        let right = &mut cols[1];
                        hw_row(
                            right,
                            "⚙  CPU",
                            &format!("{}  ·  {} núcleos", hw.cpu_brand, hw.cpu_cores),
                        );
                        if hw.cpu_freq_mhz > 0 {
                            hw_row(
                                right,
                                "⚡  Frecuencia",
                                &format!("{:.1} GHz", hw.cpu_freq_mhz as f32 / 1000.0),
                            );
                        }
                        hw_row(
                            right,
                            "💾  RAM total",
                            &format!("{:.1} GB", hw.total_ram_gb),
                        );
                    });
                }
            });
    }
}

// ── Tab: Procesos ──────────────────────────────────────────────────────────────

fn draw_tab_processes<F: FnMut(u32), G: FnMut(Option<Severity>)>(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    filter: &str,
    sev_filter: Option<Severity>,
    mut on_kill: F,
    mut on_sev_filter: G,
) {
    section_header(
        ui,
        "▸  Procesos dominantes  ·  ordenados por severidad, I/O, RAM, CPU",
    );
    ui.add_space(6.0);

    // Filtros de severidad rápidos
    ui.horizontal(|ui| {
        let sel_none = sev_filter.is_none();
        if ui
            .add(
                egui::Button::new(RichText::new("Todos").size(11.5).color(if sel_none {
                    TEXT_PRI
                } else {
                    TEXT_MUT
                }))
                .fill(if sel_none {
                    BG_CARD
                } else {
                    Color32::TRANSPARENT
                })
                .stroke(Stroke::new(
                    1.0,
                    if sel_none {
                        BORDER
                    } else {
                        Color32::TRANSPARENT
                    },
                ))
                .rounding(Rounding::same(4.0)),
            )
            .clicked()
        {
            on_sev_filter(None);
        }
        // Sin glifo de color: los símbolos geométricos (■ ▲ ●) no están en la
        // fuente y salían como "□". El color del texto/relleno ya distingue.
        for (label, sev, fg, bg) in [
            ("Crítico", Severity::Critical, C_CR_FG, C_CR_BG),
            ("Aviso", Severity::Warning, C_WN_FG, C_WN_BG),
            ("Sano", Severity::Healthy, C_OK_FG, C_OK_BG),
        ] {
            let selected = sev_filter == Some(sev);
            if ui
                .add(
                    egui::Button::new(RichText::new(label).size(11.5).color(if selected {
                        fg
                    } else {
                        TEXT_MUT
                    }))
                    .fill(if selected { bg } else { Color32::TRANSPARENT })
                    .stroke(Stroke::new(
                        1.0,
                        if selected {
                            fg.linear_multiply(0.5)
                        } else {
                            Color32::TRANSPARENT
                        },
                    ))
                    .rounding(Rounding::same(4.0)),
                )
                .clicked()
            {
                on_sev_filter(Some(sev));
            }
        }

        // Contador de resultados
        let count = snap
            .processes
            .iter()
            .filter(|p| matches_filter(&p.name, &p.exe_path, filter))
            .filter(|p| sev_filter.is_none() || p.severity == sev_filter.unwrap())
            .count();
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{count} proceso{}",
                if count != 1 { "s" } else { "" }
            ))
            .size(11.0)
            .color(TEXT_MUT),
        );
    });
    ui.add_space(6.0);

    // Cabecera de columnas
    table_header(
        ui,
        &[
            ("Proceso", W_NAME),
            ("PID", W_PID),
            ("CPU %", W_PCT),
            ("", W_BAR),
            ("RAM MB", W_MB),
            ("", W_BAR),
            ("W MB", W_MB),
            ("R MB", W_MB),
            ("Score", W_SCORE),
            ("", W_ACTION),
        ],
    );

    let mut to_kill: Option<u32> = None;
    let total_ram_mb = snap.overview.memory_total_gb.max(0.1) * 1024.0;

    egui::ScrollArea::vertical()
        .id_source("tab_procs")
        .show(ui, |ui| {
            for (i, p) in snap
                .processes
                .iter()
                .filter(|p| matches_filter(&p.name, &p.exe_path, filter))
                .filter(|p| sev_filter.is_none() || p.severity == sev_filter.unwrap())
                .take(30)
                .enumerate()
            {
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };
                let fg = sev_fg(p.severity);

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(6.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Nombre + tooltip con exe_path y razones
                            let short = trunc(&p.name, 22);
                            let resp = ui.add_sized(
                                [W_NAME, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{} {short}", sev_dot(p.severity)))
                                        .color(fg)
                                        .strong(),
                                ),
                            );
                            resp.on_hover_ui(|ui| {
                                ui.set_max_width(420.0);
                                ui.label(RichText::new(&p.name).strong().color(TEXT_PRI));
                                ui.label(
                                    RichText::new(&p.exe_path)
                                        .small()
                                        .monospace()
                                        .color(TEXT_MUT),
                                );
                                if let Some(cmdline) = &p.command_line {
                                    ui.separator();
                                    ui.label(
                                        RichText::new("Línea de comandos:")
                                            .size(10.5)
                                            .color(TEXT_MUT),
                                    );
                                    ui.label(
                                        RichText::new(cmdline).small().monospace().color(C_BL_FG),
                                    );
                                }
                                if !p.reasons.is_empty() {
                                    ui.separator();
                                    for r in &p.reasons {
                                        ui.label(RichText::new(r).small().color(TEXT_SEC));
                                    }
                                }
                            });

                            // PID
                            ui.add_sized(
                                [W_PID, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{}", p.pid))
                                        .monospace()
                                        .size(11.0)
                                        .color(TEXT_MUT),
                                ),
                            );

                            // CPU % + barra
                            ui.add_sized(
                                [W_PCT, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", p.cpu_percent))
                                        .size(12.0)
                                        .color(fg),
                                ),
                            );
                            pbar(ui, p.cpu_percent / 100.0, fg, W_BAR);

                            // RAM + barra
                            ui.add_sized(
                                [W_MB, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.0}", p.memory_mb))
                                        .size(12.0)
                                        .color(TEXT_SEC),
                                ),
                            );
                            pbar(
                                ui,
                                (p.memory_mb / total_ram_mb).min(1.0),
                                sev_fg(Severity::Warning),
                                W_BAR,
                            );

                            // Write MB
                            ui.add_sized(
                                [W_MB, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", p.io_write_mb_delta))
                                        .size(12.0)
                                        .color(if p.io_write_mb_delta > 10.0 {
                                            fg
                                        } else {
                                            TEXT_MUT
                                        }),
                                ),
                            );

                            // Read MB
                            ui.add_sized(
                                [W_MB, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", p.io_read_mb_delta))
                                        .size(12.0)
                                        .color(TEXT_MUT),
                                ),
                            );

                            // Score
                            ui.add_sized(
                                [W_SCORE, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{}", p.score)).size(12.0).color(fg),
                                ),
                            );

                            // Acción
                            if p.can_terminate
                                && action_btn(ui, "Finalizar", C_CR_BG, C_CR_FG).clicked()
                            {
                                to_kill = Some(p.pid);
                            }
                        });
                    });

                // Separador sutil
                ui.add(egui::Separator::default().spacing(0.0));
            }
        });

    if let Some(pid) = to_kill {
        on_kill(pid);
    }
}

// ── Tab: Conexiones ────────────────────────────────────────────────────────────

fn draw_tab_connections<F: FnMut(&str)>(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    filter: &str,
    only_public: &mut bool,
    mut on_block: F,
) {
    section_header(
        ui,
        "▸  Conexiones activas  ·  foco en IP pública y rutas poco confiables",
    );
    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.checkbox(
            only_public,
            RichText::new("Solo IP públicas").color(TEXT_SEC),
        )
        .on_hover_text("Ocultar conexiones a IPs privadas / localhost");
        let total = snap.connections.len();
        let shown = snap
            .connections
            .iter()
            .filter(|c| !*only_public || c.is_public_remote)
            .filter(|c| matches_filter(&c.process_name, &c.remote_address, filter))
            .count();
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{shown} conexión{}",
                if shown != 1 { "es" } else { "" }
            ))
            .size(11.0)
            .color(TEXT_MUT),
        );
        if shown < total {
            ui.label(
                RichText::new(format!("de {total} totales"))
                    .size(11.0)
                    .color(TEXT_MUT),
            );
        }
    });
    ui.add_space(6.0);

    table_header(
        ui,
        &[
            ("Proceso", W_NAME),
            ("PID", W_PID),
            ("Proto", W_PROTO),
            ("Estado", W_STATE),
            ("Local", W_ADDR),
            ("Remoto", W_ADDR),
            ("", W_ACTION),
        ],
    );

    let mut to_block: Option<String> = None;

    egui::ScrollArea::vertical()
        .id_source("tab_conns")
        .show(ui, |ui| {
            for (i, c) in snap
                .connections
                .iter()
                .filter(|c| !*only_public || c.is_public_remote)
                .filter(|c| {
                    matches_filter(
                        &c.process_name,
                        &format!("{} {}", c.remote_address, c.exe_path),
                        filter,
                    )
                })
                .take(30)
                .enumerate()
            {
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };
                let fg = sev_fg(c.severity);

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(6.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Nombre proceso + tooltip exe
                            let short = trunc(&c.process_name, 22);
                            let resp = ui.add_sized(
                                [W_NAME, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{} {short}", sev_dot(c.severity)))
                                        .color(fg)
                                        .strong(),
                                ),
                            );
                            resp.on_hover_ui(|ui| {
                                ui.set_max_width(360.0);
                                ui.label(RichText::new(&c.process_name).strong().color(TEXT_PRI));
                                ui.label(
                                    RichText::new(&c.exe_path)
                                        .small()
                                        .monospace()
                                        .color(TEXT_MUT),
                                );
                                ui.label(RichText::new(&c.reason).small().color(TEXT_SEC));
                            });

                            // PID
                            ui.add_sized(
                                [W_PID, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{}", c.pid))
                                        .monospace()
                                        .size(11.0)
                                        .color(TEXT_MUT),
                                ),
                            );

                            // Protocolo
                            ui.add_sized(
                                [W_PROTO, 18.0],
                                egui::Label::new(
                                    RichText::new(&c.protocol).size(11.5).color(C_BL_FG),
                                ),
                            );

                            // Estado
                            ui.add_sized(
                                [W_STATE, 18.0],
                                egui::Label::new(
                                    RichText::new(&c.state).size(11.0).color(TEXT_SEC),
                                ),
                            );

                            // Local
                            let local_short = trunc(&c.local_address, 22);
                            let lr = ui.add_sized(
                                [W_ADDR, 18.0],
                                egui::Label::new(
                                    RichText::new(&local_short)
                                        .monospace()
                                        .size(11.0)
                                        .color(TEXT_MUT),
                                ),
                            );
                            if c.local_address.len() > 22 {
                                lr.on_hover_text(&c.local_address);
                            }

                            // Remoto
                            let remote_short = trunc(&c.remote_address, 22);
                            let rr = ui.add_sized(
                                [W_ADDR, 18.0],
                                egui::Label::new(
                                    RichText::new(&remote_short)
                                        .monospace()
                                        .size(11.0)
                                        .color(if c.is_public_remote { fg } else { TEXT_MUT }),
                                ),
                            );
                            if c.remote_address.len() > 22 {
                                rr.on_hover_text(&c.remote_address);
                            }

                            // Bloquear
                            if c.is_public_remote
                                && action_btn(ui, "Bloquear", C_CR_BG, C_CR_FG).clicked()
                            {
                                to_block = Some(c.remote_address.clone());
                            }
                        });
                    });
                ui.add(egui::Separator::default().spacing(0.0));
            }
        });

    if let Some(ip) = to_block {
        on_block(&ip);
    }
}

// ── Tab: Temporales ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_tab_temp(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    filter: &str,
    confirm: &mut bool,
    result: &Option<String>,
    execute: &mut bool,
    docker_scan: &Option<DockerScan>,
    docker_prune_confirm: &mut Option<DockerPruneKind>,
    docker_result: &Option<String>,
    docker_action: &mut Option<DockerUiAction>,
) {
    section_header(
        ui,
        tr(
            "▸  Archivos temporales  ·  instaladores, actualizaciones, exportaciones",
            "▸  Temporary files  ·  installers, updates, exports",
        ),
    );
    ui.add_space(8.0);

    // ── Limpieza segura de %TEMP% (solo tu carpeta, >24h, salta lo en uso) ─────
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if !*confirm {
                    if ui
                        .add(egui::Button::new(
                            RichText::new("🗑  Limpiar %TEMP% (>24h, no en uso)")
                                .size(12.5)
                                .color(TEXT_PRI),
                        ))
                        .on_hover_text(
                            "Borra de tu carpeta %TEMP% solo lo no modificado en 24h; \
                             salta lo bloqueado. No toca el sistema ni Windows Update.",
                        )
                        .clicked()
                    {
                        *confirm = true;
                    }
                    ui.label(
                        RichText::new("Seguro: solo tu %TEMP%, salta archivos en uso.")
                            .size(10.5)
                            .color(TEXT_MUT),
                    );
                } else {
                    ui.label(
                        RichText::new("¿Confirmar? Se borrará lo no usado (>24h) de tu %TEMP%.")
                            .size(12.0)
                            .strong()
                            .color(C_WN_FG),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Sí, limpiar").size(12.0).color(TEXT_PRI),
                            )
                            .fill(C_CR_BG),
                        )
                        .clicked()
                    {
                        *execute = true;
                    }
                    if ui
                        .add(egui::Button::new(
                            RichText::new("Cancelar").size(12.0).color(TEXT_SEC),
                        ))
                        .clicked()
                    {
                        *confirm = false;
                    }
                }
            });
            if let Some(msg) = result {
                ui.add_space(4.0);
                ui.label(RichText::new(msg).size(11.0).color(C_OK_FG));
            }
        });
    ui.add_space(10.0);

    table_header(
        ui,
        &[
            ("Ruta", 340.0),
            ("Tamaño", W_MB + 20.0),
            ("", W_BAR),
            ("Archivos", W_MB),
            ("Nota", 0.0), // expansible
        ],
    );

    egui::ScrollArea::vertical()
        .id_source("tab_temp")
        .show(ui, |ui| {
            for (i, e) in snap
                .temp
                .top_entries
                .iter()
                .filter(|e| matches_filter(&e.path, &e.note, filter))
                .enumerate()
            {
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };
                let fg = sev_fg(e.severity);

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(6.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Ruta truncada con tooltip
                            let short = trunc(&e.path, 46);
                            let resp = ui.add_sized(
                                [340.0, 18.0],
                                egui::Label::new(
                                    RichText::new(&short).monospace().size(11.5).color(TEXT_SEC),
                                ),
                            );
                            if e.path.len() > 46 {
                                resp.on_hover_text(&e.path);
                            }

                            // Tamaño
                            ui.add_sized(
                                [W_MB + 20.0, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1} MB", e.size_mb))
                                        .size(12.0)
                                        .color(fg),
                                ),
                            );

                            // Barra de tamaño
                            pbar(ui, (e.size_mb / 2000.0).min(1.0), fg, W_BAR);

                            // Archivos
                            ui.add_sized(
                                [W_MB, 18.0],
                                egui::Label::new(
                                    RichText::new(format!("{}", e.file_count))
                                        .size(11.5)
                                        .color(TEXT_MUT),
                                ),
                            );

                            // Nota (resto del espacio)
                            let note_short = trunc(&e.note, 50);
                            let nr = ui.label(
                                RichText::new(&note_short)
                                    .size(11.0)
                                    .color(TEXT_MUT)
                                    .italics(),
                            );
                            if e.note.len() > 50 {
                                nr.on_hover_text(&e.note);
                            }
                        });
                    });
                ui.add(egui::Separator::default().spacing(0.0));
            }
        });

    if !snap.temp.limitations.is_empty() {
        ui.add_space(6.0);
        for lim in &snap.temp.limitations {
            ui.label(RichText::new(lim).small().italics().color(TEXT_MUT));
        }
    }

    // ── Docker: otro gran consumidor de disco, a menudo invisible ──────────────
    ui.add_space(18.0);
    draw_docker_section(
        ui,
        docker_scan,
        docker_prune_confirm,
        docker_result,
        docker_action,
    );
}

/// Sección Docker dentro del tab Temporales: imágenes, volúmenes y espacio
/// recuperable, con purga guiada segura (dangling + caché de build).
fn draw_docker_section(
    ui: &mut egui::Ui,
    scan: &Option<DockerScan>,
    prune_confirm: &mut Option<DockerPruneKind>,
    result: &Option<String>,
    action: &mut Option<DockerUiAction>,
) {
    section_header(
        ui,
        tr(
            "▸  Docker  ·  imágenes, volúmenes y espacio recuperable",
            "▸  Docker  ·  images, volumes and reclaimable space",
        ),
    );
    ui.add_space(8.0);

    let Some(scan) = scan else {
        // Aún no se ha escaneado: tarjeta con botón.
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(tr(
                            "Docker acumula capas de imágenes, cachés de build y volúmenes que no \
                             aparecen en las carpetas temporales. Escanea para ver cuánto ocupa y \
                             qué puedes liberar sin riesgo.",
                            "Docker piles up image layers, build caches and volumes that never show \
                             up in the temp folders. Scan to see how much it uses and what you can \
                             safely reclaim.",
                        ))
                        .size(12.0)
                        .color(TEXT_SEC),
                    )
                    .wrap(true),
                );
                ui.add_space(8.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new(tr("Escanear Docker", "Scan Docker"))
                                .size(12.5)
                                .color(C_BL_FG),
                        )
                        .fill(C_BL_BG)
                        .stroke(Stroke::new(1.0, C_BL_FG.linear_multiply(0.4)))
                        .rounding(Rounding::same(5.0)),
                    )
                    .clicked()
                {
                    *action = Some(DockerUiAction::Scan);
                }
            });
        return;
    };

    if !scan.available {
        // Docker no instalado o daemon caído.
        egui::Frame::none()
            .fill(C_WN_BG)
            .stroke(Stroke::new(1.0, C_WN_FG.linear_multiply(0.4)))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            scan.message
                                .as_deref()
                                .unwrap_or("Docker no está disponible."),
                        )
                        .size(12.0)
                        .color(C_WN_FG),
                    )
                    .wrap(true),
                );
                ui.add_space(8.0);
                if action_btn(ui, tr("Reintentar", "Retry"), C_BL_BG, C_BL_FG).clicked() {
                    *action = Some(DockerUiAction::Scan);
                }
            });
        return;
    }

    // ── Resumen: ocupado / recuperable + botón de reescaneo ────────────────────
    egui::Frame::none()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                docker_stat(
                    ui,
                    tr("Ocupado", "Used"),
                    &fmt_size_mb(scan.total_size_mb()),
                    TEXT_PRI,
                );
                ui.add_space(16.0);
                let recl = scan.total_reclaimable_mb();
                docker_stat(
                    ui,
                    tr("Recuperable", "Reclaimable"),
                    &fmt_size_mb(recl),
                    if recl > 100.0 { C_WN_FG } else { C_OK_FG },
                );
                ui.add_space(16.0);
                docker_stat(
                    ui,
                    tr("Imágenes colgantes", "Dangling images"),
                    &scan.dangling_count().to_string(),
                    if scan.dangling_count() > 0 {
                        C_WN_FG
                    } else {
                        TEXT_SEC
                    },
                );
                ui.add_space(16.0);
                if action_btn(ui, tr("Reescanear", "Rescan"), C_BL_BG, C_BL_FG).clicked() {
                    *action = Some(DockerUiAction::Scan);
                }
            });
        });
    ui.add_space(8.0);

    // ── Barra segmentada por categoría (Images / Containers / Volumes / Cache) ──
    if scan.total_size_mb() > 0.5 {
        docker_category_bar(ui, scan);
        ui.add_space(8.0);
    }

    // ── Categorías (tabla compacta) ────────────────────────────────────────────
    for c in &scan.categories {
        ui.horizontal(|ui| {
            ui.add_sized(
                [150.0, 18.0],
                egui::Label::new(RichText::new(&c.kind).size(12.0).color(TEXT_SEC)),
            );
            ui.add_sized(
                [70.0, 18.0],
                egui::Label::new(
                    RichText::new(format!("{}/{}", c.active, c.total))
                        .size(11.5)
                        .color(TEXT_MUT),
                ),
            );
            ui.add_sized(
                [90.0, 18.0],
                egui::Label::new(
                    RichText::new(fmt_size_mb(c.size_mb))
                        .size(12.0)
                        .color(TEXT_PRI),
                ),
            );
            let recl = c.reclaimable_mb;
            if recl > 0.5 {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        tr("recuperable", "reclaimable"),
                        fmt_size_mb(recl)
                    ))
                    .size(11.0)
                    .color(C_WN_FG),
                );
            }
        });
        ui.add_space(2.0);
    }

    // ── Imágenes más grandes ───────────────────────────────────────────────────
    if !scan.images.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new(tr("Imágenes más grandes", "Largest images"))
                .size(12.0)
                .strong()
                .color(TEXT_SEC),
        );
        ui.add_space(4.0);
        for img in scan.images.iter().take(8) {
            ui.horizontal(|ui| {
                let (name, name_color) = if img.dangling {
                    (tr("<colgante>", "<dangling>").to_owned(), C_WN_FG)
                } else {
                    (format!("{}:{}", img.repository, img.tag), TEXT_SEC)
                };
                let short = trunc(&name, 42);
                let resp = ui.add_sized(
                    [300.0, 18.0],
                    egui::Label::new(
                        RichText::new(&short)
                            .monospace()
                            .size(11.5)
                            .color(name_color),
                    ),
                );
                if name.len() > 42 {
                    resp.on_hover_text(&name);
                }
                ui.add_sized(
                    [80.0, 18.0],
                    egui::Label::new(
                        RichText::new(fmt_size_mb(img.size_mb))
                            .size(12.0)
                            .color(TEXT_PRI),
                    ),
                );
                ui.label(
                    RichText::new(&img.created)
                        .size(11.0)
                        .italics()
                        .color(TEXT_MUT),
                );
            });
            ui.add_space(1.0);
        }
    }

    // ── Volúmenes (solo lectura — contienen datos) ─────────────────────────────
    if !scan.volumes.is_empty() {
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{} ({})",
                tr("Volúmenes — revisión manual", "Volumes — manual review"),
                scan.volumes.len()
            ))
            .size(12.0)
            .strong()
            .color(TEXT_SEC),
        );
        ui.label(
            RichText::new(tr(
                "Los volúmenes guardan datos persistentes de contenedores (bases de datos, etc.). \
                 No se borran desde aquí; revísalos y elimina manualmente los que ya no uses.",
                "Volumes hold persistent container data (databases, etc.). They are never deleted \
                 from here; review them and remove the unused ones manually.",
            ))
            .size(10.5)
            .italics()
            .color(TEXT_MUT),
        );
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            for v in scan.volumes.iter().take(24) {
                pill(ui, &trunc(&v.name, 28), TEXT_SEC, BG_CARD);
            }
        });
    }

    // ── Purga guiada segura (2 pasos) ──────────────────────────────────────────
    ui.add_space(10.0);
    egui::Frame::none()
        .fill(BG_PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(tr("Purga segura", "Safe cleanup"))
                    .size(12.5)
                    .strong()
                    .color(TEXT_PRI),
            );
            ui.label(
                RichText::new(tr(
                    "Solo elimina lo regenerable: imágenes colgantes y caché de build. Nunca toca \
                     imágenes etiquetadas ni volúmenes.",
                    "Only removes regenerable data: dangling images and build cache. Never touches \
                     tagged images or volumes.",
                ))
                .size(10.5)
                .color(TEXT_MUT),
            );
            ui.add_space(6.0);
            docker_prune_row(
                ui,
                DockerPruneKind::Images,
                tr("Purgar imágenes colgantes", "Prune dangling images"),
                prune_confirm,
                action,
            );
            docker_prune_row(
                ui,
                DockerPruneKind::Cache,
                tr("Purgar caché de build", "Prune build cache"),
                prune_confirm,
                action,
            );
        });

    if let Some(msg) = result {
        ui.add_space(6.0);
        let color = if msg.starts_with('❌') {
            C_CR_FG
        } else {
            C_OK_FG
        };
        ui.label(RichText::new(msg).size(11.5).color(color));
    }
}

/// Métrica compacta etiqueta + valor para el resumen de Docker.
fn docker_stat(ui: &mut egui::Ui, label: &str, value: &str, value_color: Color32) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(10.5).color(TEXT_MUT));
        ui.label(RichText::new(value).size(15.0).strong().color(value_color));
    });
}

/// Barra horizontal segmentada por categoría de Docker (estilo almacenamiento).
fn docker_category_bar(ui: &mut egui::Ui, scan: &DockerScan) {
    let total = scan.total_size_mb().max(0.001);
    let colors = [C_BL_FG, C_OK_FG, C_WN_FG, ACCENT];
    let width = ui.available_width().clamp(200.0, 760.0);
    let h = 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(5.0), BG_PANEL);
    let mut x = rect.left();
    for (i, c) in scan.categories.iter().enumerate() {
        let frac = (c.size_mb / total).clamp(0.0, 1.0) as f32;
        let seg_w = rect.width() * frac;
        if seg_w > 0.5 {
            let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top()), Vec2::new(seg_w, h));
            ui.painter()
                .rect_filled(seg, Rounding::same(2.0), colors[i % colors.len()]);
            x += seg_w;
        }
    }
    // Leyenda
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        for (i, c) in scan.categories.iter().enumerate() {
            if c.size_mb <= 0.5 {
                continue;
            }
            let (dot, _) = ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
            ui.painter()
                .rect_filled(dot, Rounding::same(2.0), colors[i % colors.len()]);
            ui.label(
                RichText::new(format!("{} · {}", c.kind, fmt_size_mb(c.size_mb)))
                    .size(10.5)
                    .color(TEXT_SEC),
            );
            ui.add_space(8.0);
        }
    });
}

/// Fila de purga con confirmación de 2 pasos, reutilizando el patrón de %TEMP%.
fn docker_prune_row(
    ui: &mut egui::Ui,
    kind: DockerPruneKind,
    label: &str,
    confirm: &mut Option<DockerPruneKind>,
    action: &mut Option<DockerUiAction>,
) {
    ui.horizontal_wrapped(|ui| {
        if *confirm == Some(kind) {
            ui.label(
                RichText::new(tr("¿Confirmar?", "Confirm?"))
                    .size(12.0)
                    .strong()
                    .color(C_WN_FG),
            );
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(tr("Sí, purgar", "Yes, prune"))
                            .size(11.5)
                            .color(TEXT_PRI),
                    )
                    .fill(C_CR_BG),
                )
                .clicked()
            {
                *action = Some(DockerUiAction::Prune(kind));
            }
            if ui
                .add(egui::Button::new(
                    RichText::new(tr("Cancelar", "Cancel"))
                        .size(11.5)
                        .color(TEXT_SEC),
                ))
                .clicked()
            {
                *confirm = None;
            }
        } else if ui
            .add(
                egui::Button::new(RichText::new(label).size(11.5).color(TEXT_PRI))
                    .fill(BG_CARD)
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .clicked()
        {
            *confirm = Some(kind);
        }
    });
}

/// Formatea un tamaño en MB como MB o GB legible.
fn fmt_size_mb(mb: f64) -> String {
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{:.0} MB", mb)
    }
}

// ── Tab: ETW / WPR ─────────────────────────────────────────────────────────────

fn draw_tab_precision(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    precision_note: &mut String,
    precision_action: &mut Option<PrecisionAction>,
) {
    let p = &snap.precision;
    let recording = p.is_recording;

    section_header(ui, "▸  Captura de precisión ETW / WPR");
    ui.add_space(8.0);

    egui::Frame::none()
        .fill(if recording { C_WN_BG } else { BG_CARD })
        .stroke(Stroke::new(
            1.0,
            if recording {
                C_WN_FG.linear_multiply(0.5)
            } else {
                BORDER
            },
        ))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            // Estado y herramientas
            ui.horizontal_wrapped(|ui| {
                tool_chip(ui, "WPR", p.wpr_available);
                tool_chip(ui, "WPA", p.wpa_available);
                tool_chip(ui, "Tracerpt", p.tracerpt_available);
                ui.add_space(12.0);
                let (txt, label) = if recording {
                    (C_WN_FG, "GRABANDO")
                } else {
                    (TEXT_MUT, "En espera")
                };
                // Punto de estado pintado (los glifos ●/○ no están en la fuente).
                let (dot_rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 4.0, txt);
                ui.add_space(3.0);
                ui.label(RichText::new(label).strong().size(13.0).color(txt));
            });

            ui.add_space(10.0);
            ui.label(RichText::new(&p.guidance).color(TEXT_SEC).size(13.0));

            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Trazas:").color(TEXT_MUT).size(11.5));
                ui.label(
                    RichText::new(&p.traces_directory)
                        .monospace()
                        .size(11.5)
                        .color(TEXT_SEC),
                );
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Motor:").color(TEXT_MUT).size(11.5));
                ui.label(RichText::new(&p.analyzer_label).size(11.5).color(TEXT_SEC));
            });

            if let Some(path) = &p.last_trace_path {
                info_row(ui, "Último ETL:", path);
            }
            if let Some(path) = &p.last_analysis_path {
                info_row_ok(ui, "Resumen:", path);
            }
            if !p.status_detail.is_empty() {
                ui.label(RichText::new(&p.status_detail).small().color(TEXT_MUT));
            }

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Descripción:").color(TEXT_SEC).size(13.0));
                ui.add_sized(
                    [420.0, 28.0],
                    egui::TextEdit::singleline(precision_note)
                        .hint_text("Ej: disco al 100% mientras Windows Update descarga")
                        .text_color(TEXT_PRI),
                );
            });

            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if p.wpr_available
                    && !recording
                    && action_btn(ui, "Iniciar captura", C_OK_BG, C_OK_FG).clicked()
                {
                    *precision_action = Some(PrecisionAction::Start);
                }
                if p.wpr_available && recording {
                    if action_btn(ui, "Detener y guardar", C_WN_BG, C_WN_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Stop);
                    }
                    if action_btn(ui, "×  Cancelar", C_CR_BG, C_CR_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Cancel);
                    }
                }
                if !recording
                    && p.tracerpt_available
                    && p.last_trace_path.is_some()
                    && action_btn(ui, "⚡  Analizar ETL", C_BL_BG, C_BL_FG).clicked()
                {
                    *precision_action = Some(PrecisionAction::Analyze);
                }
            });
        });

    // Análisis de traza si existe
    if let Some(ta) = &snap.trace_analysis {
        ui.add_space(16.0);
        section_header(ui, "▸  Resumen del último ETL procesado");
        ui.add_space(8.0);
        draw_trace_analysis(ui, ta);
    }
}

fn draw_trace_analysis(ui: &mut egui::Ui, ta: &TraceAnalysisSummary) {
    egui::Frame::none()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let sev = ta
                    .findings
                    .first()
                    .map(|f| f.severity)
                    .unwrap_or(Severity::Healthy);
                pill(ui, &ta.headline, sev_fg(sev), sev_bg(sev));
                pill(
                    ui,
                    &format!("{} eventos", ta.total_events),
                    TEXT_SEC,
                    BG_ROW_ALT,
                );
                pill(ui, &ta.confidence, C_WN_FG, C_WN_BG);
            });

            ui.add_space(6.0);
            info_row(ui, "ETL:", &ta.etl_path);
            info_row(ui, "Salida:", &ta.output_directory);

            if !ta.findings.is_empty() {
                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("Hallazgos ({})", ta.findings.len()))
                        .strong()
                        .color(TEXT_PRI),
                );
                egui::ScrollArea::vertical()
                    .id_source("etl_findings")
                    .max_height(180.0)
                    .show(ui, |ui| {
                        for f in &ta.findings {
                            ui.add_space(4.0);
                            let fg = sev_fg(f.severity);
                            egui::Frame::none()
                                .fill(sev_bg(f.severity))
                                .stroke(Stroke::new(1.0, fg.linear_multiply(0.3)))
                                .rounding(Rounding::same(5.0))
                                .inner_margin(Margin::same(8.0))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(&f.title).strong().color(fg));
                                    ui.label(RichText::new(&f.detail).small().color(TEXT_SEC));
                                    ui.label(
                                        RichText::new(format!("Evidencia: {}", f.evidence))
                                            .small()
                                            .monospace()
                                            .color(TEXT_MUT),
                                    );
                                });
                        }
                    });
            }

            ui.add_space(10.0);
            ui.columns(3, |cols| {
                trace_processes_col(&mut cols[0], &ta.hot_processes);
                trace_paths_col(&mut cols[1], &ta.hot_paths);
                trace_context_col(&mut cols[2], ta);
            });
        });
}

fn trace_processes_col(ui: &mut egui::Ui, procs: &[TraceProcessSummary]) {
    ui.label(
        RichText::new("Procesos repetidos")
            .strong()
            .size(12.0)
            .color(TEXT_SEC),
    );
    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .id_source("tp")
        .max_height(180.0)
        .show(ui, |ui| {
            for p in procs.iter().take(6) {
                let fg = sev_fg(p.severity);
                egui::Frame::none()
                    .fill(sev_bg(p.severity))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&p.name).strong().color(fg));
                        ui.label(
                            RichText::new(format!("× {}  {}", p.occurrences, trunc(&p.reason, 30)))
                                .small()
                                .color(TEXT_SEC),
                        );
                    });
                ui.add_space(2.0);
            }
        });
}

fn trace_paths_col(ui: &mut egui::Ui, paths: &[TracePathSummary]) {
    ui.label(
        RichText::new("Rutas repetidas")
            .strong()
            .size(12.0)
            .color(TEXT_SEC),
    );
    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .id_source("tpa")
        .max_height(180.0)
        .show(ui, |ui| {
            for p in paths.iter().take(6) {
                let fg = sev_fg(p.severity);
                egui::Frame::none()
                    .fill(sev_bg(p.severity))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::same(6.0))
                    .show(ui, |ui| {
                        let short = trunc(&p.path, 32);
                        let resp = ui.label(RichText::new(&short).small().strong().color(fg));
                        if p.path.len() > 32 {
                            resp.on_hover_text(&p.path);
                        }
                        ui.label(
                            RichText::new(format!("{}  × {}", p.category, p.occurrences))
                                .small()
                                .color(TEXT_MUT),
                        );
                    });
                ui.add_space(2.0);
            }
        });
}

fn trace_context_col(ui: &mut egui::Ui, ta: &TraceAnalysisSummary) {
    ui.label(
        RichText::new("Proveedores ETW")
            .strong()
            .size(12.0)
            .color(TEXT_SEC),
    );
    ui.add_space(4.0);
    if !ta.providers.is_empty() {
        let max_count = ta
            .providers
            .iter()
            .map(|(_, c)| *c)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        egui::ScrollArea::vertical()
            .id_source("etl_prov")
            .max_height(100.0)
            .show(ui, |ui| {
                for (name, count) in ta.providers.iter().take(10) {
                    ui.horizontal(|ui| {
                        let short = trunc(name, 24);
                        ui.add_sized(
                            [120.0, 14.0],
                            egui::Label::new(RichText::new(&short).size(10.5).color(TEXT_SEC)),
                        );
                        pbar(ui, *count as f32 / max_count, C_BL_FG, 60.0);
                        ui.label(RichText::new(format!("{count}")).size(10.0).color(TEXT_MUT));
                    });
                }
            });
        ui.add_space(6.0);
    }

    if !ta.indicators.is_empty() {
        ui.label(
            RichText::new("Indicadores")
                .size(11.0)
                .strong()
                .color(TEXT_SEC),
        );
        ui.add_space(2.0);
        for ind in ta.indicators.iter().take(6) {
            ui.label(RichText::new(format!("· {ind}")).size(10.5).color(TEXT_MUT));
        }
        ui.add_space(4.0);
    }

    if !ta.public_ips.is_empty() {
        ui.label(
            RichText::new("IPs públicas")
                .size(11.0)
                .strong()
                .color(C_BL_FG),
        );
        for ip in ta.public_ips.iter().take(5) {
            ui.label(RichText::new(ip).small().monospace().color(TEXT_SEC));
        }
        ui.add_space(4.0);
    }
    for lim in ta.limitations.iter().take(3) {
        ui.label(RichText::new(lim).small().italics().color(TEXT_MUT));
    }
}

// ── Tab: Historial ─────────────────────────────────────────────────────────────

fn draw_tab_history(
    ui: &mut egui::Ui,
    rows: &[SnapshotRow],
    filter: &mut String,
    compare_a: &mut Option<usize>,
    compare_b: &mut Option<usize>,
) {
    section_header(
        ui,
        "▸  Historial de capturas  ·  últimas 60 entradas guardadas",
    );
    ui.add_space(8.0);

    if rows.is_empty() {
        ui.label(
            RichText::new("Sin historial aún — el historial se acumula con cada refresco.")
                .color(TEXT_MUT),
        );
        return;
    }

    // Buscador dentro del historial
    ui.horizontal(|ui| {
        draw_search_icon(ui, 14.0);
        ui.add_space(4.0);
        ui.add_sized(
            [220.0, 24.0],
            egui::TextEdit::singleline(filter)
                .hint_text("Filtrar por proceso o fecha…")
                .text_color(TEXT_PRI),
        );
        ui.add_space(8.0);
        let total = rows.len();
        let needle = filter.trim().to_ascii_lowercase();
        let shown = if needle.is_empty() {
            total
        } else {
            rows.iter()
                .filter(|r| {
                    r.dominant_process.to_ascii_lowercase().contains(&needle)
                        || r.collected_at.contains(&needle)
                })
                .count()
        };
        ui.label(
            RichText::new(format!("{shown} / {total}"))
                .size(11.0)
                .color(TEXT_MUT),
        );
    });
    ui.add_space(6.0);

    // Cabecera de columnas
    table_header(
        ui,
        &[
            ("Fecha / Hora", 150.0),
            ("CPU %", W_PCT + 10.0),
            ("RAM GB", W_MB + 10.0),
            ("I/O W MB", W_MB + 10.0),
            ("Temp MB", W_MB + 10.0),
            ("Proceso dominante", 200.0),
            ("Alertas", W_SCORE),
            ("", W_ACTION + 30.0),
        ],
    );

    egui::ScrollArea::vertical()
        .id_source("tab_hist")
        .show(ui, |ui| {
            let needle = filter.trim().to_ascii_lowercase();
            for (i, row) in rows
                .iter()
                .filter(|r| {
                    needle.is_empty()
                        || r.dominant_process.to_ascii_lowercase().contains(&needle)
                        || r.collected_at.contains(&needle)
                })
                .enumerate()
            {
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };
                let (fg, bg) = if row.has_critical {
                    (C_CR_FG, C_CR_BG)
                } else if row.alerts_count > 0 {
                    (C_WN_FG, C_WN_BG)
                } else {
                    (C_OK_FG, BG_CARD)
                };

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(6.0, 4.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Fecha/hora (truncar a HH:MM:SS si es larga)
                            let ts = if row.collected_at.len() > 19 {
                                &row.collected_at[..19]
                            } else {
                                &row.collected_at
                            };
                            ui.add_sized(
                                [150.0, 16.0],
                                egui::Label::new(
                                    RichText::new(ts).monospace().size(11.0).color(TEXT_SEC),
                                ),
                            );

                            // CPU
                            ui.add_sized(
                                [W_PCT + 10.0, 16.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", row.cpu_usage))
                                        .size(11.5)
                                        .color(sev_fg(severity_for_value(
                                            row.cpu_usage,
                                            55.0,
                                            80.0,
                                        ))),
                                ),
                            );

                            // RAM
                            let ram_pct = row.memory_used_gb / row.memory_total_gb.max(0.1) * 100.0;
                            ui.add_sized(
                                [W_MB + 10.0, 16.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", row.memory_used_gb))
                                        .size(11.5)
                                        .color(sev_fg(severity_for_value(ram_pct, 70.0, 88.0))),
                                ),
                            );

                            // I/O Write
                            ui.add_sized(
                                [W_MB + 10.0, 16.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.1}", row.io_write_mb_delta))
                                        .size(11.5)
                                        .color(sev_fg(severity_for_value(
                                            row.io_write_mb_delta,
                                            80.0,
                                            220.0,
                                        ))),
                                ),
                            );

                            // Temp
                            ui.add_sized(
                                [W_MB + 10.0, 16.0],
                                egui::Label::new(
                                    RichText::new(format!("{:.0}", row.temp_total_mb))
                                        .size(11.5)
                                        .color(TEXT_MUT),
                                ),
                            );

                            // Proceso dominante
                            let dp_short = trunc(&row.dominant_process, 26);
                            let resp = ui.add_sized(
                                [200.0, 16.0],
                                egui::Label::new(
                                    RichText::new(&dp_short).size(11.0).color(TEXT_SEC),
                                ),
                            );
                            if row.dominant_process.len() > 26 {
                                resp.on_hover_text(&row.dominant_process);
                            }

                            // Alertas
                            ui.add_sized(
                                [W_SCORE, 16.0],
                                egui::Label::new(if row.alerts_count > 0 {
                                    RichText::new(format!("{}", row.alerts_count))
                                        .size(11.5)
                                        .color(fg)
                                } else {
                                    RichText::new("—").size(11.5).color(TEXT_MUT)
                                }),
                            );

                            // Botones Comparar A / B
                            let is_a = *compare_a == Some(i);
                            let is_b = *compare_b == Some(i);
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(if is_a { "A ✅" } else { "A" })
                                            .size(11.0)
                                            .color(if is_a { C_BL_FG } else { TEXT_MUT }),
                                    )
                                    .fill(if is_a { C_BL_BG } else { BG_CARD })
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Marcar como punto A para comparación")
                                .clicked()
                            {
                                *compare_a = if is_a { None } else { Some(i) };
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(if is_b { "B ✅" } else { "B" })
                                            .size(11.0)
                                            .color(if is_b { C_WN_FG } else { TEXT_MUT }),
                                    )
                                    .fill(if is_b { C_WN_BG } else { BG_CARD })
                                    .rounding(Rounding::same(4.0)),
                                )
                                .on_hover_text("Marcar como punto B para comparación")
                                .clicked()
                            {
                                *compare_b = if is_b { None } else { Some(i) };
                            }
                        });

                        // Barra de severidad como acento lateral
                        if row.has_critical || row.alerts_count > 0 {
                            let r = ui.min_rect();
                            ui.painter().line_segment(
                                [
                                    egui::pos2(r.left(), r.top()),
                                    egui::pos2(r.left(), r.bottom()),
                                ],
                                Stroke::new(3.0, bg),
                            );
                        }
                    });
                ui.add(egui::Separator::default().spacing(0.0));
            }
        });

    // Panel de comparación
    if let (Some(ai), Some(bi)) = (*compare_a, *compare_b)
        && let (Some(row_a), Some(row_b)) = (rows.get(ai), rows.get(bi))
    {
        ui.add_space(14.0);
        section_header(ui, "▸  Comparación A vs B");
        ui.add_space(8.0);
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.columns(3, |cols| {
                    cols[0].label(RichText::new("Métrica").strong().size(12.0).color(TEXT_MUT));
                    cols[1].label(
                        RichText::new(format!(
                            "A  {}",
                            &row_a.collected_at.chars().take(19).collect::<String>()
                        ))
                        .strong()
                        .size(12.0)
                        .color(C_BL_FG),
                    );
                    cols[2].label(
                        RichText::new(format!(
                            "B  {}",
                            &row_b.collected_at.chars().take(19).collect::<String>()
                        ))
                        .strong()
                        .size(12.0)
                        .color(C_WN_FG),
                    );
                });
                ui.separator();
                for (label, va, vb) in [
                    ("CPU %", row_a.cpu_usage, row_b.cpu_usage),
                    ("RAM GB", row_a.memory_used_gb, row_b.memory_used_gb),
                    ("I/O W MB", row_a.io_write_mb_delta, row_b.io_write_mb_delta),
                    ("Temp MB", row_a.temp_total_mb, row_b.temp_total_mb),
                ] {
                    let delta = vb - va;
                    let delta_col = if delta > 0.5 {
                        C_CR_FG
                    } else if delta < -0.5 {
                        C_OK_FG
                    } else {
                        TEXT_MUT
                    };
                    ui.columns(3, |cols| {
                        cols[0].label(RichText::new(label).size(12.0).color(TEXT_SEC));
                        cols[1].label(RichText::new(format!("{va:.1}")).size(12.0).color(C_BL_FG));
                        cols[2].label(
                            RichText::new(format!(
                                "{vb:.1}  ({}{delta:.1})",
                                if delta >= 0.0 { "+" } else { "" }
                            ))
                            .size(12.0)
                            .color(delta_col),
                        );
                    });
                }
                ui.separator();
                ui.columns(3, |cols| {
                    cols[0].label(RichText::new("Alertas").size(12.0).color(TEXT_SEC));
                    cols[1].label(
                        RichText::new(format!("{}", row_a.alerts_count))
                            .size(12.0)
                            .color(C_BL_FG),
                    );
                    let diff = row_b.alerts_count as i64 - row_a.alerts_count as i64;
                    cols[2].label(
                        RichText::new(format!(
                            "{}  ({}{diff})",
                            row_b.alerts_count,
                            if diff >= 0 { "+" } else { "" }
                        ))
                        .size(12.0)
                        .color(if diff > 0 {
                            C_CR_FG
                        } else if diff < 0 {
                            C_OK_FG
                        } else {
                            TEXT_MUT
                        }),
                    );
                });
            });
    }
}

// ── Tab: Servicios ─────────────────────────────────────────────────────────────

fn draw_tab_services<F: FnMut(&str)>(ui: &mut egui::Ui, snap: &SystemSnapshot, mut on_stop: F) {
    section_header(
        ui,
        "▸  Servicios  ·  correlaciona con Windows Update, BITS, Delivery Optimization",
    );
    ui.add_space(8.0);

    let mut to_stop: Option<String> = None;

    if snap.services.is_empty() {
        empty_state(
            ui,
            "No se detectaron servicios relevantes en el último escaneo.",
        );
    }

    for svc in &snap.services {
        let sev = service_severity(svc);
        let fg = sev_fg(sev);
        let bg = sev_bg(sev);

        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.3)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    draw_service_icon(ui, sev, 14.0);
                    ui.add_space(4.0);
                    ui.label(RichText::new(&svc.display_name).strong().color(TEXT_PRI));
                    pill(ui, &svc.status, fg, bg);
                    pill(
                        ui,
                        &format!("Inicio {}", svc.start_type),
                        TEXT_MUT,
                        BG_ROW_ALT,
                    );
                    if is_stoppable_service(svc)
                        && svc.status.eq_ignore_ascii_case("Running")
                        && action_btn(ui, "Detener", C_WN_BG, C_WN_FG).clicked()
                    {
                        to_stop = Some(svc.name.clone());
                    }
                });
            });
        ui.add_space(3.0);
    }

    if let Some(svc) = to_stop {
        on_stop(&svc);
    }

    ui.add_space(12.0);
    section_header(ui, "▸  Eventos recientes del sistema");
    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .id_source("tab_events")
        .show(ui, |ui| {
            if snap.events.is_empty() {
                empty_state(ui, "Sin eventos recientes del sistema.");
            }
            for (i, evt) in snap.events.iter().take(15).enumerate() {
                let sev = if evt.level.eq_ignore_ascii_case("Error") {
                    Severity::Critical
                } else {
                    Severity::Warning
                };
                let fg = sev_fg(sev);
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(8.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(sev_dot(sev)).color(fg));
                            ui.label(
                                RichText::new(format!(
                                    "{}  ·  {}  ·  ID {}",
                                    evt.timestamp, evt.provider, evt.id
                                ))
                                .strong()
                                .size(11.0)
                                .color(fg),
                            );
                        });
                        let msg_short = trunc(&evt.message, 100);
                        let mr = ui.label(RichText::new(&msg_short).small().color(TEXT_SEC));
                        if evt.message.len() > 100 {
                            mr.on_hover_text(&evt.message);
                        }
                    });
                ui.add(egui::Separator::default().spacing(0.0));
            }
        });
}

// ── Tab: Autostart ────────────────────────────────────────────────────────────

fn draw_tab_autostart(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    filter: &str,
    accept_baseline: &mut bool,
) {
    section_header(
        ui,
        "▸  Autostart  ·  entradas de registro Run, carpetas Startup y tareas programadas",
    );
    ui.add_space(6.0);

    // Barra de resumen por tipo
    let entries = &snap.persistence_entries;
    if entries.is_empty() {
        ui.add_space(24.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("🚀")
                    .size(40.0)
                    .color(TEXT_MUT.linear_multiply(0.5)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("No se encontraron entradas de autostart")
                    .size(13.0)
                    .color(TEXT_MUT),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    "Registro Run vacío · Carpetas Startup vacías · Sin tareas detectadas",
                )
                .size(11.0)
                .color(TEXT_MUT.linear_multiply(0.6)),
            );
        });
        return;
    }

    // Contador y filtro de riesgo
    let filtered: Vec<&PersistenceEntry> = entries
        .iter()
        .filter(|e| {
            filter.is_empty()
                || e.name
                    .to_ascii_lowercase()
                    .contains(&filter.to_ascii_lowercase())
                || e.command
                    .to_ascii_lowercase()
                    .contains(&filter.to_ascii_lowercase())
                || e.entry_kind
                    .to_ascii_lowercase()
                    .contains(&filter.to_ascii_lowercase())
        })
        .collect();

    // Chips de resumen: total + cuántos son críticos/warning
    let n_critical = entries
        .iter()
        .filter(|e| matches!(e.severity, RiskLevel::Critical | RiskLevel::High))
        .count();
    let n_warn = entries
        .iter()
        .filter(|e| matches!(e.severity, RiskLevel::Medium))
        .count();

    // Conteo de cambios respecto a la baseline conocida
    let n_added = entries
        .iter()
        .filter(|e| e.change_status == PersistenceChange::Added)
        .count();
    let n_modified = entries
        .iter()
        .filter(|e| e.change_status == PersistenceChange::Modified)
        .count();
    let n_removed = entries
        .iter()
        .filter(|e| e.change_status == PersistenceChange::Removed)
        .count();
    let n_changes = n_added + n_modified + n_removed;
    let n_active = entries.len() - n_removed;

    ui.horizontal_wrapped(|ui| {
        pill(
            ui,
            &format!("{} entradas activas", n_active),
            TEXT_SEC,
            BG_CARD,
        );
        if n_critical > 0 {
            pill(ui, &format!("{} sospechosas", n_critical), C_CR_FG, C_CR_BG);
        }
        if n_warn > 0 {
            pill(ui, &format!("{} a revisar", n_warn), C_WN_FG, C_WN_BG);
        }
        if !filter.is_empty() {
            pill(
                ui,
                &format!("{} visibles", filtered.len()),
                C_BL_FG,
                C_BL_BG,
            );
        }
    });
    ui.add_space(8.0);

    // Banner de cambios vs baseline conocida + acción para aceptar el estado actual
    if n_changes > 0 {
        egui::Frame::none()
            .fill(C_CR_BG)
            .stroke(Stroke::new(1.0, C_CR_FG.linear_multiply(0.4)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("⚠").color(C_CR_FG).size(14.0));
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "{n_changes} cambio(s) de autoarranque vs baseline conocida:",
                        ))
                        .size(12.0)
                        .strong()
                        .color(TEXT_PRI),
                    );
                    if n_added > 0 {
                        pill(ui, &format!("+{n_added} nuevas"), C_CR_FG, C_CR_BG);
                    }
                    if n_modified > 0 {
                        pill(ui, &format!("~{n_modified} modificadas"), C_WN_FG, C_WN_BG);
                    }
                    if n_removed > 0 {
                        pill(ui, &format!("−{n_removed} eliminadas"), TEXT_MUT, BG_CARD);
                    }
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui
                        .add(egui::Button::new(
                            RichText::new("✅ Aceptar estado actual como baseline")
                                .size(12.0)
                                .color(TEXT_PRI),
                        ))
                        .on_hover_text(
                            "Marca el estado actual de autoarranque como \"bueno conocido\". \
                             Los cambios listados dejarán de reportarse.",
                        )
                        .clicked()
                    {
                        *accept_baseline = true;
                    }
                    ui.label(
                        RichText::new(
                            "Revisa cada cambio antes de aceptar: una entrada nueva puede ser \
                             persistencia de malware.",
                        )
                        .size(10.5)
                        .color(TEXT_MUT),
                    );
                });
            });
        ui.add_space(8.0);
    } else if n_active > 0 {
        // Sin cambios: confirmación tranquila de que hay baseline y coincide.
        ui.horizontal(|ui| {
            ui.label(RichText::new("✅").color(C_OK_FG).size(12.0));
            ui.add_space(4.0);
            ui.label(
                RichText::new("Sin cambios respecto a la baseline conocida.")
                    .size(11.0)
                    .color(TEXT_MUT),
            );
        });
        ui.add_space(6.0);
    }

    // Cabecera de columnas
    table_header(
        ui,
        &[
            ("", 18.0),
            ("Nombre", 180.0),
            ("Tipo", 200.0),
            ("Comando / Ruta", 340.0),
            ("En disco", 64.0),
        ],
    );

    egui::ScrollArea::vertical()
        .id_source("tab_autostart")
        .show(ui, |ui| {
            for (i, entry) in filtered.iter().enumerate() {
                let sev = entry.severity.to_severity();
                let fg = sev_fg(sev);
                let row_bg = if i % 2 == 0 { BG_APP } else { BG_ROW_ALT };

                egui::Frame::none()
                    .fill(row_bg)
                    .inner_margin(Margin::symmetric(6.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Dot de severidad
                            ui.add_sized(
                                [18.0, 18.0],
                                egui::Label::new(RichText::new(sev_dot(sev)).size(10.0).color(fg)),
                            );

                            // Nombre
                            let short_name = trunc(&entry.name, 24);
                            let name_resp = ui.add_sized(
                                [180.0, 18.0],
                                egui::Label::new(
                                    RichText::new(&short_name)
                                        .size(12.0)
                                        .strong()
                                        .color(TEXT_PRI),
                                ),
                            );
                            if entry.name.len() > 24 {
                                name_resp.on_hover_text(&entry.name);
                            }

                            // Badge de cambio vs baseline (NUEVA / MODIFICADA / ELIMINADA)
                            if entry.change_status.is_change() {
                                let (cfg, cbg) = match entry.change_status {
                                    PersistenceChange::Added => (C_CR_FG, C_CR_BG),
                                    PersistenceChange::Modified => (C_WN_FG, C_WN_BG),
                                    PersistenceChange::Removed | PersistenceChange::Unchanged => {
                                        (TEXT_MUT, BG_CARD)
                                    }
                                };
                                pill(ui, entry.change_status.label(), cfg, cbg);
                            }

                            // Tipo (pill con origen)
                            {
                                let kind_short = if entry.entry_kind.contains("RunOnce") {
                                    "RunOnce"
                                } else if entry.entry_kind.contains("HKCU") {
                                    "Registro (Usuario)"
                                } else if entry.entry_kind.contains("HKLM") {
                                    "Registro (Sistema)"
                                } else if entry.entry_kind.contains("All Users") {
                                    "Startup (Todos)"
                                } else if entry.entry_kind.contains("Current User") {
                                    "Startup (Usuario)"
                                } else if entry.entry_kind.contains("Scheduled") {
                                    "Tarea programada"
                                } else {
                                    &entry.entry_kind
                                };
                                let (kfg, kbg) = if entry.entry_kind.contains("HKLM")
                                    || entry.entry_kind.contains("Scheduled")
                                {
                                    (C_WN_FG, C_WN_BG)
                                } else {
                                    (C_BL_FG, C_BL_BG)
                                };
                                ui.allocate_ui_with_layout(
                                    Vec2::new(200.0, 18.0),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| pill(ui, kind_short, kfg, kbg),
                                );
                            }

                            // Comando / ruta
                            let short_cmd = trunc(&entry.command, 48);
                            let cmd_resp = ui.add_sized(
                                [340.0, 18.0],
                                egui::Label::new(
                                    RichText::new(&short_cmd).size(11.5).monospace().color(
                                        if sev == Severity::Healthy {
                                            TEXT_SEC
                                        } else {
                                            fg
                                        },
                                    ),
                                ),
                            );
                            // Tooltip con comando completo + nota si existe
                            if entry.command.len() > 48 || !entry.note.is_empty() {
                                cmd_resp.on_hover_ui(|ui| {
                                    ui.set_max_width(480.0);
                                    ui.label(
                                        RichText::new(&entry.command)
                                            .monospace()
                                            .size(11.0)
                                            .color(TEXT_PRI),
                                    );
                                    if !entry.note.is_empty() {
                                        ui.separator();
                                        ui.label(
                                            RichText::new(&entry.note).size(11.0).color(C_WN_FG),
                                        );
                                    }
                                });
                            }

                            // Existe en disco
                            let (disk_txt, disk_col) = if entry.exists_on_disk {
                                ("✅ Sí", C_OK_FG)
                            } else {
                                ("❌ No", C_CR_FG)
                            };
                            ui.add_sized(
                                [64.0, 18.0],
                                egui::Label::new(
                                    RichText::new(disk_txt).size(11.5).color(disk_col),
                                ),
                            );
                        });
                    });
            }
        });

    // Nota informativa al pie
    ui.add_space(12.0);
    egui::Frame::none()
        .fill(C_BL_BG)
        .stroke(Stroke::new(1.0, C_BL_FG.linear_multiply(0.3)))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("ℹ").color(C_BL_FG).size(13.0));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(
                        "Las entradas de tipo Registro (Sistema) requieren privilegios \
                         de administrador para modificarse. \
                         Las entradas marcadas \"❌ No\" apuntan a archivos que ya no existen \
                         y pueden limpiarse de forma segura.",
                    )
                    .size(11.0)
                    .color(TEXT_SEC),
                );
            });
        });
}

// ── Tab: Acerca ────────────────────────────────────────────────────────────────

// ── Tab: Manual ────────────────────────────────────────────────────────────────

/// Callout informativo con fondo suave para el manual.
fn manual_note(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(C_BL_BG)
        .stroke(Stroke::new(1.0, C_BL_FG.linear_multiply(0.3)))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new(text).size(12.0).color(TEXT_SEC)).wrap(true));
        });
}

/// Entrada del manual: icono + título + descripción.
fn manual_item(ui: &mut egui::Ui, icon: &str, icon_color: Color32, title: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [26.0, 20.0],
            egui::Label::new(RichText::new(icon).size(15.0).color(icon_color)),
        );
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(13.0).strong().color(TEXT_PRI));
            ui.add(egui::Label::new(RichText::new(desc).size(11.5).color(TEXT_SEC)).wrap(true));
        });
    });
    ui.add_space(9.0);
}

/// Entrada del manual con el "porqué": icono + título + qué hace + por qué importa.
fn manual_item_why(
    ui: &mut egui::Ui,
    icon: &str,
    icon_color: Color32,
    title: &str,
    what: &str,
    why: &str,
) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [26.0, 20.0],
            egui::Label::new(RichText::new(icon).size(15.0).color(icon_color)),
        );
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(13.5).strong().color(TEXT_PRI));
            ui.add(egui::Label::new(RichText::new(what).size(11.5).color(TEXT_SEC)).wrap(true));
            ui.add(
                egui::Label::new(
                    RichText::new(format!("{} {}", tr("Por qué:", "Why:"), why))
                        .size(11.0)
                        .italics()
                        .color(TEXT_MUT),
                )
                .wrap(true),
            );
        });
    });
    ui.add_space(10.0);
}

fn draw_tab_manual(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new("📖").size(24.0));
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.label(
                RichText::new(tr("Manual de uso", "User manual"))
                    .size(20.0)
                    .strong()
                    .color(TEXT_PRI),
            );
            ui.label(
                RichText::new(tr(
                    "Qué hace cada parte y —sobre todo— por qué",
                    "What each part does and —above all— why",
                ))
                .size(12.0)
                .color(TEXT_MUT),
            );
        });
    });
    ui.add_space(10.0);

    manual_note(
        ui,
        tr(
            "RootCause es un monitor forense ligero para Windows. Su filosofía: diagnostica primero la \
             causa dominante de lentitud o comportamiento raro, y solo después actúa —siempre con \
             confirmación y registro. No es un \"limpiador mágico\": prioriza explicar la causa real. \
             Complementa, no reemplaza, a un antivirus o EDR.",
            "RootCause is a lightweight forensic monitor for Windows. Its philosophy: first diagnose \
             the dominant cause of slowness or odd behavior, and only then act —always with \
             confirmation and logging. It is not a \"magic cleaner\": it prioritizes explaining the \
             real cause. It complements, not replaces, an antivirus or EDR.",
        ),
    );
    ui.add_space(16.0);

    section_header(ui, tr("Léelo en 30 segundos", "Read it in 30 seconds"));
    ui.add_space(8.0);
    manual_note(
        ui,
        tr(
            "1) Mira el banner de veredicto del Resumen: verde = tranquilo. 2) Si es ámbar o rojo, \
             baja a \"Dónde mirar primero\": son las alertas ordenadas por importancia. 3) Abre \
             Procesos o Conexiones para ver el detalle. 4) Actúa solo si hace falta, con las acciones \
             seguras (siempre confirmadas). El porqué de este orden: evita que \"apagues\" algo antes \
             de entender qué lo causaba.",
            "1) Look at the verdict banner on Overview: green = relax. 2) If it's amber or red, scroll \
             to \"Where to look first\": alerts ranked by importance. 3) Open Processes or Connections \
             for detail. 4) Act only if needed, using the safe actions (always confirmed). Why this \
             order: it stops you from \"killing\" something before understanding what caused it.",
        ),
    );
    ui.add_space(16.0);

    section_header(ui, tr("Las pestañas", "The tabs"));
    ui.add_space(8.0);
    manual_item_why(
        ui,
        "📊",
        ACCENT,
        tr("Resumen", "Overview"),
        tr(
            "Banner de veredicto, salud 0–100, tarjetas de CPU/RAM/Disco/Red/Temporales y la lista \"Dónde mirar primero\".",
            "Verdict banner, 0–100 health, CPU/RAM/Disk/Network/Temp cards and the \"Where to look first\" list.",
        ),
        tr(
            "Es tu punto de partida: un vistazo decide si hace falta investigar o no.",
            "It's your starting point: one glance decides whether you need to investigate.",
        ),
    );
    manual_item_why(
        ui,
        "⚙",
        ACCENT,
        tr("Procesos", "Processes"),
        tr(
            "Procesos por severidad, con CPU, RAM, escritura de disco y score de riesgo. Puedes finalizar uno (con confirmación).",
            "Processes by severity, with CPU, RAM, disk writes and a risk score. You can terminate one (with confirmation).",
        ),
        tr(
            "El score combina varias señales, así lo peligroso sube arriba sin que revises 200 filas.",
            "The score combines several signals, so the dangerous ones rise to the top without scanning 200 rows.",
        ),
    );
    manual_item_why(
        ui,
        "🌐",
        ACCENT,
        tr("Conexiones", "Connections"),
        tr(
            "Conexiones de red activas por proceso (netstat enriquecido con nombre y ruta). Puedes bloquear una IP con el firewall.",
            "Active network connections per process (netstat enriched with name and path). You can block an IP via the firewall.",
        ),
        tr(
            "Saber QUÉ proceso habla con una IP pública es la mitad del diagnóstico de exfiltración o C2.",
            "Knowing WHICH process talks to a public IP is half the diagnosis of exfiltration or C2.",
        ),
    );
    manual_item_why(
        ui,
        "🗑",
        ACCENT,
        tr("Temporales / Almacenamiento", "Temporary / Storage"),
        tr(
            "Carpetas temporales que crecen (%TEMP%, Windows Temp, Windows Update) y ahora el espacio de Docker (imágenes, volúmenes, caché).",
            "Growing temp folders (%TEMP%, Windows Temp, Windows Update) and now Docker space (images, volumes, cache).",
        ),
        tr(
            "El disco lleno degrada TODO Windows; Docker suele ser el culpable oculto en equipos de desarrollo.",
            "A full disk degrades ALL of Windows; Docker is often the hidden culprit on developer machines.",
        ),
    );
    manual_item_why(
        ui,
        "🎯",
        ACCENT,
        "ETW / WPR",
        tr(
            "Modo de precisión: inicia/detiene/resume una traza ETL con Windows Performance Recorder y heurísticas locales.",
            "Precision mode: start/stop/summarize an ETL trace with Windows Performance Recorder and local heuristics.",
        ),
        tr(
            "Cuando las métricas no bastan, una traza ETW ve a nivel de kernel qué causó el pico exacto.",
            "When metrics aren't enough, an ETW trace sees at kernel level what caused the exact spike.",
        ),
    );
    manual_item_why(
        ui,
        "🔧",
        ACCENT,
        tr("Servicios", "Services"),
        tr(
            "Servicios de seguridad relevantes (Defender, Windows Update, BITS…) con su estado; los cambios se reportan como alertas.",
            "Relevant security services (Defender, Windows Update, BITS…) with their status; changes are reported as alerts.",
        ),
        tr(
            "Un servicio de seguridad detenido \"de repente\" es una señal clásica de compromiso.",
            "A security service that stops \"out of nowhere\" is a classic sign of compromise.",
        ),
    );
    manual_item_why(
        ui,
        "🚀",
        ACCENT,
        "Autostart",
        tr(
            "Todo lo que arranca con Windows: Registro Run/RunOnce, carpetas Startup y tareas programadas. Marca cambios vs baseline.",
            "Everything that starts with Windows: Run/RunOnce registry, Startup folders and scheduled tasks. Flags changes vs baseline.",
        ),
        tr(
            "La persistencia es cómo el malware sobrevive al reinicio; vigilar el autoarranque la delata.",
            "Persistence is how malware survives a reboot; watching autostart exposes it.",
        ),
    );
    manual_item_why(
        ui,
        "🕒",
        ACCENT,
        tr("Historial", "History"),
        tr(
            "Capturas guardadas localmente en SQLite. Compara dos momentos (A vs B) para ver la evolución.",
            "Snapshots stored locally in SQLite. Compare two moments (A vs B) to see the evolution.",
        ),
        tr(
            "\"Empezó ayer\" es una pista enorme: comparar A/B convierte una corazonada en evidencia.",
            "\"It started yesterday\" is a huge clue: comparing A/B turns a hunch into evidence.",
        ),
    );
    manual_item_why(
        ui,
        "⚙",
        ACCENT,
        tr("Configuración", "Settings"),
        tr(
            "Idioma (español / inglés), umbrales de detección, anomalías e intervalo de refresco. Se guarda sin reiniciar.",
            "Language (Spanish / English), detection thresholds, anomalies and refresh interval. Saved without restarting.",
        ),
        tr(
            "Cada equipo tiene un \"normal\" distinto; ajustar umbrales evita falsos positivos o puntos ciegos.",
            "Every machine has a different \"normal\"; tuning thresholds avoids false positives or blind spots.",
        ),
    );
    manual_item(
        ui,
        "📖",
        ACCENT,
        tr("Manual", "Manual"),
        tr("Esta pantalla.", "This screen."),
    );
    manual_item(
        ui,
        "ℹ",
        ACCENT,
        tr("Acerca", "About"),
        tr(
            "Versión, autor, stack técnico, atajos y salud del propio agente.",
            "Version, author, tech stack, shortcuts and the agent's own health.",
        ),
    );

    ui.add_space(16.0);
    section_header(
        ui,
        tr(
            "Detección de cambios (baseline)",
            "Change detection (baseline)",
        ),
    );
    ui.add_space(8.0);
    manual_note(
        ui,
        tr(
            "RootCause guarda una \"foto de referencia\" (estado bueno conocido) de tu autoarranque y de \
             tus servicios. La primera vez se siembra en silencio. Después, cualquier cambio se marca como \
             NUEVA, MODIFICADA o ELIMINADA y genera una alerta, hasta que aceptas la nueva baseline. El \
             porqué: el malware no siempre consume CPU —a veces solo AÑADE una entrada de arranque y \
             espera. Comparar contra un estado bueno conocido lo delata aunque sea sigiloso.",
            "RootCause keeps a \"reference snapshot\" (known-good state) of your autostart and your \
             services. The first time it is seeded silently. Afterwards, any change is flagged as NEW, \
             MODIFIED or REMOVED and raises an alert until you accept the new baseline. The why: malware \
             doesn't always burn CPU —sometimes it just ADDS a startup entry and waits. Comparing against \
             a known-good state exposes it even when it's stealthy.",
        ),
    );

    ui.add_space(16.0);
    section_header(ui, tr("Docker (liberar disco)", "Docker (free disk)"));
    ui.add_space(8.0);
    manual_note(
        ui,
        tr(
            "En la pestaña Temporales, \"Escanear Docker\" muestra imágenes, volúmenes y espacio \
             recuperable. La purga segura solo borra lo regenerable: imágenes colgantes (sin etiqueta) y \
             caché de build. Los volúmenes NO se borran desde la app —contienen datos (bases de datos, \
             etc.)— y se listan para que decidas tú. El porqué de esta línea: liberar espacio nunca debe \
             costarte datos que no sabías que importaban.",
            "In the Storage tab, \"Scan Docker\" shows images, volumes and reclaimable space. Safe cleanup \
             only removes regenerable data: dangling (untagged) images and build cache. Volumes are NOT \
             deleted from the app —they hold data (databases, etc.)— and are listed so you decide. The why \
             behind this line: freeing space must never cost you data you didn't know mattered.",
        ),
    );

    ui.add_space(16.0);
    section_header(
        ui,
        tr(
            "Acciones seguras (siempre auditadas)",
            "Safe actions (always audited)",
        ),
    );
    ui.add_space(8.0);
    manual_item(
        ui,
        "•",
        C_BL_FG,
        tr("Finalizar proceso", "Terminate process"),
        tr(
            "Termina un proceso por PID. Nunca finaliza procesos críticos del sistema.",
            "Ends a process by PID. Never terminates critical system processes.",
        ),
    );
    manual_item(
        ui,
        "•",
        C_BL_FG,
        tr("Bloquear IP", "Block IP"),
        tr(
            "Crea una regla de firewall para una IP remota.",
            "Creates a firewall rule for a remote IP.",
        ),
    );
    manual_item(
        ui,
        "•",
        C_BL_FG,
        tr("Detener servicio", "Stop service"),
        tr(
            "Solo servicios de una lista permitida (bits, dosvc, sysmain, wuauserv).",
            "Only services from an allow-list (bits, dosvc, sysmain, wuauserv).",
        ),
    );
    manual_item(
        ui,
        "•",
        C_BL_FG,
        tr("Limpiar %TEMP% / Docker", "Clean %TEMP% / Docker"),
        tr(
            "Borra lo no usado de %TEMP% (>24h) y purga dangling/caché de Docker. Confirmación de 2 pasos.",
            "Removes unused %TEMP% (>24h) and prunes Docker dangling/cache. Two-step confirmation.",
        ),
    );
    manual_item(
        ui,
        "•",
        C_BL_FG,
        tr("Aceptar baseline", "Accept baseline"),
        tr(
            "Marca el estado actual de autostart o servicios como el nuevo \"bueno conocido\".",
            "Marks the current autostart or services state as the new \"known-good\".",
        ),
    );

    ui.add_space(16.0);
    section_header(ui, tr("Colores de severidad", "Severity colors"));
    ui.add_space(8.0);
    manual_item(
        ui,
        "•",
        C_OK_FG,
        tr("Verde — Saludable", "Green — Healthy"),
        tr(
            "Sin señales fuertes; comportamiento normal.",
            "No strong signals; normal behavior.",
        ),
    );
    manual_item(
        ui,
        "•",
        C_WN_FG,
        tr("Ámbar — Advertencia", "Amber — Warning"),
        tr(
            "Vale la pena revisar; consumo o cambios notables.",
            "Worth reviewing; notable usage or changes.",
        ),
    );
    manual_item(
        ui,
        "•",
        C_CR_FG,
        tr("Rojo — Crítico", "Red — Critical"),
        tr(
            "Señal fuerte: prioriza la revisión (proceso, conexión o cambio sospechoso).",
            "Strong signal: prioritize review (suspicious process, connection or change).",
        ),
    );

    ui.add_space(16.0);
    section_header(ui, tr("Desde la consola (CLI)", "From the console (CLI)"));
    ui.add_space(8.0);
    manual_note(
        ui,
        tr(
            "Todo funciona también sin interfaz. `rootcause --help` lista los comandos: status, snapshot, \
             history, autostart, services, clean-temp, docker, wpr, kill, block-ip, stop-service, config, \
             ai. El porqué: en servidores sin escritorio o dentro de scripts, el diagnóstico debe seguir \
             estando a un comando de distancia.",
            "Everything works without a GUI too. `rootcause --help` lists the commands: status, snapshot, \
             history, autostart, services, clean-temp, docker, wpr, kill, block-ip, stop-service, config, \
             ai. The why: on headless servers or inside scripts, diagnostics must stay one command away.",
        ),
    );

    ui.add_space(16.0);
    section_header(ui, tr("Privacidad", "Privacy"));
    ui.add_space(8.0);
    manual_note(
        ui,
        tr(
            "Todo es local: telemetría cero. El historial se guarda solo en tu equipo (SQLite). El \
             adaptador de IA es opcional y viene apagado por defecto. El porqué: una herramienta forense \
             que filtrara datos sería una contradicción.",
            "Everything is local: zero telemetry. History is stored only on your machine (SQLite). The AI \
             adapter is optional and off by default. The why: a forensic tool that leaked data would be a \
             contradiction.",
        ),
    );
    ui.add_space(24.0);
}

// ── Tab: Configuración ───────────────────────────────────────────────────────────

fn draw_tab_config(
    ui: &mut egui::Ui,
    cfg: &mut RootCauseConfig,
    config_path: &str,
    save_requested: &mut bool,
) {
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(14.0))
            .inner_margin(Margin::same(28.0))
            .show(ui, |ui| {
                ui.set_max_width(620.0);

                // Título
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚙").size(22.0).color(ACCENT));
                    ui.add_space(6.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(tr("Configuración", "Settings"))
                                .size(20.0)
                                .strong()
                                .color(TEXT_PRI),
                        );
                        ui.label(
                            RichText::new(tr(
                                "Idioma, umbrales de detección y comportamiento",
                                "Language, detection thresholds and behavior",
                            ))
                            .size(12.0)
                            .color(TEXT_MUT),
                        );
                    });
                });

                ui.add_space(16.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Idioma ────────────────────────────────────────────────────
                section_header(ui, tr("▸  Idioma", "▸  Language"));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(tr(
                        "Cambia al instante toda la interfaz. Se guarda automáticamente.",
                        "Switches the whole interface instantly. Saved automatically.",
                    ))
                    .size(11.0)
                    .color(TEXT_MUT),
                );
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    for lang in [Lang::Es, Lang::En] {
                        let selected = cfg.ui.language == lang;
                        let (fg, bg) = if selected {
                            (TEXT_PRI, ACCENT)
                        } else {
                            (TEXT_SEC, BG_PANEL)
                        };
                        // Sin banderas emoji: la NotoEmoji empaquetada no las trae y
                        // saldrían como tofu. La selección se indica con el fondo ACCENT.
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!(
                                        "{}  ({})",
                                        lang.native_name(),
                                        lang.code().to_uppercase()
                                    ))
                                    .size(12.5)
                                    .color(fg),
                                )
                                .fill(bg)
                                .stroke(Stroke::new(1.0, BORDER))
                                .min_size(Vec2::new(150.0, 30.0))
                                .rounding(Rounding::same(6.0)),
                            )
                            .clicked()
                        {
                            cfg.ui.language = lang;
                        }
                        ui.add_space(6.0);
                    }
                });

                ui.add_space(16.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Archivo de configuración ──────────────────────────────────
                section_header(
                    ui,
                    tr("▸  Archivo de configuración", "▸  Configuration file"),
                );
                ui.add_space(8.0);
                {
                    let path_short = if config_path.is_empty() {
                        tr("No disponible", "Not available").to_owned()
                    } else {
                        trunc(config_path, 60)
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.add_sized(
                            [120.0, 18.0],
                            egui::Label::new(
                                RichText::new(tr("Ruta", "Path")).size(12.0).color(TEXT_MUT),
                            ),
                        );
                        let resp = ui.label(
                            RichText::new(&path_short)
                                .size(11.5)
                                .monospace()
                                .color(TEXT_SEC),
                        );
                        if config_path.len() > 60 {
                            resp.on_hover_text(config_path);
                        }
                        if !config_path.is_empty()
                            && action_btn(ui, tr("Abrir", "Open"), C_BL_BG, C_BL_FG).clicked()
                        {
                            let _ = windows::powershell(&format!(
                                "Start-Process notepad.exe '{}'",
                                config_path.replace('\'', "''")
                            ));
                        }
                    });
                }

                ui.add_space(16.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Umbrales de procesos ──────────────────────────────────────
                section_header(
                    ui,
                    tr("▸  Umbrales — Procesos", "▸  Thresholds — Processes"),
                );
                ui.add_space(8.0);
                {
                    let th = &mut cfg.thresholds.process;
                    threshold_row(ui, "CPU warning", &mut th.cpu_warning_percent, "%", C_WN_FG);
                    threshold_row(
                        ui,
                        "CPU crítico",
                        &mut th.cpu_critical_percent,
                        "%",
                        C_CR_FG,
                    );
                    threshold_row(ui, "RAM warning", &mut th.memory_warning_mb, "MB", C_WN_FG);
                    threshold_row(ui, "RAM crítico", &mut th.memory_critical_mb, "MB", C_CR_FG);
                    threshold_row(
                        ui,
                        "I/O warning",
                        &mut th.io_write_warning_mb,
                        "MB/s",
                        C_WN_FG,
                    );
                    threshold_row(
                        ui,
                        "I/O crítico",
                        &mut th.io_write_critical_mb,
                        "MB/s",
                        C_CR_FG,
                    );
                }

                ui.add_space(14.0);

                // ── Detección de anomalías ────────────────────────────────────
                section_header(ui, tr("▸  Detección de anomalías", "▸  Anomaly detection"));
                ui.add_space(8.0);
                {
                    let an = &mut cfg.anomaly;
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            [120.0, 18.0],
                            egui::Label::new(
                                RichText::new(tr("Estado", "Status"))
                                    .size(12.0)
                                    .color(TEXT_MUT),
                            ),
                        );
                        let label = if an.enabled {
                            tr("Habilitada", "Enabled")
                        } else {
                            tr("Deshabilitada", "Disabled")
                        };
                        let color = if an.enabled { C_OK_FG } else { TEXT_MUT };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(label).size(11.5).color(color))
                                    .min_size(Vec2::new(110.0, 18.0)),
                            )
                            .on_hover_text(tr(
                                "Click para activar / desactivar",
                                "Click to enable / disable",
                            ))
                            .clicked()
                        {
                            an.enabled = !an.enabled;
                        }
                    });
                    ui.add_space(3.0);
                    threshold_row(
                        ui,
                        "CPU sostenida",
                        &mut an.cpu_sustained_percent,
                        "%",
                        TEXT_SEC,
                    );
                    threshold_row(
                        ui,
                        "RAM crecimiento",
                        &mut an.memory_growth_mb,
                        "MB",
                        TEXT_SEC,
                    );
                    threshold_row(
                        ui,
                        "Escritura agres.",
                        &mut an.aggressive_write_mb,
                        "MB/s",
                        TEXT_SEC,
                    );
                }
                {
                    let mut secs = cfg.collection.refresh_interval_secs as f32;
                    threshold_row(
                        ui,
                        tr("Refresco UI", "UI refresh"),
                        &mut secs,
                        "s",
                        TEXT_SEC,
                    );
                    cfg.collection.refresh_interval_secs = secs.max(1.0) as u64;
                }

                ui.add_space(16.0);

                // Botón Guardar
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new(format!("💾  {}", tr("Guardar", "Save")))
                                    .size(12.5)
                                    .color(C_OK_FG),
                            )
                            .min_size(Vec2::new(130.0, 30.0))
                            .fill(C_OK_BG),
                        )
                        .on_hover_text(tr(
                            "Persiste los cambios en el JSON y los aplica sin reiniciar",
                            "Persists changes to the JSON and applies them without restarting",
                        ))
                        .clicked()
                    {
                        *save_requested = true;
                    }
                    ui.label(
                        RichText::new(tr(
                            "Los cambios se aplican en la próxima captura.",
                            "Changes apply on the next capture.",
                        ))
                        .size(11.0)
                        .color(TEXT_MUT),
                    );
                });
            });
    });
    ui.add_space(24.0);
}

fn draw_tab_about(ui: &mut egui::Ui, hw: &HardwareInfo, snapshot: Option<&SystemSnapshot>) {
    ui.add_space(28.0);

    ui.vertical_centered(|ui| {
        // ── Tarjeta principal ─────────────────────────────────────────────────
        egui::Frame::none()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER))
            .rounding(Rounding::same(14.0))
            .inner_margin(Margin::same(32.0))
            .show(ui, |ui| {
                ui.set_max_width(620.0);

                // Logo + nombre
                ui.horizontal(|ui| {
                    draw_logo_icon(ui, 56.0);
                    ui.add_space(18.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(meta::DISPLAY_NAME)
                                .size(22.0)
                                .strong()
                                .color(TEXT_PRI),
                        );
                        ui.add_space(2.0);
                        pill(
                            ui,
                            &format!("v{}  ·  {}", meta::VERSION, meta::LICENSE),
                            C_BL_FG,
                            C_BL_BG,
                        );
                    });
                });

                ui.add_space(16.0);
                ui.label(RichText::new(meta::DESCRIPTION).size(13.0).color(TEXT_SEC));

                if let Some(snap) = snapshot {
                    ui.add_space(18.0);
                    ui.add(egui::Separator::default());
                    ui.add_space(14.0);

                    section_header(ui, "▸  Salud del agente");
                    ui.add_space(10.0);
                    draw_agent_health_block(ui, &snap.agent_health);
                }

                ui.add_space(18.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Contacto ─────────────────────────────────────────────────
                section_header(ui, "▸  Autor y contacto");
                ui.add_space(10.0);

                about_row(ui, "Autor", meta::AUTHOR, TEXT_PRI);
                if !meta::EMAIL.is_empty() {
                    about_link_row(ui, "Email", &format!("mailto:{}", meta::EMAIL), meta::EMAIL);
                }
                about_link_row(ui, "GitHub", meta::GITHUB, meta::GITHUB);
                if !meta::GITLAB.is_empty() {
                    about_link_row(ui, "GitLab", meta::GITLAB, meta::GITLAB);
                }

                ui.add_space(18.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Stack técnico ─────────────────────────────────────────────
                section_header(ui, "▸  Stack técnico");
                ui.add_space(10.0);

                about_row(ui, "Lenguaje", "Rust 2024 edition", TEXT_SEC);
                about_row(ui, "GUI", "eframe / egui 0.27  ·  modo inmediato", TEXT_SEC);
                about_row(
                    ui,
                    "Persistencia",
                    "SQLite vía rusqlite  ·  bundled",
                    TEXT_SEC,
                );
                about_row(ui, "Métricas", "sysinfo  ·  bajo consumo", TEXT_SEC);
                about_row(
                    ui,
                    "Integración Windows",
                    "PowerShell · netstat · WPR · tracerpt",
                    TEXT_SEC,
                );
                about_row(ui, "Plataforma", "Windows 10 / 11  ·  x64", TEXT_SEC);
                about_row(ui, "CI/CD", "GitHub Actions  ·  windows-latest", TEXT_SEC);

                ui.add_space(18.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Nota final ────────────────────────────────────────────────
                ui.label(
                    RichText::new(
                        "Diagnóstico primero. Intervención después.  \
                         No intenta ser un limpiador mágico — busca explicar la causa real.",
                    )
                    .size(12.5)
                    .italics()
                    .color(TEXT_MUT),
                );

                ui.add_space(8.0);

                // CLI hint
                egui::Frame::none()
                    .fill(BG_PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("  $ rootcause --help")
                                .monospace()
                                .size(12.0)
                                .color(C_OK_FG),
                        );
                        ui.label(
                            RichText::new(
                                "Disponible también como herramienta de línea de comandos",
                            )
                            .size(11.0)
                            .color(TEXT_MUT),
                        );
                    });

                ui.add_space(18.0);
                ui.add(egui::Separator::default());
                ui.add_space(14.0);

                // ── Atajos de teclado ─────────────────────────────────────────
                section_header(ui, "▸  Atajos de teclado");
                ui.add_space(10.0);

                for (shortcut, action) in [
                    ("F5", "Actualizar ahora"),
                    ("Ctrl + E", "Exportar snapshot a JSON"),
                    ("Ctrl + 1", "Ir a Resumen"),
                    ("Ctrl + 2", "Ir a Procesos"),
                    ("Ctrl + 3", "Ir a Conexiones"),
                    ("Ctrl + 4", "Ir a Temporales"),
                    ("Ctrl + 5", "Ir a ETW / WPR"),
                    ("Ctrl + 6", "Ir a Servicios"),
                    ("Ctrl + 7", "Ir a Autostart"),
                    ("Ctrl + 8", "Ir a Historial"),
                    ("Ctrl + 9", "Ir a Configuración"),
                    ("Ctrl + 0", "Ir a Manual"),
                ] {
                    ui.horizontal(|ui| {
                        egui::Frame::none()
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, BORDER))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::symmetric(8.0, 2.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(shortcut)
                                        .monospace()
                                        .size(11.5)
                                        .color(C_BL_FG),
                                );
                            });
                        ui.add_space(6.0);
                        ui.label(RichText::new(action).size(12.0).color(TEXT_SEC));
                    });
                    ui.add_space(3.0);
                }

                // ── Hardware del equipo ───────────────────────────────────────
                if !hw.host_name.is_empty() || !hw.cpu_brand.is_empty() {
                    ui.add_space(18.0);
                    ui.add(egui::Separator::default());
                    ui.add_space(14.0);

                    section_header(ui, "▸  Este equipo");
                    ui.add_space(10.0);

                    about_row(ui, "Nombre", &hw.host_name, TEXT_SEC);
                    about_row(ui, "Sistema", &hw.os_name, TEXT_SEC);
                    about_row(ui, "Versión OS", &hw.os_version, TEXT_SEC);
                    about_row(ui, "Arquitectura", &hw.architecture, TEXT_SEC);
                    about_row(
                        ui,
                        "CPU",
                        &format!("{}  ·  {} núcleos", hw.cpu_brand, hw.cpu_cores),
                        TEXT_SEC,
                    );
                    if hw.cpu_freq_mhz > 0 {
                        about_row(
                            ui,
                            "Frecuencia",
                            &format!("{:.1} GHz", hw.cpu_freq_mhz as f32 / 1000.0),
                            TEXT_SEC,
                        );
                    }
                    about_row(
                        ui,
                        "RAM total",
                        &format!("{:.1} GB", hw.total_ram_gb),
                        TEXT_SEC,
                    );
                }

                ui.add_space(6.0);
                ui.label(
                    RichText::new(tr(
                        "Ajustes y umbrales: pestaña Configuración (Ctrl+9).",
                        "Settings and thresholds: Settings tab (Ctrl+9).",
                    ))
                    .size(11.0)
                    .italics()
                    .color(TEXT_MUT),
                );
            });
    });
}

/// Fila de dato de hardware con etiqueta fija y valor.
fn hw_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [140.0, 18.0],
            egui::Label::new(RichText::new(label).size(11.5).color(TEXT_MUT)),
        );
        ui.label(RichText::new(value).size(11.5).color(TEXT_SEC));
    });
    ui.add_space(3.0);
}

/// Fila de información en el panel Acerca.
fn about_row(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [120.0, 18.0],
            egui::Label::new(RichText::new(label).size(12.0).color(TEXT_MUT)),
        );
        ui.label(RichText::new(value).size(12.0).color(color));
    });
    ui.add_space(3.0);
}

/// Fila editable con etiqueta fija + DragValue + unidad.  Usa f32 multiplicado × 1.0.
fn threshold_row(ui: &mut egui::Ui, label: &str, value: &mut f32, unit: &str, color: Color32) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [120.0, 18.0],
            egui::Label::new(RichText::new(label).size(12.0).color(TEXT_MUT)),
        );
        ui.add(
            egui::DragValue::new(value)
                .speed(1.0)
                .clamp_range(0.0_f32..=100_000.0)
                .suffix(format!(" {unit}"))
                .min_decimals(0)
                .max_decimals(0),
        )
        .on_hover_text("Arrastra para cambiar · Click y escribe el valor");
        ui.label(RichText::new("").color(color));
    });
    ui.add_space(3.0);
}

fn draw_agent_health_block(ui: &mut egui::Ui, health: &AgentHealth) {
    let (fg, bg) = match health.status {
        AgentStatus::Healthy => (C_OK_FG, C_OK_BG),
        AgentStatus::Recovered => (C_WN_FG, C_WN_BG),
        AgentStatus::Degraded => (C_CR_FG, C_CR_BG),
    };

    ui.horizontal(|ui| {
        pill(ui, health.status.label(), fg, bg);
        if health.watchdog_backoff_active {
            pill(ui, "Backoff sugerido", C_WN_FG, C_WN_BG);
        }
        if health.config_changed {
            pill(ui, "Config cambiada", C_BL_FG, C_BL_BG);
        }
    });
    ui.add_space(6.0);
    ui.label(RichText::new(&health.summary).size(12.0).color(TEXT_SEC));
    ui.add_space(8.0);
    about_row(ui, "Ultimo inicio", &health.last_start_at, TEXT_SEC);
    about_row(ui, "Ultimo heartbeat", &health.last_heartbeat_at, TEXT_SEC);
    if let Some(last_shutdown) = health.last_clean_shutdown_at.as_ref() {
        about_row(ui, "Ultimo cierre limpio", last_shutdown, TEXT_SEC);
    }
    about_row(ui, "Huella config", &health.config_fingerprint, TEXT_MUT);
    for note in health.notes.iter().take(3) {
        ui.label(
            RichText::new(format!("• {note}"))
                .size(11.5)
                .color(TEXT_MUT),
        );
    }
}

/// Fila de enlace clickable en el panel Acerca.
/// Abre la URL con el shell de Windows (`cmd /c start ""`).
fn about_link_row(ui: &mut egui::Ui, label: &str, url: &str, display: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [120.0, 18.0],
            egui::Label::new(RichText::new(label).size(12.0).color(TEXT_MUT)),
        );
        let resp = ui.add(
            egui::Button::new(RichText::new(display).size(12.0).color(C_BL_FG))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::NONE),
        );
        if resp.on_hover_text("Abrir en el navegador").clicked() {
            let _ = std::process::Command::new("cmd")
                .args(["/c", "start", "", url])
                .spawn();
        }
    });
    ui.add_space(3.0);
}

// ── Widgets de UI reutilizables ────────────────────────────────────────────────

/// Carga fuentes nativas de Windows para un aspecto de producto Windows 11:
/// **Segoe UI** como fuente proporcional principal y **Consolas** como monoespaciada.
/// Se conservan las fuentes por defecto de egui como respaldo (incluida NotoEmoji,
/// necesaria para los emoji de la UI). Si los archivos no existen (p. ej. build
/// no-Windows), no se cambia nada y se usan las fuentes por defecto.
fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut changed = false;

    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe_ui".to_owned(), egui::FontData::from_owned(bytes));
        if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            fam.insert(0, "segoe_ui".to_owned());
        }
        changed = true;
    }
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\consola.ttf") {
        fonts
            .font_data
            .insert("consolas".to_owned(), egui::FontData::from_owned(bytes));
        if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
            fam.insert(0, "consolas".to_owned());
        }
        changed = true;
    }

    if changed {
        ctx.set_fonts(fonts);
    }
}

fn apply_theme(ctx: &egui::Context) {
    let mut vis = egui::Visuals::dark();
    vis.window_fill = BG_APP;
    vis.panel_fill = BG_APP;
    vis.faint_bg_color = BG_CARD;
    vis.extreme_bg_color = BG_PANEL;
    vis.widgets.noninteractive.bg_fill = BG_CARD;
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SEC);
    vis.widgets.noninteractive.rounding = Rounding::same(4.0);
    vis.widgets.inactive.bg_fill = BG_CARD;
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SEC);
    vis.widgets.inactive.rounding = Rounding::same(4.0);
    vis.widgets.hovered.bg_fill = Color32::from_rgb(38, 46, 57);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRI);
    vis.widgets.hovered.rounding = Rounding::same(4.0);
    vis.widgets.active.bg_fill = ACCENT;
    vis.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRI);
    vis.widgets.active.rounding = Rounding::same(4.0);
    vis.selection.bg_fill = ACCENT.linear_multiply(0.35);
    vis.selection.stroke = Stroke::new(1.0, C_BL_FG);
    vis.window_rounding = Rounding::same(8.0);
    vis.window_stroke = Stroke::new(1.0, BORDER);
    vis.override_text_color = Some(TEXT_PRI);
    ctx.set_visuals(vis);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.window_margin = Margin::same(12.0);
    // Barra de scroll SÓLIDA y siempre visible (reserva espacio y dibuja pista +
    // tirador). La barra flotante por defecto es casi invisible sobre el fondo
    // oscuro y hacía creer que tabs como Resumen "no tienen scroll". Además se
    // pinta el tirador con color de primer plano (claro) y opacidad alta, porque
    // el color de fondo por defecto (BG_CARD sobre BG_PANEL) es casi indistinguible
    // del fondo de la app.
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.bar_width = 12.0;
    scroll.handle_min_length = 24.0;
    scroll.foreground_color = true;
    scroll.dormant_handle_opacity = 0.7;
    scroll.active_handle_opacity = 0.9;
    scroll.interact_handle_opacity = 1.0;
    scroll.dormant_background_opacity = 0.4;
    scroll.active_background_opacity = 0.5;
    style.spacing.scroll = scroll;
    ctx.set_style(style);
}

/// Botón de tab con indicador de selección.
/// Botón de acción primario.
fn action_btn(ui: &mut egui::Ui, label: &str, bg: Color32, fg: Color32) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(12.0).color(fg))
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.45)))
            .rounding(Rounding::same(5.0)),
    )
}

/// Botón de acción en el header (icono + texto).
fn header_btn(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(
            RichText::new(format!("{icon}  {label}"))
                .size(12.5)
                .color(C_BL_FG),
        )
        .fill(C_BL_BG)
        .stroke(Stroke::new(1.0, C_BL_FG.linear_multiply(0.4)))
        .rounding(Rounding::same(5.0)),
    )
}

/// Píldora de texto con fondo coloreado.
fn pill(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    let w = (text.chars().count() as f32 * 6.8 + 14.0).max(28.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 20.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(10.0), bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        fg,
    );
}

/// Badge de alerta con borde.
fn alert_badge(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    let w = (text.chars().count() as f32 * 7.0 + 18.0).max(40.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(11.0), bg);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(11.0),
        Stroke::new(1.0, fg.linear_multiply(0.6)),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.0),
        fg,
    );
}

/// Barra de progreso horizontal delgada.
fn pbar(ui: &mut egui::Ui, fraction: f32, color: Color32, width: f32) {
    let h = 7.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, h), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(3.5), BG_PANEL);
    if fraction > 0.005 {
        let filled_w = (rect.width() * fraction.clamp(0.0, 1.0)).max(6.0);
        let filled = egui::Rect::from_min_size(rect.min, Vec2::new(filled_w, h));
        ui.painter().rect_filled(filled, Rounding::same(3.5), color);
    }
}

/// Card de métrica con barra de progreso.
fn health_score_card(
    ui: &mut egui::Ui,
    score_fraction: f32,
    score_label: &str,
    score_fg: Color32,
    score_bg: Color32,
    width: f32,
) {
    egui::Frame::none()
        .fill(score_bg)
        .stroke(Stroke::new(1.5, score_fg.linear_multiply(0.5)))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(width, 120.0));
            ui.set_max_width(width);
            ui.vertical_centered(|ui| {
                draw_health_ring(ui, score_fraction, score_fg, 54.0);
                ui.add_space(4.0);
                ui.label(
                    RichText::new(score_label)
                        .size(13.0)
                        .strong()
                        .color(score_fg),
                );
                ui.label(
                    RichText::new("Salud del sistema")
                        .size(10.0)
                        .color(TEXT_MUT),
                );
            });
        });
}

fn overview_card(
    ui: &mut egui::Ui,
    title: &str,
    value: &str,
    subtitle: &str,
    fraction: f32,
    severity: Severity,
    width: f32,
) {
    let fg = sev_fg(severity);
    let bg = sev_bg(severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(width, 90.0));
            ui.set_max_width(width);
            // ui.vertical: el Frame hereda el layout del padre (aquí horizontal_wrapped),
            // que apilaría las etiquetas en fila y las solaparía. Forzamos columna.
            ui.vertical(|ui| {
                ui.label(RichText::new(title).size(10.0).color(TEXT_MUT).strong());
                ui.add_space(4.0);
                ui.label(RichText::new(value).size(17.0).strong().color(TEXT_PRI));
                ui.add(
                    egui::Label::new(RichText::new(subtitle).size(10.5).color(TEXT_MUT)).wrap(true),
                );
                ui.add_space(6.0);
                pbar(ui, fraction.clamp(0.0, 1.0), fg, ui.available_width() - 2.0);
            });
        });
}

/// Mini card de proceso para el overview.
fn mini_process_card(ui: &mut egui::Ui, p: &ProcessInsight, width: f32) {
    let fg = sev_fg(p.severity);
    let bg = sev_bg(p.severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(width, 80.0));
            ui.horizontal(|ui| {
                draw_proc_icon(ui, p.severity, 28.0);
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    let name_short = trunc(&p.name, 20);
                    let resp = ui.label(RichText::new(&name_short).strong().color(fg));
                    if p.name.len() > 20 {
                        resp.on_hover_text(&p.name);
                    }
                    ui.label(
                        RichText::new(format!("PID {}", p.pid))
                            .size(11.0)
                            .color(TEXT_MUT),
                    );
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("CPU {:.1}%", p.cpu_percent))
                        .size(11.5)
                        .color(fg),
                );
                ui.label(
                    RichText::new(format!("RAM {:.0}MB", p.memory_mb))
                        .size(11.5)
                        .color(TEXT_SEC),
                );
            });
            pbar(ui, p.cpu_percent / 100.0, fg, ui.available_width() - 2.0);
        });
}

fn anomaly_summary_card(ui: &mut egui::Ui, anomaly: &AnomalyEvent, width: f32) {
    let sev = anomaly.severity.to_severity();
    let fg = sev_fg(sev);
    let bg = sev_bg(sev);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            // El Frame hereda el layout del contenedor padre (horizontal_wrapped),
            // así que sin esto los Label.wrap(true) envuelven al ancho de toda la
            // fila y el texto se desborda. Un layout vertical acotado a `width`
            // fuerza el wrap al ancho real de la tarjeta.
            ui.vertical(|ui| {
                ui.set_min_width(width);
                ui.set_max_width(width);
                ui.set_min_height(125.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&anomaly.title).strong().color(fg));
                    alert_badge(ui, anomaly.severity.label(), fg, BG_CARD);
                    pill(ui, &format!("Score {}", anomaly.score), TEXT_MUT, BG_CARD);
                });
                if let Some(name) = anomaly.process_name.as_ref() {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!(
                                "{}{}",
                                name,
                                anomaly
                                    .pid
                                    .map(|pid| format!(" (PID {pid})"))
                                    .unwrap_or_default()
                            ))
                            .size(11.5)
                            .color(TEXT_SEC),
                        )
                        .wrap(true),
                    );
                }
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(RichText::new(&anomaly.summary).size(11.5).color(TEXT_SEC))
                        .wrap(true),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("Hipotesis: {}", anomaly.root_cause_hypothesis))
                            .size(11.0)
                            .color(TEXT_MUT),
                    )
                    .wrap(true),
                );
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(&anomaly.recommended_action)
                            .italics()
                            .size(11.0)
                            .color(TEXT_MUT),
                    )
                    .wrap(true),
                );
            });
        });
}

/// Cabecera de tabla con columnas.
fn table_header(ui: &mut egui::Ui, cols: &[(&str, f32)]) {
    egui::Frame::none()
        .fill(BG_PANEL)
        .inner_margin(Margin::symmetric(6.0, 6.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for &(hdr, w) in cols {
                    if w > 0.0 {
                        ui.add_sized(
                            [w, 16.0],
                            egui::Label::new(
                                RichText::new(hdr).size(11.0).strong().color(TEXT_MUT),
                            ),
                        );
                    } else {
                        ui.label(RichText::new(hdr).size(11.0).strong().color(TEXT_MUT));
                    }
                }
            });
        });
}

/// Tarjeta de sparkline con fondo, label y valor actual.
fn sparkline_card(ui: &mut egui::Ui, label: &str, values: &[f32], color: Color32, width: f32) {
    let height = 52.0;
    egui::Frame::none()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(8.0))
        .show(ui, |ui| {
            // Acotar a `width`: el Frame hereda el horizontal_wrapped del padre, así
            // que sin esto `available_width()` (usado para dibujar la línea) devuelve
            // el ancho de toda la fila y el primer sparkline se dibuja gigante.
            ui.vertical(|ui| {
                ui.set_min_width(width);
                ui.set_max_width(width);
                ui.set_min_height(height);
                // Label + último valor
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(10.0).color(TEXT_MUT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let last = values.last().copied().unwrap_or(0.0);
                        ui.label(
                            RichText::new(format!("{last:.1}"))
                                .size(12.0)
                                .strong()
                                .color(color),
                        );
                    });
                });
                ui.add_space(2.0);

                // Dibujar la línea
                let available = ui.available_width().max(40.0);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(available, 26.0), Sense::hover());
                ui.painter()
                    .rect_filled(rect, Rounding::same(3.0), BG_PANEL);

                if values.len() >= 2 {
                    let max_val = values.iter().cloned().fold(0.0_f32, f32::max).max(1.0);
                    let n = values.len();
                    let step = rect.width() / (n - 1).max(1) as f32;
                    let pts: Vec<egui::Pos2> = values
                        .iter()
                        .enumerate()
                        .map(|(i, &v)| {
                            let x = rect.left() + i as f32 * step;
                            let y = rect.bottom() - (v / max_val).clamp(0.0, 1.0) * rect.height();
                            egui::pos2(x, y)
                        })
                        .collect();
                    for w in pts.windows(2) {
                        ui.painter()
                            .line_segment([w[0], w[1]], Stroke::new(1.5, color));
                    }
                    // Punto actual
                    if let Some(&last_pt) = pts.last() {
                        ui.painter().circle_filled(last_pt, 2.5, color);
                    }
                }
            });
        });
}

/// Encabezado de sección con línea separadora.
fn section_header(ui: &mut egui::Ui, title: &str) {
    // Quita un posible glifo/icono al inicio del título (varios traían símbolos
    // Unicode que la fuente no renderiza y salían como "□"); se reemplaza por una
    // barra de acento sólida, consistente en todas las secciones.
    let clean = title
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim();
    ui.horizontal(|ui| {
        let (bar, _) = ui.allocate_exact_size(Vec2::new(3.0, 14.0), egui::Sense::hover());
        ui.painter().rect_filled(bar, Rounding::same(1.5), ACCENT);
        ui.add_space(7.0);
        ui.label(RichText::new(clean).strong().size(13.0).color(TEXT_SEC));
    });
    ui.add_space(2.0);
    let r = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [
            egui::pos2(r.left(), r.top() + 1.0),
            egui::pos2(r.right(), r.top() + 1.0),
        ],
        Stroke::new(1.0, BORDER),
    );
}

/// Chip de herramienta disponible/no disponible.
fn tool_chip(ui: &mut egui::Ui, name: &str, ok: bool) {
    let (fg, bg) = if ok {
        (C_OK_FG, C_OK_BG)
    } else {
        (TEXT_MUT, BG_ROW_ALT)
    };
    pill(
        ui,
        &format!("{name} {}", if ok { "✅" } else { "—" }),
        fg,
        bg,
    );
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    info_row_colored(ui, label, value, TEXT_SEC);
}

fn info_row_ok(ui: &mut egui::Ui, label: &str, value: &str) {
    info_row_colored(ui, label, value, C_OK_FG);
}

fn info_row_colored(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(label).size(11.5).color(TEXT_MUT));
        let short = trunc(value, 60);
        let resp = ui.label(RichText::new(&short).size(11.5).monospace().color(color));
        if value.len() > 60 {
            resp.on_hover_text(value);
        }
    });
}

fn loading_screen(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label(
            RichText::new("Capturando datos del sistema…")
                .size(16.0)
                .color(TEXT_MUT),
        );
    });
}

// ── Iconos dibujados con el painter ───────────────────────────────────────────

/// Logo RC con fondo azul.
fn draw_logo_icon(ui: &mut egui::Ui, size: f32) {
    // Marca de RootCause: radar de círculos concéntricos (igual que el icono .ico
    // y el favicon), en el azul de acento. Reemplaza el antiguo bloque "RC".
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let c = rect.center();
    let sw = (size * 0.07).max(1.5);
    ui.painter()
        .circle_stroke(c, size * 0.42, Stroke::new(sw, ACCENT));
    ui.painter()
        .circle_stroke(c, size * 0.22, Stroke::new(sw, ACCENT));
    ui.painter()
        .circle_filled(c, (size * 0.08).max(1.5), ACCENT);
}

/// Lupa de búsqueda simplificada.
fn draw_search_icon(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let center = rect.center() - Vec2::new(1.5, 1.5);
    let r = size * 0.28;
    ui.painter()
        .circle_stroke(center, r, Stroke::new(1.5, TEXT_MUT));
    ui.painter().line_segment(
        [
            center + Vec2::new(r * 0.7, r * 0.7),
            rect.right_bottom() - Vec2::new(1.0, 1.0),
        ],
        Stroke::new(1.5, TEXT_MUT),
    );
}

/// Ícono de salud: anillo circular con fracción rellena.
fn draw_health_ring(ui: &mut egui::Ui, fraction: f32, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let center = rect.center();
    let r_out = size * 0.46;
    let r_in = size * 0.30;

    // Fondo del anillo
    ui.painter().circle_stroke(
        center,
        (r_out + r_in) / 2.0,
        Stroke::new(r_out - r_in, BG_PANEL),
    );

    // Arco relleno (aproximado con segmentos)
    let steps = 48usize;
    let start = -std::f32::consts::FRAC_PI_2;
    let sweep = fraction.clamp(0.0, 1.0) * std::f32::consts::TAU;
    let mid_r = (r_out + r_in) / 2.0;
    let stroke_w = r_out - r_in;

    let points: Vec<egui::Pos2> = (0..=((steps as f32 * fraction) as usize + 1))
        .map(|i| {
            let angle = start + (i as f32 / steps as f32) * sweep;
            egui::pos2(
                center.x + mid_r * angle.cos(),
                center.y + mid_r * angle.sin(),
            )
        })
        .collect();

    for w in points.windows(2) {
        ui.painter()
            .line_segment([w[0], w[1]], Stroke::new(stroke_w, color));
    }

    // Número en el centro
    let score_text = format!("{}", (fraction * 100.0) as u8);
    ui.painter().text(
        center,
        egui::Align2::CENTER_CENTER,
        score_text,
        FontId::proportional(size * 0.30),
        color,
    );
}

/// Ícono de severidad: círculo con símbolo.
fn draw_sev_icon(ui: &mut egui::Ui, sev: Severity, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let fg = sev_fg(sev);
    let bg = sev_bg(sev);
    ui.painter().circle_filled(rect.center(), size * 0.46, bg);
    ui.painter()
        .circle_stroke(rect.center(), size * 0.46, Stroke::new(1.2, fg));
    // Símbolos de la fuente base (Ubuntu-Light): el checkmark/✕ Unicode no está en
    // la fuente y salía como "□". El color del círculo ya comunica la severidad;
    // el interior solo refuerza con glifos que sí renderizan.
    let sym = match sev {
        Severity::Healthy => "",
        Severity::Warning => "!",
        Severity::Critical => "×",
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        sym,
        FontId::proportional(size * 0.52),
        fg,
    );
}

/// Ícono de proceso: cuadrado con initial del nombre.
fn draw_proc_icon(ui: &mut egui::Ui, sev: Severity, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let fg = sev_fg(sev);
    let bg = sev_bg(sev);
    ui.painter()
        .rect_filled(rect, Rounding::same(size * 0.18), bg);
    ui.painter()
        .rect_stroke(rect, Rounding::same(size * 0.18), Stroke::new(1.0, fg));
    // Tres líneas horizontales estilo "proceso"
    for i in 0..3 {
        let y = rect.top() + (i as f32 + 1.0) * rect.height() / 4.0;
        let w = if i == 0 { 0.7 } else { 0.5 };
        ui.painter().line_segment(
            [
                egui::pos2(rect.left() + rect.width() * 0.2, y),
                egui::pos2(rect.left() + rect.width() * (0.2 + w * 0.6), y),
            ],
            Stroke::new(1.5, fg.linear_multiply(0.8)),
        );
    }
}

/// Ícono de servicio: engranaje simplificado.
fn draw_service_icon(ui: &mut egui::Ui, sev: Severity, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let fg = sev_fg(sev);
    ui.painter()
        .circle_stroke(rect.center(), size * 0.35, Stroke::new(1.5, fg));
    ui.painter().circle_filled(rect.center(), size * 0.14, fg);
}

// ── Helpers de lógica ──────────────────────────────────────────────────────────

fn compute_health_score(snap: &SystemSnapshot) -> u8 {
    let ov = &snap.overview;
    let mut score: f32 = 100.0;
    if ov.cpu_usage_percent >= 80.0 {
        score -= 22.0;
    } else if ov.cpu_usage_percent >= 55.0 {
        score -= 9.0;
    }
    let ram = ov.memory_used_gb / ov.memory_total_gb.max(0.1) * 100.0;
    if ram >= 88.0 {
        score -= 18.0;
    } else if ram >= 70.0 {
        score -= 7.0;
    }
    if ov.io_write_mb_delta >= 220.0 {
        score -= 18.0;
    } else if ov.io_write_mb_delta >= 80.0 {
        score -= 7.0;
    }
    let net = ov.network_rx_mb_delta + ov.network_tx_mb_delta;
    if net >= 80.0 {
        score -= 10.0;
    } else if net >= 15.0 {
        score -= 3.0;
    }
    let crits = snap
        .alerts
        .iter()
        .filter(|a| matches!(a.severity, Severity::Critical))
        .count();
    score -= crits as f32 * 7.0;
    if let Some(incident) = snap.incident.as_ref() {
        score -= (incident.risk_score.min(40) as f32) * 0.45;
    } else if let Some(anomaly) = snap.anomalies.first() {
        score -= (anomaly.score.min(35) as f32) * 0.35;
    }
    score.clamp(0.0, 100.0) as u8
}

fn severity_for_value(v: f32, warn: f32, crit: f32) -> Severity {
    if v >= crit {
        Severity::Critical
    } else if v >= warn {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn sev_fg(sev: Severity) -> Color32 {
    match sev {
        Severity::Healthy => C_OK_FG,
        Severity::Warning => C_WN_FG,
        Severity::Critical => C_CR_FG,
    }
}

fn sev_bg(sev: Severity) -> Color32 {
    match sev {
        Severity::Healthy => C_OK_BG,
        Severity::Warning => C_WN_BG,
        Severity::Critical => C_CR_BG,
    }
}

fn sev_dot(_sev: Severity) -> &'static str {
    // Punto de severidad como viñeta "•" (presente en la fuente base). Los glifos
    // geométricos ●/▲/■ no están en la fuente y salían como "□"; el color con el
    // que se pinta ya distingue la severidad.
    "•"
}

/// Mensaje centrado y atenuado para secciones sin datos (evita el vacío negro).
fn empty_state(ui: &mut egui::Ui, msg: &str) {
    ui.add_space(16.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(msg).size(12.5).color(TEXT_MUT));
    });
    ui.add_space(16.0);
}

fn service_severity(svc: &ServiceState) -> Severity {
    let low = svc.name.to_ascii_lowercase();
    if STOPPABLE_SERVICES.contains(&low.as_str()) && svc.status.eq_ignore_ascii_case("Running") {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn is_stoppable_service(svc: &ServiceState) -> bool {
    STOPPABLE_SERVICES.contains(&svc.name.to_ascii_lowercase().as_str())
}

fn trunc(s: &str, max: usize) -> String {
    match s.char_indices().nth(max) {
        None => s.to_owned(),
        Some((byte_offset, _)) => format!("{}…", &s[..byte_offset]),
    }
}

fn matches_filter(primary: &str, secondary: &str, filter: &str) -> bool {
    let needle = filter.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    primary.to_ascii_lowercase().contains(&needle)
        || secondary.to_ascii_lowercase().contains(&needle)
}
