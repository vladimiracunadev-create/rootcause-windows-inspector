# Resumen ETL dentro de la aplicación

Este documento explica qué hace exactamente la función **Resumir último ETL** y cuáles son sus límites reales.

---

## 1) Objetivo

La función busca reducir el tiempo hasta el primer hallazgo útil cuando ya tienes una captura `.etl` cerrada.

No pretende reemplazar WPA. Pretende responder más rápido preguntas como:

- ¿la traza apunta a Windows Update?
- ¿aparecen rutas temporales sospechosas?
- ¿hay imágenes repetidas de instaladores o binarios temporales?
- ¿se observan IP públicas en la exportación?
- ¿qué proveedor o ruta se repite más?

---

## 2) Flujo técnico

1. la app identifica el último ETL conocido,
2. exporta el ETL a `dumpfile.xml` y `summary.txt` con `tracerpt`,
3. recorre el XML exportado,
4. aplica heurísticas propias sobre:
   - rutas,
   - nombres de proceso,
   - IPs,
   - indicadores de Windows Update / Delivery Optimization,
5. genera `trace-analysis.json`,
6. muestra un resumen visual en la interfaz.

---

## 3) Artefactos generados

Por cada ETL analizado se crea una carpeta propia bajo `traces\analysis\<nombre-del-etl>` con:

- `dumpfile.xml`
- `summary.txt`
- `trace-analysis.json`

Esto deja el proceso auditable y revisable sin depender exclusivamente de la UI.

---

## 4) Qué muestra la UI

### Titular de la traza
Un encabezado corto que resume la señal dominante.

### Hallazgos principales
Hasta tres hallazgos priorizados.

### Procesos repetidos
Imágenes observadas muchas veces dentro del ETL exportado.

### Rutas repetidas
Rutas temporales, de sistema o de Windows Update que aparecen con frecuencia.

### Contexto rápido
- IPs públicas
- indicadores resumidos
- límites del análisis

---

## 5) Casos que suelen funcionar bien

- ETL donde la lentitud está muy asociada a servicing o update,
- capturas cortas donde aparece un instalador temporal,
- capturas donde un ejecutable desde `%TEMP%` deja huella evidente,
- trazas donde se repiten rutas de `SoftwareDistribution` o `DeliveryOptimization`.

---

## 6) Casos donde debes ir directo a WPA

- cuando necesitas el intervalo exacto por milisegundo,
- cuando necesitas tablas avanzadas de File I/O,
- cuando quieres call stacks,
- cuando vas a cargar símbolos,
- cuando el ETL es muy grande y la relación entre eventos requiere pivotes complejos.

---

## 7) Limitaciones técnicas

- el XML exportado por `tracerpt` no equivale a toda la potencia de WPA,
- la precisión depende de la calidad de la captura original,
- las heurísticas pueden orientar sin probar causalidad absoluta,
- una IP pública puede ser legítima,
- una ruta de sistema repetida puede ser normal si corresponde a un update real.

---

## 8) Recomendación operativa

Usa el resumen ETL como una **capa intermedia**:

1. capturas,
2. resumes,
3. decides si ya tienes suficiente evidencia,
4. si no, abres WPA.

Ese flujo ahorra tiempo y mantiene una trazabilidad profesional.
