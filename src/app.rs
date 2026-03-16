//! Capa de interfaz.
//!
//! La prioridad aquí es claridad: panel superior de estado, alertas concretas y
//! luego listas accionables. No se busca una interfaz espectacular; se busca una
//! interfaz que responda rápido y te diga dónde mirar primero.

use crate::models::{
    ConnectionInsight, PrecisionStatus, ProcessInsight, ServiceState, Severity, SystemSnapshot, TempEntry,
    TraceAnalysisSummary, TracePathSummary, TraceProcessSummary,
};
use crate::services::inspector::InspectorService;
use eframe::egui::{self, Color32, RichText, Sense};
use std::time::{Duration, Instant};

/// Acciones activadas desde la UI para el modo de precisión.
enum PrecisionAction {
    Start,
    Stop,
    Cancel,
    Analyze,
}

/// Estado completo de la aplicación.
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
}

impl RootCauseApp {
    /// Crea la app y prepara el servicio principal.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
        };

        match inspector {
            Ok(service) => {
                app.status_line = service.latest_history_line();
                app.inspector = Some(service);
            }
            Err(error) => {
                app.status_line = format!("Error inicializando el motor: {error}");
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
                    "Última captura {} | {}",
                    snapshot.collected_at.format("%Y-%m-%d %H:%M:%S"),
                    snapshot.overview.primary_reason
                );
                self.snapshot = Some(snapshot);
                self.last_refresh_at = Instant::now();
            }
            Err(error) => {
                self.status_line = format!("Fallo al capturar datos: {error}");
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
                self.status_line = format!("Reporte exportado en {path}");
            }
            Err(error) => {
                self.status_line = format!("No se pudo exportar: {error}");
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

        match result {
            Ok(message) => {
                self.status_line = message;
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(error) => {
                self.status_line = format!("No se pudo iniciar WPR: {error}");
            }
        }
    }

    fn stop_precision_capture(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.stop_precision_capture(&self.precision_note)
        };

        match result {
            Ok(message) => {
                self.status_line = message;
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(error) => {
                self.status_line = format!("No se pudo detener WPR: {error}");
            }
        }
    }

    fn cancel_precision_capture(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.cancel_precision_capture()
        };

        match result {
            Ok(message) => {
                self.status_line = message;
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(error) => {
                self.status_line = format!("No se pudo cancelar WPR: {error}");
            }
        }
    }

    fn analyze_last_trace(&mut self) {
        let result = {
            let Some(inspector) = self.inspector.as_mut() else {
                return;
            };
            inspector.analyze_last_precision_trace()
        };

        match result {
            Ok(message) => {
                self.status_line = message;
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
                self.refresh_now();
            }
            Err(error) => {
                self.status_line = format!("No se pudo resumir el ETL: {error}");
            }
        }
    }

    fn terminate_process(&mut self, pid: u32) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };

        match inspector.terminate_process(pid) {
            Ok(message) => {
                self.status_line = format!("Proceso terminado: {message}");
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(error) => {
                self.status_line = format!("No se pudo finalizar PID {pid}: {error}");
            }
        }
    }

    fn block_remote_ip(&mut self, ip: &str) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };

        match inspector.block_remote_ip(ip) {
            Ok(message) => {
                self.status_line = message;
            }
            Err(error) => {
                self.status_line = format!("No se pudo bloquear la IP: {error}");
            }
        }
    }

    fn stop_service(&mut self, service_name: &str) {
        let Some(inspector) = self.inspector.as_ref() else {
            return;
        };

        match inspector.stop_service(service_name) {
            Ok(message) => {
                self.status_line = message;
                self.last_refresh_at = Instant::now() - Duration::from_secs(self.refresh_interval_secs);
            }
            Err(error) => {
                self.status_line = format!("No se pudo detener el servicio {service_name}: {error}");
            }
        }
    }
}

impl eframe::App for RootCauseApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));

        if self.snapshot.is_none()
            || (self.auto_refresh
                && self.last_refresh_at.elapsed() >= Duration::from_secs(self.refresh_interval_secs))
        {
            self.refresh_now();
        }

        egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::from_rgb(24, 88, 201))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(155, 210, 255)))
                    .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new("RC").strong().color(Color32::WHITE));
                    });
                ui.heading("RootCause Demo");
                ui.separator();
                ui.label("Build pública de evaluación. Detecta quién está cargando disco, memoria, temporales, red o una traza ETL reciente.");
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if ui.button("Actualizar ahora").clicked() {
                    self.refresh_now();
                }
                if ui.button("Exportar JSON").clicked() {
                    self.export_snapshot();
                }
                ui.checkbox(&mut self.auto_refresh, "Auto refresco");
                ui.add(egui::Slider::new(&mut self.refresh_interval_secs, 3..=15).text("segundos"));
                ui.checkbox(&mut self.only_public_connections, "Solo IP públicas");
                ui.label("Filtro:");
                ui.add_sized([260.0, 28.0], egui::TextEdit::singleline(&mut self.filter_text));
            });
        });

        egui::TopBottomPanel::bottom("bottom-status").show(ctx, |ui| {
            ui.label(self.status_line.clone());
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(snapshot) = self.snapshot.clone() else {
                ui.label("Cargando captura inicial...");
                return;
            };

            draw_overview(ui, &snapshot);
            ui.add_space(10.0);

            let mut precision_action: Option<PrecisionAction> = None;
            draw_precision_section(ui, &snapshot.precision, &mut self.precision_note, &mut precision_action);
            ui.add_space(10.0);

            if let Some(trace_analysis) = &snapshot.trace_analysis {
                draw_trace_analysis_section(ui, trace_analysis);
                ui.add_space(10.0);
            }

            draw_alerts(ui, &snapshot);
            ui.add_space(10.0);

            let mut pid_to_kill: Option<u32> = None;
            ui.columns(2, |columns| {
                draw_processes_section(&mut columns[0], &snapshot, &self.filter_text, |pid| {
                    pid_to_kill = Some(pid)
                });
                draw_temp_section(&mut columns[1], &snapshot, &self.filter_text);
            });

            ui.add_space(8.0);
            let mut endpoint_to_block: Option<String> = None;
            let mut service_to_stop: Option<String> = None;
            ui.columns(2, |columns| {
                draw_connections_section(
                    &mut columns[0],
                    &snapshot,
                    &self.filter_text,
                    self.only_public_connections,
                    |ip| endpoint_to_block = Some(ip.to_owned()),
                );
                draw_events_and_services(&mut columns[1], &snapshot, |service_name| {
                    service_to_stop = Some(service_name.to_owned())
                });
            });

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
            if let Some(endpoint) = endpoint_to_block {
                self.block_remote_ip(&endpoint);
            }
            if let Some(service_name) = service_to_stop {
                self.stop_service(&service_name);
            }
        });
    }
}

fn draw_overview(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
    let overview = &snapshot.overview;
    ui.horizontal_wrapped(|ui| {
        status_card(
            ui,
            "Semáforo",
            overview.primary_severity.label(),
            &overview.primary_reason,
            overview.primary_severity,
        );
        status_card(
            ui,
            "CPU",
            &format!("{:.1}%", overview.cpu_usage_percent),
            "Uso global del equipo",
            severity_for_value(overview.cpu_usage_percent, 55.0, 80.0),
        );
        status_card(
            ui,
            "RAM",
            &format!("{:.1}/{:.1} GB", overview.memory_used_gb, overview.memory_total_gb),
            "Memoria física actualmente usada",
            severity_for_value(
                overview.memory_used_gb / overview.memory_total_gb.max(0.1) * 100.0,
                70.0,
                88.0,
            ),
        );
        status_card(
            ui,
            "Disco (I/O)",
            &format!("R {:.1} / W {:.1} MB", overview.io_read_mb_delta, overview.io_write_mb_delta),
            "Suma de I/O de procesos en el intervalo",
            severity_for_value(overview.io_write_mb_delta, 80.0, 220.0),
        );
        status_card(
            ui,
            "Red",
            &format!("↓ {:.1} / ↑ {:.1} MB", overview.network_rx_mb_delta, overview.network_tx_mb_delta),
            "Actividad observada entre refrescos",
            severity_for_value(overview.network_rx_mb_delta + overview.network_tx_mb_delta, 15.0, 80.0),
        );
        status_card(
            ui,
            "TEMP",
            &format!("{:.1} MB", overview.temp_total_mb),
            "Tamaño encontrado en TEMP/cachés vigiladas",
            severity_for_value(overview.temp_total_mb, 700.0, 2000.0),
        );
    });
}

fn draw_precision_section(
    ui: &mut egui::Ui,
    precision: &PrecisionStatus,
    precision_note: &mut String,
    precision_action: &mut Option<PrecisionAction>,
) {
    ui.heading("Modo de precisión ETW/WPR");
    egui::Frame::group(ui.style())
        .fill(if precision.wpr_available {
            if precision.is_recording {
                Color32::from_rgb(73, 60, 18)
            } else {
                Color32::from_rgb(18, 46, 34)
            }
        } else {
            Color32::from_rgb(45, 45, 45)
        })
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                metric_chip(
                    ui,
                    format!("WPR {}", if precision.wpr_available { "OK" } else { "NO" }),
                    if precision.wpr_available { Severity::Healthy } else { Severity::Warning },
                );
                metric_chip(
                    ui,
                    format!("WPA {}", if precision.wpa_available { "OK" } else { "NO" }),
                    if precision.wpa_available { Severity::Healthy } else { Severity::Warning },
                );
                metric_chip(
                    ui,
                    format!("Tracerpt {}", if precision.tracerpt_available { "OK" } else { "NO" }),
                    if precision.tracerpt_available { Severity::Healthy } else { Severity::Warning },
                );
                metric_chip(
                    ui,
                    format!("Captura {}", if precision.is_recording { "ACTIVA" } else { "INACTIVA" }),
                    if precision.is_recording { Severity::Warning } else { Severity::Healthy },
                );
            });

            ui.add_space(6.0);
            ui.label(precision.guidance.clone());
            ui.label(RichText::new(format!("Carpeta de trazas: {}", precision.traces_directory)).small().monospace());
            ui.label(RichText::new(format!("Motor de resumen: {}", precision.analyzer_label)).small());
            if let Some(path) = &precision.last_trace_path {
                ui.label(RichText::new(format!("Último ETL: {path}")).small().monospace());
            }
            if let Some(path) = &precision.last_analysis_path {
                ui.label(RichText::new(format!("Último resumen: {path}")).small().monospace());
            }
            ui.label(RichText::new(precision.status_detail.clone()).small());

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.label("Descripción de la captura:");
                ui.add_sized(
                    [420.0, 28.0],
                    egui::TextEdit::singleline(precision_note)
                        .hint_text("Ej: disco al 100% mientras Windows Update descarga"),
                );
            });

            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if precision.wpr_available && !precision.is_recording && ui.button("Iniciar captura WPR").clicked() {
                    *precision_action = Some(PrecisionAction::Start);
                }
                if precision.wpr_available && precision.is_recording && ui.button("Detener y guardar ETL").clicked() {
                    *precision_action = Some(PrecisionAction::Stop);
                }
                if precision.wpr_available && precision.is_recording && ui.button("Cancelar captura").clicked() {
                    *precision_action = Some(PrecisionAction::Cancel);
                }
                if !precision.is_recording
                    && precision.tracerpt_available
                    && precision.last_trace_path.is_some()
                    && ui.button("Resumir último ETL").clicked()
                {
                    *precision_action = Some(PrecisionAction::Analyze);
                }
            });
        });
}

fn draw_trace_analysis_section(ui: &mut egui::Ui, analysis: &TraceAnalysisSummary) {
    ui.heading("Resumen del último ETL procesado");
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(25, 32, 42))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let severity = analysis
                    .findings
                    .first()
                    .map(|item| item.severity)
                    .unwrap_or(Severity::Healthy);
                metric_chip(ui, analysis.headline.clone(), severity);
                metric_chip(ui, format!("Eventos {}", analysis.total_events), Severity::Healthy);
                metric_chip(ui, analysis.confidence.clone(), Severity::Warning);
            });
            ui.add_space(6.0);
            ui.label(RichText::new(format!("ETL: {}", analysis.etl_path)).small().monospace());
            ui.label(RichText::new(format!("Salida: {}", analysis.output_directory)).small().monospace());

            if !analysis.findings.is_empty() {
                ui.add_space(8.0);
                ui.label(RichText::new("Hallazgos principales").strong());
                for finding in analysis.findings.iter().take(3) {
                    egui::Frame::group(ui.style())
                        .fill(severity_bg(finding.severity))
                        .show(ui, |ui| {
                            ui.label(RichText::new(&finding.title).strong());
                            ui.label(finding.detail.clone());
                            ui.label(RichText::new(format!("Evidencia: {}", finding.evidence)).small().monospace());
                        });
                    ui.add_space(4.0);
                }
            }

            ui.columns(3, |columns| {
                draw_trace_processes(&mut columns[0], &analysis.hot_processes);
                draw_trace_paths(&mut columns[1], &analysis.hot_paths);
                draw_trace_context(&mut columns[2], analysis);
            });
        });
}

fn draw_trace_processes(ui: &mut egui::Ui, processes: &[TraceProcessSummary]) {
    ui.label(RichText::new("Procesos repetidos").strong());
    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
        for process in processes.iter().take(6) {
            egui::Frame::group(ui.style())
                .fill(severity_bg(process.severity))
                .show(ui, |ui| {
                    ui.label(RichText::new(&process.name).strong());
                    ui.label(format!("Apariciones: {}", process.occurrences));
                    ui.label(process.reason.clone());
                });
            ui.add_space(4.0);
        }
    });
}

fn draw_trace_paths(ui: &mut egui::Ui, paths: &[TracePathSummary]) {
    ui.label(RichText::new("Rutas repetidas").strong());
    egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
        for path in paths.iter().take(6) {
            egui::Frame::group(ui.style())
                .fill(severity_bg(path.severity))
                .show(ui, |ui| {
                    ui.label(RichText::new(&path.path).small().strong());
                    ui.label(format!("{} | {} apariciones", path.category, path.occurrences));
                });
            ui.add_space(4.0);
        }
    });
}

fn draw_trace_context(ui: &mut egui::Ui, analysis: &TraceAnalysisSummary) {
    ui.label(RichText::new("Contexto rápido").strong());
    if !analysis.public_ips.is_empty() {
        ui.label(RichText::new("IPs públicas").small().strong());
        for ip in analysis.public_ips.iter().take(5) {
            ui.label(ip);
        }
        ui.add_space(6.0);
    }
    if !analysis.indicators.is_empty() {
        ui.label(RichText::new("Indicadores").small().strong());
        for indicator in analysis.indicators.iter().take(5) {
            ui.label(indicator);
        }
        ui.add_space(6.0);
    }
    if !analysis.limitations.is_empty() {
        ui.label(RichText::new("Límites").small().strong());
        for limit in analysis.limitations.iter().take(3) {
            ui.label(RichText::new(limit).small().italics());
        }
    }
}

fn draw_alerts(ui: &mut egui::Ui, snapshot: &SystemSnapshot) {
    ui.heading("Dónde mirar primero");
    for alert in snapshot.alerts.iter().take(5) {
        let fill = severity_bg(alert.severity);
        egui::Frame::group(ui.style())
            .fill(fill)
            .inner_margin(egui::Margin::same(10.0))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&alert.title).strong().color(severity_fg(alert.severity)));
                    if let Some(pid) = alert.pid {
                        ui.label(RichText::new(format!("PID {pid}")).monospace());
                    }
                });
                ui.label(alert.detail.clone());
                ui.label(RichText::new(alert.hint.clone()).italics());
                if let Some(path) = &alert.path {
                    ui.label(RichText::new(path).small().monospace());
                }
            });
        ui.add_space(6.0);
    }
}

fn draw_processes_section<F: FnMut(u32)>(
    ui: &mut egui::Ui,
    snapshot: &SystemSnapshot,
    filter_text: &str,
    mut on_terminate: F,
) {
    ui.heading("Procesos dominantes");
    ui.label("Ordenados por severidad, escritura, memoria y CPU.");
    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
        for process in snapshot
            .processes
            .iter()
            .filter(|process| matches_filter(&process.name, &process.exe_path, filter_text))
            .take(18)
        {
            process_row(ui, process, &mut on_terminate);
            ui.add_space(6.0);
        }
    });
}

fn process_row<F: FnMut(u32)>(ui: &mut egui::Ui, process: &ProcessInsight, on_terminate: &mut F) {
    egui::Frame::group(ui.style())
        .fill(severity_bg(process.severity))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&process.name).strong());
                ui.label(RichText::new(format!("PID {}", process.pid)).monospace());
                ui.label(RichText::new(process.category.clone()).small());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if process.can_terminate && ui.button("Finalizar").clicked() {
                        on_terminate(process.pid);
                    }
                });
            });
            ui.horizontal_wrapped(|ui| {
                metric_chip(ui, format!("CPU {:.1}%", process.cpu_percent), process.severity);
                metric_chip(ui, format!("RAM {:.0} MB", process.memory_mb), process.severity);
                metric_chip(ui, format!("W {:.1} MB", process.io_write_mb_delta), process.severity);
                metric_chip(ui, format!("R {:.1} MB", process.io_read_mb_delta), process.severity);
                metric_chip(ui, format!("Puntaje {}", process.score), process.severity);
            });
            ui.label(RichText::new(process.exe_path.clone()).small().monospace());
            ui.label(process.reasons.join(" | "));
        });
}

fn draw_temp_section(ui: &mut egui::Ui, snapshot: &SystemSnapshot, filter_text: &str) {
    ui.heading("Archivos y carpetas temporales");
    ui.label("Aquí suele aparecer basura de instaladores, actualizaciones y exportaciones pesadas.");
    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
        for entry in snapshot
            .temp
            .top_entries
            .iter()
            .filter(|entry| matches_filter(&entry.path, &entry.note, filter_text))
        {
            temp_row(ui, entry);
            ui.add_space(6.0);
        }
    });

    if !snapshot.temp.limitations.is_empty() {
        ui.add_space(6.0);
        for limitation in &snapshot.temp.limitations {
            ui.label(RichText::new(limitation).small().italics());
        }
    }
}

fn temp_row(ui: &mut egui::Ui, entry: &TempEntry) {
    egui::Frame::group(ui.style())
        .fill(severity_bg(entry.severity))
        .show(ui, |ui| {
            ui.label(RichText::new(&entry.path).strong().small());
            ui.horizontal_wrapped(|ui| {
                metric_chip(ui, format!("{:.1} MB", entry.size_mb), entry.severity);
                metric_chip(ui, format!("{} archivos", entry.file_count), entry.severity);
            });
            ui.label(entry.note.clone());
        });
}

fn draw_connections_section<F: FnMut(&str)>(
    ui: &mut egui::Ui,
    snapshot: &SystemSnapshot,
    filter_text: &str,
    only_public_connections: bool,
    mut on_block_ip: F,
) {
    ui.heading("Conexiones activas");
    ui.label("El foco está en procesos con IP pública y rutas poco confiables.");
    egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
        let filtered: Vec<&ConnectionInsight> = snapshot
            .connections
            .iter()
            .filter(|connection| !only_public_connections || connection.is_public_remote)
            .filter(|connection| {
                matches_filter(
                    &connection.process_name,
                    &format!("{} {}", connection.remote_address, connection.exe_path),
                    filter_text,
                )
            })
            .take(18)
            .collect();

        for connection in filtered {
            egui::Frame::group(ui.style())
                .fill(severity_bg(connection.severity))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&connection.process_name).strong());
                        ui.label(RichText::new(format!("PID {}", connection.pid)).monospace());
                        ui.label(format!("{} {}", connection.protocol, connection.state));
                        if connection.is_public_remote && ui.button("Bloquear IP").clicked() {
                            on_block_ip(&connection.remote_address);
                        }
                    });
                    ui.label(format!("{} -> {}", connection.local_address, connection.remote_address));
                    ui.label(connection.reason.clone());
                    ui.label(RichText::new(connection.exe_path.clone()).small().monospace());
                });
            ui.add_space(6.0);
        }
    });
}

fn draw_events_and_services<F: FnMut(&str)>(ui: &mut egui::Ui, snapshot: &SystemSnapshot, mut on_stop_service: F) {
    ui.heading("Servicios y eventos recientes");
    ui.label("Sirven para correlacionar lentitud con Windows Update, Delivery Optimization, BITS o errores del sistema.");

    for service in &snapshot.services {
        let severity = service_severity(service);
        egui::Frame::group(ui.style())
            .fill(severity_bg(severity))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(&service.display_name).strong());
                    metric_chip(ui, service.status.clone(), severity);
                    ui.label(RichText::new(format!("Inicio {}", service.start_type)).small());
                    if is_stoppable_service(service)
                        && service.status.eq_ignore_ascii_case("Running")
                        && ui.button("Detener temporalmente").clicked()
                    {
                        on_stop_service(&service.name);
                    }
                });
            });
        ui.add_space(4.0);
    }

    ui.separator();
    egui::ScrollArea::vertical().max_height(230.0).show(ui, |ui| {
        for event in snapshot.events.iter().take(10) {
            let severity = if event.level.eq_ignore_ascii_case("Error") {
                Severity::Critical
            } else {
                Severity::Warning
            };
            egui::Frame::group(ui.style())
                .fill(severity_bg(severity))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("{} | {} | ID {}", event.timestamp, event.provider, event.id)).strong().small());
                    ui.label(event.message.clone());
                });
            ui.add_space(4.0);
        }
    });
}

fn status_card(ui: &mut egui::Ui, title: &str, value: &str, subtitle: &str, severity: Severity) {
    egui::Frame::group(ui.style())
        .fill(severity_bg(severity))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_size(egui::vec2(210.0, 92.0));
            let dot = RichText::new("●").color(severity_fg(severity)).size(22.0);
            ui.horizontal(|ui| {
                ui.label(dot);
                ui.label(RichText::new(title).strong());
            });
            ui.label(RichText::new(value).size(18.0).strong());
            ui.label(RichText::new(subtitle).small());
        });
}

fn metric_chip(ui: &mut egui::Ui, text: String, severity: Severity) {
    let width = 132.0_f32.max((text.chars().count() as f32 * 7.2).min(280.0));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 24.0), Sense::hover());
    ui.painter().rect_filled(rect, 8.0, severity_fg(severity).linear_multiply(0.15));
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::TextStyle::Body.resolve(ui.style()),
        Color32::WHITE,
    );
}

fn severity_for_value(value: f32, warning_threshold: f32, critical_threshold: f32) -> Severity {
    if value >= critical_threshold {
        Severity::Critical
    } else if value >= warning_threshold {
        Severity::Warning
    } else {
        Severity::Healthy
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
    let lowered = service.name.to_ascii_lowercase();
    ["wuauserv", "bits", "dosvc", "sysmain"].contains(&lowered.as_str())
}

fn severity_bg(severity: Severity) -> Color32 {
    match severity {
        Severity::Healthy => Color32::from_rgb(18, 46, 34),
        Severity::Warning => Color32::from_rgb(73, 60, 18),
        Severity::Critical => Color32::from_rgb(76, 24, 24),
    }
}

fn severity_fg(severity: Severity) -> Color32 {
    match severity {
        Severity::Healthy => Color32::from_rgb(57, 201, 126),
        Severity::Warning => Color32::from_rgb(232, 186, 53),
        Severity::Critical => Color32::from_rgb(234, 91, 91),
    }
}

fn matches_filter(primary: &str, secondary: &str, filter_text: &str) -> bool {
    let needle = filter_text.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }

    primary.to_ascii_lowercase().contains(&needle) || secondary.to_ascii_lowercase().contains(&needle)
}
