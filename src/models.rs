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

/// Severidad específica del módulo de anomalías y riesgo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Bajo",
            Self::Medium => "Medio",
            Self::High => "Alto",
            Self::Critical => "Critico",
        }
    }

    pub fn to_severity(self) -> Severity {
        match self {
            Self::Low => Severity::Healthy,
            Self::Medium => Severity::Warning,
            Self::High | Self::Critical => Severity::Critical,
        }
    }
}

/// Estado operativo del propio agente RootCause.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AgentStatus {
    #[default]
    Healthy,
    Degraded,
    Recovered,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Saludable",
            Self::Degraded => "Degradado",
            Self::Recovered => "Recuperado",
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

/// Información de hardware del equipo, recopilada una sola vez al iniciar.
///
/// Se usa en el tab Overview y en el CLI para contextualizar las métricas.
#[derive(Debug, Clone, Default)]
pub struct HardwareInfo {
    pub os_name: String,
    pub os_version: String,
    pub host_name: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_freq_mhz: u64,
    pub total_ram_gb: f32,
    pub architecture: String,
}

/// Fila resumida del historial SQLite, lista para mostrar en la UI.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SnapshotRow {
    #[allow(dead_code)]
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

/// Evidencia atómica asociada a un incidente resumido.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncidentEvidence {
    pub kind: String,
    pub label: String,
    pub value: String,
}

/// Estado de una entrada de autoarranque respecto a la baseline conocida.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum PersistenceChange {
    /// Igual que en la baseline (o sin baseline todavía).
    #[default]
    Unchanged,
    /// Entrada nueva que no estaba en la baseline.
    Added,
    /// Entrada que existía pero cambió de comando.
    Modified,
    /// Entrada que estaba en la baseline y ya no aparece.
    Removed,
}

impl PersistenceChange {
    /// Etiqueta corta para UI/CLI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Unchanged => "",
            Self::Added => "NUEVA",
            Self::Modified => "MODIFICADA",
            Self::Removed => "ELIMINADA",
        }
    }

    /// `true` si representa un cambio respecto a la baseline.
    pub fn is_change(self) -> bool {
        !matches!(self, Self::Unchanged)
    }
}

/// Entrada observable de persistencia básica en Windows.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistenceEntry {
    pub entry_kind: String,
    pub location: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub target_path: Option<String>,
    #[serde(default)]
    pub exists_on_disk: bool,
    #[serde(default)]
    pub severity: RiskLevel,
    #[serde(default)]
    pub note: String,
    /// Estado de cambio respecto a la baseline de autoarranque conocida.
    #[serde(default)]
    pub change_status: PersistenceChange,
}

/// Evento atómico del módulo de detección anómala.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnomalyEvent {
    pub event_id: String,
    pub detected_at: DateTime<Utc>,
    #[serde(default)]
    pub severity: RiskLevel,
    #[serde(default)]
    pub score: u16,
    #[serde(default)]
    pub status: String,
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub process_name: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub parent_pid: Option<u32>,
    #[serde(default)]
    pub parent_name: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub exe_path: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub cpu_percent: Option<f32>,
    #[serde(default)]
    pub memory_mb: Option<f32>,
    #[serde(default)]
    pub io_write_mb_delta: Option<f32>,
    #[serde(default)]
    pub unique_public_remotes: Option<usize>,
    #[serde(default)]
    pub unique_private_remotes: Option<usize>,
    pub summary: String,
    pub root_cause_hypothesis: String,
    pub recommended_action: String,
    #[serde(default)]
    pub evidence: Vec<IncidentEvidence>,
}

/// Resumen persistible de un incidente o degradación detectada.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncidentSummary {
    pub incident_id: String,
    pub fingerprint: String,
    pub collected_at: DateTime<Utc>,
    pub severity: Severity,
    pub kind: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub root_cause_hypothesis: String,
    pub probable_causes: Vec<String>,
    pub recommended_actions: Vec<String>,
    pub evidence: Vec<IncidentEvidence>,
    #[serde(default)]
    pub risk_level: Option<RiskLevel>,
    #[serde(default)]
    pub risk_score: u16,
    #[serde(default)]
    pub anomaly_count: usize,
    #[serde(default)]
    pub anomaly_types: Vec<String>,
    #[serde(default)]
    pub anomaly_events: Vec<AnomalyEvent>,
    #[serde(default)]
    pub ai_advice: Option<AiIncidentAdvice>,
}

/// Respuesta opcional de un adaptador IA desacoplado del motor principal.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiIncidentAdvice {
    pub provider: String,
    pub model: String,
    pub summary: String,
    pub probable_causes: Vec<String>,
    pub suggested_actions: Vec<String>,
    pub confidence: String,
    pub warnings: Vec<String>,
    pub generated_at: String,
}

/// Registro de acciones ejecutadas desde la app o CLI.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditRecord {
    pub occurred_at: String,
    pub action: String,
    pub target: String,
    pub success: bool,
    pub detail: String,
}

/// Estado resumido de resiliencia del propio agente.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentHealth {
    #[serde(default)]
    pub status: AgentStatus,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub last_start_at: String,
    #[serde(default)]
    pub last_heartbeat_at: String,
    #[serde(default)]
    pub last_clean_shutdown_at: Option<String>,
    #[serde(default)]
    pub config_fingerprint: String,
    #[serde(default)]
    pub config_changed: bool,
    #[serde(default)]
    pub unexpected_shutdown_detected: bool,
    #[serde(default)]
    pub watchdog_backoff_active: bool,
    #[serde(default)]
    pub consecutive_unexpected_stops: u32,
    #[serde(default)]
    pub notes: Vec<String>,
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
    #[serde(default)]
    pub agent_health: AgentHealth,
    pub processes: Vec<ProcessInsight>,
    pub temp: TempOverview,
    pub connections: Vec<ConnectionInsight>,
    pub events: Vec<EventRecord>,
    pub services: Vec<ServiceState>,
    #[serde(default)]
    pub persistence_entries: Vec<PersistenceEntry>,
    #[serde(default)]
    pub anomalies: Vec<AnomalyEvent>,
    #[serde(default)]
    pub incident: Option<IncidentSummary>,
    pub precision: PrecisionStatus,
    pub trace_analysis: Option<TraceAnalysisSummary>,
}
