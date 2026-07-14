# 👔 Documento para reclutadores

> Guía rápida para reclutadores y líderes técnicos: qué es RootCause, qué problema resuelve y qué capacidades profesionales demuestra su autor.

## 1. Resumen ejecutivo

**RootCause** es un proyecto de software para **Windows** orientado a detectar y explicar causas de degradación del sistema como consumo excesivo de disco, memoria, crecimiento anormal de temporales, actividad de red sospechosa y procesos conflictivos.

Está construido en **Rust** con una interfaz gráfica ligera y una arquitectura preparada para crecer hacia análisis de mayor precisión sobre trazas del sistema.

> No se plantea como un "limpiador mágico", sino como una herramienta de **diagnóstico, observabilidad y toma de decisiones**.

---

## 2. Qué problema de negocio / usuario resuelve

Muchos usuarios y equipos de soporte observan síntomas vagos:

- "el computador está lento",
- "Windows se traba",
- "el disco vive lleno o al 100%",
- "internet se pone lento sin razón visible",
- "aparecen procesos raros o instaladores temporales".

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
- atajos de teclado en la GUI (`Ctrl+1…9` y `Ctrl+0`),
- tab Acerca con metadatos del producto,
- información de hardware del equipo,
- tab Autostart integrado: registro Run y carpetas Startup con severidad heurística, y **detección de cambios de autoarranque contra una baseline conocida** (persistida en SQLite) que clasifica cada entrada como nueva, modificada o eliminada y levanta alertas `persistence-change`.

> 💡 **Diferenciador técnico clave** frente a herramientas equivalentes gratuitas: la detección de cambios de autoarranque contra una baseline conocida y persistida.

### Escalabilidad técnica
- ruta clara para crecer a análisis más fino,
- modelo de capas,
- preparación para madurez de producto superior.

---

## 4. Tecnologías usadas

| Tecnología | Rol en el proyecto |
|---|---|
| **Rust** | Lenguaje compilado, nativo y de alto rendimiento, elegido por seguridad de memoria, eficiencia y mantenibilidad. |
| **egui / eframe** | Bibliotecas para la interfaz gráfica de escritorio; GUI liviana y moderna sin depender de una aplicación web pesada. |
| **SQLite** | Base de datos embebida y ligera para persistencia local. |
| **PowerShell / herramientas Windows** | Interacción con capacidades del sistema operativo, especialmente en automatización y análisis complementario. |
| **WPR / ETL / WPA** | Ruta de precisión para captura y análisis profundo de trazas (ver detalle abajo). |
| **GitHub Actions** | Automatización de validaciones y build en CI. |

### Ruta de precisión: WPR / ETL / WPA

| Sigla | Nombre | Función |
|---|---|---|
| **WPR** | Windows Performance Recorder | Herramienta para capturar trazas. |
| **ETL** | Event Trace Log | Archivo de traza generado. |
| **WPA** | Windows Performance Analyzer | Herramienta para análisis profundo de trazas. |

---

## 5. Glosario de siglas, explicado simple

| Sigla | Significado | Explicación |
|---|---|---|
| **GUI** | Graphical User Interface | Interfaz gráfica de usuario. |
| **CI** | Continuous Integration | Integración continua. Validación automática del proyecto al subir cambios. |
| **CD** | Continuous Delivery / Deployment | Entrega o despliegue continuo. |
| **YAML / YML** | — | Formato de texto usado para configurar pipelines, por ejemplo en GitHub Actions. |
| **ETW** | Event Tracing for Windows | Tecnología de trazas del sistema operativo Windows. |
| **WPR** | Windows Performance Recorder | Captura trazas del sistema. |
| **WPA** | Windows Performance Analyzer | Analiza esas trazas. |
| **ETL** | Event Trace Log | Archivo que guarda eventos trazados del sistema. |
| **PID** | Process Identifier | Número identificador de un proceso. |
| **I/O** | Input / Output | Lectura y escritura de datos, por ejemplo en disco o red. |
| **JSON** | — | Formato estructurado de texto usado para exportar información. |
| **SQLite** | — | Motor de base de datos embebido en un solo archivo. |
| **Inno Setup** | — | Herramienta clásica para crear instaladores `.exe` de Windows. |

---

## 6. Qué no intenta ser el proyecto

Es importante para un reclutador entender también lo que el proyecto **no** vende:

- no promete reparación mágica,
- no reemplaza completamente herramientas forenses profundas,
- no intenta ser un antivirus,
- no intenta ser un "task manager bonito" sin criterio,
- no pretende ocultar complejidad con marketing vacío.

Su propuesta es más madura:

> detectar, resumir, priorizar y documentar el problema real.

---

## 7. Arquitectura resumida

El proyecto se organiza en capas:

| Capa | Responsabilidad |
|---|---|
| **UI** | presenta estado, semáforos y acciones. |
| **Servicios** | inspección de procesos, temporales, red, Windows y ETL. |
| **Modelos** | contratos de datos. |
| **Persistencia** | evidencia histórica y exportación. |
| **Automatización** | scripts y workflows. |

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

> La ruta de escalamiento se desarrolla con más detalle en `docs/ARQUITECTURA_ESCALABILIDAD.md`.

---

## 10. En una frase

> **RootCause es una herramienta Windows escrita en Rust que busca identificar con claridad la causa dominante de problemas de rendimiento, con una arquitectura ligera, CLI completa, GUI nativa y preparada para escalar.**

---

## 11. Versión actual

**v0.18.0** — sobre todo lo anterior, dos capacidades forenses de cara al usuario: **reportes forenses de actividad** (un módulo que vuelca la captura del sistema a un Markdown fechado —veredicto de salud, incidentes/anomalías, alertas «dónde mirar primero», cambios de autoarranque vs baseline, procesos de mayor riesgo, conexiones salientes a IP pública y temporales—, disponible con un botón en la barra superior, por CLI (`rootcause report`) y **automáticamente al cambiar el día**; son *indicios con evidencia, no veredictos*); y una **optimización segura de un clic** al estilo PC Manager pero honesta (confirmación de 2 pasos; limpia `%TEMP%` solo >24 h y no en uso, y purga lo regenerable de Docker; muestra los MB reales liberados; **no toca RAM, datos ni volúmenes**). El manual interno se amplió para ambas y se aseguró la compatibilidad con Rust 1.97.

**v0.17.0** — un **rediseño completo de la interfaz** con estética **Windows 11 / Fluent**, inspirado en la calidez de PC Manager sin copiarlo y **conservando toda la densidad de datos**: barra lateral (NavigationView) con navegación agrupada e **iconos de línea** dibujados a mano (sin emoji), **tipografía nativa Segoe UI** + Consolas, el **logo del radar** (círculos concéntricos del icono) en el header y la ventana, y **modos de tema Claro / Oscuro / Windows** (este último sigue el tema del sistema) seleccionables en Configuración. Los colores pasan a *tokens* en runtime con el azul del icono (`#1f6feb`) como acento en los tres modos. El proceso se validó fase por fase en CI y con capturas del binario real.

**v0.16.0** — sobre todo lo anterior, un **icono de bandeja del sistema** (System Tray) en la edición GUI: un punto de color que refleja la salud global (verde / ámbar / rojo), tooltip con el veredicto y el score, y un menú contextual con Mostrar ventana, Actualizar, Exportar y Salir. Se crea en el hilo del event-loop y su creación es no fatal (si el SO lo rechaza, la app sigue sin bandeja). Con esto el módulo `tray.rs` deja de ser un skeleton y pasa a ser una función real.

**v0.15.0** — sobre todo lo anterior, e inspirado en la UX de Microsoft PC Manager (sin clonarlo): interfaz bilingüe español / inglés con selector persistente y un nuevo tab **Configuración** (Ctrl+9) que reúne idioma, umbrales, anomalías y refresco; un **banner de veredicto** tipo hero en el Resumen (aro de salud + titular + causa dominante); gestión del espacio de **Docker** en el tab Temporales (imágenes, volúmenes y espacio recuperable vía `docker system df`, con purga guiada segura de 2 pasos que solo borra imágenes *dangling* y caché de build —nunca volúmenes, que contienen datos— y comando `rootcause docker [--json|--prune-images|--prune-cache]`); y un **Manual** reescrito que explica el *porqué* de cada parte. Ahora 11 tabs.

**v0.14.0** — sobre todo lo anterior, un overhaul de UI (ventana dimensionada al área de trabajo real del monitor, barras de scroll sólidas y visibles, barrido de glifos que la fuente incluida no renderizaba, layout del Resumen sin desbordes), nuevo tab Manual (guía integrada; 10 tabs en total), limpieza segura de `%TEMP%` desde la GUI y por CLI (`rootcause clean-temp`), y un fix real de colección: los tabs Servicios y Eventos ya no salen vacíos (PowerShell devolvía exit code ≠ 0 ante errores no-terminantes pese a emitir datos válidos) y la salida se fuerza a UTF-8 (los acentos ya no se corrompen).

**v0.13.0** — ediciones GUI/CLI-only, módulo PowerShell, manifests Scoop/Winget/Chocolatey, extensión VS Code, configuración local segura, historial SQLite con incidentes y auditoría, correlación técnica ligera, alertas nativas, detección heurística de comportamiento anómalo y posible actividad maliciosa (V1), salud del agente con heartbeat local, detección de cierre abrupto previo, integridad básica de configuración, backoff recomendado, tab Autostart con tareas programadas y umbrales editables inline, detección de cambios de autoarranque (baseline SQLite, clasificación NUEVA/MODIFICADA/ELIMINADA, alertas `persistence-change`, aceptación via UI y CLI), motor genérico de baseline reutilizable con su primera superficie nueva —detección de cambios en servicios de Windows (StartMode + ruta del binario, alertas `service-change`, `rootcause services [--json] [--accept]`)— y catálogo de artefactos/publicación alineado entre repo, release y landing.
