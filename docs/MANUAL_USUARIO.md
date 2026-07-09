# 📘 Manual de usuario — RootCause Windows Inspector

Este manual explica **qué es RootCause, qué resuelve y qué significa cada cosa**, en
lenguaje claro. No necesitas ser técnico para leerlo: cada término difícil se define
la primera vez que aparece. Si algo sigue sin quedar claro, es un fallo del manual —
no tuyo.

> **En una frase:** RootCause te dice, con evidencia, **qué está degradando tu PC con
> Windows** (qué proceso, carpeta, servicio, conexión o cambio) — y solo después te
> deja actuar, con confirmación.

---

## 1. El problema que resuelve

Windows, cuando va lento o raro, no te da una respuesta clara. Ves el disco al 100 %,
el ventilador a tope o la RAM llena, pero **no sabes quién tiene la culpa**. El
Administrador de tareas muestra números sueltos; no explica la **causa**.

RootCause responde preguntas concretas en una sola ventana:

- ¿Qué proceso está cargando el disco **ahora mismo**?
- ¿Qué carpeta temporal creció y cuánto?
- ¿Esa lentitud viene de Windows Update, del antivirus, de Docker?
- ¿Qué programa mantiene conexiones a Internet que no reconozco?
- ¿Cambió algo en el arranque de mi equipo sin que yo lo hiciera?

## 2. La idea central: diagnóstico primero, intervención después

RootCause **primero explica** la causa dominante del problema y **solo después** te
ofrece actuar — siempre con confirmación y dejando registro.

**Qué NO es**, para evitar malentendidos:

- **No es un antivirus ni un EDR.** No elimina malware. Detecta *señales* sospechosas
  y te las muestra para que decidas; complementa a tu antivirus, no lo reemplaza.
- **No es un "limpiador mágico".** No promete acelerar tu PC borrando cosas al azar.
  Borra solo lo que es demostrablemente seguro, y te dice qué y por qué.

---

## 3. Glosario — los términos difíciles, en simple

| Término | Qué significa (claro) |
|---|---|
| **Causa raíz** | El motivo *real* y de fondo de un síntoma, no el síntoma. Si el disco está al 100 %, la causa raíz es *qué proceso* lo está escribiendo. |
| **Captura / snapshot** | Una "foto" del estado del equipo en un instante: CPU, RAM, procesos, conexiones, etc. RootCause toma capturas cada pocos segundos. |
| **Severidad** | Qué tan importante es algo. Se codifica en color: 🟢 **verde** = normal, 🟡 **ámbar** = conviene revisar, 🔴 **rojo** = revísalo ya. |
| **Salud del sistema (0–100)** | Un puntaje resumido del estado general. Alto = tranquilo; bajo = hay señales que atender. |
| **Proceso** | Un programa en ejecución (p. ej. `chrome.exe`). Cada uno tiene un **PID**. |
| **PID** | *Process ID*: el número único que Windows le da a cada proceso mientras corre. |
| **I/O de disco** | *Entrada/Salida*: cuánto lee (R) y escribe (W) un proceso en el disco. La escritura alta sostenida suele ser la culpable del "disco al 100 %". |
| **Incidente** | Un evento problemático que RootCause detecta y guarda con su evidencia (qué proceso, qué señales, hipótesis de causa). |
| **Anomalía** | Un comportamiento fuera de lo normal según reglas locales (p. ej. CPU alta sostenida, crecimiento raro de memoria, tráfico saliente inusual). |
| **Línea base (baseline)** | Una "foto de referencia" de un estado bueno conocido. RootCause la usa para detectar **cambios**: compara el ahora contra esa referencia. |
| **Persistencia / autoarranque** | Todo lo que se ejecuta solo al encender Windows (registro `Run`, carpetas de inicio, tareas programadas). El malware suele esconderse aquí para "sobrevivir" reinicios. |
| **Servicio de Windows** | Un programa que corre en segundo plano sin ventana (p. ej. Windows Update, el antivirus). |
| **ETW / WPR / traza ETL** | Herramientas de Windows para grabar con precisión qué hace el sistema por dentro. **ETW** = el mecanismo de eventos; **WPR** = el grabador; **ETL** = el archivo de la grabación. RootCause las maneja por ti, sin abrir herramientas externas. |
| **%TEMP%** | Tu carpeta de archivos temporales. Muchos programas dejan basura ahí y crece con el tiempo. |
| **Firewall** | El "portero" de red de Windows. RootCause puede crear una regla para **bloquear** una IP concreta. |
| **Imagen de Docker** | Una plantilla de la que se crean contenedores. Ocupa mucho disco y no aparece en las carpetas temporales normales. |
| **Imagen "dangling"** | Una imagen de Docker **sin etiqueta**, que quedó huérfana. Es seguro borrarla: no la usa nadie y se regenera. |
| **Caché de build** | Restos que Docker guarda para acelerar futuras construcciones. Se puede borrar sin perder datos; se regenera. |
| **Volumen de Docker** | Donde un contenedor **guarda sus datos** (bases de datos, archivos). RootCause **nunca lo borra solo**, porque ahí hay información que puede importarte. |

---

## 4. Recorrido por la interfaz

La ventana tiene una **barra lateral** a la izquierda (estilo Windows 11) con **11
secciones**, agrupadas por tema. Arriba de cada sección aparece su **título** y los
controles (Actualizar, Exportar, filtro, avisos). Cada sección se explica igual:
**qué es · qué ves · por qué importa**.

### Resumen
- **Qué es:** la vista de inicio, el "de un vistazo".
- **Qué ves:** un **banner de veredicto** (¿está tranquilo o hay que revisar?) con el
  puntaje de salud; tarjetas de CPU, RAM, Disco, Red y Temporales; y la lista
  **"Dónde mirar primero"** con las alertas más importantes.
- **Por qué importa:** en 5 segundos sabes si hace falta investigar o no.

### Procesos
- **Qué es:** los programas en ejecución, ordenados por riesgo.
- **Qué ves:** CPU, RAM, escritura a disco y un **score de riesgo** por proceso.
- **Por qué importa:** el score combina varias señales, así lo peligroso sube arriba
  sin que revises cientos de filas. Puedes **finalizar** un proceso (con confirmación).

### Conexiones
- **Qué es:** las conexiones de red activas, por proceso.
- **Qué ves:** qué programa habla con qué dirección (IP), con filtro de IPs públicas.
- **Por qué importa:** saber *qué* proceso se conecta a Internet es media
  investigación de una fuga de datos o un programa no deseado. Puedes **bloquear** una IP.

### Temporales
- **Qué es:** las carpetas que crecen y comen disco — incluida la sección **Docker**.
- **Qué ves:** `%TEMP%`, cachés de Windows Update, y el espacio de Docker (imágenes,
  volúmenes, recuperable).
- **Por qué importa:** el disco lleno degrada **todo** Windows. Puedes **limpiar tu
  %TEMP%** (seguro) y **purgar Docker** (ver sección 5).

### ETW / WPR
- **Qué es:** el "modo de precisión" para cuando la observación normal no basta.
- **Qué ves:** botones para iniciar / detener / analizar una grabación (traza ETL).
- **Por qué importa:** una traza ETW ve, a nivel del núcleo de Windows, qué causó
  exactamente un pico — sin que tengas que instalar ni abrir herramientas externas.

### Servicios
- **Qué es:** los servicios de seguridad relevantes (Defender, Windows Update, BITS…).
- **Qué ves:** su estado; y los **cambios** contra la línea base se reportan como alertas.
- **Por qué importa:** un servicio de seguridad detenido "de repente" es una señal
  clásica de compromiso.

### Autostart
- **Qué es:** todo lo que arranca con Windows (registro Run, carpetas de inicio, tareas).
- **Qué ves:** cada entrada con su severidad y si el archivo existe en disco; y si
  algo **cambió** respecto a la línea base (NUEVA / MODIFICADA / ELIMINADA).
- **Por qué importa:** es donde el malware se instala para sobrevivir a los reinicios.

### Historial
- **Qué es:** las capturas guardadas localmente (en una base de datos SQLite en tu equipo).
- **Qué ves:** puedes **comparar dos momentos (A vs B)** para ver cómo evolucionó el PC.
- **Por qué importa:** "¿desde cuándo va lento?" se responde comparando antes/después.

### Configuración
- **Qué es:** las preferencias.
- **Qué ves:** **Apariencia** (modos de tema Claro / Oscuro / Windows), **Idioma**
  (español / inglés) y los **umbrales** de detección (a partir de qué valores algo se
  marca ámbar o rojo).
- **Por qué importa:** adaptas la app a tu gusto y a tu equipo, sin reiniciar.

### Manual
- **Qué es:** una guía breve dentro de la propia app (versión resumida de este documento).

### Acerca
- **Qué es:** versión, autor, stack técnico, atajos de teclado y estado del propio agente.

---

## 5. Docker: por qué ocupa tanto y qué es seguro purgar

**Docker** es una herramienta de desarrollo que empaqueta programas en "contenedores".
En equipos de desarrollo suele ser **uno de los mayores consumidores de disco
ocultos**: acumula imágenes viejas y cachés que no ves en las carpetas temporales
normales.

En el tab **Temporales**, RootCause te muestra (leyendo del propio Docker):

- **Ocupado total** y **espacio recuperable**.
- Desglose por categoría: **Imágenes**, **Contenedores**, **Volúmenes**, **Caché de build**.
- La lista de imágenes (las más grandes primero) y de volúmenes.

**Purga guiada segura (2 pasos):** RootCause solo borra lo que es **regenerable**:

- ✅ **Imágenes "dangling"** (huérfanas, sin etiqueta) — nadie las usa.
- ✅ **Caché de build** — se vuelve a generar en la próxima construcción.
- ❌ **Volúmenes: nunca se borran automáticamente**, porque contienen **datos** (p. ej.
  la base de datos de un contenedor). Se listan para que **tú** decidas.

Desde consola: `rootcause docker` (ver el uso), `rootcause docker --prune-images`
(borra dangling), `rootcause docker --prune-cache` (borra caché de build).

---

## 6. Acciones seguras (siempre con confirmación y registro)

Toda acción que modifica algo pide confirmación y queda auditada localmente:

| Acción | Qué hace | Qué **no** hace |
|---|---|---|
| **Finalizar proceso** | Cierra un proceso por su PID. | No toca procesos críticos del sistema. |
| **Bloquear IP** | Crea una regla en el firewall de Windows para una IP. | No bloquea rangos ni afecta otras conexiones. |
| **Detener servicio** | Detiene un servicio de una lista permitida (bits, dosvc, sysmain, wuauserv). | No detiene servicios fuera de esa lista. |
| **Limpiar %TEMP%** | Borra lo no usado en 24 h de **tu** carpeta temporal. | No toca el sistema ni Windows Update; salta lo que esté en uso. |
| **Purgar Docker** | Borra imágenes dangling y/o caché de build. | Nunca borra volúmenes (tus datos). |
| **Aceptar línea base** | Marca el estado actual (autostart o servicios) como el nuevo "bueno conocido". | No cambia nada del sistema; solo actualiza la referencia de comparación. |

---

## 7. Apariencia e idioma

- **Modos de tema** (en Configuración → Apariencia): **Claro**, **Oscuro** (azul
  profundo de la marca) y **Windows** (sigue el tema claro/oscuro de tu sistema). El
  color de acento se toma del icono de RootCause en los tres.
- **Idioma:** español o inglés; cambia toda la interfaz al instante y se guarda solo.

---

## 8. Desde la consola (CLI)

Todo funciona también sin interfaz — útil para scripts o servidores. `rootcause --help`
lista todo. Los más usados:

| Comando | Qué hace |
|---|---|
| `rootcause status [--json]` | Estado actual del sistema. |
| `rootcause snapshot` | Captura completa en JSON. |
| `rootcause history [N]` | Últimas N capturas. |
| `rootcause autostart [--json] [--accept]` | Entradas de autoarranque y cambios vs baseline. |
| `rootcause services [--json] [--accept]` | Cambios en servicios vs baseline. |
| `rootcause clean-temp [--yes]` | Limpieza de `%TEMP%` (sin `--yes` solo simula). |
| `rootcause docker [--json\|--prune-images\|--prune-cache]` | Uso de disco de Docker y purga segura. |
| `rootcause wpr start\|stop\|analyze` | Modo de precisión ETW/WPR. |
| `rootcause kill <PID>` · `block-ip <IP>` · `stop-service <n>` | Acciones controladas (requieren administrador). |

---

## 9. Requisitos y permisos

- **Windows 10 / 11 (x64).**
- Algunas funciones (finalizar procesos del sistema, bloquear IP, detener servicios,
  capturas WPR) requieren **ejecutar como administrador**. La observación normal no.
- **SmartScreen:** al abrir por primera vez, Windows puede advertir "editor
  desconocido" porque el binario **aún no tiene firma digital**. Es esperable: *Más
  información → Ejecutar de todas formas*. (La firma digital está planificada para v1.0.)

## 10. Privacidad

- **Todo es local. Telemetría: cero.** Nada sale de tu equipo.
- El historial se guarda solo en tu PC (SQLite).
- El adaptador de IA es **opcional y viene apagado** por defecto.

---

## 11. Preguntas frecuentes

**¿Por qué mi disco está al 100 %?** Abre **Resumen** → mira la tarjeta Disco y "Dónde
mirar primero"; luego **Procesos** ordena por escritura para ver el culpable. Suele ser
Windows Update, un indexador, un antivirus o Docker.

**¿Es seguro usar los botones de limpieza?** Sí. Limpiar `%TEMP%` solo borra lo tuyo no
usado en 24 h; la purga de Docker solo borra lo regenerable (nunca volúmenes).

**¿RootCause me protege de virus?** No directamente: **te avisa** de señales
sospechosas (procesos raros, cambios de autoarranque, conexiones inusuales) para que
actúes o uses tu antivirus. No elimina malware por sí mismo.

**¿Consume muchos recursos?** No. Está escrito en Rust y pensado para observación
frecuente y de bajo consumo.

---

_¿Falta algo o un término sigue sin quedar claro? Es mejorable: la meta de este manual
es que **cualquiera** entienda qué hace RootCause y por qué._
