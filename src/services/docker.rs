//! Inspección de espacio ocupado por Docker (imágenes, volúmenes, cachés).
//!
//! Docker suele ser uno de los mayores consumidores silenciosos de disco en una
//! máquina de desarrollo: capas de imágenes viejas, cachés de build y volúmenes
//! huérfanos se acumulan sin que aparezcan en las carpetas temporales clásicas.
//! Este módulo se apoya en el propio CLI de Docker (`docker system df`,
//! `docker images`, `docker volume ls`) para no añadir dependencias ni hablar el
//! protocolo del daemon a mano.
//!
//! Filosofía de acciones (coherente con el resto de RootCause): la app solo
//! purga lo verdaderamente seguro —imágenes *dangling* (sin etiqueta) y la caché
//! de build—, ambas regenerables. Los volúmenes contienen datos persistentes de
//! contenedores, así que se **listan para revisión manual** pero nunca se borran
//! automáticamente.

use std::process::Command;

/// Bandera Win32 `CREATE_NO_WINDOW`: evita que cada invocación de `docker` haga
/// parpadear una ventana de consola cuando se lanza desde la GUI.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Categoría del resumen de `docker system df` (Images, Containers, Local
/// Volumes, Build Cache).
#[derive(Debug, Clone)]
pub struct DockerCategory {
    pub kind: String,
    pub total: u32,
    pub active: u32,
    pub size_mb: f64,
    pub reclaimable_mb: f64,
}

/// Una imagen local.
#[derive(Debug, Clone)]
pub struct DockerImage {
    pub repository: String,
    pub tag: String,
    pub id: String,
    pub size_mb: f64,
    pub created: String,
    /// `true` si es una imagen colgante (`<none>:<none>`), candidata segura a purga.
    pub dangling: bool,
}

/// Un volumen local (los tamaños individuales requieren `system df -v`, costoso;
/// se muestra el nombre y el driver, y el total recuperable sale del resumen).
#[derive(Debug, Clone)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
}

/// Resultado de un escaneo de Docker.
#[derive(Debug, Clone)]
pub struct DockerScan {
    /// `true` si Docker está instalado y el daemon respondió.
    pub available: bool,
    /// Mensaje de estado o error legible (p. ej. "Docker Desktop no está en ejecución").
    pub message: Option<String>,
    pub categories: Vec<DockerCategory>,
    pub images: Vec<DockerImage>,
    pub volumes: Vec<DockerVolume>,
}

impl DockerScan {
    fn unavailable(message: impl Into<String>) -> Self {
        Self {
            available: false,
            message: Some(message.into()),
            categories: Vec::new(),
            images: Vec::new(),
            volumes: Vec::new(),
        }
    }

    /// Espacio total recuperable (suma de todas las categorías), en MB.
    pub fn total_reclaimable_mb(&self) -> f64 {
        self.categories.iter().map(|c| c.reclaimable_mb).sum()
    }

    /// Espacio total ocupado por Docker (suma de tamaños de todas las categorías), en MB.
    pub fn total_size_mb(&self) -> f64 {
        self.categories.iter().map(|c| c.size_mb).sum()
    }

    /// Número de imágenes colgantes (candidatas a purga segura).
    pub fn dangling_count(&self) -> usize {
        self.images.iter().filter(|i| i.dangling).count()
    }
}

/// Resultado interno de ejecutar un comando `docker`.
enum Run {
    /// El binario `docker` no existe en el PATH.
    Missing,
    /// El comando corrió pero devolvió error (daemon caído, permisos, etc.).
    Err(String),
    /// Salida estándar del comando.
    Ok(String),
}

fn run_docker(args: &[&str]) -> Run {
    let mut cmd = Command::new("docker");
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    match cmd.output() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Run::Missing,
        Err(e) => Run::Err(e.to_string()),
        Ok(out) => {
            if out.status.success() {
                Run::Ok(String::from_utf8_lossy(&out.stdout).into_owned())
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
                Run::Err(if stderr.is_empty() {
                    "docker devolvió un código de error".to_owned()
                } else {
                    stderr
                })
            }
        }
    }
}

/// Escanea el uso de disco de Docker. Nunca falla: si Docker no está disponible,
/// devuelve un [`DockerScan`] con `available = false` y un mensaje explicativo.
pub fn scan() -> DockerScan {
    let df = match run_docker(&["system", "df", "--format", "{{json .}}"]) {
        Run::Missing => {
            return DockerScan::unavailable(
                "Docker no está instalado o no está en el PATH. Instálalo para gestionar su espacio.",
            );
        }
        Run::Err(e) => {
            return DockerScan::unavailable(format!(
                "Docker no responde ({e}). ¿Está Docker Desktop en ejecución?"
            ));
        }
        Run::Ok(out) => out,
    };

    let categories = parse_categories(&df);
    let images = match run_docker(&["images", "--format", "{{json .}}"]) {
        Run::Ok(out) => parse_images(&out),
        _ => Vec::new(),
    };
    let volumes = match run_docker(&["volume", "ls", "--format", "{{json .}}"]) {
        Run::Ok(out) => parse_volumes(&out),
        _ => Vec::new(),
    };

    DockerScan {
        available: true,
        message: None,
        categories,
        images,
        volumes,
    }
}

/// Purga las imágenes colgantes (`docker image prune -f`). Seguro: solo elimina
/// capas sin etiqueta que ningún contenedor usa.
pub fn prune_dangling_images() -> Result<String, String> {
    match run_docker(&["image", "prune", "-f"]) {
        Run::Ok(out) => Ok(summarize_prune(&out)),
        Run::Missing => Err("Docker no está instalado.".to_owned()),
        Run::Err(e) => Err(e),
    }
}

/// Purga la caché de build (`docker builder prune -f`). Seguro: la caché se
/// regenera en el siguiente build.
pub fn prune_build_cache() -> Result<String, String> {
    match run_docker(&["builder", "prune", "-f"]) {
        Run::Ok(out) => Ok(summarize_prune(&out)),
        Run::Missing => Err("Docker no está instalado.".to_owned()),
        Run::Err(e) => Err(e),
    }
}

// ── Parsers ────────────────────────────────────────────────────────────────────

fn parse_categories(raw: &str) -> Vec<DockerCategory> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            Some(DockerCategory {
                kind: field(&v, "Type"),
                total: field(&v, "TotalCount").parse().unwrap_or(0),
                active: field(&v, "Active").parse().unwrap_or(0),
                size_mb: parse_size_mb(&field(&v, "Size")),
                reclaimable_mb: parse_size_mb(&field(&v, "Reclaimable")),
            })
        })
        .collect()
}

fn parse_images(raw: &str) -> Vec<DockerImage> {
    let mut images: Vec<DockerImage> = raw
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            let repository = field(&v, "Repository");
            let tag = field(&v, "Tag");
            let dangling = repository == "<none>" || tag == "<none>";
            Some(DockerImage {
                repository,
                tag,
                id: field(&v, "ID"),
                size_mb: parse_size_mb(&field(&v, "Size")),
                created: field(&v, "CreatedSince"),
                dangling,
            })
        })
        .collect();
    // Más grandes primero: es lo que el usuario quiere ver para liberar espacio.
    images.sort_by(|a, b| {
        b.size_mb
            .partial_cmp(&a.size_mb)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    images
}

fn parse_volumes(raw: &str) -> Vec<DockerVolume> {
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let v: serde_json::Value = serde_json::from_str(line).ok()?;
            Some(DockerVolume {
                name: field(&v, "Name"),
                driver: field(&v, "Driver"),
            })
        })
        .collect()
}

/// Lee un campo string de un objeto JSON de docker, devolviendo "" si falta.
fn field(v: &serde_json::Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .trim()
        .to_owned()
}

/// Convierte un tamaño humano de docker ("1.2GB", "142MB", "1.5GB (75%)", "0B")
/// a megabytes. Ignora sufijos entre paréntesis y espacios.
fn parse_size_mb(raw: &str) -> f64 {
    // "1.5GB (75%)" → "1.5GB"
    let head = raw.split(['(', ' ']).next().unwrap_or(raw).trim();
    if head.is_empty() || head.eq_ignore_ascii_case("n/a") {
        return 0.0;
    }
    let split_at = head
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(head.len());
    let (num, unit) = head.split_at(split_at);
    let value: f64 = num.trim().parse().unwrap_or(0.0);
    match unit.trim().to_ascii_lowercase().as_str() {
        "b" | "" => value / (1024.0 * 1024.0),
        "kb" | "k" | "kib" => value / 1024.0,
        "mb" | "m" | "mib" => value,
        "gb" | "g" | "gib" => value * 1024.0,
        "tb" | "t" | "tib" => value * 1024.0 * 1024.0,
        _ => value, // unidad desconocida: asumir MB para no exagerar
    }
}

/// Extrae la línea "Total reclaimed space" de la salida de un `prune`, o devuelve
/// un resumen genérico si no aparece.
fn summarize_prune(raw: &str) -> String {
    for line in raw.lines().rev() {
        let line = line.trim();
        if line.to_ascii_lowercase().contains("reclaimed") {
            return line.to_owned();
        }
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "Purga completada. Nada que liberar.".to_owned()
    } else {
        trimmed
            .lines()
            .last()
            .unwrap_or("Purga completada.")
            .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_size_variantes() {
        assert!((parse_size_mb("142MB") - 142.0).abs() < 0.01);
        assert!((parse_size_mb("1.5GB") - 1536.0).abs() < 0.01);
        assert!((parse_size_mb("1.5GB (75%)") - 1536.0).abs() < 0.01);
        assert_eq!(parse_size_mb("0B"), 0.0);
        assert_eq!(parse_size_mb("N/A"), 0.0);
        assert_eq!(parse_size_mb(""), 0.0);
        assert!((parse_size_mb("512kB") - 0.5).abs() < 0.01);
    }

    #[test]
    fn parse_categories_de_system_df() {
        let raw = r#"{"Active":"2","Reclaimable":"1.2GB (75%)","Size":"1.6GB","TotalCount":"5","Type":"Images"}
{"Active":"0","Reclaimable":"0B","Size":"0B","TotalCount":"0","Type":"Containers"}"#;
        let cats = parse_categories(raw);
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].kind, "Images");
        assert_eq!(cats[0].total, 5);
        assert_eq!(cats[0].active, 2);
        assert!((cats[0].reclaimable_mb - 1228.8).abs() < 1.0);
    }

    #[test]
    fn parse_images_marca_dangling_y_ordena() {
        let raw = r#"{"Repository":"nginx","Tag":"latest","ID":"aaa","Size":"142MB","CreatedSince":"2 weeks ago"}
{"Repository":"<none>","Tag":"<none>","ID":"bbb","Size":"900MB","CreatedSince":"3 days ago"}"#;
        let imgs = parse_images(raw);
        assert_eq!(imgs.len(), 2);
        // Ordenadas por tamaño desc: la de 900MB primero.
        assert_eq!(imgs[0].id, "bbb");
        assert!(imgs[0].dangling);
        assert!(!imgs[1].dangling);
    }
}
