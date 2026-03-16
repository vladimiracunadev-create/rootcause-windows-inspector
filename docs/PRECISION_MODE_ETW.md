# Modo de precisión ETW / WPR

Este documento explica cómo usar el modo de precisión del proyecto y qué esperar realmente de él.

---

## 1) Qué significa “modo de precisión” aquí

En la aplicación, “modo de precisión” significa usar **WPR** para capturar una traza ETW más rica que la observación liviana.

La app no reemplaza a WPA como analizador profundo. Lo que hace es:

- facilitar el arranque de la captura,
- facilitar el guardado del ETL,
- registrar contexto,
- mantener una carpeta de trazas organizada,
- permitir un **primer resumen** del ETL cuando `tracerpt` está disponible,
- dejar clara la ruta de análisis posterior.

---

## 2) Cuándo conviene activarlo

Úsalo cuando la UI liviana no alcance para responder:

- qué archivo exacto se estaba tocando,
- cuál fue la secuencia temporal precisa,
- qué pasó durante una ventana breve de lentitud,
- cómo correlacionar proceso, disco, CPU y red,
- si una actualización, instalador o actividad de fondo coincide realmente con el síntoma.

---

## 3) Qué no debes esperar

No debes esperar que la UI principal reemplace a WPA.

La versión actual:

- **sí** inicia/detiene la captura,
- **sí** conserva el ETL,
- **sí** resume el último ETL con `tracerpt` si está disponible,
- **sí** deja artefactos intermedios auditables,
- **no** implementa análisis interactivo completo de gráficas ETW,
- **no** sustituye pivotes, símbolos o vistas avanzadas de WPA.

---

## 4) Perfil usado por defecto

Se usa como referencia operativa:

```text
GeneralProfile -filemode
```

Motivo:

- es una base razonable,
- es más segura para un primer diagnóstico general,
- evita empezar con perfiles demasiado agresivos.

---

## 5) Ruta desde la interfaz

### Iniciar
Desde la UI, en la sección **Modo de precisión ETW/WPR**:

1. escribe una descripción breve del problema,
2. pulsa **Iniciar captura WPR**,
3. reproduce el síntoma real.

### Detener
Cuando el síntoma ocurra:

1. pulsa **Detener y guardar ETL**,
2. el archivo `.etl` se guardará en la carpeta de trazas,
3. si `tracerpt` está presente, pulsa **Resumir último ETL**.

### Cancelar
Usa **Cancelar captura** si arrancaste la sesión por error o no lograste reproducir el problema.

---

## 6) Ruta por scripts

### Iniciar

```powershell
.\scripts\wpr-start-general.ps1 -ProblemDescription "Disco al 100% mientras Windows Update trabaja"
```

### Detener

```powershell
.\scripts\wpr-stop-general.ps1 -ProblemDescription "Disco al 100% mientras Windows Update trabaja"
```

### Exportar el último ETL a XML + summary

```powershell
.\scripts\analyze-last-etl.ps1
```

### Abrir último ETL

```powershell
.\scripts\wpa-open-latest.ps1
```

---

## 7) Artefactos generados

### Captura cruda
- `.etl`

### Análisis auxiliar
- `dumpfile.xml`
- `summary.txt`
- `trace-analysis.json`

Estos artefactos quedan organizados por nombre base del ETL bajo la carpeta `traces\analysis`.

---

## 8) Buenas prácticas

- captura solo la ventana del problema,
- usa descripciones cortas pero concretas,
- no acumules ETL enormes innecesariamente,
- guarda JSON y ETL juntos cuando el caso sea importante,
- si el problema dura segundos, inicia antes y detén apenas ocurra,
- si el problema no reaparece, no dejes WPR grabando “por si acaso”.

---

## 9) Riesgos y costos operativos

### Tamaño
Las trazas ETL pueden crecer rápido.

### Impacto
Aunque WPR es la ruta correcta para precisión, sigue teniendo costo. No conviene dejarlo corriendo mucho tiempo en un equipo ya comprometido.

### Permisos
En algunos entornos se requerirán privilegios de administrador.

### Resumen ETL
La fase de resumen crea archivos adicionales y, dependiendo del volumen del ETL, puede tardar y consumir disco temporalmente.

---

## 10) Flujo recomendado de análisis

1. observar con la UI liviana,
2. identificar sospechoso principal,
3. activar WPR si persiste duda,
4. reproducir el síntoma,
5. guardar ETL,
6. resumir el ETL desde la app,
7. abrir en WPA si el caso todavía exige mayor precisión,
8. correlacionar con el JSON exportado y con la hora del evento.

---

## 11) Qué revisar en WPA si el resumen local no basta

Según el caso, revisa:

- CPU Usage,
- Disk Usage,
- File I/O,
- procesos dominantes,
- marcas temporales ligadas al síntoma,
- ventanas donde el equipo se degradó,
- símbolos si necesitas stacks o llamadas.

---

## 12) Resultado esperado

El modo de precisión no “arregla” el problema por sí solo. Su objetivo es permitirte responder con mucha más confianza:

- qué estaba pasando,
- quién lo causó,
- en qué momento,
- con qué intensidad,
- y si valía la pena intervenir.
