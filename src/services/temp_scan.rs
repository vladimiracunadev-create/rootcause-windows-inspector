//! Escaneo de carpetas temporales.
//!
//! Objetivo: detectar rápidamente qué zonas de TEMP están creciendo y cuánto
//! pesan, sin convertir el monitor en otra carga pesada. Por eso el escaneo
//! usa límites razonables de profundidad y conteo de archivos.

use crate::config::TempThresholds;
use crate::models::{Severity, TempCleanResult, TempEntry, TempOverview};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

const MAX_FILES_PER_ENTRY: usize = 20_000;

/// Construye el resumen de temporales más relevante para la UI.
pub fn scan_temp_overview(thresholds: &TempThresholds) -> Result<TempOverview> {
    let roots = candidate_roots();
    let mut top_entries = Vec::new();
    let mut limitations = Vec::new();
    let mut total_bytes: u64 = 0;

    for root in &roots {
        if !root.exists() {
            continue;
        }

        match scan_root(root, thresholds) {
            Ok((entry_total, mut entries)) => {
                total_bytes = total_bytes.saturating_add(entry_total);
                top_entries.append(&mut entries);
            }
            Err(error) => {
                limitations.push(format!(
                    "No fue posible escanear {}: {}",
                    root.display(),
                    error
                ));
            }
        }
    }

    top_entries.sort_by(|a, b| b.size_mb.total_cmp(&a.size_mb));
    top_entries.truncate(12);

    Ok(TempOverview {
        total_mb: bytes_to_mb(total_bytes),
        roots_scanned: roots
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        top_entries,
        limitations,
    })
}

/// Devuelve raíces candidatas conocidas para Windows.
pub fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(user_temp) = std::env::var_os("TEMP") {
        roots.push(PathBuf::from(user_temp));
    }
    roots.push(PathBuf::from(r"C:\Windows\Temp"));
    roots.push(PathBuf::from(r"C:\Windows\SoftwareDistribution\Download"));
    roots.push(PathBuf::from(
        r"C:\ProgramData\Microsoft\Windows\DeliveryOptimization\Cache",
    ));

    roots
}

/// Escanea una raíz concreta y devuelve el total y sus elementos más pesados.
fn scan_root(root: &Path, thresholds: &TempThresholds) -> Result<(u64, Vec<TempEntry>)> {
    let mut total_bytes: u64 = 0;
    let mut rows = Vec::new();

    for child in fs::read_dir(root)? {
        let child = match child {
            Ok(value) => value,
            Err(_) => continue,
        };
        let path = child.path();
        let (size_bytes, file_count) = accumulate_path(&path);
        total_bytes = total_bytes.saturating_add(size_bytes);

        rows.push(TempEntry {
            path: path.display().to_string(),
            size_mb: bytes_to_mb(size_bytes),
            file_count,
            severity: classify_temp_size(size_bytes, thresholds),
            note: note_for_path(&path, size_bytes),
        });
    }

    rows.sort_by(|a, b| b.size_mb.total_cmp(&a.size_mb));
    rows.truncate(8);

    Ok((total_bytes, rows))
}

/// Suma el tamaño de archivos de forma recursiva con tope de archivos.
fn accumulate_path(path: &Path) -> (u64, u64) {
    if path.is_file() {
        let bytes = path.metadata().map(|meta| meta.len()).unwrap_or_default();
        return (bytes, 1);
    }

    let mut total_bytes: u64 = 0;
    let mut file_count: u64 = 0;

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .take(MAX_FILES_PER_ENTRY)
    {
        if let Ok(metadata) = entry.metadata()
            && metadata.is_file()
        {
            total_bytes = total_bytes.saturating_add(metadata.len());
            file_count = file_count.saturating_add(1);
        }
    }

    (total_bytes, file_count)
}

/// Limpia SOLO la carpeta `%TEMP%` del usuario. Borra las entradas de nivel
/// superior no modificadas en las últimas `older_than_hours` horas; salta lo que
/// esté en uso (bloqueado) o sin permisos. Con `dry_run` cuenta sin borrar.
///
/// Seguridad: opera exclusivamente dentro de `%TEMP%` (nunca `C:\Windows\Temp`,
/// el sistema ni `SoftwareDistribution`). En Windows, un archivo con un handle
/// abierto no puede borrarse → la operación falla y se salta, por lo que
/// "no en uso" es intrínsecamente seguro.
pub fn clean_user_temp(older_than_hours: u64, dry_run: bool) -> TempCleanResult {
    let mut result = TempCleanResult {
        dry_run,
        ..Default::default()
    };

    let temp = match std::env::var_os("TEMP") {
        Some(value) => PathBuf::from(value),
        None => return result,
    };
    if !temp.is_dir() {
        return result;
    }

    let cutoff =
        SystemTime::now().checked_sub(Duration::from_secs(older_than_hours.saturating_mul(3600)));

    let entries = match fs::read_dir(&temp) {
        Ok(entries) => entries,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        // Guarda de antigüedad: saltar lo modificado recientemente. Si no se puede
        // leer la fecha, se salta por seguridad (nunca borrar a ciegas).
        let too_recent = match (entry.metadata().and_then(|m| m.modified()).ok(), cutoff) {
            (Some(modified), Some(cut)) => modified > cut,
            _ => true,
        };
        if too_recent {
            result.skipped_recent = result.skipped_recent.saturating_add(1);
            continue;
        }

        let size_bytes = accumulate_path(&path).0;
        let size_mb = size_bytes as f32 / (1024.0 * 1024.0);

        if dry_run {
            result.deleted_count = result.deleted_count.saturating_add(1);
            result.freed_mb += size_mb;
            continue;
        }

        let removed = if path.is_dir() {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };

        match removed {
            Ok(()) => {
                result.deleted_count = result.deleted_count.saturating_add(1);
                result.freed_mb += size_mb;
            }
            Err(err) => {
                use std::io::ErrorKind;
                // ERROR_SHARING_VIOLATION (32) / ERROR_LOCK_VIOLATION (33) = en uso.
                let in_use = matches!(err.kind(), ErrorKind::PermissionDenied)
                    || matches!(err.raw_os_error(), Some(32) | Some(33));
                if in_use {
                    result.skipped_in_use = result.skipped_in_use.saturating_add(1);
                } else {
                    result.error_count = result.error_count.saturating_add(1);
                }
            }
        }
    }

    result
}

/// Traduce peso en severidad visual.
pub fn classify_temp_size(bytes: u64, thresholds: &TempThresholds) -> Severity {
    let size_mb = bytes_to_mb(bytes);
    if size_mb >= thresholds.critical_mb {
        Severity::Critical
    } else if size_mb >= thresholds.warning_mb {
        Severity::Warning
    } else {
        Severity::Healthy
    }
}

/// Genera una explicación breve según el tipo de ruta.
fn note_for_path(path: &Path, size_bytes: u64) -> String {
    let display = path.display().to_string().to_ascii_lowercase();

    if display.contains("softwaredistribution") {
        return "Descargas de Windows Update; pueden explicar actividad de disco y red en segundo plano".to_owned();
    }
    if display.contains("deliveryoptimization") {
        return "Caché de Delivery Optimization; puede crecer cuando Windows comparte o descarga actualizaciones".to_owned();
    }
    if display.contains("temp") && size_bytes > 500 * 1024 * 1024 {
        return "Volumen temporal alto; revisar si quedó basura de instaladores, compresiones o exportaciones".to_owned();
    }

    "Elemento temporal/caché observado en el escaneo actual".to_owned()
}

fn bytes_to_mb(bytes: u64) -> f32 {
    bytes as f32 / (1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clasifica_severidad_de_temporales() {
        let thresholds = TempThresholds::default();
        assert_eq!(
            classify_temp_size(10 * 1024 * 1024, &thresholds),
            Severity::Healthy
        );
        assert_eq!(
            classify_temp_size(600 * 1024 * 1024, &thresholds),
            Severity::Warning
        );
        assert_eq!(
            classify_temp_size(2 * 1024 * 1024 * 1024, &thresholds),
            Severity::Critical
        );
    }

    #[test]
    fn incluye_rutas_clave_de_windows() {
        let roots = candidate_roots();
        let text = roots
            .iter()
            .map(|p| p.display().to_string().to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(
                "
",
            );
        assert!(text.contains("windows\\temp"));
        assert!(text.contains("softwaredistribution"));
        assert!(text.contains("deliveryoptimization"));
    }
}
