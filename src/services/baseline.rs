//! Motor genérico de detección de cambios contra baseline.
//!
//! Generaliza el patrón que estrenó el tab Autostart: cada "superficie
//! vigilada" (servicios, tareas, hosts, claves de registro…) aporta una lista
//! de [`WatchedItem`], y este motor la compara contra una foto de referencia
//! ("estado bueno conocido") guardada en SQLite, clasificando cada ítem como
//! NUEVA / MODIFICADA / ELIMINADA. La primera foto se siembra en silencio; los
//! cambios son pegajosos hasta que el usuario acepta la baseline.

use crate::models::{AnomalyEvent, IncidentEvidence, PersistenceChange, RiskLevel, WatchedItem};
use crate::services::persistence::PersistenceStore;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Descripción de una superficie vigilada para construir textos de eventos.
pub struct SurfaceSpec {
    /// Id corto: se usa como `kind` de anomalía (`<id>-change`) y clave de tabla.
    pub id: &'static str,
    /// Título del evento cuando aparece un ítem nuevo.
    pub title_added: &'static str,
    /// Título del evento cuando un ítem cambia de valor.
    pub title_modified: &'static str,
    /// Título del evento cuando un ítem desaparece.
    pub title_removed: &'static str,
    /// Sustantivo con artículo para el resumen: "El servicio", "La tarea".
    pub summary_noun: &'static str,
}

/// Compara `items` contra la baseline de la superficie y los anota con su
/// estado de cambio. Si la baseline está vacía (primera vez), la siembra con el
/// estado actual y no marca nada. Añade ítems sintéticos para los eliminados.
/// Devuelve `true` si había una baseline previa contra la que comparar.
pub fn diff_surface(
    store: &PersistenceStore,
    surface_id: &str,
    items: &mut Vec<WatchedItem>,
) -> bool {
    let baseline = match store.load_baseline(surface_id) {
        Ok(baseline) => baseline,
        Err(_) => return false,
    };

    if baseline.is_empty() {
        // Primera foto: aceptar todo como baseline "buena conocida".
        let _ = store.replace_baseline(surface_id, items);
        return false;
    }

    let mut current_keys = HashSet::new();
    for item in items.iter_mut() {
        current_keys.insert(item.key.clone());
        item.change_status = match baseline.get(&item.key) {
            None => PersistenceChange::Added,
            Some(base) if base.value != item.value => PersistenceChange::Modified,
            Some(_) => PersistenceChange::Unchanged,
        };
    }

    // Ítems que estaban en la baseline y ya no aparecen: sintéticos eliminados.
    for (key, base) in &baseline {
        if !current_keys.contains(key) {
            let mut removed = base.clone();
            removed.change_status = PersistenceChange::Removed;
            items.push(removed);
        }
    }

    true
}

/// Construye un evento de anomalía para un ítem cambiado. A diferencia de las
/// heurísticas, no depende de "sospecha": cualquier cambio se reporta para dar
/// control explícito al usuario. Devuelve `None` para ítems sin cambios.
pub fn surface_change_event(
    detected_at: DateTime<Utc>,
    spec: &SurfaceSpec,
    item: &WatchedItem,
) -> Option<AnomalyEvent> {
    let (severity, score, title, verb) = match item.change_status {
        PersistenceChange::Added => (RiskLevel::High, 72_u16, spec.title_added, "apareció"),
        PersistenceChange::Modified => (RiskLevel::High, 68, spec.title_modified, "cambió"),
        PersistenceChange::Removed => (RiskLevel::Medium, 45, spec.title_removed, "desapareció"),
        PersistenceChange::Unchanged => return None,
    };

    Some(AnomalyEvent {
        event_id: format!(
            "anom-{}-{}chg-{}",
            detected_at.timestamp_millis(),
            spec.id,
            item.key
        ),
        detected_at,
        severity,
        score,
        status: "open".to_owned(),
        kind: format!("{}-change", spec.id),
        title: title.to_owned(),
        process_name: None,
        pid: None,
        parent_pid: None,
        parent_name: None,
        user: None,
        exe_path: Some(item.detail.clone()),
        sha256: None,
        cpu_percent: None,
        memory_mb: None,
        io_write_mb_delta: None,
        unique_public_remotes: None,
        unique_private_remotes: None,
        summary: format!(
            "{} '{}' {} respecto a la baseline conocida.",
            spec.summary_noun, item.label, verb
        ),
        root_cause_hypothesis:
            "cambio en una superficie vigilada respecto al estado bueno conocido".to_owned(),
        recommended_action:
            "Verificar el origen del cambio y aceptar la baseline solo si es legítimo.".to_owned(),
        evidence: vec![
            IncidentEvidence {
                kind: "item".to_owned(),
                label: "Elemento".to_owned(),
                value: item.label.clone(),
            },
            IncidentEvidence {
                kind: "detail".to_owned(),
                label: "Detalle".to_owned(),
                value: item.detail.clone(),
            },
            IncidentEvidence {
                kind: "change".to_owned(),
                label: "Cambio".to_owned(),
                value: item.change_status.label().to_owned(),
            },
        ],
    })
}
