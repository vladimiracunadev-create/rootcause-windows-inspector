//! Persistencia histórica en SQLite.
//!
//! La base no intenta guardar todo el universo de datos, solo lo necesario para
//! comparar tendencias y revisar qué proceso dominaba cuando apareció la lentitud.

use crate::models::{SnapshotRow, SystemSnapshot};
use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{Connection, params};
use std::fs;
use std::path::{Path, PathBuf};

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
    pub fn persist_snapshot(&self, snapshot: &SystemSnapshot) -> Result<()> {
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

        // Retención: mantener solo las últimas 1 000 filas para evitar crecimiento ilimitado.
        connection.execute(
            r#"
            DELETE FROM snapshots
            WHERE id NOT IN (
                SELECT id FROM snapshots
                ORDER BY id DESC
                LIMIT 1000
            )
            "#,
            [],
        )?;

        Ok(())
    }

    /// Exporta el historial reciente a un archivo JSON como copia de seguridad.
    ///
    /// El JSON se escribe junto al archivo SQLite con el nombre
    /// `rootcause-history-backup.json`. Se usa como respaldo de último recurso:
    /// si la base SQLite se corrompe se puede recuperar el historial de aquí.
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
            "#,
        )?;
        Ok(())
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
}
