# Documento para reclutadores

## 1. Resumen ejecutivo

**RootCause** es un proyecto de software para **Windows** orientado a detectar y explicar causas de degradación del sistema como consumo excesivo de disco, memoria, crecimiento anormal de temporales, actividad de red sospechosa y procesos conflictivos.

Está construido en **Rust** con una interfaz gráfica ligera y una arquitectura preparada para crecer hacia análisis de mayor precisión sobre trazas del sistema.

No se plantea como un “limpiador mágico”, sino como una herramienta de **diagnóstico, observabilidad y toma de decisiones**.

---

## 2. Qué problema de negocio / usuario resuelve

Muchos usuarios y equipos de soporte observan síntomas vagos:

- “el computador está lento”,
- “Windows se traba”,
- “el disco vive lleno o al 100%”,
- “internet se pone lento sin razón visible”,
- “aparecen procesos raros o instaladores temporales”.

Windows ofrece información repartida en varias herramientas, pero rara vez en una vista única, entendible y accionable.

Este proyecto busca reducir ese problema entregando:

- correlación entre proceso, ruta, red y servicios,
- priorización visual por severidad,
- evidencia exportable,
- y un camino a análisis más profundo cuando el caso lo exige.

---

## 3. Qué demuestra profesionalmente este proyecto

Este proyecto demuestra capacidades en varias áreas al mismo tiempo:

### Ingeniería de software
- separación por módulos,
- documentación profunda,
- diseño orientado a responsabilidades,
- cuidado por mantenibilidad.

### Desarrollo de producto
- foco en problema real,
- interfaz sobria,
- decisiones de UX técnica,
- priorización de legibilidad sobre sobrecarga visual.

### Plataforma Windows
- interacción con procesos,
- servicios,
- eventos,
- red,
- y trazas ETL/WPR.

### Calidad y entrega
- scripts de build,
- guía de instalación,
- empaquetado,
- CI en GitHub Actions,
- checklist de release,
- landing page del producto,
- análisis y hardening de seguridad (command injection, validación de entradas antes de invocar PowerShell).

### CLI y experiencia de usuario
- interfaz de línea de comandos completa (`rootcause --help`),
- atajos de teclado en la GUI,
- tab Acerca con metadatos del producto,
- información de hardware del equipo.

### Escalabilidad técnica
- ruta clara para crecer a análisis más fino,
- modelo de capas,
- preparación para madurez de producto superior.

---

## 4. Tecnologías usadas

### Rust
Lenguaje compilado, nativo y de alto rendimiento, elegido por seguridad de memoria, eficiencia y mantenibilidad.

### egui / eframe
Bibliotecas para la interfaz gráfica de escritorio.

Se usaron para construir una GUI liviana y moderna sin depender de una aplicación web pesada.

### SQLite
Base de datos embebida y ligera para persistencia local.

### PowerShell / herramientas Windows
Se usan para interactuar con capacidades del sistema operativo, especialmente en automatización y análisis complementario.

### WPR / ETL / WPA
Se utilizan como ruta de precisión.

- **WPR**: Windows Performance Recorder. Herramienta para capturar trazas.
- **ETL**: Event Trace Log. Archivo de traza generado.
- **WPA**: Windows Performance Analyzer. Herramienta para análisis profundo de trazas.

### GitHub Actions
Automatización de validaciones y build en CI.

---

## 5. Glosario de siglas, explicado simple

### GUI
**Graphical User Interface**. Interfaz gráfica de usuario.

### CI
**Continuous Integration**. Integración continua. Validación automática del proyecto al subir cambios.

### CD
**Continuous Delivery / Deployment**. Entrega o despliegue continuo.

### YAML / YML
Formato de texto usado para configurar pipelines, por ejemplo en GitHub Actions.

### ETW
**Event Tracing for Windows**. Tecnología de trazas del sistema operativo Windows.

### WPR
**Windows Performance Recorder**. Captura trazas del sistema.

### WPA
**Windows Performance Analyzer**. Analiza esas trazas.

### ETL
Archivo que guarda eventos trazados del sistema.

### PID
**Process Identifier**. Número identificador de un proceso.

### I/O
**Input / Output**. Lectura y escritura de datos, por ejemplo en disco o red.

### JSON
Formato estructurado de texto usado para exportar información.

### SQLite
Motor de base de datos embebido en un solo archivo.

### Inno Setup
Herramienta clásica para crear instaladores `.exe` de Windows.

---

## 6. Qué no intenta ser el proyecto

Es importante para un reclutador entender también lo que el proyecto **no** vende:

- no promete reparación mágica,
- no reemplaza completamente herramientas forenses profundas,
- no intenta ser un antivirus,
- no intenta ser un “task manager bonito” sin criterio,
- no pretende ocultar complejidad con marketing vacío.

Su propuesta es más madura:

> detectar, resumir, priorizar y documentar el problema real.

---

## 7. Arquitectura resumida

El proyecto se organiza en capas:

- **UI**: presenta estado, semáforos y acciones.
- **Servicios**: inspección de procesos, temporales, red, Windows y ETL.
- **Modelos**: contratos de datos.
- **Persistencia**: evidencia histórica y exportación.
- **Automatización**: scripts y workflows.

Esto favorece:

- mantenibilidad,
- extensibilidad,
- pruebas por áreas,
- y evolución a producto más robusto.

---

## 8. Por qué tiene valor para reclutamiento

Un reclutador o líder técnico puede leer este proyecto como evidencia de que su autor puede trabajar en:

- software de escritorio,
- observabilidad,
- soporte técnico avanzado,
- arquitectura modular,
- herramientas Windows,
- documentación seria,
- automatización de entrega,
- y diseño de productos útiles, no solo académicos.

También es un proyecto con narrativa clara: parte de un dolor concreto y propone una solución razonable, escalable y documentada.

---

## 9. Posibilidades de escalamiento

El proyecto está bien posicionado para crecer hacia:

- correlación temporal más fina,
- mejor análisis automático de ETL,
- reglas más inteligentes de severidad,
- reportes enriquecidos,
- firma digital,
- telemetría opcional,
- distribución profesional,
- y compatibilidad validada en más entornos Windows.

La ruta de escalamiento se desarrolla con más detalle en `docs/ARQUITECTURA_ESCALABILIDAD.md`.

---

## 10. En una frase

**RootCause es una herramienta Windows escrita en Rust que busca identificar con claridad la causa dominante de problemas de rendimiento, con una arquitectura ligera, CLI completa, GUI nativa y preparada para escalar.**

---

## 11. Versión actual

**v0.6.0** — CLI completa, atajos de teclado, tab Acerca, características del equipo, sparklines, historial SQLite, comparación A vs B, filtro de severidad, notificaciones toast, correlación process↔cmdline, landing page del producto, hardening de seguridad (validación de IPs y servicios antes de ejecutar scripts PowerShell).

