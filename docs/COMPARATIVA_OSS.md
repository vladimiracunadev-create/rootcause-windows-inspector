# 🧭 RootCause frente al open source de seguridad/DFIR

> Comparativa de herramientas open source afines a RootCause: qué son, qué **tomar** y
> qué **evitar** de cada una, y las **oportunidades priorizadas** para mejorar el
> producto antes de lanzarlo. Datos verificados con búsqueda web (finales de 2025 /
> mediados de 2026); las fuentes están al pie. Los puntos no confirmados al 100% se
> marcan con ⚠️.

**Dónde encaja RootCause:** software **forense de ciberseguridad para Windows**,
**ligero, local y cero-telemetría**, un solo binario, que trata **la distorsión de
recursos como indicio temprano** (procesos/CPU/RAM/IO/red/temporales) + **baseline** de
autoarranque y servicios + ETW/WPR + heurísticas de comportamiento. **No es antivirus
ni EDR.** La investigación confirma que **ningún proyecto ocupa exactamente ese nicho**:
los grandes son plataformas agente+servidor; los ligeros hacen *consultas*, no
detección temprana por comportamiento. Ese es su diferenciador defendible.

---

## Tabla comparativa

| Proyecto | Lenguaje | Licencia | Arquitectura | Windows | Qué es |
|---|---|---|---|---|---|
| **RootCause** | Rust | Apache-2.0 | **Local, 1 binario** | Nativo | Forense de recursos + baseline + ETW, cero-telemetría |
| osquery | C++ | Apache-2.0 **o** GPL-2.0 | Local (o flota opc.) | Nativo | El SO como tablas SQL consultables |
| Velociraptor | Go | **AGPLv3** | Agente + servidor | 1.ª clase | DFIR/hunting con VQL + ETW |
| Wazuh | C | GPLv2 | Agente+servidor+indexer+dashboard | Agente | SIEM/XDR: FIM, ATT&CK, drift |
| OSSEC | C | GPLv2 | Agente (needs servidor) | Parcial | HIDS clásico: FIM, rootcheck |
| Sysmon | C/C++ (cerrado) | **Freeware, NO OSS** | Driver+servicio → Event Log | Nativo | Telemetría ETW (procesos, red, registro) |
| BLUESPAWN | C++ | **GPLv3** (+deps NC) | Local | Nativo | Hunt/Monitor/Mitigate ATT&CK (α) |
| PersistenceSniper | PowerShell | MIT + Commons Clause | Local (one-shot) | Nativo | Caza de persistencia (~60 técnicas) |
| Sigma (SigmaHQ) | YAML (formato) | DRL 1.1 / Apache | — (estándar) | Amplio | Reglas de detección portables |
| Hayabusa | **Rust** | **AGPLv3** | Local (CLI) | Nativo | Timeline/hunting EVTX + Sigma |
| Chainsaw | **Rust** | **GPL-3.0** | Local (CLI) | Nativo | Hunting EVTX/MFT + Sigma |
| bottom (`btm`) | **Rust** | MIT | Local (TUI) | Nativo | Monitor de sistema de terminal |

---

## Qué tomar / qué evitar (por proyecto)

### Plataformas DFIR/endpoint

**osquery** (C++, Apache-2.0 **o** GPL-2.0; activo ~5.22, Linux Foundation; **local** con `osqueryi`, sin servidor).
- ✅ **Tomar:** el modelo *"el SO como tablas consultables"* y su modo **local sin telemetría** — el ejemplo canónico de lo que RootCause quiere ser. Inspira un esquema de datos que el usuario pueda "preguntar".
- ⛔ **Evitar:** su despliegue de **flota** (fleet manager). Y ojo: osquery hace *snapshots/consultas*, no heurística temporal de comportamiento — ahí RootCause se diferencia.

**Velociraptor** (Go, **AGPLv3**; muy activo, Rapid7; **agente+servidor**, Windows de 1.ª clase, mucho ETW).
- ✅ **Tomar:** su **catálogo de artefactos forenses Windows** (autoruns, servicios, tareas, registro, ETW, prefetch) como lista de "qué mirar"; valida tu baseline y tu uso de ETW.
- ⛔ **Evitar:** arquitectura **agente+servidor** con GUI central (peso que rechazas) y **no copiar código** (AGPLv3, copyleft de red). Toma ideas, no líneas.

**Wazuh** (C, GPLv2; muy activo, 4.14.2 ene-2026; **agente+servidor+indexer+dashboard**).
- ✅ **Tomar:** conceptos ligeros de **FIM**, **detección de drift de configuración** y **mapeo a MITRE ATT&CK** como etiquetado de hallazgos.
- ⛔ **Evitar:** es el arquetipo de lo que **NO** debes ser (stack pesado, telemetría centralizada). Úsalo solo como *taxonomía de qué detectar*, no de arquitectura.

**OSSEC** (C, GPLv2; mantenido por Atomicorp; Windows solo como **agente que necesita servidor** Linux). ⚠️ *fechas de release 4.0/4.1 no corroboradas al 100%.*
- ✅ **Tomar:** su **FIM**, detección de rootkits y sobre todo el diseño de **reglas/decodificadores declarativos** (XML) — idea válida si algún día quieres heurísticas configurables por el usuario sin recompilar.
- ⛔ **Evitar:** el modelo Windows-agente-necesita-servidor. RootCause aporta lo que OSSEC no: distorsión de recursos como indicio temprano, sin servidor.

### Hunting específico de Windows

**Sysmon** (Microsoft; **gratuito pero NO open source** — EULA Sysinternals; *Sysmon for Linux* sí es MIT/eBPF; muy mantenido).
- ✅ **Tomar (brújula de heurísticas ETW que RootCause puede *inferir* sin instalar driver):** los **Event IDs de alto valor** — ID 8 *CreateRemoteThread* e ID 10 *ProcessAccess* (acceso a `lsass.exe` → robo de credenciales, ATT&CK T1003), ID 25 *ProcessTampering* (hollowing, T1055), ID 9 *RawAccessRead* (lectura cruda de disco → exfiltración), ID 7 *ImageLoad* (DLL sideloading), ID 19-21 *WmiEvent* (persistencia WMI), ID 12-14 *RegistryEvent* (autoarranque), ID 22 *DnsQuery*, ID 2 *FileCreateTime* (timestomping).
- ⛔ **Evitar:** instalar un **driver kernel** boot-start (contradice ligero/local); Sysmon **solo emite**, no analiza, y su modelo reenvía a un SIEM.

**BLUESPAWN** (C++, **GPLv3** + `signature-base` CC-BY-NC; Windows; estado **alfa**). ⚠️ *cadencia de commits reciente no confirmada.*
- ✅ **Tomar:** el **mapeo explícito a MITRE ATT&CK** de cada hallazgo (patrón clave, barato y muy rentable); y el uso de **PE-Sieve/YARA** para detectar inyección/hollowing en memoria (encaja con "distorsión de recursos").
- ⛔ **Evitar:** su naturaleza de **EDR que mitiga/actúa** y sus falsos positivos de alfa; quédate con la taxonomía ATT&CK, no con la respuesta activa. Cuidado con arrastrar dependencias **no comerciales** (NC) que contaminan la licencia.

**PersistenceSniper** (PowerShell, MIT + **Commons Clause**; muy activo, v1.17.1, **~60 técnicas**; solo Windows).
- ✅ **Tomar:** su catálogo es la **referencia para expandir tu baseline de autoarranque**. Técnicas más allá de Run keys/servicios: **WMI Event Subscriptions**, **Scheduled Tasks**, **Winlogon** (Shell/Userinit/Notify), **AppInit_DLLs**, **IFEO / SilentProcessExit**, **COM hijacking**, **LSA providers/Security Packages**, **Netsh helper DLLs**, **Print Monitors/Processors**, **Office add-ins/templates**, **BITS jobs**, **Screensaver/Debugger hijacks**, **Time Providers**, carpetas Startup — cada una mapea a ATT&CK T1547/T1546/T1053/T1543.
- ⛔ **Evitar:** su modelo es enumeración *one-shot*; RootCause debe integrarlas como **diff contra baseline** (detección de *cambio*), no como listado. No reusar código bajo Commons Clause si hay intención comercial.

**Sigma** (formato YAML, **DRL 1.1** permisiva + toolchain Apache/MIT; **+15.000 reglas**; corpus muy Windows/Sysmon).
- ✅ **Tomar:** adoptarlo como **capa opcional de reglas declarativas**: RootCause podría **ingerir el corpus DRL** y evaluarlo contra su telemetría, ganando detecciones "de la comunidad" sin escribirlas a mano y **hablando el idioma de la industria** — todo local, coherente con cero-telemetría.
- ⛔ **Evitar:** muchas reglas asumen **campos de Sysmon**; sin Sysmon, RootCause necesitaría un **adaptador** que mapee su telemetría a los `logsource`/campos Sigma. No lo uses como motor pesado que reemplace tus heurísticas nativas; úsalo como capa encima.

### Herramientas Rust (referencia, no integrables)

**Hayabusa** (Rust, **AGPLv3**; producción, v3.10.0 jul-2026; EVTX + Sigma, único con Sigma correlation v2).
**Chainsaw** (Rust, **GPL-3.0**; producción, v2.16.0 may-2025; EVTX/MFT + Sigma vía `evtx` + `tau-engine`).
- ✅ **Tomar:** son la **referencia arquitectónica** de "motor de reglas Sigma sobre EVTX en Rust". Chainsaw muestra cómo combinar la crate `evtx` con un motor de matching.
- ⛔ **Evitar (crítico):** **AGPLv3 y GPL-3.0 son incompatibles con la Apache-2.0 de RootCause** para enlazado/integración. **No embeber su código.** Solo como herramientas externas que el usuario ejecute, o inspiración de diseño.

**bottom** (Rust, MIT; monitor de sistema TUI; usa `sysinfo`). Es una **app de usuario final**, no una librería → **inspiración de UX**, no código reusable.

### Librerías Rust — el material real para mejorar RootCause

**`windows` (windows-rs)** — MIT **o** Apache-2.0 ✅ compatible; oficial de Microsoft, muy activo (0.62.2 oct-2025), producción. **Máximo ROI.**
- Reemplaza el **shelleo de PowerShell/netstat** por **APIs nativas**: Servicios (`EnumServicesStatusEx`), Registro (`RegOpenKeyEx`), sesiones/usuarios (`WTSEnumerateSessions`), procesos/tokens, y **red** (`GetExtendedTcpTable`/`GetExtendedUdpTable` en vez de parsear `netstat`). Elimina de raíz los problemas de exit-code/encoding que ya sufrimos. Migración **incremental por módulo**.

**`evtx` (omerbenamram)** — ⚠️ licencia probable MIT/Apache (*verificar*); muy activo (v0.12.2 jul-2026), producción; es la base de Chainsaw/Hayabusa.
- Leer **Event Logs (EVTX) directamente en Rust** en vez de `wevtutil`/`Get-WinEvent`. Bajo riesgo, habilita análisis histórico de logs sin shellear.

**`ferrisetw`** — MIT **o** Apache-2.0 ✅ compatible; v1.2.0 (~2024), ⚠️ mantenedor único, actividad reciente no confirmada; funcional pero de nicho.
- **ETW en vivo estilo Sysmon** (creación de procesos, conexiones, carga de módulos) sin WPR/tracerpt en batch. **Reservas:** requiere **administrador**, cambia de *batch* a *streaming*, un solo mantenedor. → candidato de **v2.0** (encaja con el Windows Service).

**`sysinfo`** — MIT ✅; ya en uso, muy sano (0.37.2). **Mantener**, fijar a 0.37.x y vigilar breaking changes de la API entre versiones menores. No usarla para lo que windows-rs hace mejor.

**Motores Sigma en Rust** (`sigma-rust`, `sigmars`, `sigma-rs`; `tau-engine` de Chainsaw) — ⚠️ licencias variadas (*verificar cada una*), madurez mixta/jóvenes.
- La vía para pasar de **heurísticas hardcodeadas** a **reglas Sigma declarativas**. Con `evtx`, sería el mayor salto de detección. **Riesgo:** aún menos maduras que los motores de Chainsaw/Hayabusa. → prototipo, **no bloquear v1.0**.

---

## Compatibilidad de licencias (RootCause = Apache-2.0)

| Puedes **embeber/enlazar** (permisiva) | **NO** embeber (copyleft) — solo usar/mirar |
|---|---|
| `windows` (MIT/Apache), `ferrisetw` (MIT/Apache), `sysinfo` (MIT), `evtx` (⚠️ verificar), reglas **Sigma** (DRL 1.1), `bottom` (MIT, pero es app) | **Velociraptor** (AGPLv3), **Hayabusa** (AGPLv3), **Chainsaw** (GPL-3.0), **Wazuh/OSSEC** (GPLv2), **BLUESPAWN** (GPLv3), código de **osquery** bajo la rama GPL |

> Regla práctica: de los proyectos GPL/AGPL toma **ideas y taxonomía** (qué detectar,
> cómo etiquetar), nunca **código**. Los crates MIT/Apache sí se integran.

---

## Oportunidades priorizadas (antes de lanzar y después)

### Antes de v1.0 — bajo riesgo, alto ROI
1. **Migrar recolectores a `windows-rs`** (servicios, registro, red, procesos): mata la fragilidad del shelleo PowerShell/netstat (exit-code, encoding). Incremental.
2. **Etiquetar las 16 señales con MITRE ATT&CK** (T1486 ransomware, T1496 cryptojacking, T1547/T1546 persistencia, T1055 injection, T1003 credenciales, T1071 C2, T1046 escaneo…). Barato, alto impacto de credibilidad.
3. **Expandir el baseline de persistencia** con las técnicas de PersistenceSniper (WMI subs, IFEO, COM hijack, LSA, Netsh DLLs…), implementadas como *diff vs baseline*.
4. **`evtx`** para leer Event Logs en Rust (sin `Get-WinEvent`).
5. **Fijar `sysinfo` 0.37.x**.

### v2.0 — mayor esfuerzo, encaja con el Windows Service
6. **`ferrisetw`**: ETW en vivo estilo Sysmon (reemplaza WPR/tracerpt batch).
7. **Motor de reglas Sigma** (`sigma-rust`/`sigmars`): detección declarativa y extensible sobre la telemetría.
8. **FIM** (integridad de archivos) como nueva superficie del motor de baseline.

### Qué NO copiar (para no perder la identidad)
- Arquitectura **agente+servidor**, indexer, dashboard central (Wazuh/OSSEC/Velociraptor).
- **Telemetría centralizada** (rompe el cero-telemetría).
- **Respuesta activa / EDR** que mitiga por su cuenta (BLUESPAWN).
- **Drivers kernel** boot-start (Sysmon).
- Cualquier **código GPL/AGPL** embebido.

---

## Fuentes
- osquery: <https://github.com/osquery/osquery> · licencia <https://github.com/osquery/osquery/blob/master/LICENSE>
- Velociraptor: <https://github.com/velocidex/velociraptor/releases> · <https://www.rapid7.com/products/velociraptor/>
- Wazuh: <https://github.com/wazuh/wazuh> · <https://documentation.wazuh.com/current/release-notes/release-4-14-2.html>
- OSSEC: <https://github.com/ossec/ossec-hids> · <https://www.ossec.net/ossec-downloads/>
- Sysmon: <https://learn.microsoft.com/en-us/sysinternals/downloads/sysmon> · Sysmon for Linux <https://github.com/microsoft/SysmonForLinux> · licencia <https://learn.microsoft.com/en-us/sysinternals/license-terms>
- BLUESPAWN: <https://github.com/ION28/BLUESPAWN>
- PersistenceSniper: <https://github.com/last-byte/PersistenceSniper> · detections <https://github.com/last-byte/PersistenceSniper/wiki/3-%E2%80%90-Detections>
- Sigma: <https://github.com/SigmaHQ/sigma> · DRL 1.1 <https://github.com/SigmaHQ/Detection-Rule-License>
- Hayabusa: <https://github.com/Yamato-Security/hayabusa/releases>
- Chainsaw: <https://github.com/WithSecureLabs/chainsaw>
- bottom: <https://github.com/ClementTsang/bottom> · <https://crates.io/crates/bottom>
- windows-rs: <https://crates.io/crates/windows> · <https://github.com/microsoft/windows-rs/releases>
- ferrisetw: <https://crates.io/crates/ferrisetw> · <https://github.com/n4r1b/ferrisetw>
- sysinfo: <https://crates.io/crates/sysinfo>
- evtx: <https://crates.io/crates/evtx> · <https://github.com/omerbenamram/evtx>
- Sigma en Rust: <https://crates.io/crates/sigma-rust> · <https://docs.rs/tau-engine>

> ⚠️ **A verificar antes de citar como definitivo:** versión exacta de `bottom`;
> licencia exacta de la crate `evtx`; actividad reciente de `ferrisetw`; licencias
> individuales de las crates Sigma/`tau-engine`; fechas de release de OSSEC 4.0/4.1.

_Ver también: [`DETECCION_AMENAZAS.md`](DETECCION_AMENAZAS.md) (amenaza → detección hoy) y [`ROADMAP.md`](ROADMAP.md)._
