# Análisis, descripción y documentación del repositorio

## 1. Propósito del repositorio

Este repositorio contiene el código fuente, la documentación y los scripts de apoyo para **RootCause**, una aplicación de escritorio para **Windows** escrita en **Rust**.

Su objetivo no es “limpiar el PC” de forma ciega, sino **detectar la causa dominante de la lentitud**, especialmente cuando el usuario percibe síntomas como:

- disco al 100%,
- memoria ocupada de forma anormal,
- crecimiento de archivos temporales,
- actividad de red sospechosa,
- actualizaciones de Windows corriendo en segundo plano,
- procesos que afectan al rendimiento sin explicarlo claramente.

En otras palabras, el repositorio busca resolver un problema real de soporte y observabilidad:

> **explicar con evidencia qué está degradando el equipo, antes de intervenir.**

---

## 2. Qué contiene este repositorio

El repositorio está dividido en cuatro grandes áreas:

### A. Código fuente (`src/`)
Contiene la aplicación en Rust.

### B. Documentación (`docs/`)
Contiene el material operativo, técnico, arquitectónico y de apoyo.

### C. Scripts (`scripts/`)
Contiene automatizaciones para build, verificación, empaquetado y trabajo con ETL/WPR.

### D. Empaquetado (`packaging/`)
Contiene la configuración del instalador de Windows.

---

## 3. Problema que resuelve

Este proyecto intenta responder preguntas muy concretas que normalmente quedan repartidas entre varias herramientas de Windows:

- ¿Qué proceso está escribiendo más en disco?
- ¿Qué carpeta temporal está creciendo?
- ¿Windows Update está detrás del problema?
- ¿Hay conexiones remotas que parecen fuera de lugar?
- ¿Hay un ejecutable lanzado desde una ruta temporal?
- ¿Puedo capturar una evidencia más precisa sin volver el monitor más pesado que el problema?

Muchas herramientas muestran “síntomas”. Este proyecto intenta mostrar **síntoma + posible causa + evidencia + acción controlada**.

---

## 4. Estructura del repositorio

```text
rootcause-windows-inspector/
├── Cargo.toml
├── README.md
├── LICENSE
├── SECURITY.md
├── docs/
│   ├── ARCHITECTURE.md
│   ├── BUILD_WINDOWS.md
│   ├── CI_GITHUB.md
│   ├── COMMANDS.md
│   ├── HEURISTICAS.md
│   ├── OPERACION.md
│   ├── PACKAGING_WINDOWS.md
│   ├── PRECISION_MODE_ETW.md
│   ├── RELEASE_CHECKLIST.md
│   ├── REQUIREMENTS.md
│   ├── ROADMAP.md
│   ├── TESTING_WINDOWS.md
│   ├── TRACE_SUMMARY_ETL.md
│   ├── TROUBLESHOOTING.md
│   └── (documentos nuevos orientados a usuario, novato y reclutador)
├── packaging/
│   └── windows/
│       └── RootCause.iss
├── scripts/
│   ├── build-release.*
│   ├── verify-environment.*
│   ├── quality-gates.ps1
│   ├── package-*.ps1
│   ├── wpr-*.ps1
│   ├── analyze-last-etl.*
│   └── wpa-open-latest.ps1
└── src/
    ├── main.rs
    ├── app.rs
    ├── cli.rs
    ├── meta.rs
    ├── models.rs
    ├── config.rs
    ├── i18n.rs
    └── services/
        ├── ai.rs
        ├── anomaly.rs
        ├── baseline.rs
        ├── docker.rs
        ├── etl.rs
        ├── inspector.rs
        ├── network.rs
        ├── persistence.rs
        ├── resilience.rs
        ├── rules.rs
        ├── temp_scan.rs
        ├── tray.rs
        └── windows.rs
```

---

## 5. Explicación funcional por capas

### 5.1 Capa de presentación
Archivo principal relacionado:

- `src/app.rs`

Responsabilidad:

- dibujar la interfaz gráfica,
- mostrar el semáforo,
- presentar tablas y hallazgos,
- ofrecer acciones como detener captura, analizar ETL o bloquear IP,
- resumir la información para que no se vuelva una pantalla técnica ilegible.

### 5.2 Capa de dominio y modelos
Archivo principal relacionado:

- `src/models.rs`

Responsabilidad:

- definir estructuras compartidas,
- modelar hallazgos,
- mantener consistencia entre servicios y UI,
- servir como contrato interno entre módulos.

### 5.3 Capa de inspección del sistema
Archivos relacionados:

- `src/services/inspector.rs`
- `src/services/temp_scan.rs`
- `src/services/network.rs`
- `src/services/windows.rs`

Responsabilidad:

- observar procesos,
- medir actividad,
- revisar rutas temporales,
- detectar conexiones activas,
- consultar servicios y eventos Windows,
- convertir observaciones técnicas en señales entendibles.

### 5.4 Capa de persistencia
Archivo relacionado:

- `src/services/persistence.rs`

Responsabilidad:

- guardar histórico,
- mantener evidencia,
- soportar exportación y análisis posterior,
- evitar que la información se pierda entre sesiones.

### 5.5 Capa de precisión ETW/WPR/ETL
Archivo relacionado:

- `src/services/etl.rs`

Responsabilidad:

- controlar la captura de trazas,
- registrar el estado del modo de precisión,
- resumir información exportada desde ETL,
- ofrecer una ruta más profunda que la observación liviana.

---

## 6. Flujo de trabajo del producto

### Flujo normal
1. La aplicación toma una muestra.
2. Se consultan procesos, temporales, red y estado de Windows.
3. Se asignan severidades.
4. Se calcula un semáforo general.
5. Se presentan procesos y rutas relevantes.
6. El usuario decide si observar, intervenir o capturar más evidencia.

### Flujo de precisión
1. El usuario detecta un problema intermitente o complejo.
2. Activa el modo de precisión.
3. Se inicia una captura WPR.
4. Se detiene la captura cuando aparece el síntoma.
5. Se genera un ETL.
6. Se exporta información resumible.
7. La app produce un resumen de apoyo.
8. Si aún falta detalle, se abre WPA para análisis profundo.

---

## 7. Fortalezas del repositorio

### A. Buen encaje con un problema real
No es un ejercicio académico aislado. Apunta a una molestia cotidiana de soporte técnico y usuarios avanzados de Windows.

### B. Tecnología bien elegida
Rust es una opción fuerte para una herramienta de escritorio que necesita:

- bajo consumo,
- binario nativo,
- seguridad de memoria,
- mantenibilidad,
- capacidad de crecer sin arrastrar runtimes pesados.

### C. Ruta de precisión profesional
La presencia de WPR/ETL/WPA le da una salida seria cuando el modo liviano no basta.

### D. Buena capacidad de explicación
El proyecto no se enfoca solo en “optimizar”; se enfoca en **explicar por qué**.

### E. Potencial demostrable para portafolio
Sirve tanto como producto técnico real como muestra profesional de arquitectura, observabilidad, UX técnica y automatización de build.

---

## 8. Riesgos y límites actuales

Aunque el repositorio es sólido conceptualmente, tiene límites naturales:

### A. Dependencia de Windows real para validación completa
No se puede garantizar comportamiento real sin compilar y ejecutar en Windows.

### B. Precisión total de archivo/proceso/tiempo
Sin análisis más profundo del ETL o integración más avanzada con proveedores de eventos, algunos casos seguirán requiriendo WPA.

### C. Riesgo de acciones destructivas
Matar procesos, bloquear IP o detener servicios debe manejarse con listas permitidas, advertencias y control fino.

### D. Requiere documentación muy buena
Como mezcla observación liviana con trazas avanzadas, la documentación no es opcional: es parte del producto.

---

## 9. Estado de madurez recomendado

Este repositorio debería entenderse como una base **profesionalmente estructurada** para un producto Windows real, con tres niveles de madurez:

### Nivel 1: demostración técnica seria
- build local,
- uso manual,
- pruebas funcionales básicas,
- empaquetado portable.

### Nivel 2: producto utilizable
- CI estable,
- instalador,
- pruebas manuales más amplias,
- checklist de release,
- validación de consumo de recursos.

### Nivel 3: producto robusto / comercializable
- firma digital,
- telemetría opcional y transparente,
- instalador pulido,
- rollback,
- mejor correlación ETL,
- reportes más ricos,
- pruebas en varias versiones de Windows.

---

## 10. Qué revisar primero si quieres auditarlo

### Si te importa compilación
- `Cargo.toml`
- `.github/workflows/`
- `docs/BUILD_WINDOWS.md`
- `docs/CI_GITHUB.md`

### Si te importa arquitectura
- `docs/ARCHITECTURE.md`
- `docs/ARQUITECTURA_ESCALABILIDAD.md`
- `src/models.rs`
- `src/services/`

### Si te importa operación real
- `docs/OPERACION.md`
- `docs/HEURISTICAS.md`
- `docs/PRECISION_MODE_ETW.md`
- `docs/TRACE_SUMMARY_ETL.md`

### Si te importa producto / portafolio
- `README.md`
- `docs/RECLUTADORES.md`
- `docs/ROADMAP.md`

---

## 11. Conclusión

Este repositorio está bien posicionado para transformarse en un software Windows útil, entendible y profesional, porque combina:

- una necesidad concreta,
- una tecnología adecuada,
- una interfaz sobria,
- una arquitectura escalable,
- una ruta de precisión seria,
- y documentación capaz de hablarle a distintos públicos.

Su valor principal no está en “limpiar archivos”, sino en **detectar, explicar y priorizar la causa del conflicto**.

