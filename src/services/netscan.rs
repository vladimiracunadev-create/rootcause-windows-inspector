//! Exploración de la red local: equipos cercanos (estilo nmap, honesto).
//!
//! Filosofía del módulo, coherente con el resto de RootCause: nada de captura de
//! paquetes ni escaneo agresivo por defecto. La vista *pasiva* solo lee la tabla
//! de vecinos (ARP/NDP) que el propio Windows ya mantiene —barata y silenciosa—;
//! el barrido *profundo* (opcional, bajo demanda) despierta al segmento con pings
//! para descubrir equipos que aún no estaban en la tabla, y resuelve nombres.
//!
//! Por qué importa para seguridad: muchas amenazas no llegan por Internet sino
//! por el **mismo segmento de red** —un equipo comprometido que escanea a sus
//! vecinos (movimiento lateral), un dispositivo no autorizado que se enchufa, un
//! punto de acceso pirata que suplanta la puerta de enlace—. Comparar los equipos
//! presentes contra una baseline de **red conocida** convierte "apareció alguien
//! nuevo cerca de mí" en una señal accionable, igual que el tab Autostart hace con
//! la persistencia. Es un **indicio con evidencia**, no un veredicto.
//!
//! La parte de aquí es pura y testeable: recibe el JSON que produce
//! `windows::network_scan_raw` y devuelve el modelo ya clasificado. La ejecución
//! real (PowerShell / Get-NetNeighbor / ping) vive en `windows.rs`, y el cruce
//! contra la baseline en `inspector.rs`.

use crate::models::{
    AnomalyEvent, IncidentEvidence, NetworkDevice, NetworkScan, PersistenceChange, RiskLevel,
    Severity, WatchedItem,
};
use chrono::{DateTime, Utc};
use serde_json::Value;

/// Separador interno para empaquetar `ip␟vendor␟hostname` en `WatchedItem.detail`
/// (unidad ASCII 0x1F: no aparece en IPs, nombres de fabricante ni hostnames).
const PACK_SEP: char = '\u{1f}';

/// Construye el modelo de red a partir del JSON del recolector nativo.
///
/// El JSON esperado (una sola vuelta de PowerShell) tiene la forma:
/// `{ adapter, localIp, localMac, prefix, gateway, devices:[{ip,mac,state,host}] }`.
/// Ante JSON vacío o inválido devuelve un escaneo vacío con una limitación anotada
/// (nunca entra en pánico: la app debe seguir viva aunque la red no se pueda leer).
pub fn scan_from_json(raw: &str, deep: bool, scanned_at: &str) -> NetworkScan {
    let mut scan = NetworkScan {
        scanned_at: scanned_at.to_owned(),
        deep,
        ..Default::default()
    };

    let value: Value = match serde_json::from_str(raw.trim()) {
        Ok(value) if !raw.trim().is_empty() => value,
        _ => {
            scan.limitations.push(
                "No se pudo leer la red local (¿sin adaptador con puerta de enlace, o \
                 Get-NetNeighbor no disponible?)."
                    .to_owned(),
            );
            return scan;
        }
    };

    scan.adapter_name = string_field(&value, "adapter");
    scan.local_ip = string_field(&value, "localIp");
    scan.local_mac = normalize_mac(&string_field(&value, "localMac"));
    scan.subnet_prefix = string_field(&value, "prefix");
    scan.gateway_ip = string_field(&value, "gateway");

    let interface = scan.adapter_name.clone();
    let gateway = scan.gateway_ip.clone();
    let local_ip = scan.local_ip.clone();
    let local_mac = scan.local_mac.clone();

    if local_ip.is_empty() {
        scan.limitations.push(
            "Sin adaptador activo con puerta de enlace por defecto (equipo sin red o solo con \
             adaptadores virtuales)."
                .to_owned(),
        );
    }

    // El propio equipo casi nunca aparece en la tabla de vecinos: lo añadimos de
    // forma sintética para dar contexto ("este eres tú").
    if !local_ip.is_empty() {
        let mut me = NetworkDevice {
            ip: local_ip.clone(),
            mac: local_mac.clone(),
            hostname: String::new(),
            vendor: vendor_from_mac(&local_mac).to_owned(),
            state: "Local".to_owned(),
            interface: interface.clone(),
            is_self: true,
            ..Default::default()
        };
        classify_device(&mut me);
        scan.devices.push(me);
    }

    if let Some(list) = value.get("devices").and_then(Value::as_array) {
        for entry in list {
            let ip = string_field(entry, "ip");
            if ip.is_empty() || ip == local_ip {
                continue;
            }
            let mac = normalize_mac(&string_field(entry, "mac"));
            let hostname = string_field(entry, "host");
            let mut device = NetworkDevice {
                is_gateway: !gateway.is_empty() && ip == gateway,
                is_self: false,
                vendor: vendor_from_mac(&mac).to_owned(),
                state: string_field(entry, "state"),
                interface: interface.clone(),
                hostname,
                mac,
                ip,
                ..Default::default()
            };
            classify_device(&mut device);
            scan.devices.push(device);
        }
    }

    // Orden estable y legible: primero el propio equipo, luego el gateway, luego
    // por severidad (los nuevos suben) y finalmente por IP.
    scan.devices.sort_by(|a, b| {
        b.is_self
            .cmp(&a.is_self)
            .then_with(|| b.is_gateway.cmp(&a.is_gateway))
            .then_with(|| b.severity.cmp(&a.severity))
            .then_with(|| ip_sort_key(&a.ip).cmp(&ip_sort_key(&b.ip)))
    });

    scan.total_devices = scan.devices.iter().filter(|d| !d.is_self).count();
    scan.new_devices = scan
        .devices
        .iter()
        .filter(|d| d.change_status == PersistenceChange::Added)
        .count();
    scan
}

/// Asigna severidad y motivo humano según el rol del dispositivo y su estado de
/// cambio respecto a la baseline. Se llama tanto al parsear (cambio = Unchanged)
/// como después del cruce con la baseline en el inspector.
pub fn classify_device(device: &mut NetworkDevice) {
    let (severity, reason) = if device.is_self {
        (Severity::Healthy, "Este equipo (tú).")
    } else if device.change_status == PersistenceChange::Added {
        if device.is_gateway {
            (
                Severity::Critical,
                "La puerta de enlace cambió de identidad (MAC nueva): posible suplantación \
                 de router o punto de acceso pirata. Verifícalo.",
            )
        } else {
            (
                Severity::Warning,
                "Dispositivo NUEVO en tu red respecto a la baseline conocida: confirma que lo \
                 reconoces; si no, aíslalo y revisa el punto de acceso.",
            )
        }
    } else if device.change_status == PersistenceChange::Removed {
        (
            Severity::Healthy,
            "Estaba en la red conocida y ya no responde (equipo apagado o desconectado).",
        )
    } else if device.is_gateway {
        (Severity::Healthy, "Puerta de enlace (router) del segmento.")
    } else {
        (Severity::Healthy, "Dispositivo conocido de la red local.")
    };
    device.severity = severity;
    device.reason = reason.to_owned();
}

/// Clave estable de un dispositivo a lo largo del tiempo: su MAC normalizada.
/// Si no hay MAC (raro), cae a la IP para no perder la fila.
pub fn device_key(device: &NetworkDevice) -> String {
    if device.mac.is_empty() {
        format!("ip:{}", device.ip)
    } else {
        device.mac.clone()
    }
}

/// Construye los ítems vigilados (uno por dispositivo, excepto el propio equipo)
/// para el motor genérico de baseline. `value` es constante a propósito: un cambio
/// de IP por DHCP es normal y NO debe marcarse como cambio; lo que importa es que
/// aparezca (o desaparezca) una MAC. El `detail` empaqueta ip/vendor/hostname para
/// poder reconstruir un dispositivo "eliminado" sin volver a escanear.
pub fn device_watch_items(devices: &[NetworkDevice]) -> Vec<WatchedItem> {
    devices
        .iter()
        .filter(|device| !device.is_self && device.change_status != PersistenceChange::Removed)
        .map(|device| WatchedItem {
            key: device_key(device),
            value: String::new(),
            label: display_name(device),
            detail: format!(
                "{}{sep}{}{sep}{}",
                device.ip,
                device.vendor,
                device.hostname,
                sep = PACK_SEP
            ),
            ..Default::default()
        })
        .collect()
}

/// Reconstruye un dispositivo "eliminado" a partir del `WatchedItem` guardado en
/// la baseline (para mostrar en la tabla que un equipo conocido dejó de responder).
pub fn device_from_watch_item(item: &WatchedItem) -> NetworkDevice {
    let mut parts = item.detail.split(PACK_SEP);
    let ip = parts.next().unwrap_or("").to_owned();
    let vendor = parts.next().unwrap_or("").to_owned();
    let hostname = parts.next().unwrap_or("").to_owned();
    let mac = if item.key.starts_with("ip:") {
        String::new()
    } else {
        item.key.clone()
    };
    let mut device = NetworkDevice {
        ip,
        mac,
        hostname,
        vendor,
        state: "Ausente".to_owned(),
        change_status: PersistenceChange::Removed,
        ..Default::default()
    };
    classify_device(&mut device);
    device
}

/// Evento de anomalía para un dispositivo nuevo/desconocido en el segmento. No
/// depende de "sospecha de malware": reporta el hecho para dar control explícito,
/// igual que los cambios de autoarranque o de servicios. Solo para altas (Added).
pub fn new_device_event(
    detected_at: DateTime<Utc>,
    device: &NetworkDevice,
) -> Option<AnomalyEvent> {
    if device.change_status != PersistenceChange::Added || device.is_self {
        return None;
    }
    let (severity, score) = if device.is_gateway {
        (RiskLevel::High, 74_u16)
    } else {
        (RiskLevel::Medium, 55)
    };
    let name = display_name(device);
    let vendor = if device.vendor.is_empty() {
        "fabricante desconocido".to_owned()
    } else {
        device.vendor.clone()
    };

    Some(AnomalyEvent {
        event_id: format!(
            "anom-{}-netdev-{}",
            detected_at.timestamp_millis(),
            device_key(device)
        ),
        detected_at,
        severity,
        score,
        status: "open".to_owned(),
        kind: "unknown-device".to_owned(),
        title: if device.is_gateway {
            "Puerta de enlace suplantada (MAC nueva)".to_owned()
        } else {
            "Dispositivo desconocido en la red local".to_owned()
        },
        summary: format!(
            "Apareció '{name}' ({ip}, {mac}, {vendor}) en tu segmento; no estaba en la baseline \
             de red conocida.",
            ip = device.ip,
            mac = device.mac,
        ),
        root_cause_hypothesis: if device.is_gateway {
            "posible suplantación de la puerta de enlace (ARP spoofing / rogue AP)".to_owned()
        } else {
            "un equipo no reconocido se unió al mismo segmento de red".to_owned()
        },
        recommended_action: if device.is_gateway {
            "Verifica la MAC real de tu router; si no coincide, desconéctate de esa red y revisa \
             posibles puntos de acceso piratas."
                .to_owned()
        } else {
            "Confirma que el equipo te pertenece. Si no lo reconoces, aíslalo del segmento y revisa \
             quién tiene acceso al punto de red / Wi-Fi."
                .to_owned()
        },
        exe_path: None,
        evidence: vec![
            IncidentEvidence {
                kind: "ip".to_owned(),
                label: "IP".to_owned(),
                value: device.ip.clone(),
            },
            IncidentEvidence {
                kind: "mac".to_owned(),
                label: "MAC".to_owned(),
                value: device.mac.clone(),
            },
            IncidentEvidence {
                kind: "vendor".to_owned(),
                label: "Fabricante".to_owned(),
                value: vendor,
            },
            IncidentEvidence {
                kind: "host".to_owned(),
                label: "Nombre".to_owned(),
                value: device.hostname.clone(),
            },
        ],
        ..Default::default()
    })
}

/// Nombre a mostrar: hostname si se resolvió, si no la IP.
pub fn display_name(device: &NetworkDevice) -> String {
    if device.hostname.is_empty() || device.hostname == "Este equipo" {
        if device.is_self {
            "Este equipo".to_owned()
        } else {
            device.ip.clone()
        }
    } else {
        device.hostname.clone()
    }
}

/// Normaliza una MAC a mayúsculas con separador `-` (ej. `aa:bb:cc` → `AA-BB-CC`).
pub fn normalize_mac(mac: &str) -> String {
    mac.trim().replace(':', "-").to_ascii_uppercase()
}

/// Fabricante aproximado a partir del prefijo OUI (primeros 3 octetos) de la MAC.
/// Lista curada y pequeña de prefijos frecuentes en equipos domésticos/oficina;
/// es orientativa (no una base OUI completa). Devuelve "" si no se reconoce.
pub fn vendor_from_mac(mac: &str) -> &'static str {
    let norm = normalize_mac(mac);
    let prefix: String = norm.chars().take(8).collect(); // "AA-BB-CC"
    match prefix.as_str() {
        "00-1A-11" | "3C-5A-B4" | "F4-F5-E8" | "DA-A1-19" => "Google",
        "F0-9F-C2" | "B4-FB-E4" | "68-D7-9A" | "24-A4-3C" | "44-D9-E7" | "80-2A-A8" => "Ubiquiti",
        "00-05-69" | "00-0C-29" | "00-50-56" | "00-1C-14" => "VMware",
        "08-00-27" | "0A-00-27" => "VirtualBox",
        "00-15-5D" => "Microsoft Hyper-V",
        "00-1B-63" | "3C-07-54" | "A4-83-E7" | "F0-18-98" | "D0-81-7A" | "AC-BC-32" => "Apple",
        "FC-FB-FB" | "50-EB-F6" | "B8-27-EB" | "DC-A6-32" | "E4-5F-01" => "Raspberry Pi",
        "00-1D-D8" | "00-50-F2" | "60-45-BD" | "C8-3A-35" => "Microsoft",
        "50-C7-BF" | "AC-84-C6" | "10-BE-F5" | "34-60-F9" => "TP-Link",
        "00-18-4D" | "C0-56-27" | "20-E5-2A" => "NETGEAR",
        "2C-3A-E8" | "A0-20-A6" | "CC-50-E3" | "24-62-AB" => "Espressif (IoT)",
        "00-16-6C" | "78-BD-BC" | "8C-77-12" | "50-85-69" => "Samsung",
        _ => "",
    }
}

/// Devuelve una clave ordenable a partir de una IPv4 (para ordenar por dirección).
fn ip_sort_key(ip: &str) -> u32 {
    let mut key = 0_u32;
    for part in ip.split('.') {
        key = key.wrapping_shl(8) | (part.parse::<u32>().unwrap_or(0) & 0xFF);
    }
    key
}

fn string_field(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "adapter": "Wi-Fi",
        "localIp": "192.168.1.15",
        "localMac": "aa:bb:cc:dd:ee:ff",
        "prefix": "192.168.1",
        "gateway": "192.168.1.1",
        "devices": [
            {"ip":"192.168.1.1","mac":"11-22-33-44-55-66","state":"Reachable","host":"router.local"},
            {"ip":"192.168.1.42","mac":"B8-27-EB-00-11-22","state":"Stale","host":""}
        ]
    }"#;

    #[test]
    fn parsea_y_marca_self_y_gateway() {
        let scan = scan_from_json(SAMPLE, false, "2026-07-15T00:00:00Z");
        assert_eq!(scan.local_ip, "192.168.1.15");
        assert_eq!(scan.subnet_prefix, "192.168.1");
        // self + gateway + 1 dispositivo = 3 filas; self no cuenta en total.
        assert_eq!(scan.devices.len(), 3);
        assert_eq!(scan.total_devices, 2);
        assert!(scan.devices.iter().any(|d| d.is_self));
        assert!(
            scan.devices
                .iter()
                .any(|d| d.is_gateway && d.ip == "192.168.1.1")
        );
        // El B8-27-EB es Raspberry Pi por OUI.
        assert!(
            scan.devices
                .iter()
                .any(|d| d.ip == "192.168.1.42" && d.vendor == "Raspberry Pi")
        );
    }

    #[test]
    fn json_invalido_no_paniquea() {
        let scan = scan_from_json("", false, "t");
        assert!(scan.devices.is_empty());
        assert!(!scan.limitations.is_empty());
    }

    #[test]
    fn normaliza_mac() {
        assert_eq!(normalize_mac("aa:bb:cc:dd:ee:ff"), "AA-BB-CC-DD-EE-FF");
        assert_eq!(normalize_mac(" 11-22-33 "), "11-22-33");
    }

    #[test]
    fn dispositivo_nuevo_genera_evento_y_removido_no() {
        let mut nuevo = NetworkDevice {
            ip: "192.168.1.77".to_owned(),
            mac: "DE-AD-BE-EF-00-01".to_owned(),
            change_status: PersistenceChange::Added,
            ..Default::default()
        };
        classify_device(&mut nuevo);
        assert_eq!(nuevo.severity, Severity::Warning);
        let now = Utc::now();
        assert!(new_device_event(now, &nuevo).is_some());

        let mut ido = NetworkDevice {
            ip: "192.168.1.78".to_owned(),
            change_status: PersistenceChange::Removed,
            ..Default::default()
        };
        classify_device(&mut ido);
        assert!(new_device_event(now, &ido).is_none());
    }

    #[test]
    fn watch_item_roundtrip_de_removido() {
        let scan = scan_from_json(SAMPLE, false, "t");
        let items = device_watch_items(&scan.devices);
        // self excluido; quedan gateway + raspberry.
        assert_eq!(items.len(), 2);
        let reconstruido = device_from_watch_item(&items[0]);
        assert_eq!(reconstruido.change_status, PersistenceChange::Removed);
        assert!(!reconstruido.ip.is_empty());
    }
}
