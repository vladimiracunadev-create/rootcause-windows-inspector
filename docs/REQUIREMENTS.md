# Requisitos

Este documento consolida los requisitos mínimos, recomendados y operativos del proyecto.

---

## 1) Requisitos mínimos de ejecución

### Sistema operativo
- Windows 10 x64
- o Windows 11 x64

### CPU
- 2 núcleos físicos o equivalente

### RAM
- 8 GB

### Disco
- SSD recomendado incluso para el mínimo práctico
- al menos 2 GB libres para operar con comodidad

### Pantalla
- 1366x768 como mínimo funcional

### Permisos
- usuario estándar para observación básica
- administrador cuando quieras bloquear IP, detener servicios o ejecutar ciertas capturas ETW

---

## 2) Requisitos recomendados de ejecución

### Sistema operativo
- Windows 11 x64 actualizado

### CPU
- 4 núcleos o más

### RAM
- 16 GB o más

### Disco
- SSD NVMe o SSD SATA
- 10 GB libres si usarás capturas ETL y empaquetado local

### Pantalla
- 1920x1080 o mayor

---

## 3) Requisitos máximos razonables de caso de uso

No hay un “máximo oficial” del software, pero para operación profesional cómoda se recomienda:

- 32 GB RAM si harás análisis paralelo,
- 20 GB o más libres si tomarás múltiples ETL,
- CPU moderna de escritorio si además compilarás y empaquetarás en el mismo equipo.

---

## 4) Requisitos de build

### Obligatorios
- Rust estable
- Cargo
- Rustup
- toolchain MSVC funcional

### Fuertemente recomendados
- Visual Studio Build Tools / Visual Studio con C++
- PowerShell
- Git

### Para revisión de calidad
- `cargo fmt`
- `cargo clippy`
- `cargo test`

---

## 5) Requisitos para modo de precisión

### Obligatorio para activar ETW desde scripts/UI
- `wpr.exe` instalado y accesible en PATH

### Recomendado para análisis profundo posterior
- `wpa.exe` instalado

### Recomendado para resumen ETL automatizado
- `tracerpt.exe` disponible

### Espacio sugerido
- 5 GB libres adicionales si piensas capturar trazas medianas o repetidas

---

## 6) Requisitos para empaquetado

### Portable ZIP
- release compilado

### CLI-only ZIP
- release CLI-only compilado
- `cargo build --release --no-default-features --target-dir target/cli`

### Inno Setup
- release compilado
- Inno Setup instalado
- `ISCC.exe` disponible

### PowerShell module
- `rootcause.exe` disponible si se va a usar el módulo como integración

### VS Code Extension
- Node.js
- `npm`
- `rootcause.exe` disponible en PATH o configurable en `rootcause.executablePath`

### Hashing
- PowerShell o utilitario equivalente para SHA-256

---

## 7) Requisitos de red

No requiere conectividad permanente para operar en modo base.

Solo analiza:

- conexiones ya existentes,
- información local del sistema,
- herramientas nativas.

No depende de nube para funcionar.

---

## 8) Requisitos de almacenamiento interno del software

### SQLite
El historial local se guarda en la carpeta de datos del usuario.

### JSON
Los snapshots exportados van a Descargas o Documentos.

### ETL
Las trazas WPR se guardan en la carpeta de trazas del proyecto / aplicación.

### Análisis ETL
Los artefactos auxiliares (`dumpfile.xml`, `summary.txt`, `trace-analysis.json`) se guardan bajo `traces\analysis`.

---

## 9) Requisitos mínimos del usuario objetivo

### Usuario final técnico
- quiere entender por qué el PC se pone lento,
- tolera lectura básica de procesos / servicios,
- necesita una UI clara.

### Usuario avanzado
- exportará JSON,
- usará WPR/WPA,
- comparará capturas,
- tomará decisiones con más contexto.

### Mantenedor / desarrollador
- compila desde fuente,
- empaqueta,
- itera documentación,
- valida releases.

---

## 10) Límites operativos importantes

### La UI liviana no es un parser ETL completo
Sirve para orientar y priorizar, no para reemplazar WPA.

### El resumen ETL integrado no equivale a una sesión completa de WPA
Su objetivo es reducir el tiempo al primer hallazgo útil, no sustituir pivotes, símbolos, stacks o regiones de tiempo complejas.

### El escaneo TEMP es deliberadamente acotado
No busca indexar todo el disco.

### `netstat` no equivale a inspección de red forense profunda
Es útil, pero no reemplaza herramientas más especializadas.

### Detener servicios o procesos no es una solución universal
Se usa como mitigación puntual y validación de causa, no como receta automática.

---

## 11) Huella esperada del software

El objetivo del proyecto es mantener una huella razonable para que el propio monitor no empeore el problema.

Aun así, la huella real dependerá de:

- cantidad de procesos activos en el sistema,
- tamaño de carpetas temporales vigiladas,
- frecuencia de refresco,
- presencia o no de captura WPR,
- presencia o no de análisis ETL.

### Recomendación práctica
- usa refresco entre 4 y 8 segundos,
- activa WPR solo cuando el síntoma de verdad lo justifique,
- resume ETL solo sobre capturas cerradas,
- evita dejar acumuladas trazas muy grandes.


## 12) Requisitos para CI/CD

### Runner GitHub Actions recomendado
- `windows-latest`

### Componentes Rust requeridos en CI
- `rustfmt`
- `clippy`

### Empaquetado automático
- Inno Setup instalable en el runner
- espacio suficiente para `target/` y `build/`

### Recomendación
- fijar `Cargo.lock` después del primer build exitoso local
- no hacer release automático si los quality gates no pasaron
