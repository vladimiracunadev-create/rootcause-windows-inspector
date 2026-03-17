//! Modelos de dominio del monitor.
//!
//! Se mantienen deliberadamente explícitos y serializables para cuatro fines:
//! 1. Renderizar la interfaz de forma estable.
//! 2. Exportar evidencia en JSON.
//! 3. Persistir resúmenes históricos en SQLite.
//! 4. Resumir capturas ETL de forma que puedan leerse sin abrir WPA en todo momento.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Severidad visual para el semáforo principal y para tablas detalladas.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    #[default]
    Healthy,
    Warning,
    Critical,
}

impl Severity {
    #[allow(dead_code)]
    /// Texto humano para la interfaz.
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Verde",
            Self::Warning => "Amarillo",
            Self::Critical => "Rojo",
        }
    }
}

/// Resumen del estado global del equipo en una instantánea concreta.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemOverview {
    pub cpu_usage_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub network_rx_mb_delta: f32,
    pub network_tx_mb_delta: f32,
    pub io_read_mb_delta: f32,
    pub io_write_mb_delta: f32,
    pub temp_total_mb: f32,
    pub primary_severity: Severity,
    pub primary_reason: String,
}

/// Hallazgo resumido que explica por qué un proceso o condición merece atención.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Alert {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub pid: Option<u32>,
    pub path: Option<String>,
    pub hint: String,
}

/// Vista enriquecida por proceso.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessInsight {
    pub pid: u32,
    pub name: String,
    pub exe_path: String,
    pub parent_pid: Option<u32>,
    pub cpu_percent: f32,
    pub memory_mb: f32,
    pub io_read_mb_delta: f32,
    pub io_write_mb_delta: f32,
    pub status: String,
    pub category: String,
    pub severity: Severity,
    pub score: u8,
    pub can_terminate: bool,
    pub reasons: Vec<String>,
    /// Línea de comandos completa del proceso, obtenida bajo demanda para procesos críticos.
    #[serde(default)]
    pub command_line: Option<String>,
}

/// Fila resumida del historial SQLite, lista para mostrar en la UI.
#[derive(Debug, Clone, Default)]
pub struct SnapshotRow {
    pub id: i64,
    pub collected_at: String,
    pub cpu_usage: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub io_write_mb_delta: f32,
    pub temp_total_mb: f32,
    pub dominant_process: String,
    pub alerts_count: usize,
    pub has_critical: bool,
}

/// Elemento medible dentro de carpetas temporales o cachés de riesgo.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TempEntry {
    pub path: String,
    pub size_mb: f32,
    pub file_count: u64,
    pub severity: Severity,
    pub note: String,
}

/// Resumen de carpetas temporales relevantes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TempOverview {
    pub total_mb: f32,
    pub roots_scanned: Vec<String>,
    pub top_entries: Vec<TempEntry>,
    pub limitations: Vec<String>,
}

/// Conexión observada a partir de netstat.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionInsight {
    pub protocol: String,
    pub local_address: String,
    pub remote_address: String,
    pub state: String,
    pub pid: u32,
    pub process_name: String,
    pub exe_path: String,
    pub severity: Severity,
    pub reason: String,
    pub is_public_remote: bool,
}

/// Evento reciente de Windows que puede ayudar a correlacionar lentitud o fallas.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventRecord {
    pub timestamp: String,
    pub provider: String,
    pub id: u32,
    pub level: String,
    pub message: String,
}

/// Estado de servicios relevantes para actualizaciones y uso intensivo en segundo plano.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceState {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub start_type: String,
}

/// Hallazgo derivado del análisis posterior de una traza ETL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceFinding {
    pub severity: Severity,
    pub title: String,
    pub detail: String,
    pub evidence: String,
}

/// Proceso o imagen repetido dentro de una traza analizada.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceProcessSummary {
    pub name: String,
    pub occurrences: u64,
    pub severity: Severity,
    pub reason: String,
}

/// Ruta o artefacto repetido dentro de la traza.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TracePathSummary {
    pub path: String,
    pub category: String,
    pub occurrences: u64,
    pub severity: Severity,
}

/// Resumen legible de una traza ETL procesada por herramientas del sistema.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceAnalysisSummary {
    pub engine: String,
    pub analyzed_at: String,
    pub etl_path: String,
    pub output_directory: String,
    pub raw_xml_path: Option<String>,
    pub raw_summary_path: Option<String>,
    pub total_events: u64,
    pub headline: String,
    pub confidence: String,
    pub findings: Vec<TraceFinding>,
    pub hot_processes: Vec<TraceProcessSummary>,
    pub hot_paths: Vec<TracePathSummary>,
    pub public_ips: Vec<String>,
    pub providers: Vec<(String, u64)>,
    pub indicators: Vec<String>,
    pub limitations: Vec<String>,
}

/// Estado del modo de precisión basado en WPR/WPA + análisis de ETL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrecisionStatus {
    pub wpr_available: bool,
    pub wpa_available: bool,
    pub tracerpt_available: bool,
    pub is_recording: bool,
    pub recommended_profile: String,
    pub traces_directory: String,
    pub last_trace_path: Option<String>,
    pub last_analysis_path: Option<String>,
    pub analyzer_label: String,
    pub status_detail: String,
    pub guidance: String,
}

/// Instantánea completa que la UI consume y que también puede exportarse.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    pub collected_at: DateTime<Utc>,
    pub overview: SystemOverview,
    pub alerts: Vec<Alert>,
    pub processes: Vec<ProcessInsight>,
    pub temp: TempOverview,
    pub connections: Vec<ConnectionInsight>,
    pub events: Vec<EventRecord>,
    pub services: Vec<ServiceState>,
    pub precision: PrecisionStatus,
    pub trace_analysis: Option<TraceAnalysisSummary>,
}
