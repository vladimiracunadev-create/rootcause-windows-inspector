# Troubleshooting

Este documento agrupa los problemas más frecuentes de build, ejecución, captura y empaquetado.

---

## 1) `cargo` no se reconoce

### Síntoma
```text
'cargo' no se reconoce como un comando interno o externo
```

### Causa probable
Rustup no está instalado o la terminal no fue reiniciada.

### Acción
- instala Rustup,
- cierra y abre la terminal,
- verifica con `cargo --version`.

---

## 2) Error de linker MSVC

### Síntoma típico
mensajes ligados a `link.exe`, `cl.exe` o toolchain MSVC.

### Acción
- instala Build Tools / Visual Studio,
- incluye **Desktop development with C++**,
- prueba desde Developer PowerShell for VS.

---

## 3) PowerShell bloquea scripts

### Acción temporal

```powershell
Set-ExecutionPolicy -Scope Process Bypass
```

No lo cambies globalmente si no hace falta.

---

## 4) WPR no existe

### Síntoma
la UI informa que el modo de precisión no está disponible o `wpr` no se reconoce.

### Acción
- instala Windows Performance Toolkit,
- verifica con `wpr -profiles` y `wpr -status`.

---

## 5) WPA no existe

### Síntoma
puedes capturar ETL pero no abrirlo fácilmente en el mismo equipo.

### Acción
- instala Windows Performance Analyzer,
- o mueve el ETL a otro equipo que sí tenga WPA.

---

## 6) `wpr -status` muestra sesión activa que no recuerdas

### Acción
verifica antes de lanzar una nueva captura:

```powershell
wpr -status
```

Si corresponde, cancela:

```powershell
wpr -cancel
```

---

## 7) El ETL sale enorme

### Causa probable
la captura duró demasiado.

### Acción
- reduce la ventana de observación,
- inicia justo antes del síntoma,
- detén apenas ocurra.

---

## 8) No puedes finalizar un proceso

### Posibles causas
- proceso protegido,
- permisos insuficientes,
- el proceso ya murió,
- es un proceso del sistema.

### Acción
- valida si es realmente tu objetivo,
- eleva privilegios si corresponde,
- usa exportación JSON antes de insistir en intervenir.

---

## 9) No puedes detener un servicio

### Posibles causas
- falta de permisos,
- política local,
- servicio no autorizado por la UI,
- dependencia del sistema.

### Acción
- ejecuta como administrador si hace falta,
- confirma el nombre del servicio,
- recuerda que la UI solo permite ciertos servicios concretos.

---

## 10) No puedes bloquear una IP

### Posibles causas
- permisos de firewall,
- IP mal parseada,
- política corporativa restrictiva.

### Acción
- prueba con privilegios elevados,
- valida la IP manualmente,
- revisa reglas existentes.

---

## 11) El instalador no se genera

### Revisión rápida
- confirma `target\release\rootcause.exe`
- confirma `ISCC.exe`
- ejecuta `scripts\package-inno.ps1`
- revisa si `build\installer\` fue creada

---

## 12) La UI abre pero faltan datos

### Posibles causas
- restricciones de PowerShell,
- netstat limitado por políticas,
- actividad real todavía no presente,
- servicios o eventos no accesibles.

### Acción
- reproduce el problema real,
- exporta JSON,
- contrasta con scripts manuales,
- usa WPR si la observación normal no basta.

---

## 13) La app parece pesada

### Acción
- aumenta el intervalo de refresco,
- filtra menos,
- no dejes WPR corriendo si no estás capturando un caso,
- evita abrirla junto con herramientas pesadas de análisis.
