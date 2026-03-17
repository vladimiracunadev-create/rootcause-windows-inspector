//! Metadatos del producto — versión, autor, contacto.
//!
//! Actualiza `EMAIL` y `GITLAB` antes de distribuir el software.

/// Versión del software (se sincroniza automáticamente con Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Nombre visible en la UI, el CLI y los instaladores.
pub const DISPLAY_NAME: &str = "RootCause Windows Inspector";

/// Descripción breve para el tab Acerca y el `--help` del CLI.
pub const DESCRIPTION: &str = "Monitor forense ligero para Windows. Detecta la causa dominante de lentitud: \
     procesos, temporales, conexiones de red y trazas ETL.";

/// Autor principal.
pub const AUTHOR: &str = "Vladimir Acuña";

/// Correo de contacto.
/// TODO: completar con dirección real antes de distribución pública.
pub const EMAIL: &str = "";

/// URL del repositorio en GitHub.
pub const GITHUB: &str = "https://github.com/vladimiracunadev-create/rootcause-windows-inspector";

/// URL del perfil / repositorio en GitLab.
/// TODO: confirmar o corregir la URL antes de distribución pública.
pub const GITLAB: &str = "https://gitlab.com/vladimiracunadev-create";

/// Licencia del software.
pub const LICENSE: &str = "Apache License 2.0";
