//! Capa de interfaz — diseño con tabs, estilo PC Manager dark.
//!
//! Estructura: barra superior con logo + controles, barra de tabs horizontal,
//! cada tab dibuja su contenido con tablas, progress bars y tooltips para
//! nombres o rutas largas. Sin scroll horizontal.

use crate::meta;
use crate::models::{
    AnomalyEvent, HardwareInfo, ProcessInsight, ServiceState, Severity, SnapshotRow,
    SystemSnapshot, TraceAnalysisSummary, TracePathSummary, TraceProcessSummary,
};
use crate::services::inspector::InspectorService;
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
    History,
    About,
}

impl Tab {
    const ALL: &'static [(Tab, &'static str, &'static str)] = &[
        (Tab::Overview, "◈", "Resumen"),
        (Tab::Processes, "⚙", "Procesos"),
        (Tab::Connections, "◎", "Conexiones"),
        (Tab::TempFiles, "▤", "Temporales"),
        (Tab::Precision, "◉", "ETW / WPR"),
        (Tab::Services, "◧", "Servicios"),
        (Tab::History, "◑", "Historial"),
        (Tab::About, "ℹ", "Acerca"),
    ];
}

// ── Acciones de precisión ──────────────────────────────────────────────────────

enum PrecisionAction {
    Start,
    Stop,
    Cancel,
    Analyze,
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
}

impl RootCauseApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
        };
        match inspector {
            Ok(svc) => {
                app.hardware_info = svc.get_hardware_info();
                app.status_line = svc.latest_history_line();
                app.refresh_interval_secs = svc.config().collection.refresh_interval_secs;
                app.notifications_enabled = svc.config().alerting.notify_on_critical;
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
}

// ── Loop principal ─────────────────────────────────────────────────────────────

impl eframe::App for RootCauseApp {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(BG_APP).to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            // Ctrl+1..8 → Cambiar de tab
            for (key, idx) in [
                (egui::Key::Num1, 0usize),
                (egui::Key::Num2, 1),
                (egui::Key::Num3, 2),
                (egui::Key::Num4, 3),
                (egui::Key::Num5, 4),
                (egui::Key::Num6, 5),
                (egui::Key::Num7, 6),
                (egui::Key::Num8, 7),
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
            && let Some(&(tab, _, _)) = Tab::ALL.get(idx)
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

        draw_header(self, ctx);
        draw_tabbar(self, ctx);
        draw_statusbar(self, ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BG_APP)
                    .inner_margin(Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                // El tab Acerca no necesita snapshot — se muestra siempre.
                if self.active_tab == Tab::About {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| draw_tab_about(ui, &self.hardware_info));
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
                                self.only_public_connections,
                                |ip| ip_to_block = Some(ip.to_owned()),
                            ),
                            Tab::TempFiles => {
                                draw_tab_temp(ui, &snapshot, &self.filter_text);
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
                            Tab::History => draw_tab_history(
                                ui,
                                &self.history_rows,
                                &mut self.history_filter,
                                &mut self.history_compare_a,
                                &mut self.history_compare_b,
                            ),
                            // About se gestiona antes del guard de snapshot — nunca llega aquí.
                            Tab::About => {}
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
                        if let Some(new_sev) = sev_filter_change {
                            self.proc_severity_filter = new_sev;
                        }
                    });
            });
    }
}

// ── Header ─────────────────────────────────────────────────────────────────────

fn draw_header(app: &mut RootCauseApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("header")
        .frame(
            egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(16.0, 10.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo icon
                draw_logo_icon(ui, 32.0);
                ui.add_space(8.0);
                ui.label(
                    RichText::new("RootCause")
                        .size(16.0)
                        .strong()
                        .color(TEXT_PRI),
                );
                ui.label(
                    RichText::new("Windows Inspector")
                        .size(11.0)
                        .color(TEXT_MUT),
                );

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);

                // Botones principales
                if accion_btn(ui, "⟳", "Actualizar").clicked() {
                    app.refresh_now();
                }
                if accion_btn(ui, "↓", "Exportar JSON").clicked() {
                    app.export_snapshot();
                }

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);

                // Auto refresco
                ui.checkbox(&mut app.auto_refresh, RichText::new("Auto").color(TEXT_SEC));
                ui.add(
                    egui::Slider::new(&mut app.refresh_interval_secs, 3..=30)
                        .text(RichText::new("s").color(TEXT_MUT))
                        .clamp_to_range(true),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(
                    &mut app.only_public_connections,
                    RichText::new("Solo IP públicas").color(TEXT_SEC),
                );

                ui.add_space(4.0);
                ui.checkbox(
                    &mut app.notifications_enabled,
                    RichText::new("🔔").color(TEXT_SEC),
                )
                .on_hover_text("Activar notificaciones toast cuando el estado sea Crítico");

                ui.add_space(8.0);
                // Buscador
                draw_search_icon(ui, 14.0);
                ui.add_space(4.0);
                ui.add_sized(
                    [200.0, 26.0],
                    egui::TextEdit::singleline(&mut app.filter_text)
                        .hint_text("Filtrar por nombre o ruta…")
                        .text_color(TEXT_PRI),
                );

                // Badges de alerta en el header
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

// ── Barra de tabs ──────────────────────────────────────────────────────────────

fn draw_tabbar(app: &mut RootCauseApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("tabbar")
        .frame(
            egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(12.0, 0.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for &(tab, icon, label) in Tab::ALL {
                    let selected = app.active_tab == tab;
                    if tab_btn(ui, icon, label, selected).clicked() {
                        app.active_tab = tab;
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
                ui.label(RichText::new("●").color(dot).size(9.0));
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
        (C_OK_FG, C_OK_BG, "Saludable")
    } else if score >= 50 {
        (C_WN_FG, C_WN_BG, "Advertencia")
    } else {
        (C_CR_FG, C_CR_BG, "Crítico")
    };

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
                "↓{:.1}  ↑{:.1} MB",
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
                    "↓{:.1}  ↑{:.1} MB",
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
                    hw_row(ui, "🪟  Sistema", &hw.os_name);
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
                        hw_row(left, "🪟  Sistema", &hw.os_name);
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
        for (label, sev, fg, bg) in [
            ("■ Crítico", Severity::Critical, C_CR_FG, C_CR_BG),
            ("▲ Aviso", Severity::Warning, C_WN_FG, C_WN_BG),
            ("● Sano", Severity::Healthy, C_OK_FG, C_OK_BG),
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
                                (p.memory_mb / 16384.0).min(1.0),
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
    only_public: bool,
    mut on_block: F,
) {
    section_header(
        ui,
        "▸  Conexiones activas  ·  foco en IP pública y rutas poco confiables",
    );
    ui.add_space(8.0);

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
                .filter(|c| !only_public || c.is_public_remote)
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

fn draw_tab_temp(ui: &mut egui::Ui, snap: &SystemSnapshot, filter: &str) {
    section_header(
        ui,
        "▸  Archivos temporales  ·  instaladores, actualizaciones, exportaciones",
    );
    ui.add_space(8.0);

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
                let (dot, txt, label) = if recording {
                    ("●", C_WN_FG, "GRABANDO")
                } else {
                    ("○", TEXT_MUT, "En espera")
                };
                ui.label(
                    RichText::new(format!("{dot} {label}"))
                        .strong()
                        .size(13.0)
                        .color(txt),
                );
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
                    && action_btn(ui, "▶  Iniciar captura", C_OK_BG, C_OK_FG).clicked()
                {
                    *precision_action = Some(PrecisionAction::Start);
                }
                if p.wpr_available && recording {
                    if action_btn(ui, "■  Detener y guardar", C_WN_BG, C_WN_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Stop);
                    }
                    if action_btn(ui, "✕  Cancelar", C_CR_BG, C_CR_FG).clicked() {
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
                                        RichText::new(if is_a { "A ✓" } else { "A" })
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
                                        RichText::new(if is_b { "B ✓" } else { "B" })
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

// ── Tab: Acerca ────────────────────────────────────────────────────────────────

fn draw_tab_about(ui: &mut egui::Ui, hw: &HardwareInfo) {
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
                    ("Ctrl + 7", "Ir a Historial"),
                    ("Ctrl + 8", "Ir a Acerca"),
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
    ctx.set_style(style);
}

/// Botón de tab con indicador de selección.
fn tab_btn(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let (fg, bg) = if selected {
        (TEXT_PRI, BG_CARD)
    } else {
        (TEXT_SEC, Color32::TRANSPARENT)
    };
    let resp = ui.add(
        egui::Button::new(
            RichText::new(format!("{icon}  {label}"))
                .size(12.5)
                .color(fg),
        )
        .fill(bg)
        .stroke(if selected {
            Stroke::new(1.0, BORDER)
        } else {
            Stroke::NONE
        })
        .rounding(Rounding::same(5.0)),
    );
    // Línea de acento inferior para el tab activo
    if selected {
        let r = resp.rect;
        ui.painter().line_segment(
            [
                egui::pos2(r.left() + 4.0, r.bottom() + 1.0),
                egui::pos2(r.right() - 4.0, r.bottom() + 1.0),
            ],
            Stroke::new(2.0, ACCENT),
        );
    }
    resp
}

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
fn accion_btn(ui: &mut egui::Ui, icon: &str, label: &str) -> egui::Response {
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
            ui.label(RichText::new(title).size(10.0).color(TEXT_MUT).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(17.0).strong().color(TEXT_PRI));
            ui.add(egui::Label::new(RichText::new(subtitle).size(10.5).color(TEXT_MUT)).wrap(true));
            ui.add_space(6.0);
            pbar(ui, fraction.clamp(0.0, 1.0), fg, ui.available_width() - 2.0);
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
            ui.set_min_size(Vec2::new(width, 125.0));
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
            ui.set_min_size(Vec2::new(width, height));
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
}

/// Encabezado de sección con línea separadora.
fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.label(RichText::new(title).strong().size(13.0).color(TEXT_SEC));
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
        &format!("{name} {}", if ok { "✓" } else { "—" }),
        fg,
        bg,
    );
}

fn info_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(label).size(11.5).color(TEXT_MUT));
        let short = trunc(value, 60);
        let resp = ui.label(RichText::new(&short).size(11.5).monospace().color(TEXT_SEC));
        if value.len() > 60 {
            resp.on_hover_text(value);
        }
    });
}

fn info_row_ok(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(label).size(11.5).color(TEXT_MUT));
        let short = trunc(value, 60);
        let resp = ui.label(RichText::new(&short).size(11.5).monospace().color(C_OK_FG));
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
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(6.0), ACCENT);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "RC",
        FontId::proportional(size * 0.38),
        TEXT_PRI,
    );
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
    let sym = match sev {
        Severity::Healthy => "✓",
        Severity::Warning => "!",
        Severity::Critical => "✕",
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

fn sev_dot(sev: Severity) -> &'static str {
    match sev {
        Severity::Healthy => "●",
        Severity::Warning => "▲",
        Severity::Critical => "■",
    }
}

fn service_severity(svc: &ServiceState) -> Severity {
    let low = svc.name.to_ascii_lowercase();
    if ["wuauserv", "bits", "dosvc", "sysmain"].contains(&low.as_str())
        && svc.status.eq_ignore_ascii_case("Running")
    {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn is_stoppable_service(svc: &ServiceState) -> bool {
    ["wuauserv", "bits", "dosvc", "sysmain"].contains(&svc.name.to_ascii_lowercase().as_str())
}

fn trunc(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        s.to_owned()
    } else {
        format!("{}…", chars[..max].iter().collect::<String>())
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
