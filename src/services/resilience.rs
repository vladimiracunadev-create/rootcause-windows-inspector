//! Estado local de resiliencia del agente.
//!
//! Mantiene una señal mínima de salud sin introducir servicios externos:
//! heartbeat local, detección de cierre abrupto previo y evidencia básica de
//! cambios en la configuración operativa.

use crate::config::ResilienceConfig;
use crate::models::{AgentHealth, AgentStatus, AuditRecord};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

const STATE_FILE: &str = "rootcause-agent-state.json";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct AgentStateFile {
    #[serde(default)]
    last_start_at: Option<String>,
    #[serde(default)]
    last_heartbeat_at: Option<String>,
    #[serde(default)]
    last_shutdown_at: Option<String>,
    #[serde(default)]
    clean_shutdown: bool,
    #[serde(default)]
    config_fingerprint: String,
    #[serde(default)]
    unexpected_stop_times: Vec<String>,
}

pub struct ResilienceMonitor {
    state_path: PathBuf,
    config_path: PathBuf,
    config: ResilienceConfig,
    state: AgentStateFile,
    health: AgentHealth,
    last_persisted_heartbeat: DateTime<Utc>,
}

impl ResilienceMonitor {
    pub fn new(app_name: &str, config_path: &Path, config: &ResilienceConfig) -> Result<Self> {
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .context("No fue posible obtener la carpeta de datos del usuario")?
            .join(app_name);
        fs::create_dir_all(&base_dir)?;

        let state_path = base_dir.join(STATE_FILE);
        let previous = load_state_file(&state_path);
        let now = Utc::now();
        let config_fingerprint = fingerprint_of_file(config_path);
        let previous_last_start_at = previous
            .as_ref()
            .and_then(|state| state.last_start_at.clone());

        let mut status = AgentStatus::Healthy;
        let mut summary = "Heartbeat local y control básico de resiliencia activos.".to_owned();
        let mut notes = vec![format!(
            "Heartbeat persistido cada {} s; ventana de reinicio {} s.",
            config.heartbeat_interval_secs, config.restart_window_secs
        )];
        let unexpected_shutdown_detected = previous
            .as_ref()
            .map(|state| !state.clean_shutdown)
            .unwrap_or(false);
        let mut config_changed = false;
        let mut watchdog_backoff_active = false;

        let mut unexpected_stop_times = previous
            .as_ref()
            .map(|state| state.unexpected_stop_times.clone())
            .unwrap_or_default();
        prune_old_timestamps(&mut unexpected_stop_times, config.restart_window_secs, now);

        let stale_heartbeat_detected = previous
            .as_ref()
            .and_then(|state| state.last_heartbeat_at.as_ref())
            .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
            .map(|ts| {
                now.signed_duration_since(ts.with_timezone(&Utc))
                    .num_seconds()
                    > config.stale_after_secs as i64
            })
            .unwrap_or(false);

        if stale_heartbeat_detected && status == AgentStatus::Healthy {
            status = AgentStatus::Degraded;
            summary =
                "El ultimo heartbeat registrado quedo demasiado antiguo y conviene revisar continuidad."
                    .to_owned();
            notes.push(
                "La sesion previa dejo un heartbeat fuera del umbral esperado para resiliencia."
                    .to_owned(),
            );
        }

        if unexpected_shutdown_detected {
            unexpected_stop_times.push(now.to_rfc3339());
            prune_old_timestamps(&mut unexpected_stop_times, config.restart_window_secs, now);
            status = AgentStatus::Recovered;
            summary =
                "Se detectó un cierre abrupto previo; la sesión actual recuperó el monitoreo."
                    .to_owned();
            notes.push(
                "Revisa historial, incidentes y auditoría si el cierre no fue intencional."
                    .to_owned(),
            );
        }

        if config.watch_config_integrity
            && previous
                .as_ref()
                .map(|state| {
                    !state.config_fingerprint.is_empty()
                        && state.config_fingerprint != config_fingerprint
                })
                .unwrap_or(false)
        {
            config_changed = true;
            if status == AgentStatus::Healthy {
                status = AgentStatus::Degraded;
                summary = "La configuración operativa cambió desde la última ejecución.".to_owned();
            }
            notes.push(
                "Cambio de configuración pendiente de revisión: puede ser legítimo o requerir validación."
                    .to_owned(),
            );
        }

        if unexpected_stop_times.len() >= usize::from(config.max_restarts_in_window) {
            watchdog_backoff_active = true;
            status = AgentStatus::Degraded;
            summary = format!(
                "Se detectaron {} reinicios/cierres abruptos recientes; conviene revisar estabilidad antes de insistir.",
                unexpected_stop_times.len()
            );
            notes.push(
                "Backoff recomendado: evitar bucles de reinicio hasta revisar la causa dominante."
                    .to_owned(),
            );
        }

        if let Some(previous_start) = previous_last_start_at {
            notes.push(format!("Sesion previa registrada en {previous_start}."));
        }

        let health = AgentHealth {
            status,
            summary: summary.clone(),
            last_start_at: now.to_rfc3339(),
            last_heartbeat_at: now.to_rfc3339(),
            last_clean_shutdown_at: previous.and_then(|state| state.last_shutdown_at),
            config_fingerprint: config_fingerprint.clone(),
            config_changed,
            unexpected_shutdown_detected,
            watchdog_backoff_active,
            consecutive_unexpected_stops: unexpected_stop_times.len() as u32,
            notes,
        };

        let state = AgentStateFile {
            last_start_at: Some(now.to_rfc3339()),
            last_heartbeat_at: Some(now.to_rfc3339()),
            last_shutdown_at: None,
            clean_shutdown: false,
            config_fingerprint,
            unexpected_stop_times,
        };

        let monitor = Self {
            state_path,
            config_path: config_path.to_path_buf(),
            config: config.clone(),
            state,
            health,
            last_persisted_heartbeat: now,
        };
        monitor.persist()?;
        Ok(monitor)
    }

    pub fn health(&self) -> &AgentHealth {
        &self.health
    }

    pub fn startup_audits(&self) -> Vec<AuditRecord> {
        let mut records = vec![AuditRecord {
            occurred_at: self.health.last_start_at.clone(),
            action: "agent-start".to_owned(),
            target: "rootcause-agent".to_owned(),
            success: true,
            detail: self.health.summary.clone(),
        }];

        if self.health.unexpected_shutdown_detected {
            records.push(AuditRecord {
                occurred_at: self.health.last_start_at.clone(),
                action: "agent-recovery".to_owned(),
                target: "rootcause-agent".to_owned(),
                success: true,
                detail: "Se detectó un cierre abrupto previo y el agente volvió a iniciar."
                    .to_owned(),
            });
        }

        if self.health.config_changed {
            records.push(AuditRecord {
                occurred_at: self.health.last_start_at.clone(),
                action: "config-integrity-change".to_owned(),
                target: self.config_path.display().to_string(),
                success: true,
                detail: format!(
                    "La huella local cambió a {}.",
                    self.health.config_fingerprint
                ),
            });
        }

        if self.health.watchdog_backoff_active {
            records.push(AuditRecord {
                occurred_at: self.health.last_start_at.clone(),
                action: "agent-watchdog-backoff".to_owned(),
                target: "rootcause-agent".to_owned(),
                success: true,
                detail: "Se activó backoff recomendado por reinicios/cierres abruptos repetidos."
                    .to_owned(),
            });
        }

        records
    }

    pub fn heartbeat(&mut self) -> Result<Vec<AuditRecord>> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        let now = Utc::now();
        let mut audits = Vec::new();
        self.health.last_heartbeat_at = now.to_rfc3339();
        self.state.last_heartbeat_at = Some(self.health.last_heartbeat_at.clone());

        if self.config.watch_config_integrity {
            let current = fingerprint_of_file(&self.config_path);
            if current != self.health.config_fingerprint {
                self.health.config_fingerprint = current.clone();
                self.state.config_fingerprint = current.clone();
                if !self.health.config_changed {
                    self.health.config_changed = true;
                    if self.health.status == AgentStatus::Healthy {
                        self.health.status = AgentStatus::Degraded;
                        self.health.summary =
                            "Se detectó un cambio de configuración durante la ejecución."
                                .to_owned();
                    }
                    self.health.notes.push(
                        "La configuración cambió mientras el agente estaba activo.".to_owned(),
                    );
                    audits.push(AuditRecord {
                        occurred_at: now.to_rfc3339(),
                        action: "config-integrity-change".to_owned(),
                        target: self.config_path.display().to_string(),
                        success: true,
                        detail: format!("Nueva huella local detectada: {current}."),
                    });
                }
            }
        }

        if now
            .signed_duration_since(self.last_persisted_heartbeat)
            .num_seconds()
            >= self.config.heartbeat_interval_secs as i64
        {
            self.persist()?;
            self.last_persisted_heartbeat = now;
        }

        Ok(audits)
    }

    pub fn shutdown(&mut self) -> Result<AuditRecord> {
        let now = Utc::now();
        self.state.clean_shutdown = true;
        self.state.last_shutdown_at = Some(now.to_rfc3339());
        self.state.last_heartbeat_at = Some(now.to_rfc3339());
        self.health.last_clean_shutdown_at = Some(now.to_rfc3339());
        self.health.last_heartbeat_at = now.to_rfc3339();
        self.persist()?;

        Ok(AuditRecord {
            occurred_at: now.to_rfc3339(),
            action: "agent-stop".to_owned(),
            target: "rootcause-agent".to_owned(),
            success: true,
            detail: "Cierre limpio registrado.".to_owned(),
        })
    }

    fn persist(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.state)?;
        fs::write(&self.state_path, json)
            .with_context(|| format!("No se pudo escribir {}", self.state_path.display()))?;
        Ok(())
    }
}

fn load_state_file(path: &Path) -> Option<AgentStateFile> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str::<AgentStateFile>(&text).ok()
}

fn prune_old_timestamps(values: &mut Vec<String>, restart_window_secs: u64, now: DateTime<Utc>) {
    values.retain(|raw| {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|ts| {
                now.signed_duration_since(ts.with_timezone(&Utc))
                    .num_seconds()
                    <= restart_window_secs as i64
            })
            .unwrap_or(false)
    });
}

fn fingerprint_of_file(path: &Path) -> String {
    let body = fs::read_to_string(path).unwrap_or_else(|_| "missing-config".to_owned());
    fingerprint(&body)
}

fn fingerprint(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_es_estable_para_el_mismo_texto() {
        assert_eq!(fingerprint("demo"), fingerprint("demo"));
        assert_ne!(fingerprint("demo"), fingerprint("demo-2"));
    }

    #[test]
    fn prune_descarta_timestamps_viejos() {
        let now = Utc::now();
        let mut values = vec![
            (now - chrono::TimeDelta::seconds(30)).to_rfc3339(),
            (now - chrono::TimeDelta::seconds(999)).to_rfc3339(),
        ];
        prune_old_timestamps(&mut values, 120, now);
        assert_eq!(values.len(), 1);
    }
}
