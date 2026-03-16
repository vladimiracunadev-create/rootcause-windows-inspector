# Arquitectura

Este documento explica la estructura técnica del proyecto y cómo se relacionan sus capas.

---

## 1) Objetivo arquitectónico

La arquitectura busca equilibrar cuatro cosas al mismo tiempo:

1. **bajo consumo**,
2. **claridad de código**,
3. **capacidad de diagnóstico real**,
4. **ruta profesional a mayor precisión**.

No se eligió una base web pesada ni Electron porque el propio monitor no debe convertirse en una fuente adicional de consumo.

---

## 2) Stack principal

### Lenguaje
- Rust

### GUI
- `eframe/egui`

### Persistencia
- SQLite vía `rusqlite`

### Métricas locales
- `sysinfo`

### Integración Windows
- PowerShell
- `netstat`
- `taskkill`
- `wpr`
- `wpa`
- `tracerpt`

---

## 3) Capas del código

```text
src/
├── main.rs
├── app.rs
├── models.rs
└── services/
    ├── mod.rs
    ├── inspector.rs
    ├── network.rs
    ├── persistence.rs
    ├── temp_scan.rs
    ├── etl.rs
    └── windows.rs
```

---

## 4) Responsabilidad por archivo

### `main.rs`
Punto de entrada. Crea la ventana principal y registra la app.

### `app.rs`
Capa de interfaz.

Responsabilidades:
- layout general,
- acciones del usuario,
- semáforo,
- tablas principales,
- control del modo de precisión,
- vista de resumen ETL.

### `models.rs`
Modelos serializables.

Responsabilidades:
- snapshot de sistema,
- procesos,
- conexiones,
- temporales,
- servicios,
- estado de precisión,
- resumen ETL.

### `services/inspector.rs`
Orquestador principal.

Responsabilidades:
- refrescar métricas,
- calcular deltas,
- ensamblar el snapshot completo,
- exponer acciones de UI,
- coordinar ETL + resumen.

### `services/network.rs`
Parsea `netstat` y clasifica conexiones.

### `services/temp_scan.rs`
Escanea rutas temporales y cachés relevantes.

### `services/windows.rs`
Adaptador de utilidades nativas de Windows.

Responsabilidades:
- PowerShell,
- `netstat`,
- `taskkill`,
- firewall,
- servicios,
- WPR,
- `tracerpt`.

### `services/etl.rs`
Resumen asistido del ETL.

Responsabilidades:
- analizar `dumpfile.xml`,
- extraer rutas, imágenes, IPs e indicadores,
- generar `trace-analysis.json`,
- producir un resumen consumible por la UI.

### `services/persistence.rs`
Persistencia local SQLite.

---

## 5) Flujo de datos

### Observación liviana
1. `app.rs` pide refresh,
2. `inspector.rs` consulta procesos, red, temporales, eventos y servicios,
3. arma `SystemSnapshot`,
4. persiste un resumen en SQLite,
5. devuelve datos a la UI.

### Modo de precisión
1. `app.rs` dispara captura WPR,
2. `windows.rs` ejecuta `wpr`,
3. se guarda `.etl`,
4. `app.rs` dispara resumen ETL,
5. `windows.rs` ejecuta `tracerpt`,
6. `etl.rs` analiza `dumpfile.xml`,
7. se genera `trace-analysis.json`,
8. `inspector.rs` lo reincorpora al snapshot y la UI lo muestra.

---

## 6) Decisiones de diseño clave

### GUI nativa ligera
Para minimizar consumo y complejidad.

### Integración con herramientas nativas
Para aprovechar lo que Windows ya expone, en vez de introducir dependencias más pesadas desde el inicio.

### Persistencia pequeña
SQLite guarda solo lo útil para comparar tendencias, no un universo de datos crudos.

### ETL como capa progresiva
No se obliga al usuario a entrar a ETW desde el primer minuto. Primero se observa; luego, si hace falta, se sube de nivel.

### Artefactos auditables
El resumen ETL produce archivos intermedios visibles y revisables.

---

## 7) Límites de la arquitectura actual

- no hay servicio de fondo persistente,
- no hay driver,
- no hay parser completo de WPA,
- no hay carga de símbolos dentro de la app,
- no hay correlación temporal fina por intervalos seleccionables dentro de la UI,
- no hay integración directa con ETWAnalyzer u otros motores externos.

---

## 8) Expansiones previstas

### Corto plazo
- mejorar heurísticas ETL,
- agregar más scripts operativos,
- ampliar validaciones de release.

### Mediano plazo
- perfiles WPR más específicos,
- resumen más granular por proveedor/evento,
- correlación con intervalos de tiempo del síntoma.

### Largo plazo
- análisis ETW más profundo,
- mayor soporte a símbolos,
- packaging más corporativo,
- firma digital y pipeline de liberación formal.
