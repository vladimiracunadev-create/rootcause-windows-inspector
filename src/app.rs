//! Capa de interfaz — diseño con tabs, estilo PC Manager dark.
//!
//! Estructura: barra superior con logo + controles, barra de tabs horizontal,
//! cada tab dibuja su contenido con tablas, progress bars y tooltips para
//! nombres o rutas largas. Sin scroll horizontal.

use crate::models::{
    ProcessInsight, ServiceState, Severity, SystemSnapshot, TraceAnalysisSummary, TracePathSummary,
    TraceProcessSummary,
};
use crate::services::inspector::InspectorService;
use eframe::egui::{self, Color32, FontId, Margin, RichText, Rounding, Sense, Stroke, Vec2};
use std::time::{Duration, Instant};

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
}

impl Tab {
    const ALL: &'static [(Tab, &'static str, &'static str)] = &[
        (Tab::Overview, "◈", "Resumen"),
        (Tab::Processes, "⚙", "Procesos"),
        (Tab::Connections, "◎", "Conexiones"),
        (Tab::TempFiles, "▤", "Temporales"),
        (Tab::Precision, "◉", "ETW / WPR"),
        (Tab::Services, "◧", "Servicios"),
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
        };
        match inspector {
            Ok(svc) => {
                app.status_line = svc.latest_history_line();
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
                self.snapshot = Some(snap);
                self.last_refresh_at = Instant::now();
            }
            Err(e) => {
                self.status_line = format!("Error al capturar: {e}");
                self.status_is_error = true;
            }
        }
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
                self.status_line = format!("Exportado → {path}");
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

        if self.snapshot.is_none()
            || (self.auto_refresh
                && self.last_refresh_at.elapsed()
                    >= Duration::from_secs(self.refresh_interval_secs))
        {
            self.refresh_now();
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

                        match self.active_tab {
                            Tab::Overview => draw_tab_overview(ui, &snapshot),
                            Tab::Processes => {
                                draw_tab_processes(ui, &snapshot, &self.filter_text, |pid| {
                                    pid_to_kill = Some(pid)
                                })
                            }
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

fn draw_tab_overview(ui: &mut egui::Ui, snap: &SystemSnapshot) {
    let ov = &snap.overview;
    let score = compute_health_score(snap);
    let (score_fg, score_bg, score_label) = if score >= 80 {
        (C_OK_FG, C_OK_BG, "Saludable")
    } else if score >= 50 {
        (C_WN_FG, C_WN_BG, "Advertencia")
    } else {
        (C_CR_FG, C_CR_BG, "Crítico")
    };

    // ── Fila 1: score + cards de métricas ─────────────────────────────────────
    ui.horizontal_wrapped(|ui| {
        // Score card (más grande)
        egui::Frame::none()
            .fill(score_bg)
            .stroke(Stroke::new(1.5, score_fg.linear_multiply(0.5)))
            .rounding(Rounding::same(10.0))
            .inner_margin(Margin::same(14.0))
            .show(ui, |ui| {
                ui.set_min_size(Vec2::new(140.0, 120.0));
                ui.vertical_centered(|ui| {
                    draw_health_ring(ui, score as f32 / 100.0, score_fg, 54.0);
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

        ui.add_space(4.0);

        // Cards métricas con barra de progreso integrada
        let ram_pct = ov.memory_used_gb / ov.memory_total_gb.max(0.1) * 100.0;
        let net = ov.network_rx_mb_delta + ov.network_tx_mb_delta;

        overview_card(
            ui,
            "CPU",
            &format!("{:.1}%", ov.cpu_usage_percent),
            "Uso global del procesador",
            ov.cpu_usage_percent / 100.0,
            severity_for_value(ov.cpu_usage_percent, 55.0, 80.0),
        );
        overview_card(
            ui,
            "RAM",
            &format!("{:.1} / {:.1} GB", ov.memory_used_gb, ov.memory_total_gb),
            &format!("{ram_pct:.0}% utilizado"),
            ram_pct / 100.0,
            severity_for_value(ram_pct, 70.0, 88.0),
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
        );
        overview_card(
            ui,
            "TEMP",
            &format!("{:.0} MB", ov.temp_total_mb),
            "TEMP / cachés vigiladas",
            ov.temp_total_mb / 2000.0,
            severity_for_value(ov.temp_total_mb, 700.0, 2000.0),
        );
    });

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
                        ui.label(RichText::new(path).small().monospace().color(TEXT_MUT));
                    }
                });
            ui.add_space(4.0);
        }
    }

    // ── Top 3 procesos críticos (vista rápida) ─────────────────────────────────
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
        ui.horizontal_wrapped(|ui| {
            for p in top_procs {
                mini_process_card(ui, p);
            }
        });
    }
}

// ── Tab: Procesos ──────────────────────────────────────────────────────────────

fn draw_tab_processes<F: FnMut(u32)>(
    ui: &mut egui::Ui,
    snap: &SystemSnapshot,
    filter: &str,
    mut on_kill: F,
) {
    section_header(
        ui,
        "▸  Procesos dominantes  ·  ordenados por severidad, I/O, RAM, CPU",
    );
    ui.add_space(8.0);

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
                                ui.set_max_width(360.0);
                                ui.label(RichText::new(&p.name).strong().color(TEXT_PRI));
                                ui.label(
                                    RichText::new(&p.exe_path)
                                        .small()
                                        .monospace()
                                        .color(TEXT_MUT),
                                );
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
                            if p.can_terminate {
                                if action_btn(ui, "Finalizar", C_CR_BG, C_CR_FG).clicked() {
                                    to_kill = Some(p.pid);
                                }
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
                if p.wpr_available && !recording {
                    if action_btn(ui, "▶  Iniciar captura", C_OK_BG, C_OK_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Start);
                    }
                }
                if p.wpr_available && recording {
                    if action_btn(ui, "■  Detener y guardar", C_WN_BG, C_WN_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Stop);
                    }
                    if action_btn(ui, "✕  Cancelar", C_CR_BG, C_CR_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Cancel);
                    }
                }
                if !recording && p.tracerpt_available && p.last_trace_path.is_some() {
                    if action_btn(ui, "⚡  Analizar ETL", C_BL_BG, C_BL_FG).clicked() {
                        *precision_action = Some(PrecisionAction::Analyze);
                    }
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
                ui.label(RichText::new("Hallazgos").strong().color(TEXT_PRI));
                for f in ta.findings.iter().take(3) {
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
        RichText::new("Contexto")
            .strong()
            .size(12.0)
            .color(TEXT_SEC),
    );
    ui.add_space(4.0);
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
    if !ta.indicators.is_empty() {
        ui.label(
            RichText::new("Indicadores")
                .size(11.0)
                .strong()
                .color(C_WN_FG),
        );
        for ind in ta.indicators.iter().take(5) {
            ui.label(RichText::new(ind).small().color(TEXT_SEC));
        }
        ui.add_space(4.0);
    }
    for lim in ta.limitations.iter().take(3) {
        ui.label(RichText::new(lim).small().italics().color(TEXT_MUT));
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
fn overview_card(
    ui: &mut egui::Ui,
    title: &str,
    value: &str,
    subtitle: &str,
    fraction: f32,
    severity: Severity,
) {
    let fg = sev_fg(severity);
    let bg = sev_bg(severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(10.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(170.0, 90.0));
            ui.label(RichText::new(title).size(10.0).color(TEXT_MUT).strong());
            ui.add_space(4.0);
            ui.label(RichText::new(value).size(17.0).strong().color(TEXT_PRI));
            ui.label(RichText::new(subtitle).size(10.5).color(TEXT_MUT));
            ui.add_space(6.0);
            pbar(ui, fraction.clamp(0.0, 1.0), fg, ui.available_width() - 2.0);
        });
}

/// Mini card de proceso para el overview.
fn mini_process_card(ui: &mut egui::Ui, p: &ProcessInsight) {
    let fg = sev_fg(p.severity);
    let bg = sev_bg(p.severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(220.0, 80.0));
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
    score.max(0.0).min(100.0) as u8
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
