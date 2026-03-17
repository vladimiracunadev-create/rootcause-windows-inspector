//! Escaneo de carpetas temporales.
//!
//! Objetivo: detectar rápidamente qué zonas de TEMP están creciendo y cuánto
//! pesan, sin convertir el monitor en otra carga pesada. Por eso el escaneo
//! usa límites razonables de profundidad y conteo de archivos.

use crate::models::{Severity, TempEntry, TempOverview};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MAX_FILES_PER_ENTRY: usize = 20_000;

/// Construye el resumen de temporales más relevante para la UI.
pub fn scan_temp_overview() -> Result<TempOverview> {
    let roots = candidate_roots();
    let mut top_entries = Vec::new();
    let mut limitations = Vec::new();
    let mut total_bytes: u64 = 0;

    for root in &roots {
        if !root.exists() {
            continue;
        }

        match scan_root(root) {
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
fn scan_root(root: &Path) -> Result<(u64, Vec<TempEntry>)> {
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
            severity: classify_temp_size(size_bytes),
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

/// Traduce peso en severidad visual.
pub fn classify_temp_size(bytes: u64) -> Severity {
    match bytes {
        0..=262_143_999 => Severity::Healthy,
        262_144_000..=1_073_741_823 => Severity::Warning,
        _ => Severity::Critical,
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
        assert_eq!(classify_temp_size(10 * 1024 * 1024), Severity::Healthy);
        assert_eq!(classify_temp_size(600 * 1024 * 1024), Severity::Warning);
        assert_eq!(
            classify_temp_size(2 * 1024 * 1024 * 1024),
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
