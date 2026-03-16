//! Capa de interfaz — tema profesional dark.
//!
//! Prioridad: claridad inmediata, jerarquía visual sólida y respuesta rápida.
//! Inspirado en el lenguaje visual de herramientas de observabilidad modernas.

use crate::models::{
    ConnectionInsight, PrecisionStatus, ProcessInsight, ServiceState, Severity, SystemSnapshot,
    TempEntry, TraceAnalysisSummary, TracePathSummary, TraceProcessSummary,
};
use crate::services::inspector::InspectorService;
use eframe::egui::{self, Color32, FontId, Margin, RichText, Rounding, Sense, Stroke, Vec2};
use std::time::{Duration, Instant};

// ── Paleta ────────────────────────────────────────────────────────────────────

const BG_APP: Color32 = Color32::from_rgb(13, 17, 23);
const BG_PANEL: Color32 = Color32::from_rgb(22, 27, 34);
const BG_CARD: Color32 = Color32::from_rgb(30, 37, 46);
const BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 46, 57);
const BORDER: Color32 = Color32::from_rgb(48, 54, 61);
const BORDER_ACCENT: Color32 = Color32::from_rgb(31, 111, 235);

const TEXT_PRIMARY: Color32 = Color32::from_rgb(230, 237, 243);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(139, 148, 158);
const TEXT_MUTED: Color32 = Color32::from_rgb(88, 96, 108);

const C_HEALTHY_FG: Color32 = Color32::from_rgb(63, 185, 80);
const C_HEALTHY_BG: Color32 = Color32::from_rgb(13, 43, 26);
const C_WARNING_FG: Color32 = Color32::from_rgb(210, 153, 34);
const C_WARNING_BG: Color32 = Color32::from_rgb(43, 29, 14);
const C_CRITICAL_FG: Color32 = Color32::from_rgb(248, 81, 73);
const C_CRITICAL_BG: Color32 = Color32::from_rgb(43, 14, 14);
const C_BLUE_FG: Color32 = Color32::from_rgb(88, 166, 255);
const C_BLUE_BG: Color32 = Color32::from_rgb(14, 34, 68);

// ── Acción de precisión ───────────────────────────────────────────────────────

enum PrecisionAction {
    Start,
    Stop,
    Cancel,
    Analyze,
}

// ── Estado de la aplicación ───────────────────────────────────────────────────

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
        };

        match inspector {
            Ok(service) => {
                app.status_line = service.latest_history_line();
                app.inspector = Some(service);
            }
            Err(error) => {
                app.status_line = format!("Error inicializando el motor: {error}");
                app.status_is_error = true;
            }
        }
        app
    }

    fn refresh_now(&mut self) {
        let Some(inspector) = self.inspector.as_mut() else {
            return;
        };
        match inspector.collect_snapshot() {
            Ok(snapshot) => {
                self.status_line = format!(
                    "Última captura {}  ·  {}",
                    snapshot.collected_at.format("%H:%M:%S"),
                    snapshot.overview.primary_reason
                );
                self.status_is_error = false;
                self.snapshot = Some(snapshot);
                self.last_refresh_at = Instant::now();
            }
            Err(error) => {
                self.status_line = format!("Fallo al capturar: {error}");
                self.status_is_error = true;
            }
        }
    }

    fn export_snapshot(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            self.status_line = "Aún no hay datos para exportar".to_owned();
            return;
        };
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };
        match inspector.export_snapshot(snapshot) {
            Ok(path) => {
                self.status_line = format!("Reporte exportado → {path}");
                self.status_is_error = false;
            }
            Err(error) => {
                self.status_line = format!("No se pudo exportar: {error}");
                self.status_is_error = true;
            }
        }
    }

    fn start_precision_capture(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.start_precision_capture(&self.precision_note)
        };
        self.apply_precision_result(result, "No se pudo iniciar WPR");
    }

    fn stop_precision_capture(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.stop_precision_capture(&self.precision_note)
        };
        self.apply_precision_result(result, "No se pudo detener WPR");
    }

    fn cancel_precision_capture(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.cancel_precision_capture()
        };
        self.apply_precision_result(result, "No se pudo cancelar WPR");
    }

    fn analyze_last_trace(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.analyze_last_precision_trace()
        };
        self.apply_precision_result(result, "No se pudo resumir el ETL");
    }

    fn apply_precision_result(&mut self, result: anyhow::Result<String>, error_prefix: &str) {
        match result {
            Ok(message) => {
                self.status_line = message;
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(error) => {
                self.status_line = format!("{error_prefix}: {error}");
                self.status_is_error = true;
            }
        }
    }

    fn terminate_process(&mut self, pid: u32) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };
        match inspector.terminate_process(pid) {
            Ok(msg) => {
                self.status_line = format!("Proceso terminado  ·  {msg}");
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(error) => {
                self.status_line = format!("No se pudo finalizar PID {pid}: {error}");
                self.status_is_error = true;
            }
        }
    }

    fn block_remote_ip(&mut self, ip: &str) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };
        match inspector.block_remote_ip(ip) {
            Ok(msg) => {
                self.status_line = msg;
                self.status_is_error = false;
            }
            Err(error) => {
                self.status_line = format!("No se pudo bloquear la IP: {error}");
                self.status_is_error = true;
            }
        }
    }

    fn stop_service(&mut self, name: &str) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };
        match inspector.stop_service(name) {
            Ok(msg) => {
                self.status_line = msg;
                self.status_is_error = false;
                self.last_refresh_at =
                    Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(error) => {
                self.status_line = format!("No se pudo detener {name}: {error}");
                self.status_is_error = true;
            }
        }
    }
}

// ── Loop principal ────────────────────────────────────────────────────────────

impl eframe::App for RootCauseApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
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

        draw_topbar(self, ctx);
        draw_statusbar(self, ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_APP))
            .show(ctx, |ui| {
                let Some(snapshot) = self.snapshot.clone() else {
                    loading_screen(ui);
                    return;
                };

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(12.0);
                        draw_overview(ui, &snapshot);
                        section_gap(ui);
                        let mut precision_action: Option<PrecisionAction> = None;
                        draw_precision_section(
                            ui,
                            &snapshot.precision,
                            &mut self.precision_note,
                            &mut precision_action,
                        );
                        if let Some(trace_analysis) = &snapshot.trace_analysis {
                            section_gap(ui);
                            draw_trace_analysis_section(ui, trace_analysis);
                        }
                        section_gap(ui);
                        draw_alerts(ui, &snapshot);
                        section_gap(ui);

                        let mut pid_to_kill: Option<u32> = None;
                        ui.columns(2, |cols| {
                            draw_processes_section(
                                &mut cols[0],
                                &snapshot,
                                &self.filter_text,
                                |pid| pid_to_kill = Some(pid),
                            );
                            draw_temp_section(&mut cols[1], &snapshot, &self.filter_text);
                        });

                        section_gap(ui);
                        let mut endpoint_to_block: Option<String> = None;
                        let mut service_to_stop: Option<String> = None;
                        ui.columns(2, |cols| {
                            draw_connections_section(
                                &mut cols[0],
                                &snapshot,
                                &self.filter_text,
                                self.only_public_connections,
                                |ip| endpoint_to_block = Some(ip.to_owned()),
                            );
                            draw_events_and_services(&mut cols[1], &snapshot, |svc| {
                                service_to_stop = Some(svc.to_owned())
                            });
                        });
                        ui.add_space(20.0);

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
                        if let Some(ep) = endpoint_to_block {
                            self.block_remote_ip(&ep);
                        }
                        if let Some(svc) = service_to_stop {
                            self.stop_service(&svc);
                        }
                    });
            });
    }
}

// ── Barra superior ────────────────────────────────────────────────────────────

fn draw_topbar(app: &mut RootCauseApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("topbar")
        .frame(
            egui::Frame::none()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER))
                .inner_margin(Margin::symmetric(16.0, 10.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Logo
                let logo_size = Vec2::splat(32.0);
                let (logo_rect, _) = ui.allocate_exact_size(logo_size, Sense::hover());
                ui.painter().rect_filled(logo_rect, Rounding::same(6.0), BORDER_ACCENT);
                ui.painter().text(
                    logo_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "RC",
                    FontId::proportional(13.0),
                    TEXT_PRIMARY,
                );
                ui.add_space(10.0);

                // Nombre
                ui.label(
                    RichText::new("RootCause")
                        .size(17.0)
                        .strong()
                        .color(TEXT_PRIMARY),
                );
                ui.label(
                    RichText::new("Inspector de Windows")
                        .size(12.0)
                        .color(TEXT_MUTED),
                );

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(12.0);

                // Controles
                if styled_button(ui, "⟳  Actualizar", C_BLUE_BG, C_BLUE_FG).clicked() {
                    app.refresh_now();
                }
                if styled_button(ui, "↓  Exportar JSON", BG_CARD, TEXT_SECONDARY).clicked() {
                    app.export_snapshot();
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(&mut app.auto_refresh, RichText::new("Auto").color(TEXT_SECONDARY));
                ui.add(
                    egui::Slider::new(&mut app.refresh_interval_secs, 3..=30)
                        .text(RichText::new("s").color(TEXT_MUTED))
                        .clamp_to_range(true),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.checkbox(
                    &mut app.only_public_connections,
                    RichText::new("Solo IP públicas").color(TEXT_SECONDARY),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label(RichText::new("🔍").size(13.0));
                ui.add_sized(
                    [220.0, 26.0],
                    egui::TextEdit::singleline(&mut app.filter_text)
                        .hint_text("Filtrar…")
                        .text_color(TEXT_PRIMARY),
                );
            });
        });
}

// ── Barra inferior de estado ──────────────────────────────────────────────────

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
                let (dot_color, label_color) = if app.status_is_error {
                    (C_CRITICAL_FG, C_CRITICAL_FG)
                } else {
                    (C_HEALTHY_FG, TEXT_SECONDARY)
                };
                ui.label(RichText::new("●").color(dot_color).size(10.0));
                ui.label(RichText::new(&app.status_line).size(12.0).color(label_color));
            });
        });
}

// ── Resumen de métricas (cards) ───────────────────────────────────────────────

fn draw_overview(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
    let overview = &snapshot.overview;
    section_header(ui, "▪  Métricas del sistema");
    ui.add_space(8.0);
    ui.horizontal_wrapped(|ui| {
        metric_card(
            ui,
            "ESTADO",
            overview.primary_severity.label(),
            &overview.primary_reason,
            overview.primary_severity,
        );
        metric_card(
            ui,
            "CPU",
            &format!("{:.1}%", overview.cpu_usage_percent),
            "Uso global",
            severity_for_value(overview.cpu_usage_percent, 55.0, 80.0),
        );
        let ram_pct =
            overview.memory_used_gb / overview.memory_total_gb.max(0.1) * 100.0;
        metric_card(
            ui,
            "RAM",
            &format!("{:.1} / {:.1} GB", overview.memory_used_gb, overview.memory_total_gb),
            &format!("{ram_pct:.0}% utilizado"),
            severity_for_value(ram_pct, 70.0, 88.0),
        );
        metric_card(
            ui,
            "DISCO  I/O",
            &format!("R {:.1}  W {:.1} MB", overview.io_read_mb_delta, overview.io_write_mb_delta),
            "Suma de procesos en el intervalo",
            severity_for_value(overview.io_write_mb_delta, 80.0, 220.0),
        );
        metric_card(
            ui,
            "RED",
            &format!("↓{:.1}  ↑{:.1} MB", overview.network_rx_mb_delta, overview.network_tx_mb_delta),
            "Entre refrescos",
            severity_for_value(
                overview.network_rx_mb_delta + overview.network_tx_mb_delta,
                15.0,
                80.0,
            ),
        );
        metric_card(
            ui,
            "TEMP",
            &format!("{:.1} MB", overview.temp_total_mb),
            "TEMP / cachés vigiladas",
            severity_for_value(overview.temp_total_mb, 700.0, 2000.0),
        );
    });
}

// ── Sección ETW / WPR ─────────────────────────────────────────────────────────

fn draw_precision_section(
    ui: &mut egui::Ui,
    precision: &PrecisionStatus,
    precision_note: &mut String,
    precision_action: &mut Option<PrecisionAction>,
) {
    let recording = precision.is_recording;
    let accent = if recording { C_WARNING_FG } else { C_BLUE_FG };

    section_header(ui, "▪  Captura de precisión  ETW / WPR");
    ui.add_space(8.0);

    card_frame(if recording { C_WARNING_BG } else { BG_CARD }).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            tool_badge(
                ui,
                "WPR",
                precision.wpr_available,
            );
            tool_badge(ui, "WPA", precision.wpa_available);
            tool_badge(ui, "Tracerpt", precision.tracerpt_available);
            ui.add_space(8.0);
            let status_text = if recording { "● GRABANDO" } else { "○ En espera" };
            ui.label(
                RichText::new(status_text)
                    .strong()
                    .color(accent)
                    .size(13.0),
            );
        });

        ui.add_space(8.0);
        ui.label(RichText::new(&precision.guidance).color(TEXT_SECONDARY).size(13.0));
        ui.label(
            RichText::new(format!("Trazas → {}", precision.traces_directory))
                .small()
                .monospace()
                .color(TEXT_MUTED),
        );
        ui.label(
            RichText::new(format!("Motor → {}", precision.analyzer_label))
                .small()
                .color(TEXT_MUTED),
        );
        if let Some(path) = &precision.last_trace_path {
            ui.label(
                RichText::new(format!("Último ETL → {path}"))
                    .small()
                    .monospace()
                    .color(TEXT_SECONDARY),
            );
        }
        if let Some(path) = &precision.last_analysis_path {
            ui.label(
                RichText::new(format!("Resumen    → {path}"))
                    .small()
                    .monospace()
                    .color(C_HEALTHY_FG),
            );
        }
        if !precision.status_detail.is_empty() {
            ui.label(RichText::new(&precision.status_detail).small().color(TEXT_MUTED));
        }

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Descripción:").color(TEXT_SECONDARY).size(13.0));
            ui.add_sized(
                [400.0, 26.0],
                egui::TextEdit::singleline(precision_note)
                    .hint_text("Ej: disco al 100% mientras Windows Update descarga")
                    .text_color(TEXT_PRIMARY),
            );
        });

        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            if precision.wpr_available && !recording {
                if styled_button(ui, "▶  Iniciar captura WPR", C_HEALTHY_BG, C_HEALTHY_FG)
                    .clicked()
                {
                    *precision_action = Some(PrecisionAction::Start);
                }
            }
            if precision.wpr_available && recording {
                if styled_button(ui, "■  Detener y guardar ETL", C_WARNING_BG, C_WARNING_FG)
                    .clicked()
                {
                    *precision_action = Some(PrecisionAction::Stop);
                }
                if styled_button(ui, "✕  Cancelar captura", C_CRITICAL_BG, C_CRITICAL_FG)
                    .clicked()
                {
                    *precision_action = Some(PrecisionAction::Cancel);
                }
            }
            if !recording && precision.tracerpt_available && precision.last_trace_path.is_some() {
                if styled_button(ui, "⚡  Analizar último ETL", C_BLUE_BG, C_BLUE_FG).clicked() {
                    *precision_action = Some(PrecisionAction::Analyze);
                }
            }
        });
    });
}

// ── Análisis de traza ─────────────────────────────────────────────────────────

fn draw_trace_analysis_section(ui: &mut egui::Ui, analysis: &TraceAnalysisSummary) {
    section_header(ui, "▪  Resumen ETL procesado");
    ui.add_space(8.0);

    card_frame(BG_CARD).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let sev = analysis
                .findings
                .first()
                .map(|f| f.severity)
                .unwrap_or(Severity::Healthy);
            badge(ui, &analysis.headline, severity_fg(sev), severity_bg(sev));
            badge(
                ui,
                &format!("{} eventos", analysis.total_events),
                TEXT_SECONDARY,
                BG_CARD_HOVER,
            );
            badge(ui, &analysis.confidence, C_WARNING_FG, C_WARNING_BG);
        });

        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("ETL: {}", analysis.etl_path))
                .small()
                .monospace()
                .color(TEXT_MUTED),
        );
        ui.label(
            RichText::new(format!("Salida: {}", analysis.output_directory))
                .small()
                .monospace()
                .color(TEXT_MUTED),
        );

        if !analysis.findings.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("Hallazgos principales")
                    .strong()
                    .color(TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            for finding in analysis.findings.iter().take(3) {
                card_frame(severity_bg(finding.severity)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("▸")
                                .color(severity_fg(finding.severity)),
                        );
                        ui.label(
                            RichText::new(&finding.title)
                                .strong()
                                .color(severity_fg(finding.severity)),
                        );
                    });
                    ui.label(RichText::new(&finding.detail).color(TEXT_SECONDARY));
                    ui.label(
                        RichText::new(format!("Evidencia: {}", finding.evidence))
                            .small()
                            .monospace()
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(4.0);
            }
        }

        ui.add_space(6.0);
        ui.columns(3, |cols| {
            draw_trace_processes(&mut cols[0], &analysis.hot_processes);
            draw_trace_paths(&mut cols[1], &analysis.hot_paths);
            draw_trace_context(&mut cols[2], analysis);
        });
    });
}

fn draw_trace_processes(ui: &mut egui::Ui, processes: &[TraceProcessSummary]) {
    col_label(ui, "Procesos repetidos");
    egui::ScrollArea::vertical()
        .id_source("trace_procs")
        .max_height(200.0)
        .show(ui, |ui| {
            for p in processes.iter().take(6) {
                card_frame(severity_bg(p.severity)).show(ui, |ui| {
                    ui.label(
                        RichText::new(&p.name)
                            .strong()
                            .color(severity_fg(p.severity)),
                    );
                    ui.label(
                        RichText::new(format!("× {}  ·  {}", p.occurrences, p.reason))
                            .small()
                            .color(TEXT_SECONDARY),
                    );
                });
                ui.add_space(3.0);
            }
        });
}

fn draw_trace_paths(ui: &mut egui::Ui, paths: &[TracePathSummary]) {
    col_label(ui, "Rutas repetidas");
    egui::ScrollArea::vertical()
        .id_source("trace_paths")
        .max_height(200.0)
        .show(ui, |ui| {
            for p in paths.iter().take(6) {
                card_frame(severity_bg(p.severity)).show(ui, |ui| {
                    ui.label(
                        RichText::new(&p.path)
                            .small()
                            .strong()
                            .color(severity_fg(p.severity)),
                    );
                    ui.label(
                        RichText::new(format!("{}  ·  × {}", p.category, p.occurrences))
                            .small()
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(3.0);
            }
        });
}

fn draw_trace_context(ui: &mut egui::Ui, analysis: &TraceAnalysisSummary) {
    col_label(ui, "Contexto rápido");
    if !analysis.public_ips.is_empty() {
        ui.label(RichText::new("IPs públicas").small().strong().color(C_BLUE_FG));
        for ip in analysis.public_ips.iter().take(5) {
            ui.label(RichText::new(ip).small().monospace().color(TEXT_SECONDARY));
        }
        ui.add_space(4.0);
    }
    if !analysis.indicators.is_empty() {
        ui.label(
            RichText::new("Indicadores")
                .small()
                .strong()
                .color(C_WARNING_FG),
        );
        for indicator in analysis.indicators.iter().take(5) {
            ui.label(RichText::new(indicator).small().color(TEXT_SECONDARY));
        }
        ui.add_space(4.0);
    }
    if !analysis.limitations.is_empty() {
        ui.label(RichText::new("Límites").small().strong().color(TEXT_MUTED));
        for limit in analysis.limitations.iter().take(3) {
            ui.label(RichText::new(limit).small().italics().color(TEXT_MUTED));
        }
    }
}

// ── Alertas ───────────────────────────────────────────────────────────────────

fn draw_alerts(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
    if snapshot.alerts.is_empty() {
        return;
    }
    section_header(ui, "▪  Dónde mirar primero");
    ui.add_space(8.0);
    for alert in snapshot.alerts.iter().take(5) {
        let fg = severity_fg(alert.severity);
        let bg = severity_bg(alert.severity);
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.35)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(severity_icon(alert.severity))
                            .color(fg)
                            .size(16.0),
                    );
                    ui.label(RichText::new(&alert.title).strong().color(fg).size(14.0));
                    if let Some(pid) = alert.pid {
                        badge(ui, &format!("PID {pid}"), TEXT_MUTED, BG_CARD_HOVER);
                    }
                });
                ui.add_space(2.0);
                ui.label(RichText::new(&alert.detail).color(TEXT_SECONDARY));
                ui.label(RichText::new(&alert.hint).italics().color(TEXT_MUTED).size(12.0));
                if let Some(path) = &alert.path {
                    ui.label(
                        RichText::new(path)
                            .small()
                            .monospace()
                            .color(TEXT_MUTED),
                    );
                }
            });
        ui.add_space(5.0);
    }
}

// ── Procesos ──────────────────────────────────────────────────────────────────

fn draw_processes_section<F: FnMut(u32)>(
    ui: &mut egui::Ui,
    snapshot: &SystemSnapshot,
    filter_text: &str,
    mut on_terminate: F,
) {
    section_header(ui, "▪  Procesos dominantes");
    ui.label(
        RichText::new("Ordenados por severidad, escritura, memoria y CPU")
            .small()
            .color(TEXT_MUTED),
    );
    ui.add_space(6.0);
    egui::ScrollArea::vertical()
        .id_source("procs")
        .max_height(380.0)
        .show(ui, |ui| {
            for process in snapshot
                .processes
                .iter()
                .filter(|p| matches_filter(&p.name, &p.exe_path, filter_text))
                .take(18)
            {
                process_row(ui, process, &mut on_terminate);
                ui.add_space(4.0);
            }
        });
}

fn process_row<F: FnMut(u32)>(ui: &mut egui::Ui, p: &ProcessInsight, on_terminate: &mut F) {
    let fg = severity_fg(p.severity);
    let bg = severity_bg(p.severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.25)))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(severity_icon(p.severity)).color(fg));
                ui.label(RichText::new(&p.name).strong().color(TEXT_PRIMARY));
                badge(ui, &format!("PID {}", p.pid), TEXT_MUTED, BG_CARD_HOVER);
                badge(ui, &p.category, TEXT_MUTED, BG_CARD_HOVER);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if p.can_terminate
                        && styled_button(ui, "Finalizar", C_CRITICAL_BG, C_CRITICAL_FG).clicked()
                    {
                        on_terminate(p.pid);
                    }
                });
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                metric_pill(ui, &format!("CPU  {:.1}%", p.cpu_percent), fg);
                metric_pill(ui, &format!("RAM  {:.0} MB", p.memory_mb), fg);
                metric_pill(ui, &format!("W  {:.1} MB", p.io_write_mb_delta), fg);
                metric_pill(ui, &format!("R  {:.1} MB", p.io_read_mb_delta), fg);
                metric_pill(ui, &format!("Score  {}", p.score), fg);
            });
            ui.add_space(2.0);
            ui.label(
                RichText::new(&p.exe_path)
                    .small()
                    .monospace()
                    .color(TEXT_MUTED),
            );
            if !p.reasons.is_empty() {
                ui.label(
                    RichText::new(p.reasons.join("  ·  "))
                        .small()
                        .color(TEXT_SECONDARY),
                );
            }
        });
}

// ── Temporales ────────────────────────────────────────────────────────────────

fn draw_temp_section(ui: &mut egui::Ui, snapshot: &SystemSnapshot, filter_text: &str) {
    section_header(ui, "▪  Archivos temporales");
    ui.label(
        RichText::new("Basura de instaladores, actualizaciones y exportaciones")
            .small()
            .color(TEXT_MUTED),
    );
    ui.add_space(6.0);
    egui::ScrollArea::vertical()
        .id_source("temp")
        .max_height(380.0)
        .show(ui, |ui| {
            for entry in snapshot
                .temp
                .top_entries
                .iter()
                .filter(|e| matches_filter(&e.path, &e.note, filter_text))
            {
                temp_row(ui, entry);
                ui.add_space(4.0);
            }
        });
    if !snapshot.temp.limitations.is_empty() {
        ui.add_space(4.0);
        for lim in &snapshot.temp.limitations {
            ui.label(RichText::new(lim).small().italics().color(TEXT_MUTED));
        }
    }
}

fn temp_row(ui: &mut egui::Ui, entry: &TempEntry) {
    let fg = severity_fg(entry.severity);
    let bg = severity_bg(entry.severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.25)))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(&entry.path)
                    .strong()
                    .small()
                    .color(TEXT_PRIMARY),
            );
            ui.add_space(3.0);
            ui.horizontal_wrapped(|ui| {
                metric_pill(ui, &format!("{:.1} MB", entry.size_mb), fg);
                metric_pill(ui, &format!("{} archivos", entry.file_count), fg);
            });
            ui.add_space(2.0);
            ui.label(RichText::new(&entry.note).small().color(TEXT_SECONDARY));
        });
}

// ── Conexiones ────────────────────────────────────────────────────────────────

fn draw_connections_section<F: FnMut(&str)>(
    ui: &mut egui::Ui,
    snapshot: &SystemSnapshot,
    filter_text: &str,
    only_public: bool,
    mut on_block_ip: F,
) {
    section_header(ui, "▪  Conexiones activas");
    ui.label(
        RichText::new("Foco en procesos con IP pública y rutas poco confiables")
            .small()
            .color(TEXT_MUTED),
    );
    ui.add_space(6.0);
    egui::ScrollArea::vertical()
        .id_source("conns")
        .max_height(380.0)
        .show(ui, |ui| {
            let items: Vec<&ConnectionInsight> = snapshot
                .connections
                .iter()
                .filter(|c| !only_public || c.is_public_remote)
                .filter(|c| {
                    matches_filter(
                        &c.process_name,
                        &format!("{} {}", c.remote_address, c.exe_path),
                        filter_text,
                    )
                })
                .take(18)
                .collect();

            for conn in items {
                let fg = severity_fg(conn.severity);
                let bg = severity_bg(conn.severity);
                egui::Frame::none()
                    .fill(bg)
                    .stroke(Stroke::new(1.0, fg.linear_multiply(0.25)))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                RichText::new(&conn.process_name)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            badge(
                                ui,
                                &format!("PID {}", conn.pid),
                                TEXT_MUTED,
                                BG_CARD_HOVER,
                            );
                            badge(
                                ui,
                                &format!("{} {}", conn.protocol, conn.state),
                                TEXT_MUTED,
                                BG_CARD_HOVER,
                            );
                            if conn.is_public_remote {
                                if styled_button(ui, "Bloquear IP", C_CRITICAL_BG, C_CRITICAL_FG)
                                    .clicked()
                                {
                                    on_block_ip(&conn.remote_address);
                                }
                            }
                        });
                        ui.label(
                            RichText::new(format!(
                                "{} → {}",
                                conn.local_address, conn.remote_address
                            ))
                            .small()
                            .monospace()
                            .color(TEXT_SECONDARY),
                        );
                        ui.label(RichText::new(&conn.reason).small().color(TEXT_MUTED));
                        ui.label(
                            RichText::new(&conn.exe_path)
                                .small()
                                .monospace()
                                .color(TEXT_MUTED),
                        );
                    });
                ui.add_space(4.0);
            }
        });
}

// ── Servicios y eventos ───────────────────────────────────────────────────────

fn draw_events_and_services<F: FnMut(&str)>(
    ui: &mut egui::Ui,
    snapshot: &SystemSnapshot,
    mut on_stop_service: F,
) {
    section_header(ui, "▪  Servicios  ·  Eventos recientes");
    ui.label(
        RichText::new(
            "Correlaciona lentitud con Windows Update, BITS, Delivery Optimization o errores",
        )
        .small()
        .color(TEXT_MUTED),
    );
    ui.add_space(6.0);

    for svc in &snapshot.services {
        let sev = service_severity(svc);
        let fg = severity_fg(sev);
        let bg = severity_bg(sev);
        egui::Frame::none()
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.25)))
            .rounding(Rounding::same(6.0))
            .inner_margin(Margin::same(8.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(&svc.display_name)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    badge(ui, &svc.status, fg, bg);
                    badge(
                        ui,
                        &format!("Inicio {}", svc.start_type),
                        TEXT_MUTED,
                        BG_CARD_HOVER,
                    );
                    if is_stoppable_service(svc)
                        && svc.status.eq_ignore_ascii_case("Running")
                        && styled_button(ui, "Detener", C_WARNING_BG, C_WARNING_FG).clicked()
                    {
                        on_stop_service(&svc.name);
                    }
                });
            });
        ui.add_space(3.0);
    }

    ui.add_space(8.0);
    col_label(ui, "Eventos del sistema");
    ui.add_space(4.0);
    egui::ScrollArea::vertical()
        .id_source("events")
        .max_height(240.0)
        .show(ui, |ui| {
            for evt in snapshot.events.iter().take(10) {
                let sev = if evt.level.eq_ignore_ascii_case("Error") {
                    Severity::Critical
                } else {
                    Severity::Warning
                };
                let fg = severity_fg(sev);
                egui::Frame::none()
                    .fill(severity_bg(sev))
                    .stroke(Stroke::new(1.0, fg.linear_multiply(0.2)))
                    .rounding(Rounding::same(4.0))
                    .inner_margin(Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!(
                                "{}  ·  {}  ·  ID {}",
                                evt.timestamp, evt.provider, evt.id
                            ))
                            .strong()
                            .small()
                            .color(fg),
                        );
                        ui.label(RichText::new(&evt.message).small().color(TEXT_SECONDARY));
                    });
                ui.add_space(3.0);
            }
        });
}

// ── Widgets reutilizables ─────────────────────────────────────────────────────

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = BG_APP;
    visuals.panel_fill = BG_APP;
    visuals.faint_bg_color = BG_CARD;
    visuals.extreme_bg_color = BG_PANEL;
    visuals.code_bg_color = BG_CARD;
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.active.bg_fill = BORDER_ACCENT;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.selection.bg_fill = BORDER_ACCENT.linear_multiply(0.4);
    visuals.selection.stroke = Stroke::new(1.0, C_BLUE_FG);
    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);
    visuals.override_text_color = Some(TEXT_PRIMARY);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.window_margin = Margin::same(12.0);
    ctx.set_style(style);
}

fn card_frame(fill: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(fill)
        .stroke(Stroke::new(1.0, BORDER))
        .rounding(Rounding::same(6.0))
        .inner_margin(Margin::same(10.0))
}

fn styled_button(
    ui: &mut egui::Ui,
    label: &str,
    bg: Color32,
    fg: Color32,
) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).color(fg).size(12.5))
            .fill(bg)
            .stroke(Stroke::new(1.0, fg.linear_multiply(0.5)))
            .rounding(Rounding::same(5.0)),
    )
}

fn badge(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    let text_shape = RichText::new(text).size(11.5).color(fg);
    let galley = ui.painter().layout_no_wrap(
        text.to_owned(),
        FontId::proportional(11.5),
        fg,
    );
    let width = galley.size().x + 12.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 20.0), Sense::hover());
    ui.painter().rect_filled(rect, Rounding::same(10.0), bg);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.5),
        fg,
    );
    let _ = text_shape;
}

fn metric_pill(ui: &mut egui::Ui, text: &str, fg: Color32) {
    let chars = text.chars().count() as f32;
    let width = (chars * 7.5 + 16.0).max(60.0).min(240.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(11.0), fg.linear_multiply(0.12));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        FontId::proportional(11.5),
        fg,
    );
}

fn metric_card(ui: &mut egui::Ui, title: &str, value: &str, subtitle: &str, severity: Severity) {
    let fg = severity_fg(severity);
    let bg = severity_bg(severity);
    egui::Frame::none()
        .fill(bg)
        .stroke(Stroke::new(1.0, fg.linear_multiply(0.4)))
        .rounding(Rounding::same(8.0))
        .inner_margin(Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(190.0, 80.0));
            ui.label(RichText::new(title).size(10.5).color(TEXT_MUTED).strong());
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").color(fg).size(18.0));
                ui.label(RichText::new(value).size(17.0).strong().color(TEXT_PRIMARY));
            });
            ui.add_space(2.0);
            ui.label(RichText::new(subtitle).size(11.0).color(TEXT_MUTED));
        });
}

fn section_header(ui: &mut egui::Ui, title: &str) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(title)
                .strong()
                .size(13.0)
                .color(TEXT_SECONDARY),
        );
    });
    ui.add_space(2.0);
    let stroke_rect = ui.available_rect_before_wrap();
    let line_y = stroke_rect.top() + 1.0;
    ui.painter().line_segment(
        [
            egui::pos2(stroke_rect.left(), line_y),
            egui::pos2(stroke_rect.right(), line_y),
        ],
        Stroke::new(1.0, BORDER),
    );
}

fn col_label(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).strong().size(12.0).color(TEXT_SECONDARY));
    ui.add_space(4.0);
}

fn tool_badge(ui: &mut egui::Ui, name: &str, available: bool) {
    let (fg, bg) = if available {
        (C_HEALTHY_FG, C_HEALTHY_BG)
    } else {
        (TEXT_MUTED, BG_CARD_HOVER)
    };
    badge(ui, &format!("{name} {}", if available { "✓" } else { "—" }), fg, bg);
}

fn section_gap(ui: &mut egui::Ui) {
    ui.add_space(18.0);
}

fn loading_screen(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label(
            RichText::new("Capturando datos del sistema…")
                .size(16.0)
                .color(TEXT_MUTED),
        );
    });
}

// ── Helpers de severidad ──────────────────────────────────────────────────────

fn severity_for_value(value: f32, warn: f32, crit: f32) -> Severity {
    if value >= crit {
        Severity::Critical
    } else if value >= warn {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn severity_fg(severity: Severity) -> Color32 {
    match severity {
        Severity::Healthy => C_HEALTHY_FG,
        Severity::Warning => C_WARNING_FG,
        Severity::Critical => C_CRITICAL_FG,
    }
}

fn severity_bg(severity: Severity) -> Color32 {
    match severity {
        Severity::Healthy => C_HEALTHY_BG,
        Severity::Warning => C_WARNING_BG,
        Severity::Critical => C_CRITICAL_BG,
    }
}

fn severity_icon(severity: Severity) -> &'static str {
    match severity {
        Severity::Healthy => "✓",
        Severity::Warning => "⚠",
        Severity::Critical => "✕",
    }
}

fn service_severity(service: &ServiceState) -> Severity {
    let lowered = service.name.to_ascii_lowercase();
    if ["wuauserv", "bits", "dosvc", "sysmain"].contains(&lowered.as_str())
        && service.status.eq_ignore_ascii_case("Running")
    {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

fn is_stoppable_service(service: &ServiceState) -> bool {
    ["wuauserv", "bits", "dosvc", "sysmain"]
        .contains(&service.name.to_ascii_lowercase().as_str())
}

fn matches_filter(primary: &str, secondary: &str, filter_text: &str) -> bool {
    let needle = filter_text.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    primary.to_ascii_lowercase().contains(&needle)
        || secondary.to_ascii_lowercase().contains(&needle)
}
