# Plan de pruebas en Windows

Este documento existe para cubrir la parte que la CI no puede garantizar por sí sola: funcionalidad observable, permisos, empaquetado y comportamiento básico de la interfaz en un Windows real.

---

## 1) Objetivo

Comprobar, en una máquina Windows real, que el software:

- abre correctamente,
- refresca datos,
- no se bloquea al operar,
- exporta evidencia,
- identifica señales básicas de lentitud,
- ejecuta acciones controladas cuando hay permisos,
- soporta la ruta de build y empaquetado documentada.

---

## 2) Ambientes mínimos a probar

### A. Usuario estándar
- [ ] ejecutar la app
- [ ] refrescar snapshot
- [ ] exportar JSON
- [ ] revisar procesos, temporales, conexiones y eventos

### B. Administrador
- [ ] finalizar un proceso de prueba
- [ ] bloquear una IP de laboratorio
- [ ] detener un servicio permitido de laboratorio
- [ ] iniciar y detener WPR si WPT está instalado

---

## 3) Smoke test mínimo

- [ ] 1. `cargo build --release`
- [ ] 2. abrir `target\release\rootcause.exe`
- [ ] 3. verificar que la ventana aparezca sin crash
- [ ] 4. pulsar `Actualizar ahora`
- [ ] 5. confirmar que cambia la línea de estado
- [ ] 6. pulsar `Exportar JSON`
- [ ] 7. confirmar que aparece el archivo exportado
- [ ] 8. cerrar y reabrir la aplicación
- [ ] 9. confirmar que el historial SQLite no rompe el arranque

---

## 4) Validación funcional

| Área | Acción de prueba | Qué validar |
|---|---|---|
| Procesos dominantes | Abrir una copia pesada de archivos o descompresión | El proceso sube en ranking |
| Temporales | Copiar un archivo grande a `%TEMP%` | Que aparezca reflejado en el escaneo |
| Red | Abrir una descarga o navegador | Que existan conexiones con PID y proceso |
| Eventos | Sistema con warnings/errors recientes | Que la sección de eventos no quede vacía |
| Servicios | Comprobar estado de `BITS`, `DoSvc`, `wuauserv`, `SysMain` | — |

---

## 5) Validación del modo de precisión

### Requisitos
- WPR instalado
- opcionalmente WPA y tracerpt

### Caso básico
- [ ] 1. iniciar captura
- [ ] 2. reproducir síntoma durante 1 a 3 minutos
- [ ] 3. detener captura
- [ ] 4. confirmar generación del `.etl`
- [ ] 5. resumir el ETL
- [ ] 6. validar que exista `trace-analysis.json`
- [ ] 7. abrir el ETL en WPA si corresponde

---

## 6) Validación de empaquetado

### Portable
- [ ] ejecutar `scripts\package-portable.ps1`
- [ ] abrir el ZIP
- [ ] validar que incluya `.exe`, docs y scripts

### Instalador
- [ ] ejecutar `scripts\package-inno.ps1`
- [ ] instalar en máquina de prueba
- [ ] abrir desde acceso directo
- [ ] desinstalar
- [ ] validar que no queden residuos inesperados fuera de la carpeta de datos del usuario

---

## 7) Criterios de salida

Se puede considerar que el repositorio está en estado profesional aceptable cuando:

- [ ] CI en GitHub Actions está en verde,
- [ ] el smoke test en Windows real pasa,
- [ ] el `.exe` abre y exporta JSON,
- [ ] el ZIP portable se genera,
- [ ] el instalador se genera e instala,
- [ ] los límites y requisitos quedaron documentados con honestidad.
