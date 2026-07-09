# 🛡️ Qué hace RootCause frente a cada amenaza (hoy)

> **Tesis:** RootCause no identifica malware por firma ni reemplaza a un antivirus/EDR.
> Su valor es otro y complementario: **cualquier distorsión de los recursos de un
> equipo —CPU, RAM, disco, red, temporales, procesos, autoarranque, servicios— puede
> ser el primer indicio de que algo sucede.** RootCause vigila esas distorsiones de
> forma **agnóstica** (no necesita saber *qué* malware es para notar que "algo se
> comporta distinto"), las correlaciona en incidentes con una hipótesis de causa, y
> deja evidencia para investigar.

Este documento es **honesto sobre los límites**: mapea cada familia de amenaza a lo
que RootCause hace **en la versión actual**, con una leyenda de cobertura clara.

## Leyenda de cobertura

| Símbolo | Significado |
|---|---|
| 🟢 | **Señal directa/fuerte** — RootCause tiene una detección específica para esto. |
| 🟡 | **Señal indirecta/parcial** — ve el *síntoma de recursos*, no el ataque en sí; útil como indicio temprano. |
| ⬜ | **Fuera de alcance** — no aplica a un endpoint Windows o requiere otra herramienta (antivirus/EDR/WAF/red). |

> Nota: las detecciones 🟢/🟡 son **heurísticas** con umbrales configurables (tab
> **Configuración**). Pueden dar falsos positivos y negativos; son *indicios para
> mirar*, no veredictos.

---

## Resumen: familia de amenaza → qué hace RootCause hoy

| Familia de amenaza | Cobertura | Señal / función de RootCause |
|---|---|---|
| **Ransomware** (cifrado en curso) | 🟢 | `aggressive-disk-write`: escritura de disco masiva y sostenida = indicio muy temprano del cifrado |
| **Cryptojacking** (minado oculto) | 🟢 | `sustained-high-cpu`: CPU alta sostenida por un proceso |
| **Persistencia** (sobrevivir reinicios) | 🟢 | Baseline de **Autostart**: `persistence-change` (NUEVA/MODIFICADA/ELIMINADA) + `suspicious-persistence` |
| **Servicio malicioso / binario secuestrado / Defender apagado** | 🟢 | Baseline de **Servicios**: `service-change`, `security-service-disabled`, `security-control-alteration` |
| **Fileless / LOLBins** (PowerShell, rundll32, mshta…) | 🟡 | `suspicious-parent-child`, `repetitive-script-execution` |
| **C2 / RAT / backdoor / exfiltración** | 🟡 | `multi-destination-outbound` + tab **Conexiones** (proceso ↔ IP) + **bloquear IP** |
| **Dropper / troyano en ubicación rara** | 🟡 | `suspicious-execution-path` (%TEMP%, AppData, Downloads…), `outside-trusted-baseline` |
| **Movimiento lateral / recon interno** | 🟡 | `local-network-scan`: un proceso hablando con muchos destinos de la LAN |
| **Malware genérico (por síntomas)** | 🟡 | `memory-growth`, score de riesgo por proceso, `correlated-anomaly` (varias señales juntas) |
| **Exfiltración por insider** | 🟡 | Red Tx elevada + `multi-destination-outbound` + proceso asociado |
| **Explotación de vulnerabilidad / zero-day** | 🟡 | No ve el exploit; sí el **resultado** (proceso nuevo, respawn, persistencia, conexión) |
| **Ingeniería social / phishing** | 🟡 | Solo el **payload post-clic** (proceso/persistencia/conexión que deja el clic) |
| **Rootkit / bootkit** | ⬜ | Se ocultan del SO; RootCause usa las APIs del SO |
| **Web/app (SQLi, XSS, CSRF…)** | ⬜ | Es del lado servidor/web; RootCause es endpoint |
| **Red interna (MITM, sniffing, ARP/DNS spoof)** | ⬜ | No inspecciona el tráfico de la LAN (solo conexiones por proceso) |
| **Cadena de suministro (pre-runtime)** | ⬜ | No escanea dependencias/CVEs (la sección Docker es higiene de espacio, no seguridad) |
| **Físico / hardware / side-channel** | ⬜ | Fuera de alcance |
| **Nube / contenedores / móvil / IoT/OT** | ⬜ | RootCause es endpoint Windows |
| **Criptográficas / de datos** | ⬜ | Fuera de alcance |
| **Emergentes / IA (deepfakes, adversarial ML)** | ⬜ | Salvo que el payload corra localmente |

---

## Detalle por familia

### Ransomware 🟢 (indicio temprano fuerte)
El cifrado masivo de archivos produce **escritura de disco agresiva y sostenida**.
RootCause lo marca como `aggressive-disk-write` (umbral MB/s configurable) y, si
coincide con una ruta de ejecución sospechosa, lo eleva en el score. Es de las
señales más valiosas: *puedes ver el síntoma mientras el cifrado está empezando*,
antes de que termine. No detiene el cifrado — pero te da la ventana para reaccionar
(finalizar el proceso, aislar el equipo).

### Cryptojacking 🟢
El minado oculto satura la CPU. RootCause lo detecta como `sustained-high-cpu`
(porcentaje + nº de muestras configurable) y lo atribuye al proceso concreto.

### Persistencia 🟢 (el punto más fuerte)
Casi todo el malware necesita **sobrevivir a los reinicios**, así que se instala en
el autoarranque. RootCause guarda una **línea base** (Run/RunOnce HKCU/HKLM, carpetas
Startup, tareas programadas) y en cada escaneo clasifica cada entrada como **NUEVA /
MODIFICADA / ELIMINADA**, generando la alerta `persistence-change` (y
`suspicious-persistence` para entradas en rutas de riesgo). Los cambios son pegajosos
hasta que aceptas la nueva baseline. **Aquí RootCause hace algo que el Administrador
de tareas no hace**: te dice *qué cambió* respecto a tu estado bueno conocido.

### Servicios: secuestro, servicio malicioso, Defender deshabilitado 🟢
Mismo motor de baseline sobre los **servicios de Windows**: vigila `StartMode` + ruta
del binario. Detecta un servicio **nuevo**, un **binario secuestrado** (misma etiqueta,
otra ruta), un cambio de modo de arranque, o que un **control de seguridad se apagó**
(`security-service-disabled`, `security-control-alteration` — p. ej. Defender/MpsSvc
deshabilitado "de repente", señal clásica de compromiso).

### Fileless / Living-off-the-Land (LOLBins) 🟡
Los ataques sin archivo usan binarios legítimos del sistema. RootCause aplica la
heurística `suspicious-parent-child`: un proceso lanzado por `powershell.exe`,
`cmd.exe`, `wscript.exe`, `cscript.exe`, `mshta.exe`, `rundll32.exe` o por apps de
Office/navegador es sospechoso; y `repetitive-script-execution` marca scripts que se
repiten. No inspecciona la memoria — es un indicio de comportamiento, no una prueba.

### C2 / RAT / backdoor / exfiltración 🟡
Un implante suele **hablar hacia fuera**. RootCause marca `multi-destination-outbound`
cuando un proceso se conecta a muchas IPs **públicas** (posible C2, exfiltración o
tu equipo actuando como bot). El tab **Conexiones** muestra *qué proceso* habla con
*qué IP* (netstat enriquecido con nombre y ruta), y permite **bloquear una IP** con el
firewall de Windows. No descifra el tráfico ni identifica el C2 por reputación.

### Dropper / troyano en ubicación sospechosa 🟡
La ejecución desde `%TEMP%`, `AppData`, `Downloads`, `Public` o `ProgramData` se marca
como `suspicious-execution-path`; y `outside-trusted-baseline` señala procesos fuera de
las rutas/nombres de confianza. Es donde el software legítimo *normalmente no* corre.

### Movimiento lateral / reconocimiento interno 🟡
`local-network-scan`: un proceso que abre conexiones a **muchos destinos de la red
local** puede estar escaneando o moviéndose lateralmente. Es un indicio, no una prueba
de intrusión.

### Malware genérico (por síntomas de recursos) 🟡
Aunque RootCause no sepa qué es, nota `memory-growth` (crecimiento anómalo de RAM),
CPU/IO fuera de rango, y sobre todo `correlated-anomaly`: cuando **varias señales
coinciden en el mismo proceso** (p. ej. ruta sospechosa + fuera de baseline +
reaparición), se eleva a un **incidente** con score de riesgo e **hipótesis de causa
raíz**. Esa correlación es la que convierte "señales sueltas" en "esto merece mirarse".

### Explotación de vulnerabilidades / zero-day 🟡
RootCause **no ve el exploit** (no es un IDS ni analiza payloads de red). Ve el
**resultado**: un proceso inesperado, un crash que reaparece (`rapid-respawn`), una
nueva entrada de persistencia o una conexión saliente. Es detección *post-explotación*
por comportamiento.

### Ingeniería social / phishing 🟡 (solo el payload)
El engaño ocurre en el correo y en la persona — fuera de alcance. Pero **lo que el
clic deja atrás sí aparece**: un adjunto que lanza PowerShell (padre sospechoso), que
crea persistencia o que abre una conexión. RootCause ve la consecuencia, no el correo.

### Fuera de alcance ⬜ (y por qué)
- **Rootkits/bootkits:** se ocultan por debajo del SO; RootCause consulta al SO, así
  que lo que el rootkit oculta, no lo ve.
- **Web/aplicaciones (SQLi, XSS, CSRF, SSRF…):** es del lado servidor; corresponde a
  un WAF/pentest.
- **Red interna (MITM, sniffing, ARP/DNS spoofing):** RootCause ve *tus* conexiones por
  proceso, no el tráfico de otros en la LAN.
- **Cadena de suministro / paquetes maliciosos (antes de ejecutarse):** no escanea
  dependencias ni CVEs. *(La sección Docker mide espacio, no seguridad de imágenes.)*
- **Físico, hardware, side-channels, nube/contenedores/móvil/IoT, criptografía,
  amenazas de IA:** requieren herramientas específicas de esos dominios.

> En todos los ⬜ hay un matiz: **si el ataque termina en un proceso que corre en este
> Windows**, su distorsión de recursos vuelve a ser visible para RootCause.

---

## Vocabulario de detección (las 16 señales reales de hoy)

Estas son las señales que RootCause emite actualmente (nombres internos → qué
significan):

| Señal | Qué significa | Amenaza que insinúa |
|---|---|---|
| `sustained-high-cpu` | CPU alta sostenida | cryptojacking, bucle malicioso |
| `memory-growth` | crecimiento anómalo de RAM | fuga/malware activo |
| `aggressive-disk-write` | escritura masiva y sostenida | **ransomware** en curso |
| `multi-destination-outbound` | muchas conexiones a IPs públicas | C2 / exfiltración / botnet |
| `local-network-scan` | muchas conexiones a la LAN | recon / movimiento lateral |
| `suspicious-execution-path` | ejecución en %TEMP%/AppData/… | dropper / troyano |
| `outside-trusted-baseline` | fuera de rutas/nombres de confianza | binario inesperado |
| `suspicious-parent-child` | lanzado por PowerShell/cmd/mshta… | fileless / LOLBins |
| `rapid-respawn` | proceso que reaparece rápido | persistencia / watchdog malicioso |
| `repetitive-script-execution` | scripts repetidos | automatización maliciosa |
| `security-control-alteration` | cambio en un control de seguridad | evasión de defensas |
| `suspicious-persistence` | persistencia en ruta de riesgo | malware que se instala |
| `persistence-change` | cambio en autoarranque vs baseline | nueva persistencia |
| `security-service-disabled` | servicio de seguridad detenido | evasión (Defender off) |
| `correlated-anomaly` | varias señales en el mismo proceso | **incidente** con hipótesis |
| `config-integrity-change` | cambió la config del propio agente | manipulación del monitor |

Además: **score de salud** y **veredicto** en el Resumen, **historial A/B** para ver
"desde cuándo cambió", y **auditoría local** de toda acción (finalizar proceso, bloquear
IP, detener servicio).

---

## Lo que RootCause **NO** hace (para no venderlo mal)

- No es antivirus ni EDR: **no elimina malware** ni bloquea ejecución por firma.
- No inspecciona tráfico de red ni descifra conexiones.
- No analiza la memoria en profundidad (no ve rootkits que se ocultan del SO).
- No escanea dependencias/CVEs ni imágenes de contenedor por vulnerabilidades.
- No hace DLP, ni cubre web, nube, móvil, IoT ni ataques físicos.
- Sus detecciones son **heurísticas** (indicios), no veredictos: úsalas para *mirar*,
  y combínalas con tu antivirus/EDR.

**En una línea:** RootCause es el sensor de *"algo se comporta distinto en este
Windows"* — el indicio temprano y agnóstico — que te dice **dónde mirar** y deja la
evidencia, no el guardia que lo detiene.

---

_Ver también: [`MANUAL_USUARIO.md`](MANUAL_USUARIO.md) (qué es cada sección),
[`HEURISTICAS.md`](HEURISTICAS.md) y [`MODULO_DETECCION_ANOMALIAS.md`](MODULO_DETECCION_ANOMALIAS.md)
(detalle técnico), y los requisitos [`REQ-SEC-001`](requirements/REQ-SEC-001-deteccion-comportamiento-anomalo.md)._
