# Heurísticas y criterios

Este documento explica cómo el proyecto clasifica señales y qué significa cada alerta.

---

## 1) Principio general

Las heurísticas **no son verdad absoluta**. Son un sistema práctico para:

- priorizar,
- reducir ruido,
- orientar la mirada,
- decidir cuándo escalar a WPR/ETW.

---

## 2) Señales por proceso

### CPU
Sube severidad cuando el proceso domina la ventana observada.

### RAM
Sube severidad cuando el consumo ya es material para el equipo.

### Escritura de disco
Es una de las señales más valiosas del proyecto.

### Ruta del ejecutable
Gana peso si el ejecutable está en:

- `%TEMP%`
- `AppData\Local\Temp`
- rutas transitorias
- staging de instaladores

### Nombre del proceso
Sube contexto si contiene patrones como:

- `update`
- `setup`
- `installer`
- `msiexec`
- `trustedinstaller`
- `dism`

---

## 3) Señales por red

### IP pública
No implica riesgo por sí sola, pero exige validación.

### Ruta temporal + IP pública
Es una combinación más seria.

### Conexión establecida
Tiene mayor peso que un estado irrelevante o residual.

---

## 4) Señales por temporales y cachés

Se vigilan especialmente:

- `%TEMP%`
- `C:\Windows\Temp`
- `SoftwareDistribution`
- `DeliveryOptimization`

Suben de prioridad cuando:

- pesan mucho,
- coinciden con lentitud,
- coinciden con update o instalación,
- muestran crecimiento fuerte.

---

## 5) Señales por servicios

Se observan con intención explicativa:

- `wuauserv`
- `BITS`
- `DoSvc`
- `TrustedInstaller`
- `SysMain`

No se marcan como “malos” por defecto. Se usan para responder:

- ¿Windows está actualizando?
- ¿hay descarga en segundo plano?
- ¿hay precarga o mantenimiento activo?

---

## 6) Cambios de autoarranque contra baseline

Además de juzgar si una entrada de autoarranque "parece sospechosa ahora", el proyecto mantiene una **baseline conocida** de persistencia: registro Run/RunOnce (HKCU/HKLM), carpetas Startup (usuario y global) y tareas programadas no-Microsoft.

### Siembra de la baseline
El primer scan con baseline vacía siembra las entradas observadas en silencio. Esa primera foto no marca cambios: solo establece el punto de referencia.

### Clasificación de cambios
En cada scan posterior se compara la persistencia observada contra la baseline y cada entrada se clasifica:

- **NUEVA** — apareció una entrada que no estaba en la baseline.
- **MODIFICADA** — la entrada existe, pero su comando cambió.
- **ELIMINADA** — la entrada estaba en la baseline y ya no aparece.
- **sin cambios** — coincide con la baseline.

Los cambios son pegajosos: siguen visibles hasta que el usuario los acepta e incorpora a la baseline.

### Severidad diferenciada
La aparición (NUEVA) y la modificación (MODIFICADA) reciben severidad más alta porque son la forma en que un mecanismo no autorizado se instala o muta para persistir. La eliminación (ELIMINADA) recibe severidad menor: quitar una entrada rara vez es un vector de ataque y suele corresponder a una desinstalación o limpieza legítima, aunque sigue siendo un cambio que conviene notar.

---

## 7) Construcción del semáforo general

### Verde
No hay una causa dominante fuerte.

### Amarillo
Hay presión relevante o patrón que merece vigilancia.

### Rojo
Hay causa dominante o correlación fuerte entre señales.

---

## 8) Cuándo escalar a WPR

Escala a WPR cuando:

- hay dudas entre dos sospechosos,
- el síntoma es breve pero muy fuerte,
- necesitas temporalidad más fina,
- necesitas acercarte al archivo o flujo exacto.

---

## 9) Límite deliberado de la heurística

La heurística base no pretende reemplazar:

- análisis ETL,
- reputación de binarios,
- análisis de malware,
- inspección profunda de red,
- observabilidad de kernel.

Su función es orientar y priorizar bien.
