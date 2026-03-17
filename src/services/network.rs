//! Parseo y clasificación de conexiones de red.
//!
//! Para minimizar dependencias y evitar capturas de bajo nivel demasiado
//! invasivas, esta primera versión usa `netstat -ano -n` y aplica heurísticas.
//! Es suficiente para responder preguntas muy prácticas:
//! - ¿qué proceso tiene conexiones activas?
//! - ¿la IP remota parece pública?
//! - ¿la ruta del ejecutable es sospechosa?

use crate::models::{ConnectionInsight, Severity};
use std::collections::HashMap;

/// Parsea el texto devuelto por `netstat -ano -n`.
pub fn parse_netstat_output(
    output: &str,
    process_names: &HashMap<u32, String>,
    process_paths: &HashMap<u32, String>,
) -> Vec<ConnectionInsight> {
    let mut rows = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with("Proto") {
            continue;
        }

        let columns: Vec<&str> = line.split_whitespace().collect();
        if columns.is_empty() {
            continue;
        }

        match columns[0] {
            "TCP" if columns.len() >= 5 => {
                let pid = columns[4].parse::<u32>().unwrap_or_default();
                let remote_address = columns[2].to_string();
                let exe_path = process_paths.get(&pid).cloned().unwrap_or_default();
                let process_name = process_names
                    .get(&pid)
                    .cloned()
                    .unwrap_or_else(|| "Desconocido".to_owned());
                let (severity, reason, is_public_remote) =
                    classify_connection(&process_name, &exe_path, &remote_address, columns[3]);

                rows.push(ConnectionInsight {
                    protocol: "TCP".to_owned(),
                    local_address: columns[1].to_owned(),
                    remote_address,
                    state: columns[3].to_owned(),
                    pid,
                    process_name,
                    exe_path,
                    severity,
                    reason,
                    is_public_remote,
                });
            }
            "UDP" if columns.len() >= 4 => {
                let pid = columns[3].parse::<u32>().unwrap_or_default();
                let remote_address = columns[2].to_string();
                let exe_path = process_paths.get(&pid).cloned().unwrap_or_default();
                let process_name = process_names
                    .get(&pid)
                    .cloned()
                    .unwrap_or_else(|| "Desconocido".to_owned());
                let (severity, reason, is_public_remote) =
                    classify_connection(&process_name, &exe_path, &remote_address, "UDP");

                rows.push(ConnectionInsight {
                    protocol: "UDP".to_owned(),
                    local_address: columns[1].to_owned(),
                    remote_address,
                    state: "ACTIVE".to_owned(),
                    pid,
                    process_name,
                    exe_path,
                    severity,
                    reason,
                    is_public_remote,
                });
            }
            _ => {}
        }
    }

    rows.sort_by(|a, b| {
        b.severity
            .cmp(&a.severity)
            .then_with(|| a.process_name.cmp(&b.process_name))
    });
    rows
}

/// Determina si una conexión debe marcarse como normal, advertencia o crítica.
pub fn classify_connection(
    process_name: &str,
    exe_path: &str,
    remote_address: &str,
    state: &str,
) -> (Severity, String, bool) {
    let remote_ip = extract_ip(remote_address);
    let is_public_remote = remote_ip.as_deref().map(is_public_ip).unwrap_or(false);

    let lower_name = process_name.to_ascii_lowercase();
    let lower_path = exe_path.to_ascii_lowercase();
    let browser = looks_like_browser(&lower_name);
    let path_is_temp =
        lower_path.contains("\\temp\\") || lower_path.contains("\\appdata\\local\\temp\\");
    let established = state.eq_ignore_ascii_case("ESTABLISHED");

    if is_public_remote && path_is_temp && established {
        return (
            Severity::Critical,
            "Conexión pública desde ejecutable ubicado en carpeta temporal".to_owned(),
            true,
        );
    }

    if is_public_remote && established && !browser {
        return (
            Severity::Warning,
            "Conexión pública activa; conviene validar si este proceso debería salir a Internet"
                .to_owned(),
            true,
        );
    }

    if is_public_remote && state.eq_ignore_ascii_case("UDP") {
        return (
            Severity::Warning,
            "Tráfico UDP hacia IP pública; revisar si corresponde a software esperado".to_owned(),
            true,
        );
    }

    (
        Severity::Healthy,
        "Conexión sin señales obvias de riesgo en esta heurística".to_owned(),
        is_public_remote,
    )
}

/// Extrae la IP de un endpoint `ip:puerto` o `[ipv6]:puerto`.
pub fn extract_ip(endpoint: &str) -> Option<String> {
    let trimmed = endpoint.trim();
    if trimmed == "*:*" || trimmed == "0.0.0.0:0" {
        return None;
    }

    if let Some(rest) = trimmed.strip_prefix('[') {
        return rest.split(']').next().map(|s| s.to_owned());
    }

    if let Some((host, _port)) = trimmed.rsplit_once(':') {
        return Some(host.to_owned());
    }

    Some(trimmed.to_owned())
}

/// Heurística local para clasificar IPs privadas/loopback vs públicas.
pub fn is_public_ip(ip: &str) -> bool {
    let ip = ip.trim().to_ascii_lowercase();

    if ip.is_empty()
        || ip == "*"
        || ip == "0.0.0.0"
        || ip == "::"
        || ip == "::1"
        || ip.starts_with("127.")
        || ip.starts_with("10.")
        || ip.starts_with("192.168.")
        || ip.starts_with("169.254.")
        || ip.starts_with("fe80:")
        || ip.starts_with("fc")
        || ip.starts_with("fd")
    {
        return false;
    }

    if let Some(rest) = ip.strip_prefix("172.") {
        if let Some(second_octet) = rest.split('.').next() {
            if let Ok(value) = second_octet.parse::<u8>() {
                if (16..=31).contains(&value) {
                    return false;
                }
            }
        }
    }

    true
}

/// Distingue navegadores comunes para no inundar la UI con falsos positivos.
pub fn looks_like_browser(name: &str) -> bool {
    ["chrome", "msedge", "firefox", "opera", "brave", "vivaldi"]
        .iter()
        .any(|needle| name.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn detecta_ip_publica_correctamente() {
        assert!(is_public_ip("8.8.8.8"));
        assert!(!is_public_ip("127.0.0.1"));
        assert!(!is_public_ip("192.168.1.3"));
        assert!(!is_public_ip("172.16.5.20"));
        assert!(is_public_ip("172.32.5.20"));
    }

    #[test]
    fn parsea_lineas_tcp_y_udp() {
        let mut names = HashMap::new();
        names.insert(1000_u32, "weird-updater.exe".to_owned());
        let mut paths = HashMap::new();
        paths.insert(
            1000_u32,
            r"C:\Users\vbav\AppData\Local\Temp\weird-updater.exe".to_owned(),
        );

        let input = r#"
  Proto  Local Address          Foreign Address        State           PID
  TCP    192.168.1.15:53010     8.8.8.8:443            ESTABLISHED     1000
  UDP    0.0.0.0:5353           *:*                                    1000
"#;

        let rows = parse_netstat_output(input, &names, &paths);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].pid, 1000);
    }
}
