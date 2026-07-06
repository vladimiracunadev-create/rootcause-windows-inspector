//! Motor ligero de detección de comportamiento anómalo.
//!
//! V1 mantiene el enfoque de RootCause:
//! - heurísticas locales,
//! - correlación básica,
//! - evidencia auditable,
//! - sin firmas masivas,
//! - sin respuesta destructiva automática.

use crate::config::AnomalyConfig;
use crate::models::{
    AnomalyEvent, ConnectionInsight, IncidentEvidence, PersistenceChange, PersistenceEntry,
    ProcessInsight, RiskLevel, ServiceState,
};
use crate::services::network;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
struct ProcessHistory {
    last_memory_mb: f32,
    high_cpu_streak: u8,
    memory_growth_streak: u8,
    aggressive_write_streak: u8,
}

#[derive(Debug, Clone)]
struct RespawnState {
    last_pid: u32,
    last_seen: DateTime<Utc>,
    rapid_respawns: u8,
}

#[derive(Debug, Clone)]
struct ScriptState {
    last_seen: DateTime<Utc>,
    repeats: u8,
}

#[derive(Default)]
pub struct AnomalyTracker {
    process_history: HashMap<u32, ProcessHistory>,
    respawn_history: HashMap<String, RespawnState>,
    script_history: HashMap<String, ScriptState>,
}

pub struct DetectionInput<'a> {
    pub collected_at: DateTime<Utc>,
    pub processes: &'a [ProcessInsight],
    pub connections: &'a [ConnectionInsight],
    pub services: &'a [ServiceState],
    pub persistence_entries: &'a [PersistenceEntry],
    pub config: &'a AnomalyConfig,
}

impl AnomalyTracker {
    fn detect_respawn(
        &mut self,
        process: &ProcessInsight,
        now: DateTime<Utc>,
        config: &AnomalyConfig,
    ) -> Option<u16> {
        let key = normalize_exe_key(process);
        if key.is_empty() {
            return None;
        }
        let window = Duration::seconds(config.respawn_window_secs as i64);
        let state = self.respawn_history.entry(key).or_insert(RespawnState {
            last_pid: process.pid,
            last_seen: now,
            rapid_respawns: 0,
        });

        if state.last_pid != process.pid && now - state.last_seen <= window {
            state.rapid_respawns = state.rapid_respawns.saturating_add(1);
        } else if now - state.last_seen > window {
            state.rapid_respawns = 0;
        }

        state.last_pid = process.pid;
        state.last_seen = now;

        if state.rapid_respawns >= config.respawn_count {
            Some(58 + (state.rapid_respawns as u16 * 6))
        } else {
            None
        }
    }

    fn detect_repetitive_script(
        &mut self,
        process: &ProcessInsight,
        now: DateTime<Utc>,
        config: &AnomalyConfig,
    ) -> Option<u16> {
        if !looks_like_script_engine(&process.name, process.command_line.as_deref()) {
            return None;
        }
        let signature = normalize_script_signature(process)?;
        let window = Duration::seconds(config.respawn_window_secs as i64);
        let state = self.script_history.entry(signature).or_insert(ScriptState {
            last_seen: now,
            repeats: 0,
        });

        if now - state.last_seen <= window {
            state.repeats = state.repeats.saturating_add(1);
        } else {
            state.repeats = 1;
        }
        state.last_seen = now;

        if state.repeats >= config.repetitive_script_count {
            Some(54 + (state.repeats as u16 * 4))
        } else {
            None
        }
    }

    pub fn analyze(&mut self, input: DetectionInput<'_>) -> Vec<AnomalyEvent> {
        if !input.config.enabled {
            return Vec::new();
        }

        let process_map: HashMap<u32, &ProcessInsight> = input
            .processes
            .iter()
            .map(|process| (process.pid, process))
            .collect();
        let network_summary = summarize_network(input.connections);
        let mut anomalies = Vec::new();

        for process in input.processes {
            let now = input.collected_at;
            let lower_path = process.exe_path.to_ascii_lowercase();
            let suspicious_path =
                path_is_suspicious(&lower_path, &process.command_line, input.config);
            let network = network_summary
                .get(&process.pid)
                .cloned()
                .unwrap_or_default();
            let parent = process
                .parent_pid
                .and_then(|pid| process_map.get(&pid).copied());

            let (high_cpu_streak, memory_growth_streak, aggressive_write_streak, memory_growth) = {
                let history = self.process_history.entry(process.pid).or_default();
                let previous_memory = history.last_memory_mb;

                if process.cpu_percent >= input.config.cpu_sustained_percent {
                    history.high_cpu_streak = history.high_cpu_streak.saturating_add(1);
                } else {
                    history.high_cpu_streak = 0;
                }
                if previous_memory > 0.0
                    && process.memory_mb >= previous_memory + input.config.memory_growth_mb
                {
                    history.memory_growth_streak = history.memory_growth_streak.saturating_add(1);
                } else {
                    history.memory_growth_streak = 0;
                }
                if process.io_write_mb_delta >= input.config.aggressive_write_mb {
                    history.aggressive_write_streak =
                        history.aggressive_write_streak.saturating_add(1);
                } else {
                    history.aggressive_write_streak = 0;
                }
                history.last_memory_mb = process.memory_mb;

                (
                    history.high_cpu_streak,
                    history.memory_growth_streak,
                    history.aggressive_write_streak,
                    (process.memory_mb - previous_memory).max(0.0),
                )
            };

            if high_cpu_streak >= input.config.cpu_sustained_samples {
                let score = (42 + high_cpu_streak as u16 * 8).min(88);
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "sustained-high-cpu",
                        title: "CPU sostenido anormal",
                        score,
                        parent,
                        summary: format!(
                            "{} mantiene {:.1}% de CPU durante {} muestras consecutivas.",
                            process.name, process.cpu_percent, high_cpu_streak
                        ),
                        root_cause_hypothesis:
                            "lentitud asociada a proceso no habitual con alto consumo sostenido"
                                .to_owned(),
                        recommended_action:
                            "Observar, validar command line y finalizar manualmente solo si el proceso no corresponde.",
                        evidence: vec![
                            metric_evidence("cpu", "CPU", format!("{:.1}%", process.cpu_percent)),
                            metric_evidence(
                                "samples",
                                "Muestras sostenidas",
                                high_cpu_streak.to_string(),
                            ),
                        ],
                        network: &network,
                    },
                ));
            }

            if memory_growth_streak >= input.config.memory_growth_samples {
                let score = (46 + memory_growth_streak as u16 * 8).min(86);
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "memory-growth",
                        title: "Crecimiento anomalo de memoria",
                        score,
                        parent,
                        summary: format!(
                            "{} incrementa memoria hasta {:.0} MB con crecimiento repetido.",
                            process.name, process.memory_mb
                        ),
                        root_cause_hypothesis:
                            "degradacion compatible con proceso persistente o script abusivo con crecimiento de memoria".to_owned(),
                        recommended_action:
                            "Revisar manualmente el proceso, preservar evidencia y escalar escaneo si no pertenece a la carga normal.",
                        evidence: vec![
                            metric_evidence(
                                "memory",
                                "Memoria",
                                format!("{:.0} MB", process.memory_mb),
                            ),
                            metric_evidence(
                                "growth",
                                "Crecimiento",
                                format!("{memory_growth:.1} MB"),
                            ),
                        ],
                        network: &network,
                    },
                ));
            }

            if aggressive_write_streak >= input.config.aggressive_write_samples {
                let score = (50 + aggressive_write_streak as u16 * 8).min(92);
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "aggressive-disk-write",
                        title: "Escritura agresiva en disco",
                        score,
                        parent,
                        summary: format!(
                            "{} escribe {:.1} MB por intervalo en forma sostenida.",
                            process.name, process.io_write_mb_delta
                        ),
                        root_cause_hypothesis:
                            "degradacion compatible con escritura masiva no autorizada o proceso de alto impacto en disco".to_owned(),
                        recommended_action:
                            "Revisar si corresponde a backup, actualizacion o cifrado no esperado; generar informe tecnico si persiste.",
                        evidence: vec![metric_evidence(
                            "disk-write",
                            "Escritura",
                            format!("{:.1} MB", process.io_write_mb_delta),
                        )],
                        network: &network,
                    },
                ));
            }

            if network.public_unique_destinations >= input.config.public_destination_count {
                let score = (56 + network.public_unique_destinations as u16 * 4).min(90);
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "multi-destination-outbound",
                        title: "Conexiones salientes inusuales",
                        score,
                        parent,
                        summary: format!(
                            "{} se conecta a {} destinos publicos en la misma ventana.",
                            process.name, network.public_unique_destinations
                        ),
                        root_cause_hypothesis:
                            "picos de red asociados a proceso fuera de linea base".to_owned(),
                        recommended_action:
                            "Validar origen del proceso, revisar destinos remotos y considerar bloqueo o escaneo externo si no corresponde.",
                        evidence: vec![metric_evidence(
                            "public-remotes",
                            "Destinos publicos",
                            network.public_unique_destinations.to_string(),
                        )],
                        network: &network,
                    },
                ));
            }

            if suspicious_path {
                let score =
                    if lower_path.contains("\\temp\\") || lower_path.contains("\\downloads\\") {
                        78
                    } else {
                        62
                    };
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "suspicious-execution-path",
                        title: "Ejecucion desde ruta sospechosa",
                        score,
                        parent,
                        summary: format!(
                            "{} se ejecuta desde una ruta atipica: {}.",
                            process.name, process.exe_path
                        ),
                        root_cause_hypothesis:
                            "actividad potencialmente no autorizada asociada a ejecucion fuera de rutas confiables".to_owned(),
                        recommended_action:
                            "Revisar manualmente la ruta, validar el origen del binario y evitar ejecucion futura si no corresponde.",
                        evidence: vec![metric_evidence("path", "Ruta", process.exe_path.clone())],
                        network: &network,
                    },
                ));
            }

            if !is_trusted_process(&process.name, &lower_path, input.config)
                && should_flag_untrusted_process(process, &network, suspicious_path)
            {
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "outside-trusted-baseline",
                        title: "Proceso fuera de linea confiable",
                        score: 60,
                        parent,
                        summary: format!(
                            "{} no coincide con la linea base confiable local y presenta actividad relevante.",
                            process.name
                        ),
                        root_cause_hypothesis:
                            "riesgo medio-alto por ejecucion no habitual con impacto operativo"
                                .to_owned(),
                        recommended_action:
                            "Revisar manualmente, contrastar con software autorizado y escanear con antivirus/EDR si hay dudas.",
                        evidence: vec![metric_evidence("path", "Ruta", process.exe_path.clone())],
                        network: &network,
                    },
                ));
            }

            if let Some(parent_process) = parent
                && suspicious_parent_child(parent_process, process, input.config)
            {
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "suspicious-parent-child",
                        title: "Relacion padre-hijo sospechosa",
                        score: 72,
                        parent: Some(parent_process),
                        summary: format!(
                            "{} fue lanzado por {}.",
                            process.name, parent_process.name
                        ),
                        root_cause_hypothesis:
                            "actividad compatible con ejecucion encadenada fuera del patron normal"
                                .to_owned(),
                        recommended_action:
                            "Revisar manualmente el proceso padre, command line y origen del archivo adjunto o launcher asociado.",
                        evidence: vec![
                            metric_evidence("parent", "Padre", parent_process.name.clone()),
                            metric_evidence("child", "Hijo", process.name.clone()),
                        ],
                        network: &network,
                    },
                ));
            }

            if let Some(score) = self.detect_respawn(process, now, input.config) {
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "rapid-respawn",
                        title: "Reaparicion rapida de proceso",
                        score: score.min(92),
                        parent,
                        summary: format!(
                            "{} reaparecio repetidamente en una ventana corta.",
                            process.name
                        ),
                        root_cause_hypothesis:
                            "degradacion compatible con proceso que se reinicia automaticamente o mecanismo de persistencia".to_owned(),
                        recommended_action:
                            "Revisar persistencia, servicio asociado y tareas programadas antes de finalizar el proceso.",
                        evidence: vec![metric_evidence(
                            "pid",
                            "PID actual",
                            process.pid.to_string(),
                        )],
                        network: &network,
                    },
                ));
            }

            if let Some(score) = self.detect_repetitive_script(process, now, input.config) {
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "repetitive-script-execution",
                        title: "Ejecucion repetitiva de scripts o comandos",
                        score: score.min(88),
                        parent,
                        summary: format!(
                            "{} repite una invocacion de script fuera del patron esperado.",
                            process.name
                        ),
                        root_cause_hypothesis:
                            "actividad compatible con automatizacion agresiva, script abusivo o intento de persistencia".to_owned(),
                        recommended_action:
                            "Revisar command line, scheduler, carpeta Startup y origen del script antes de intervenir.",
                        evidence: command_line_evidence(process),
                        network: &network,
                    },
                ));
            }

            if let Some(score) = security_control_score(process) {
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "security-control-alteration",
                        title: "Intento de alterar seguridad local",
                        score,
                        parent,
                        summary: format!(
                            "{} ejecuta comandos compatibles con cambios sobre controles de seguridad.",
                            process.name
                        ),
                        root_cause_hypothesis:
                            "alteracion compatible con desactivacion o debilitamiento de defensas locales".to_owned(),
                        recommended_action:
                            "Revisar urgentemente el comando, validar usuario/contexto y escanear el endpoint con herramientas dedicadas.",
                        evidence: command_line_evidence(process),
                        network: &network,
                    },
                ));
            }

            if network.private_unique_destinations >= input.config.local_scan_destination_count {
                let score = (52 + network.private_unique_destinations as u16 * 3).min(88);
                anomalies.push(build_process_event(
                    process,
                    ProcessEventSpec {
                        detected_at: now,
                        kind: "local-network-scan",
                        title: "Patron de exploracion agresiva en red local",
                        score,
                        parent,
                        summary: format!(
                            "{} contacta {} destinos privados distintos en poco tiempo.",
                            process.name, network.private_unique_destinations
                        ),
                        root_cause_hypothesis:
                            "actividad compatible con propagacion basica o exploracion interna no habitual".to_owned(),
                        recommended_action:
                            "Validar si corresponde a software de inventario o administracion; si no, aislar red y revisar el host.",
                        evidence: vec![metric_evidence(
                            "private-remotes",
                            "Destinos privados",
                            network.private_unique_destinations.to_string(),
                        )],
                        network: &network,
                    },
                ));
            }
        }

        if input.config.watch_persistence {
            anomalies.extend(
                input
                    .persistence_entries
                    .iter()
                    .filter_map(|entry| persistence_event(input.collected_at, entry, input.config)),
            );
        }

        anomalies.extend(input.services.iter().filter_map(|service| {
            security_service_event(input.collected_at, service, input.config)
        }));

        let correlated = correlate_anomalies(input.collected_at, &anomalies);
        anomalies.extend(correlated);

        anomalies.sort_by(|left, right| {
            right
                .severity
                .cmp(&left.severity)
                .then_with(|| right.score.cmp(&left.score))
                .then_with(|| left.kind.cmp(&right.kind))
        });

        self.process_history
            .retain(|pid, _| process_map.contains_key(pid));
        let respawn_window = Duration::seconds(input.config.respawn_window_secs as i64);
        self.respawn_history
            .retain(|_, state| input.collected_at - state.last_seen <= respawn_window);
        self.script_history
            .retain(|_, state| input.collected_at - state.last_seen <= respawn_window);

        anomalies
    }
}

#[derive(Debug, Clone, Default)]
struct ProcessNetworkSummary {
    public_unique_destinations: usize,
    private_unique_destinations: usize,
}

struct ProcessEventSpec<'a> {
    detected_at: DateTime<Utc>,
    kind: &'a str,
    title: &'a str,
    score: u16,
    parent: Option<&'a ProcessInsight>,
    summary: String,
    root_cause_hypothesis: String,
    recommended_action: &'a str,
    evidence: Vec<IncidentEvidence>,
    network: &'a ProcessNetworkSummary,
}

fn summarize_network(connections: &[ConnectionInsight]) -> HashMap<u32, ProcessNetworkSummary> {
    let mut public_map: HashMap<u32, HashSet<String>> = HashMap::new();
    let mut private_map: HashMap<u32, HashSet<String>> = HashMap::new();

    for connection in connections {
        let remote_ip = network::extract_ip(&connection.remote_address)
            .unwrap_or_else(|| connection.remote_address.clone());
        if connection.is_public_remote {
            public_map
                .entry(connection.pid)
                .or_default()
                .insert(remote_ip);
        } else if !remote_ip.is_empty()
            && remote_ip != "*"
            && !remote_ip.eq_ignore_ascii_case("0.0.0.0")
        {
            private_map
                .entry(connection.pid)
                .or_default()
                .insert(remote_ip);
        }
    }

    let mut summary = HashMap::new();
    for pid in public_map.keys().chain(private_map.keys()) {
        summary.insert(
            *pid,
            ProcessNetworkSummary {
                public_unique_destinations: public_map.get(pid).map(|s| s.len()).unwrap_or(0),
                private_unique_destinations: private_map.get(pid).map(|s| s.len()).unwrap_or(0),
            },
        );
    }
    summary
}

fn risk_from_score(score: u16) -> RiskLevel {
    match score {
        0..=24 => RiskLevel::Low,
        25..=49 => RiskLevel::Medium,
        50..=79 => RiskLevel::High,
        _ => RiskLevel::Critical,
    }
}

fn metric_evidence(kind: &str, label: &str, value: String) -> IncidentEvidence {
    IncidentEvidence {
        kind: kind.to_owned(),
        label: label.to_owned(),
        value,
    }
}

fn build_process_event(process: &ProcessInsight, spec: ProcessEventSpec<'_>) -> AnomalyEvent {
    let ProcessEventSpec {
        detected_at,
        kind,
        title,
        score,
        parent,
        summary,
        root_cause_hypothesis,
        recommended_action,
        mut evidence,
        network,
    } = spec;
    if let Some(cmdline) = process.command_line.as_ref() {
        evidence.push(metric_evidence(
            "command-line",
            "Command line",
            cmdline.clone(),
        ));
    }
    if network.public_unique_destinations > 0 {
        evidence.push(metric_evidence(
            "public-remotes",
            "Destinos publicos",
            format!("{}", network.public_unique_destinations),
        ));
    }
    if network.private_unique_destinations > 0 {
        evidence.push(metric_evidence(
            "private-remotes",
            "Destinos privados",
            format!("{}", network.private_unique_destinations),
        ));
    }

    AnomalyEvent {
        event_id: format!(
            "anom-{}-{}-{}",
            detected_at.timestamp_millis(),
            kind,
            process.pid
        ),
        detected_at,
        severity: risk_from_score(score),
        score,
        status: "open".to_owned(),
        kind: kind.to_owned(),
        title: title.to_owned(),
        process_name: Some(process.name.clone()),
        pid: Some(process.pid),
        parent_pid: process.parent_pid,
        parent_name: parent.map(|p| p.name.clone()),
        user: None,
        exe_path: Some(process.exe_path.clone()),
        sha256: None,
        cpu_percent: Some(process.cpu_percent),
        memory_mb: Some(process.memory_mb),
        io_write_mb_delta: Some(process.io_write_mb_delta),
        unique_public_remotes: Some(network.public_unique_destinations),
        unique_private_remotes: Some(network.private_unique_destinations),
        summary,
        root_cause_hypothesis,
        recommended_action: recommended_action.to_owned(),
        evidence,
    }
}

fn persistence_event(
    detected_at: DateTime<Utc>,
    entry: &PersistenceEntry,
    config: &AnomalyConfig,
) -> Option<AnomalyEvent> {
    // Las entradas sintéticas "eliminadas" no representan persistencia activa.
    if entry.change_status == PersistenceChange::Removed {
        return None;
    }
    let command_lower = entry.command.to_ascii_lowercase();
    let path_lower = entry
        .target_path
        .as_deref()
        .unwrap_or(entry.command.as_str())
        .to_ascii_lowercase();
    let suspicious_path = path_is_suspicious(&path_lower, &Some(entry.command.clone()), config);
    let script_like = looks_like_script_command(&command_lower);
    let untrusted = !is_trusted_path_only(&path_lower, config);

    if !suspicious_path && !script_like && !untrusted {
        return None;
    }

    let mut score = 42_u16;
    if suspicious_path {
        score += 18;
    }
    if script_like {
        score += 12;
    }
    if !entry.exists_on_disk {
        score += 10;
    }

    Some(AnomalyEvent {
        event_id: format!("anom-{}-persistence-{}", detected_at.timestamp_millis(), entry.name),
        detected_at,
        severity: risk_from_score(score),
        score,
        status: "open".to_owned(),
        kind: "suspicious-persistence".to_owned(),
        title: "Persistencia sospechosa".to_owned(),
        process_name: None,
        pid: None,
        parent_pid: None,
        parent_name: None,
        user: None,
        exe_path: entry.target_path.clone(),
        sha256: None,
        cpu_percent: None,
        memory_mb: None,
        io_write_mb_delta: None,
        unique_public_remotes: None,
        unique_private_remotes: None,
        summary: format!(
            "{} registra '{}' en {}.",
            entry.entry_kind, entry.name, entry.location
        ),
        root_cause_hypothesis:
            "degradacion compatible con actividad persistente no autorizada".to_owned(),
        recommended_action:
            "Revisar persistencia, validar origen del binario y escanear con antivirus/EDR si no corresponde.".to_owned(),
        evidence: vec![
            metric_evidence("location", "Ubicacion", entry.location.clone()),
            metric_evidence("command", "Comando", entry.command.clone()),
        ],
    })
}

/// Genera un evento cuando una entrada de autoarranque cambia respecto a la
/// baseline conocida (nueva, modificada o eliminada). A diferencia de
/// `persistence_event`, no depende de heurísticas de "sospecha": cualquier
/// cambio se reporta para que el usuario tenga control explícito.
pub fn persistence_change_event(
    detected_at: DateTime<Utc>,
    entry: &PersistenceEntry,
) -> Option<AnomalyEvent> {
    let (severity, score, title, verb) = match entry.change_status {
        PersistenceChange::Added => (RiskLevel::High, 70_u16, "Autoarranque nuevo", "apareció"),
        PersistenceChange::Modified => (
            RiskLevel::High,
            68,
            "Autoarranque modificado",
            "cambió de comando",
        ),
        PersistenceChange::Removed => (
            RiskLevel::Medium,
            45,
            "Autoarranque eliminado",
            "desapareció",
        ),
        PersistenceChange::Unchanged => return None,
    };

    Some(AnomalyEvent {
        event_id: format!(
            "anom-{}-persistchg-{}-{}",
            detected_at.timestamp_millis(),
            entry.change_status.label(),
            entry.name
        ),
        detected_at,
        severity,
        score,
        status: "open".to_owned(),
        kind: "persistence-change".to_owned(),
        title: title.to_owned(),
        process_name: None,
        pid: None,
        parent_pid: None,
        parent_name: None,
        user: None,
        exe_path: entry.target_path.clone(),
        sha256: None,
        cpu_percent: None,
        memory_mb: None,
        io_write_mb_delta: None,
        unique_public_remotes: None,
        unique_private_remotes: None,
        summary: format!(
            "La entrada '{}' en {} ({}) {} respecto a la baseline conocida.",
            entry.name, entry.location, entry.entry_kind, verb
        ),
        root_cause_hypothesis:
            "cambio en un punto de autoarranque respecto al estado bueno conocido".to_owned(),
        recommended_action: match entry.change_status {
            PersistenceChange::Removed =>
                "Confirmar si corresponde a una desinstalación esperada; si no, investigar."
                    .to_owned(),
            _ =>
                "Verificar el origen del binario/comando y aceptar como baseline solo si es legítimo."
                    .to_owned(),
        },
        evidence: vec![
            metric_evidence("location", "Ubicacion", entry.location.clone()),
            metric_evidence("command", "Comando", entry.command.clone()),
            metric_evidence("change", "Cambio", entry.change_status.label().to_owned()),
        ],
    })
}

fn security_service_event(
    detected_at: DateTime<Utc>,
    service: &ServiceState,
    config: &AnomalyConfig,
) -> Option<AnomalyEvent> {
    if !config
        .security_service_names
        .iter()
        .any(|name| name.eq_ignore_ascii_case(&service.name))
    {
        return None;
    }

    if service.status.eq_ignore_ascii_case("Running") {
        return None;
    }

    let critical_service = service.name.eq_ignore_ascii_case("WinDefend")
        || service.name.eq_ignore_ascii_case("MpsSvc");
    let score = if critical_service { 86 } else { 68 };

    Some(AnomalyEvent {
        event_id: format!("anom-{}-service-{}", detected_at.timestamp_millis(), service.name),
        detected_at,
        severity: risk_from_score(score),
        score,
        status: "open".to_owned(),
        kind: "security-service-disabled".to_owned(),
        title: "Servicio de seguridad alterado".to_owned(),
        process_name: None,
        pid: None,
        parent_pid: None,
        parent_name: None,
        user: None,
        exe_path: None,
        sha256: None,
        cpu_percent: None,
        memory_mb: None,
        io_write_mb_delta: None,
        unique_public_remotes: None,
        unique_private_remotes: None,
        summary: format!(
            "{} ({}) reporta estado {}.",
            service.display_name, service.name, service.status
        ),
        root_cause_hypothesis:
            "alteracion compatible con desactivacion o degradacion de mecanismos de seguridad locales"
                .to_owned(),
        recommended_action:
            "Revisar manualmente el estado del servicio, validar politica local y escalar a revision de seguridad.".to_owned(),
        evidence: vec![
            metric_evidence("service", "Servicio", service.name.clone()),
            metric_evidence("status", "Estado", service.status.clone()),
            metric_evidence("start-type", "Inicio", service.start_type.clone()),
        ],
    })
}

fn correlate_anomalies(
    detected_at: DateTime<Utc>,
    anomalies: &[AnomalyEvent],
) -> Vec<AnomalyEvent> {
    let mut groups: HashMap<String, Vec<&AnomalyEvent>> = HashMap::new();
    for anomaly in anomalies {
        let key = anomaly
            .pid
            .map(|pid| format!("pid:{pid}"))
            .or_else(|| anomaly.exe_path.as_ref().map(|path| format!("path:{path}")))
            .or_else(|| {
                anomaly
                    .process_name
                    .as_ref()
                    .map(|name| format!("name:{name}"))
            });
        if let Some(key) = key {
            groups.entry(key).or_default().push(anomaly);
        }
    }

    let mut correlated = Vec::new();
    for events in groups.into_values() {
        if events.len() < 2 {
            continue;
        }
        let kinds = dedupe(
            events
                .iter()
                .map(|event| human_kind(&event.kind).to_owned())
                .collect(),
        );
        let top = events[0];
        let score = events
            .iter()
            .take(3)
            .map(|event| event.score)
            .sum::<u16>()
            .min(100);
        correlated.push(AnomalyEvent {
            event_id: format!(
                "anom-{}-correlation-{}",
                detected_at.timestamp_millis(),
                top.pid.unwrap_or_default()
            ),
            detected_at,
            severity: risk_from_score(score),
            score,
            status: "open".to_owned(),
            kind: "correlated-anomaly".to_owned(),
            title: "Correlacion de señales anomalas".to_owned(),
            process_name: top.process_name.clone(),
            pid: top.pid,
            parent_pid: top.parent_pid,
            parent_name: top.parent_name.clone(),
            user: top.user.clone(),
            exe_path: top.exe_path.clone(),
            sha256: None,
            cpu_percent: top.cpu_percent,
            memory_mb: top.memory_mb,
            io_write_mb_delta: top.io_write_mb_delta,
            unique_public_remotes: top.unique_public_remotes,
            unique_private_remotes: top.unique_private_remotes,
            summary: format!(
                "Se correlacionaron {} señales en el mismo proceso/contexto.",
                events.len()
            ),
            root_cause_hypothesis: format!(
                "riesgo {} por combinacion de {}",
                risk_from_score(score).label().to_ascii_lowercase(),
                kinds.join(" + ")
            ),
            recommended_action:
                "Priorizar revision manual, preservar evidencia y considerar aislamiento de red o escaneo con antivirus/EDR si no corresponde al contexto.".to_owned(),
            evidence: events
                .iter()
                .take(4)
                .map(|event| metric_evidence(&event.kind, &event.title, event.summary.clone()))
                .collect(),
        });
    }
    correlated
}

fn path_is_suspicious(path: &str, command_line: &Option<String>, config: &AnomalyConfig) -> bool {
    if config
        .suspicious_path_keywords
        .iter()
        .any(|needle| path.contains(&needle.to_ascii_lowercase()))
    {
        return true;
    }
    command_line
        .as_ref()
        .map(|cmd| {
            let lower = cmd.to_ascii_lowercase();
            config
                .suspicious_path_keywords
                .iter()
                .any(|needle| lower.contains(&needle.to_ascii_lowercase()))
        })
        .unwrap_or(false)
}

fn is_trusted_process(name: &str, path: &str, config: &AnomalyConfig) -> bool {
    config
        .trusted_process_names
        .iter()
        .any(|trusted| trusted.eq_ignore_ascii_case(name))
        || is_trusted_path_only(path, config)
}

fn is_trusted_path_only(path: &str, config: &AnomalyConfig) -> bool {
    config
        .trusted_path_prefixes
        .iter()
        .any(|prefix| path.starts_with(&prefix.to_ascii_lowercase()))
}

fn should_flag_untrusted_process(
    process: &ProcessInsight,
    summary: &ProcessNetworkSummary,
    suspicious_path: bool,
) -> bool {
    suspicious_path
        || process.cpu_percent >= 30.0
        || process.io_write_mb_delta >= 40.0
        || summary.public_unique_destinations > 0
}

fn normalize_exe_key(process: &ProcessInsight) -> String {
    if !process.exe_path.is_empty() {
        process.exe_path.to_ascii_lowercase()
    } else {
        process.name.to_ascii_lowercase()
    }
}

fn looks_like_script_engine(name: &str, command_line: Option<&str>) -> bool {
    let lower_name = name.to_ascii_lowercase();
    let lower_cmd = command_line.unwrap_or_default().to_ascii_lowercase();
    [
        "powershell",
        "cmd.exe",
        "wscript",
        "cscript",
        "mshta",
        "python",
    ]
    .iter()
    .any(|needle| lower_name.contains(needle) || lower_cmd.contains(needle))
}

fn looks_like_script_command(command: &str) -> bool {
    [
        ".ps1",
        ".bat",
        ".cmd",
        ".vbs",
        ".js",
        "powershell",
        "wscript",
        "cscript",
        "mshta",
    ]
    .iter()
    .any(|needle| command.contains(needle))
}

fn normalize_script_signature(process: &ProcessInsight) -> Option<String> {
    let cmd = process.command_line.as_ref()?.trim();
    if cmd.is_empty() {
        return None;
    }
    Some(format!(
        "{}|{}",
        process.name.to_ascii_lowercase(),
        cmd.to_ascii_lowercase()
    ))
}

fn security_control_score(process: &ProcessInsight) -> Option<u16> {
    let cmd = process.command_line.as_ref()?.to_ascii_lowercase();
    let suspicious_terms = ["set-mppreference", "netsh advfirewall"];
    if suspicious_terms.iter().any(|term| cmd.contains(term)) {
        Some(84)
    } else {
        None
    }
}

fn command_line_evidence(process: &ProcessInsight) -> Vec<IncidentEvidence> {
    process
        .command_line
        .as_ref()
        .map(|cmd| vec![metric_evidence("command-line", "Command line", cmd.clone())])
        .unwrap_or_default()
}

fn suspicious_parent_child(
    parent: &ProcessInsight,
    child: &ProcessInsight,
    config: &AnomalyConfig,
) -> bool {
    let parent_name = parent.name.to_ascii_lowercase();
    let child_name = child.name.to_ascii_lowercase();
    let parent_is_interpreter = config
        .suspicious_parent_names
        .iter()
        .any(|item| item.eq_ignore_ascii_case(&parent_name));
    let child_is_interpreter = looks_like_script_engine(&child_name, child.command_line.as_deref());
    let child_is_lolbin = [
        "powershell.exe",
        "cmd.exe",
        "wscript.exe",
        "cscript.exe",
        "mshta.exe",
    ]
    .iter()
    .any(|item| child_name.contains(item));

    (parent_is_interpreter && (child_is_interpreter || child_is_lolbin))
        || ((parent_name.contains("winword")
            || parent_name.contains("excel")
            || parent_name.contains("outlook")
            || parent_name.contains("chrome")
            || parent_name.contains("msedge"))
            && (child_is_interpreter || child_is_lolbin))
}

fn dedupe(items: Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in items {
        if !out.iter().any(|existing| existing == &item) {
            out.push(item);
        }
    }
    out
}

fn human_kind(kind: &str) -> &'static str {
    match kind {
        "sustained-high-cpu" => "CPU sostenida",
        "memory-growth" => "crecimiento de memoria",
        "aggressive-disk-write" => "escritura agresiva",
        "multi-destination-outbound" => "trafico saliente",
        "suspicious-execution-path" => "ruta de ejecucion sospechosa",
        "outside-trusted-baseline" => "fuera de linea base",
        "suspicious-persistence" => "persistencia",
        "rapid-respawn" => "reaparicion automatica",
        "suspicious-parent-child" => "relacion padre-hijo sospechosa",
        "repetitive-script-execution" => "scripts repetitivos",
        "security-control-alteration" => "alteracion de seguridad",
        "local-network-scan" => "escaneo de red local",
        _ => "senales correlacionadas",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AnomalyConfig;

    fn sample_process(pid: u32, name: &str) -> ProcessInsight {
        ProcessInsight {
            pid,
            name: name.to_owned(),
            exe_path: format!(r"c:\users\demo\appdata\local\temp\{name}"),
            parent_pid: Some(40),
            cpu_percent: 82.0,
            memory_mb: 220.0,
            io_write_mb_delta: 64.0,
            command_line: Some(format!(r#""c:\temp\{name}" -nop -w hidden"#)),
            ..Default::default()
        }
    }

    #[test]
    fn correlation_event_is_generated_for_same_process_context() {
        let now = Utc::now();
        let process = sample_process(4242, "powershell.exe");
        let network = ProcessNetworkSummary::default();
        let events = vec![
            build_process_event(
                &process,
                ProcessEventSpec {
                    detected_at: now,
                    kind: "suspicious-execution-path",
                    title: "Ruta sospechosa",
                    score: 72,
                    parent: None,
                    summary: "demo".to_owned(),
                    root_cause_hypothesis: "demo".to_owned(),
                    recommended_action: "demo",
                    evidence: vec![],
                    network: &network,
                },
            ),
            build_process_event(
                &process,
                ProcessEventSpec {
                    detected_at: now,
                    kind: "multi-destination-outbound",
                    title: "Red sospechosa",
                    score: 68,
                    parent: None,
                    summary: "demo".to_owned(),
                    root_cause_hypothesis: "demo".to_owned(),
                    recommended_action: "demo",
                    evidence: vec![],
                    network: &network,
                },
            ),
        ];

        let correlated = correlate_anomalies(now, &events);
        assert_eq!(correlated.len(), 1);
        assert_eq!(correlated[0].kind, "correlated-anomaly");
        assert_eq!(correlated[0].score, 100);
    }

    #[test]
    fn summarize_network_counts_private_and_public_destinations() {
        let connections = vec![
            ConnectionInsight {
                pid: 9,
                remote_address: "8.8.8.8:443".to_owned(),
                is_public_remote: true,
                ..Default::default()
            },
            ConnectionInsight {
                pid: 9,
                remote_address: "8.8.4.4:443".to_owned(),
                is_public_remote: true,
                ..Default::default()
            },
            ConnectionInsight {
                pid: 9,
                remote_address: "192.168.1.10:445".to_owned(),
                is_public_remote: false,
                ..Default::default()
            },
        ];

        let summary = summarize_network(&connections);
        let item = summary.get(&9).expect("debe existir");
        assert_eq!(item.public_unique_destinations, 2);
        assert_eq!(item.private_unique_destinations, 1);
    }

    #[test]
    fn analyze_detects_suspicious_temp_process() {
        let now = Utc::now();
        let process = sample_process(3131, "powershell.exe");
        let mut tracker = AnomalyTracker::default();
        let anomalies = tracker.analyze(DetectionInput {
            collected_at: now,
            processes: &[process],
            connections: &[],
            services: &[],
            persistence_entries: &[],
            config: &AnomalyConfig::default(),
        });

        assert!(
            anomalies
                .iter()
                .any(|event| event.kind == "suspicious-execution-path")
        );
    }
}
