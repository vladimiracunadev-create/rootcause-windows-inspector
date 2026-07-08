//! Configuración operativa de RootCause.
//!
//! Se mantiene en JSON para evitar dependencias nuevas y porque el proyecto ya
//! usa `serde_json` en exportes, historial y análisis ETL.

use crate::i18n::Lang;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_FILE: &str = "rootcause-config.json";
const DEFAULT_APP_DIR: &str = "RootCauseInspector";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RootCauseConfig {
    #[serde(default)]
    pub collection: CollectionConfig,
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
    #[serde(default)]
    pub anomaly: AnomalyConfig,
    #[serde(default)]
    pub alerting: AlertingConfig,
    #[serde(default)]
    pub remediation: RemediationConfig,
    #[serde(default)]
    pub resilience: ResilienceConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

/// Modo de interfaz (apariencia). `windows` sigue el tema del sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Tema oscuro de marca (azul profundo del icono). Por defecto.
    #[default]
    Dark,
    /// Tema claro de marca.
    Light,
    /// Neutro Mica que sigue el tema claro/oscuro del sistema.
    Windows,
}

/// Preferencias de la interfaz gráfica: idioma y modo de apariencia.
/// Deliberadamente aislada para que el resto de la config no cargue con detalles
/// de presentación.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    /// Idioma de la interfaz (`es` por defecto, `en` disponible).
    #[serde(default)]
    pub language: Lang,
    /// Modo de apariencia (`dark` por defecto).
    #[serde(default)]
    pub theme: ThemeMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    #[serde(default = "default_refresh_interval_secs")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    #[serde(default = "default_incident_limit")]
    pub incident_limit: usize,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            refresh_interval_secs: default_refresh_interval_secs(),
            history_limit: default_history_limit(),
            incident_limit: default_incident_limit(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThresholdsConfig {
    #[serde(default)]
    pub process: ProcessThresholds,
    #[serde(default)]
    pub temp: TempThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_anomaly_cpu_sustained_percent")]
    pub cpu_sustained_percent: f32,
    #[serde(default = "default_anomaly_cpu_sustained_samples")]
    pub cpu_sustained_samples: u8,
    #[serde(default = "default_anomaly_memory_growth_mb")]
    pub memory_growth_mb: f32,
    #[serde(default = "default_anomaly_memory_growth_samples")]
    pub memory_growth_samples: u8,
    #[serde(default = "default_anomaly_aggressive_write_mb")]
    pub aggressive_write_mb: f32,
    #[serde(default = "default_anomaly_aggressive_write_samples")]
    pub aggressive_write_samples: u8,
    #[serde(default = "default_anomaly_public_destination_count")]
    pub public_destination_count: usize,
    #[serde(default = "default_anomaly_local_scan_destination_count")]
    pub local_scan_destination_count: usize,
    #[serde(default = "default_anomaly_respawn_window_secs")]
    pub respawn_window_secs: u64,
    #[serde(default = "default_anomaly_respawn_count")]
    pub respawn_count: u8,
    #[serde(default = "default_anomaly_repetitive_script_count")]
    pub repetitive_script_count: u8,
    #[serde(default = "default_suspicious_path_keywords")]
    pub suspicious_path_keywords: Vec<String>,
    #[serde(default = "default_trusted_process_names")]
    pub trusted_process_names: Vec<String>,
    #[serde(default = "default_trusted_path_prefixes")]
    pub trusted_path_prefixes: Vec<String>,
    #[serde(default = "default_suspicious_parent_names")]
    pub suspicious_parent_names: Vec<String>,
    #[serde(default = "default_security_service_names")]
    pub security_service_names: Vec<String>,
    #[serde(default = "default_true")]
    pub watch_persistence: bool,
    #[serde(default = "default_true")]
    pub watch_service_changes: bool,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            cpu_sustained_percent: default_anomaly_cpu_sustained_percent(),
            cpu_sustained_samples: default_anomaly_cpu_sustained_samples(),
            memory_growth_mb: default_anomaly_memory_growth_mb(),
            memory_growth_samples: default_anomaly_memory_growth_samples(),
            aggressive_write_mb: default_anomaly_aggressive_write_mb(),
            aggressive_write_samples: default_anomaly_aggressive_write_samples(),
            public_destination_count: default_anomaly_public_destination_count(),
            local_scan_destination_count: default_anomaly_local_scan_destination_count(),
            respawn_window_secs: default_anomaly_respawn_window_secs(),
            respawn_count: default_anomaly_respawn_count(),
            repetitive_script_count: default_anomaly_repetitive_script_count(),
            suspicious_path_keywords: default_suspicious_path_keywords(),
            trusted_process_names: default_trusted_process_names(),
            trusted_path_prefixes: default_trusted_path_prefixes(),
            suspicious_parent_names: default_suspicious_parent_names(),
            security_service_names: default_security_service_names(),
            watch_persistence: default_true(),
            watch_service_changes: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessThresholds {
    #[serde(default = "default_process_cpu_warning")]
    pub cpu_warning_percent: f32,
    #[serde(default = "default_process_cpu_critical")]
    pub cpu_critical_percent: f32,
    #[serde(default = "default_process_memory_warning")]
    pub memory_warning_mb: f32,
    #[serde(default = "default_process_memory_critical")]
    pub memory_critical_mb: f32,
    #[serde(default = "default_process_io_warning")]
    pub io_write_warning_mb: f32,
    #[serde(default = "default_process_io_critical")]
    pub io_write_critical_mb: f32,
}

impl Default for ProcessThresholds {
    fn default() -> Self {
        Self {
            cpu_warning_percent: default_process_cpu_warning(),
            cpu_critical_percent: default_process_cpu_critical(),
            memory_warning_mb: default_process_memory_warning(),
            memory_critical_mb: default_process_memory_critical(),
            io_write_warning_mb: default_process_io_warning(),
            io_write_critical_mb: default_process_io_critical(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TempThresholds {
    #[serde(default = "default_temp_warning")]
    pub warning_mb: f32,
    #[serde(default = "default_temp_critical")]
    pub critical_mb: f32,
}

impl Default for TempThresholds {
    fn default() -> Self {
        Self {
            warning_mb: default_temp_warning(),
            critical_mb: default_temp_critical(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    #[serde(default = "default_max_alerts")]
    pub max_alerts: usize,
    #[serde(default = "default_true")]
    pub notify_on_critical: bool,
    #[serde(default = "default_notification_cooldown_secs")]
    pub notification_cooldown_secs: u64,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            max_alerts: default_max_alerts(),
            notify_on_critical: default_true(),
            notification_cooldown_secs: default_notification_cooldown_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationConfig {
    #[serde(default = "default_true")]
    pub manual_actions_enabled: bool,
    #[serde(default)]
    pub automatic_actions_enabled: bool,
}

impl Default for RemediationConfig {
    fn default() -> Self {
        Self {
            manual_actions_enabled: default_true(),
            automatic_actions_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResilienceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_heartbeat_interval_secs")]
    pub heartbeat_interval_secs: u64,
    #[serde(default = "default_stale_after_secs")]
    pub stale_after_secs: u64,
    #[serde(default = "default_restart_window_secs")]
    pub restart_window_secs: u64,
    #[serde(default = "default_max_restarts_in_window")]
    pub max_restarts_in_window: u8,
    #[serde(default = "default_true")]
    pub watch_config_integrity: bool,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            heartbeat_interval_secs: default_heartbeat_interval_secs(),
            stale_after_secs: default_stale_after_secs(),
            restart_window_secs: default_restart_window_secs(),
            max_restarts_in_window: default_max_restarts_in_window(),
            watch_config_integrity: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default = "default_ai_model")]
    pub model: String,
    #[serde(default = "default_ai_api_key_env_var")]
    pub api_key_env_var: String,
    #[serde(default = "default_ai_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: String::new(),
            model: default_ai_model(),
            api_key_env_var: default_ai_api_key_env_var(),
            timeout_secs: default_ai_timeout_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    path: PathBuf,
    config: RootCauseConfig,
}

impl ConfigManager {
    pub fn load_or_default(app_name: &str) -> (Self, Option<String>) {
        let path = config_path(app_name);
        let path_display = path.display().to_string();
        match fs::read_to_string(&path) {
            Ok(raw) => match serde_json::from_str::<RootCauseConfig>(&raw) {
                Ok(config) => (Self { path, config }, None),
                Err(error) => (
                    Self {
                        path,
                        config: RootCauseConfig::default(),
                    },
                    Some(format!(
                        "Configuración inválida en {}. Se usan valores por defecto: {error}",
                        path_display
                    )),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
                Self {
                    path,
                    config: RootCauseConfig::default(),
                },
                None,
            ),
            Err(error) => (
                Self {
                    path,
                    config: RootCauseConfig::default(),
                },
                Some(format!(
                    "No se pudo leer {}. Se usan valores por defecto: {error}",
                    path_display
                )),
            ),
        }
    }

    pub fn write_default_if_missing(app_name: &str) -> anyhow::Result<PathBuf> {
        let path = config_path(app_name);
        if path.exists() {
            return Ok(path);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, example_config_json()?)?;
        Ok(path)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn config(&self) -> &RootCauseConfig {
        &self.config
    }

    /// Serializa `config` al disco en `path`.  Crea el directorio padre si no existe.
    pub fn save_to_path(path: &Path, config: &RootCauseConfig) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(config)?;
        fs::write(path, json)?;
        Ok(())
    }
}

pub fn config_path(app_name: &str) -> PathBuf {
    let resolved_app = if app_name.trim().is_empty() {
        DEFAULT_APP_DIR
    } else {
        app_name
    };
    dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(resolved_app)
        .join(DEFAULT_CONFIG_FILE)
}

pub fn example_config_json() -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(&RootCauseConfig::default())?)
}

fn default_refresh_interval_secs() -> u64 {
    5
}

fn default_history_limit() -> usize {
    1_000
}

fn default_incident_limit() -> usize {
    300
}

fn default_process_cpu_warning() -> f32 {
    30.0
}

fn default_process_cpu_critical() -> f32 {
    65.0
}

fn default_process_memory_warning() -> f32 {
    1_000.0
}

fn default_process_memory_critical() -> f32 {
    2_500.0
}

fn default_process_io_warning() -> f32 {
    40.0
}

fn default_process_io_critical() -> f32 {
    200.0
}

fn default_anomaly_cpu_sustained_percent() -> f32 {
    55.0
}

fn default_anomaly_cpu_sustained_samples() -> u8 {
    3
}

fn default_anomaly_memory_growth_mb() -> f32 {
    250.0
}

fn default_anomaly_memory_growth_samples() -> u8 {
    2
}

fn default_anomaly_aggressive_write_mb() -> f32 {
    120.0
}

fn default_anomaly_aggressive_write_samples() -> u8 {
    2
}

fn default_anomaly_public_destination_count() -> usize {
    4
}

fn default_anomaly_local_scan_destination_count() -> usize {
    8
}

fn default_anomaly_respawn_window_secs() -> u64 {
    180
}

fn default_anomaly_respawn_count() -> u8 {
    2
}

fn default_anomaly_repetitive_script_count() -> u8 {
    3
}

fn default_suspicious_path_keywords() -> Vec<String> {
    vec![
        "\\temp\\".to_owned(),
        "\\downloads\\".to_owned(),
        "\\appdata\\".to_owned(),
        "\\public\\".to_owned(),
        "\\programdata\\".to_owned(),
    ]
}

fn default_trusted_process_names() -> Vec<String> {
    vec![
        "system".to_owned(),
        "svchost.exe".to_owned(),
        "explorer.exe".to_owned(),
        "dwm.exe".to_owned(),
        "msmpeng.exe".to_owned(),
    ]
}

fn default_trusted_path_prefixes() -> Vec<String> {
    vec![
        r"c:\windows".to_owned(),
        r"c:\program files".to_owned(),
        r"c:\program files (x86)".to_owned(),
    ]
}

fn default_suspicious_parent_names() -> Vec<String> {
    vec![
        "powershell.exe".to_owned(),
        "cmd.exe".to_owned(),
        "wscript.exe".to_owned(),
        "cscript.exe".to_owned(),
        "mshta.exe".to_owned(),
        "rundll32.exe".to_owned(),
        "winword.exe".to_owned(),
        "excel.exe".to_owned(),
        "outlook.exe".to_owned(),
        "chrome.exe".to_owned(),
        "msedge.exe".to_owned(),
    ]
}

fn default_security_service_names() -> Vec<String> {
    vec![
        "WinDefend".to_owned(),
        "WdNisSvc".to_owned(),
        "MpsSvc".to_owned(),
        "wscsvc".to_owned(),
    ]
}

fn default_temp_warning() -> f32 {
    250.0
}

fn default_temp_critical() -> f32 {
    1_024.0
}

fn default_max_alerts() -> usize {
    8
}

fn default_notification_cooldown_secs() -> u64 {
    90
}

fn default_heartbeat_interval_secs() -> u64 {
    15
}

fn default_stale_after_secs() -> u64 {
    90
}

fn default_restart_window_secs() -> u64 {
    600
}

fn default_max_restarts_in_window() -> u8 {
    3
}

fn default_ai_timeout_secs() -> u64 {
    25
}

fn default_ai_model() -> String {
    "gpt-4.1-mini".to_owned()
}

fn default_ai_api_key_env_var() -> String {
    "ROOTCAUSE_AI_API_KEY".to_owned()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valores_por_defecto_son_razonables() {
        let cfg = RootCauseConfig::default();
        assert_eq!(cfg.collection.refresh_interval_secs, 5);
        assert!(
            cfg.thresholds.process.cpu_critical_percent
                > cfg.thresholds.process.cpu_warning_percent
        );
        assert!(cfg.thresholds.temp.critical_mb > cfg.thresholds.temp.warning_mb);
        assert!(cfg.anomaly.enabled);
        assert!(cfg.anomaly.cpu_sustained_samples >= 2);
        assert!(!cfg.anomaly.suspicious_path_keywords.is_empty());
        assert!(cfg.resilience.enabled);
        assert!(cfg.resilience.heartbeat_interval_secs >= 5);
        assert!(!cfg.ai.enabled);
    }
}
