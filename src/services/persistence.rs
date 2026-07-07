//! Persistencia histórica en SQLite.
//!
//! La base ahora guarda tres capas:
//! 1. snapshots compactos para tendencia,
//! 2. incidentes resumidos para correlación/evidencia,
//! 3. auditoría de acciones manuales o automáticas.

use crate::models::{
    AiIncidentAdvice, AuditRecord, IncidentSummary, PersistenceEntry, SnapshotRow, SystemSnapshot,
    WatchedItem,
};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Clave estable que identifica una entrada de autoarranque a lo largo del tiempo.
/// No incluye el comando: un cambio de comando en la misma ubicación/nombre se
/// interpreta como *modificación*, no como par eliminada+nueva.
pub fn persistence_entry_key(entry: &PersistenceEntry) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{}",
        entry.entry_kind, entry.location, entry.name
    )
}

/// Adaptador pequeño sobre SQLite.
pub struct PersistenceStore {
    db_path: PathBuf,
}

impl PersistenceStore {
    /// Crea el almacenamiento en la carpeta de datos local del usuario.
    pub fn new(app_name: &str) -> Result<Self> {
        let base_dir = dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .context("No fue posible obtener la carpeta de datos del usuario")?
            .join(app_name);
        fs::create_dir_all(&base_dir)?;

        let db_path = base_dir.join("rootcause-history.db");
        let store = Self { db_path };
        store.ensure_schema()?;
        Ok(store)
    }

    /// Devuelve la ruta física del archivo SQLite.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Guarda un resumen compacto de la instantánea actual.
    pub fn persist_snapshot(&self, snapshot: &SystemSnapshot, history_limit: usize) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        let dominant_process = snapshot
            .processes
            .first()
            .map(|process| format!("{} ({})", process.name, process.pid))
            .unwrap_or_else(|| "Sin datos".to_owned());
        let alerts_json = serde_json::to_string(&snapshot.alerts)?;

        connection.execute(
            r#"
            INSERT INTO snapshots (
                collected_at,
                cpu_usage,
                memory_used_gb,
                memory_total_gb,
                temp_total_mb,
                network_rx_mb_delta,
                network_tx_mb_delta,
                io_read_mb_delta,
                io_write_mb_delta,
                dominant_process,
                alerts_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                snapshot.collected_at.to_rfc3339(),
                snapshot.overview.cpu_usage_percent,
                snapshot.overview.memory_used_gb,
                snapshot.overview.memory_total_gb,
                snapshot.overview.temp_total_mb,
                snapshot.overview.network_rx_mb_delta,
                snapshot.overview.network_tx_mb_delta,
                snapshot.overview.io_read_mb_delta,
                snapshot.overview.io_write_mb_delta,
                dominant_process,
                alerts_json,
            ],
        )?;

        self.trim_snapshots(history_limit)?;
        Ok(())
    }

    /// Guarda un incidente resumido si no es un duplicado inmediato.
    pub fn persist_incident(
        &self,
        incident: &IncidentSummary,
        incident_limit: usize,
    ) -> Result<bool> {
        let connection = Connection::open(&self.db_path)?;
        let last_fingerprint: Option<String> = connection
            .query_row(
                "SELECT fingerprint FROM incidents ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if last_fingerprint.as_deref() == Some(incident.fingerprint.as_str()) {
            return Ok(false);
        }

        connection.execute(
            r#"
            INSERT INTO incidents (
                incident_id,
                fingerprint,
                collected_at,
                severity,
                kind,
                title,
                summary,
                payload_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &incident.incident_id,
                &incident.fingerprint,
                incident.collected_at.to_rfc3339(),
                format!("{:?}", incident.severity),
                &incident.kind,
                &incident.title,
                &incident.summary,
                serde_json::to_string(incident)?,
            ],
        )?;

        self.trim_incidents(incident_limit)?;
        Ok(true)
    }

    /// Actualiza el enriquecimiento IA del incidente más reciente con ese ID.
    pub fn update_incident_ai(&self, incident_id: &str, advice: &AiIncidentAdvice) -> Result<()> {
        let Some(mut incident) = self.load_incident_by_id(incident_id)? else {
            return Ok(());
        };
        incident.ai_advice = Some(advice.clone());

        let connection = Connection::open(&self.db_path)?;
        connection.execute(
            "UPDATE incidents SET payload_json = ?2 WHERE incident_id = ?1",
            params![incident_id, serde_json::to_string(&incident)?],
        )?;
        Ok(())
    }

    /// Devuelve las últimas N filas del historial para mostrar en la pestaña Historial.
    pub fn load_recent(&self, limit: usize) -> Result<Vec<SnapshotRow>> {
        let connection = Connection::open(&self.db_path)?;
        let mut statement = connection.prepare(
            r#"
            SELECT id, collected_at, cpu_usage, memory_used_gb, memory_total_gb,
                   io_write_mb_delta, temp_total_mb, dominant_process, alerts_json
            FROM snapshots
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )?;

        let mut rows_out = Vec::new();
        let mut rows = statement.query(params![limit as i64])?;
        while let Some(row) = rows.next()? {
            let alerts_json: String = row.get(8)?;
            let alerts_count = serde_json::from_str::<serde_json::Value>(&alerts_json)
                .ok()
                .and_then(|v| v.as_array().map(|a| a.len()))
                .unwrap_or(0);
            let has_critical = serde_json::from_str::<serde_json::Value>(&alerts_json)
                .ok()
                .and_then(|v| {
                    v.as_array().map(|a| {
                        a.iter()
                            .any(|e| e.get("severity").and_then(|s| s.as_str()) == Some("Critical"))
                    })
                })
                .unwrap_or(false);

            rows_out.push(SnapshotRow {
                id: row.get(0)?,
                collected_at: row.get(1)?,
                cpu_usage: row.get(2)?,
                memory_used_gb: row.get(3)?,
                memory_total_gb: row.get(4)?,
                io_write_mb_delta: row.get(5)?,
                temp_total_mb: row.get(6)?,
                dominant_process: row.get(7)?,
                alerts_count,
                has_critical,
            });
        }
        Ok(rows_out)
    }

    pub fn load_recent_incidents(&self, limit: usize) -> Result<Vec<IncidentSummary>> {
        let connection = Connection::open(&self.db_path)?;
        let mut statement = connection.prepare(
            r#"
            SELECT payload_json
            FROM incidents
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )?;

        let rows = statement.query_map(params![limit as i64], |row| row.get::<_, String>(0))?;
        let mut incidents = Vec::new();
        for row in rows {
            let payload = row?;
            if let Ok(incident) = serde_json::from_str::<IncidentSummary>(&payload) {
                incidents.push(incident);
            }
        }
        Ok(incidents)
    }

    pub fn latest_incident(&self) -> Result<Option<IncidentSummary>> {
        let connection = Connection::open(&self.db_path)?;
        let payload = connection
            .query_row(
                "SELECT payload_json FROM incidents ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match payload {
            Some(text) => Ok(serde_json::from_str::<IncidentSummary>(&text).ok()),
            None => Ok(None),
        }
    }

    /// Guarda un evento de auditoría.
    pub fn record_audit(&self, record: &AuditRecord) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute(
            r#"
            INSERT INTO audit_log (occurred_at, action, target, success, detail)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                &record.occurred_at,
                &record.action,
                &record.target,
                record.success,
                &record.detail,
            ],
        )?;
        Ok(())
    }

    /// Exporta el historial reciente a un archivo JSON como copia de seguridad.
    pub fn export_history_backup(&self, limit: usize) -> Result<PathBuf> {
        let rows = self.load_recent(limit)?;
        let json =
            serde_json::to_string_pretty(&rows).context("No se pudo serializar el historial")?;
        let backup_path = self
            .db_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("rootcause-history-backup.json");
        fs::write(&backup_path, json)
            .with_context(|| format!("No se pudo escribir {}", backup_path.display()))?;
        Ok(backup_path)
    }

    /// Devuelve una línea resumen fácil de mostrar en la UI.
    pub fn latest_summary_line(&self) -> Result<Option<String>> {
        let connection = Connection::open(&self.db_path)?;
        let mut statement = connection.prepare(
            r#"
            SELECT collected_at, cpu_usage, temp_total_mb, dominant_process
            FROM snapshots
            ORDER BY id DESC
            LIMIT 1
            "#,
        )?;

        let mut rows = statement.query([])?;
        if let Some(row) = rows.next()? {
            let collected_at: String = row.get(0)?;
            let cpu_usage: f32 = row.get(1)?;
            let temp_total_mb: f32 = row.get(2)?;
            let dominant_process: String = row.get(3)?;
            return Ok(Some(format!(
                "Último historial {} | CPU {:.1}% | TEMP {:.1} MB | Proceso dominante: {}",
                collected_at, cpu_usage, temp_total_mb, dominant_process
            )));
        }

        Ok(None)
    }

    /// Utilidad opcional para generar archivos de soporte fuera de SQLite.
    pub fn export_path(&self) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        dirs::download_dir()
            .or_else(dirs::document_dir)
            .unwrap_or_else(|| {
                self.db_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_path_buf()
            })
            .join(format!("rootcause-snapshot-{timestamp}.json"))
    }

    fn load_incident_by_id(&self, incident_id: &str) -> Result<Option<IncidentSummary>> {
        let connection = Connection::open(&self.db_path)?;
        let payload = connection
            .query_row(
                r#"
                SELECT payload_json
                FROM incidents
                WHERE incident_id = ?1
                ORDER BY id DESC
                LIMIT 1
                "#,
                params![incident_id],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        match payload {
            Some(text) => Ok(serde_json::from_str::<IncidentSummary>(&text).ok()),
            None => Ok(None),
        }
    }

    fn trim_snapshots(&self, keep: usize) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute(
            r#"
            DELETE FROM snapshots
            WHERE id NOT IN (
                SELECT id FROM snapshots
                ORDER BY id DESC
                LIMIT ?1
            )
            "#,
            params![keep as i64],
        )?;
        Ok(())
    }

    fn trim_incidents(&self, keep: usize) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute(
            r#"
            DELETE FROM incidents
            WHERE id NOT IN (
                SELECT id FROM incidents
                ORDER BY id DESC
                LIMIT ?1
            )
            "#,
            params![keep as i64],
        )?;
        Ok(())
    }

    fn ensure_schema(&self) -> Result<()> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                collected_at TEXT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_used_gb REAL NOT NULL,
                memory_total_gb REAL NOT NULL,
                temp_total_mb REAL NOT NULL,
                network_rx_mb_delta REAL NOT NULL,
                network_tx_mb_delta REAL NOT NULL,
                io_read_mb_delta REAL NOT NULL,
                io_write_mb_delta REAL NOT NULL,
                dominant_process TEXT NOT NULL,
                alerts_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS incidents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                incident_id TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                collected_at TEXT NOT NULL,
                severity TEXT NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_incidents_incident_id
            ON incidents(incident_id);

            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                occurred_at TEXT NOT NULL,
                action TEXT NOT NULL,
                target TEXT NOT NULL,
                success INTEGER NOT NULL,
                detail TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS persistence_baseline (
                entry_key TEXT PRIMARY KEY,
                entry_kind TEXT NOT NULL,
                location TEXT NOT NULL,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                target_path TEXT,
                first_seen TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS baseline (
                surface TEXT NOT NULL,
                entry_key TEXT NOT NULL,
                value TEXT NOT NULL,
                label TEXT NOT NULL,
                detail TEXT NOT NULL,
                first_seen TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (surface, entry_key)
            );
            "#,
        )?;
        Ok(())
    }

    /// Carga la baseline de autoarranque conocida, indexada por clave estable.
    /// Las entradas reconstruidas traen `entry_kind`, `location`, `name`,
    /// `command` y `target_path`; el resto de campos quedan por defecto.
    pub fn load_persistence_baseline(&self) -> Result<HashMap<String, PersistenceEntry>> {
        let connection = Connection::open(&self.db_path)?;
        let mut statement = connection.prepare(
            "SELECT entry_key, entry_kind, location, name, command, target_path \
             FROM persistence_baseline",
        )?;
        let rows = statement.query_map([], |row| {
            let key: String = row.get(0)?;
            let entry = PersistenceEntry {
                entry_kind: row.get(1)?,
                location: row.get(2)?,
                name: row.get(3)?,
                command: row.get(4)?,
                target_path: row.get(5)?,
                ..Default::default()
            };
            Ok((key, entry))
        })?;

        let mut baseline = HashMap::new();
        for row in rows {
            let (key, entry) = row?;
            baseline.insert(key, entry);
        }
        Ok(baseline)
    }

    /// Reemplaza por completo la baseline con el estado actual de autoarranque.
    /// Se usa para sembrar la primera foto y para "aceptar" cambios como buenos.
    pub fn replace_persistence_baseline(&self, entries: &[PersistenceEntry]) -> Result<()> {
        let mut connection = Connection::open(&self.db_path)?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM persistence_baseline", [])?;
        {
            let mut statement = transaction.prepare(
                "INSERT OR REPLACE INTO persistence_baseline \
                 (entry_key, entry_kind, location, name, command, target_path) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for entry in entries {
                if matches!(
                    entry.change_status,
                    crate::models::PersistenceChange::Removed
                ) {
                    // Nunca sembrar entradas sintéticas de tipo "eliminada".
                    continue;
                }
                statement.execute(params![
                    persistence_entry_key(entry),
                    entry.entry_kind,
                    entry.location,
                    entry.name,
                    entry.command,
                    entry.target_path,
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    /// Carga la baseline genérica de una superficie vigilada (servicios, hosts…),
    /// indexada por clave estable. Motor genérico de detección de cambios.
    pub fn load_baseline(&self, surface: &str) -> Result<HashMap<String, WatchedItem>> {
        let connection = Connection::open(&self.db_path)?;
        let mut statement = connection
            .prepare("SELECT entry_key, value, label, detail FROM baseline WHERE surface = ?1")?;
        let rows = statement.query_map([surface], |row| {
            let key: String = row.get(0)?;
            Ok((
                key.clone(),
                WatchedItem {
                    key,
                    value: row.get(1)?,
                    label: row.get(2)?,
                    detail: row.get(3)?,
                    ..Default::default()
                },
            ))
        })?;

        let mut baseline = HashMap::new();
        for row in rows {
            let (key, item) = row?;
            baseline.insert(key, item);
        }
        Ok(baseline)
    }

    /// Reemplaza por completo la baseline de una superficie con el estado actual.
    /// Se usa para sembrar la primera foto y para "aceptar" cambios como buenos.
    pub fn replace_baseline(&self, surface: &str, items: &[WatchedItem]) -> Result<()> {
        let mut connection = Connection::open(&self.db_path)?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM baseline WHERE surface = ?1", [surface])?;
        {
            let mut statement = transaction.prepare(
                "INSERT OR REPLACE INTO baseline (surface, entry_key, value, label, detail) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;
            for item in items {
                if matches!(
                    item.change_status,
                    crate::models::PersistenceChange::Removed
                ) {
                    // Nunca sembrar ítems sintéticos de tipo "eliminado".
                    continue;
                }
                statement.execute(params![
                    surface,
                    item.key,
                    item.value,
                    item.label,
                    item.detail,
                ])?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}
