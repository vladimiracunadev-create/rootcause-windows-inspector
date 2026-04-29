# Operación

Esta guía explica cómo usar el software en un caso real de lentitud de Windows.

---

## 1) Flujo mínimo recomendado

1. abre la app,
2. deja correr algunos refrescos — observa las sparklines de CPU / RAM / I/O en el tab **Overview**,
3. mira el semáforo,
4. revisa “Dónde mirar primero”,
5. ve al tab **Procesos** y usa el filtro de severidad para concentrarte en lo Critical primero,
6. identifica si el problema parece venir de:
   - proceso (revisa también el command line del proceso),
   - temporal,
   - red,
   - servicio,
   - update,
7. si ya hay capturas anteriores, compara en el tab **Historial** para ver si empeoró,
8. exporta JSON si necesitas respaldo,
9. activa modo de precisión solo si todavía no basta.

---

## 2) Interpretación rápida

### Proceso dominante
Empieza por ahí cuando:
- CPU es alta,
- memoria es alta,
- escritura es alta,
- el ejecutable sale de `%TEMP%`.

### Temporales
Empieza por ahí cuando:
- TEMP crece rápido,
- aparece `SoftwareDistribution`,
- aparece `DeliveryOptimization`,
- el equipo se pone lento durante descargas o instalaciones.

### Conexiones
Empieza por ahí cuando:
- hay IP pública inesperada,
- el ejecutable no te resulta familiar,
- la ruta es rara o temporal.

### Servicios
Empieza por ahí cuando:
- `wuauserv`, `BITS` o `DoSvc` están activos,
- el equipo va lento mientras Windows parece “hacer algo solo”.

---

## 3) Intervención prudente

### Finalizar proceso
Hazlo solo si:
- ya identificaste que no es crítico,
- no corresponde a un servicio base protegido,
- quieres validar causalidad.

### Bloquear IP
Hazlo solo si:
- la IP no corresponde a tráfico esperado,
- ya validaste el proceso asociado,
- entiendes el impacto de cortar esa salida.

### Detener servicio
Úsalo como mitigación temporal o prueba de causa, no como receta permanente.

---

## 4) Cuándo pasar a WPR

Activa WPR cuando:
- el síntoma dura poco y desaparece,
- no logras identificar archivo o secuencia,
- necesitas dejar evidencia más precisa.

No lo actives por defecto en todos los casos.

---

## 5) Flujo de caso real

### Caso A: Windows Update sospechoso
1. abre la app,
2. observa servicios,
3. revisa `SoftwareDistribution` y `DeliveryOptimization`,
4. si sigue la duda, captura ETL,
5. resume ETL,
6. si el resumen sigue ambiguo, abre WPA.

### Caso B: binario temporal sospechoso
1. revisa procesos,
2. confirma ruta en `%TEMP%`,
3. valida conexiones activas,
4. exporta JSON,
5. si necesitas evidencia adicional, captura ETL,
6. resume y documenta.

### Caso C: lentitud corta e intermitente
1. deja el monitor corriendo,
2. cuando aparezca el síntoma activa o detén WPR según el caso,
3. resume el último ETL,
4. correlaciona hora del síntoma con alertas y snapshot.

### Caso D: comparar si el problema empeoró en el tiempo
1. abre el tab **Historial**,
2. selecciona dos capturas con los botones **A** y **B**,
3. revisa el panel de comparación — deltas de CPU / RAM / I/O / Alertas en verde o rojo.

---

## 6) Uso desde la consola de Windows (CLI)

El binario `rootcause.exe` funciona también como herramienta de línea de comandos.

### Ver ayuda completa
```
rootcause --help
rootcause --version
```

### Estado y datos en tiempo real
```
rootcause status          # estado del sistema (severidad, CPU, RAM, I/O, alertas)
rootcause status --json   # lo mismo en JSON para integraciones
rootcause snapshot        # snapshot completo en JSON a stdout
rootcause snapshot --output C:\diag\snapshot.json
rootcause history [N]     # últimas N filas del historial (default 10)
rootcause history [N] --json
rootcause incidents [N]   # incidentes persistidos con evidencia resumida
rootcause export          # exporta snapshot a JSON en Descargas/Documentos
```

### Configuración e IA opcional
```
rootcause config show
rootcause config show --json
rootcause config init
rootcause ai explain-latest
```

La IA es opcional. Si no la habilitas en `rootcause-config.json`, RootCause sigue operando normal.

### Autostart y persistencia
```
rootcause autostart               # lista entradas Registro Run + Startup + Tareas programadas
rootcause autostart --json        # lo mismo en JSON para integraciones
```

### Acciones de intervención
```
rootcause kill <PID>              # finaliza proceso (respeta política de protección)
rootcause block-ip <IP>           # bloquea IP vía firewall de Windows
rootcause stop-service <nombre>   # detiene servicio permitido (bits, dosvc, sysmain, wuauserv)
```

### Captura ETL con WPR
```
rootcause wpr start [--note "descripción"]   # inicia captura ETL
rootcause wpr stop  [--note "descripción"]   # detiene y guarda .etl
rootcause wpr cancel                          # cancela sin guardar
rootcause wpr analyze                         # resume el último .etl
```

### Lanzar GUI explícitamente
```
rootcause --gui
```

---

## 7) Atajos de teclado (GUI)

| Atajo        | Acción                        |
|:-------------|:------------------------------|
| `F5`         | Actualizar ahora              |
| `Ctrl + E`   | Exportar snapshot a JSON      |
| `Ctrl + 1`   | Ir a tab Resumen              |
| `Ctrl + 2`   | Ir a tab Procesos             |
| `Ctrl + 3`   | Ir a tab Conexiones           |
| `Ctrl + 4`   | Ir a tab Temporales           |
| `Ctrl + 5`   | Ir a tab ETW / WPR            |
| `Ctrl + 6`   | Ir a tab Servicios            |
| `Ctrl + 7`   | Ir a tab Autostart            |
| `Ctrl + 8`   | Ir a tab Historial            |
| `Ctrl + 9`   | Ir a tab Acerca               |

---

## 8) Características del equipo (tab Resumen)

Al final del tab **Resumen** aparece una sección "▸ Características del equipo" con:
- nombre del equipo,
- sistema operativo y versión,
- arquitectura de CPU,
- marca, número de núcleos y frecuencia del procesador,
- RAM total.

Los mismos datos aparecen en el tab **Acerca**, junto con la tabla de atajos de teclado.

---

## 9) Funciones nuevas en v0.6

### Sparklines (tab Overview)
Muestra las últimas 60 muestras de CPU%, RAM% e I/O Write como mini-gráficos de línea. Útil para identificar picos recientes sin haber estado mirando la pantalla en ese momento.

### Filtro de severidad (tab Procesos)
Botones **Critical / Warning / Normal / Todos** encima de la tabla. Concentra la vista en lo que importa cuando hay muchos procesos activos.

### Notificaciones toast
Si hay un proceso con severidad **Critical**, la app envía una notificación de Windows en segundo plano (no congela la UI). El cooldown ahora sale de configuración local. Activar/desactivar con el checkbox 🔔 en el header.

### Command line de proceso
Los procesos Critical o con I/O > 20 MB muestran el command line completo del proceso en la tabla. Útil para distinguir instancias del mismo ejecutable lanzadas con parámetros distintos.

### Instalación silenciosa
El instalador Inno Setup ahora acepta parámetros de despliegue corporativo:
```
RootCause-Setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```

### Interfaz de línea de comandos (CLI)
El binario funciona también como CLI. `rootcause --help` muestra todos los comandos disponibles. Ahora incluye salida JSON para integraciones, consulta de incidentes persistidos, configuración local e IA opcional.

### Atajos de teclado
`F5` para refrescar, `Ctrl+E` para exportar, `Ctrl+1…9` para cambiar de tab sin usar el ratón. Ver sección 7.

### Características del equipo
Nueva sección al final del tab **Resumen** y en el tab **Acerca**: muestra OS, CPU, núcleos, frecuencia y RAM total del equipo. Ver sección 8.

### Tab Acerca
Nueva pestaña con versión del producto, autor, enlaces a GitHub/GitLab, stack técnico, atajos de teclado y hardware del equipo.

---

## 10) Tab Autostart (v0.10)

El tab **Autostart** (Ctrl+7) muestra qué programas se configuran para ejecutarse con Windows:

- **Registro Run (Usuario)** — `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- **Registro Run (Sistema)** — `HKLM\Software\Microsoft\Windows\CurrentVersion\Run`
- **Carpeta Startup (Usuario)** — `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`
- **Carpeta Startup (Todos)** — `%ProgramData%\Microsoft\Windows\Start Menu\Programs\Startup`

### Cómo leer la tabla

| Columna      | Qué indica                                                          |
|:-------------|:--------------------------------------------------------------------|
| Dot color    | Severidad heurística: verde = normal, amarillo = revisar, rojo = sospechoso |
| Nombre       | Nombre de la entrada o archivo                                       |
| Tipo         | Origen: Registro (Usuario/Sistema) o Startup (Usuario/Todos)        |
| Comando/Ruta | Ejecutable o script configurado — tooltip muestra ruta completa     |
| En disco     | ✓ = el archivo existe · ✗ = el archivo ya no existe                |

### Qué hacer con entradas "✗ No"
Las entradas que no existen en disco son residuos de software desinstalado. Se pueden limpiar manualmente desde el Editor del Registro (`regedit`) o desde `msconfig` → pestaña Inicio.

### Entradas de tipo Registro (Sistema)
Requieren privilegios de administrador para modificarse. Son más difíciles de eliminar desde una cuenta estándar y por eso se marcan con color diferente.
