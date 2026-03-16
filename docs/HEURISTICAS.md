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

## 6) Construcción del semáforo general

### Verde
No hay una causa dominante fuerte.

### Amarillo
Hay presión relevante o patrón que merece vigilancia.

### Rojo
Hay causa dominante o correlación fuerte entre señales.

---

## 7) Cuándo escalar a WPR

Escala a WPR cuando:

- hay dudas entre dos sospechosos,
- el síntoma es breve pero muy fuerte,
- necesitas temporalidad más fina,
- necesitas acercarte al archivo o flujo exacto.

---

## 8) Límite deliberado de la heurística

La heurística base no pretende reemplazar:

- análisis ETL,
- reputación de binarios,
- análisis de malware,
- inspección profunda de red,
- observabilidad de kernel.

Su función es orientar y priorizar bien.
