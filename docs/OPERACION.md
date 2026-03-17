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

## 6) Funciones nuevas en v0.6

### Sparklines (tab Overview)
Muestra las últimas 60 muestras de CPU%, RAM% e I/O Write como mini-gráficos de línea. Útil para identificar picos recientes sin haber estado mirando la pantalla en ese momento.

### Filtro de severidad (tab Procesos)
Botones **Critical / Warning / Normal / Todos** encima de la tabla. Concentra la vista en lo que importa cuando hay muchos procesos activos.

### Notificaciones toast
Si hay un proceso con severidad **Critical**, la app envía una notificación de Windows en segundo plano (no congela la UI). Cooldown de 90 segundos entre notificaciones del mismo tipo. Activar/desactivar con el checkbox 🔔 en el header.

### Command line de proceso
Los procesos Critical o con I/O > 20 MB muestran el command line completo del proceso en la tabla. Útil para distinguir instancias del mismo ejecutable lanzadas con parámetros distintos.

### Instalación silenciosa
El instalador Inno Setup ahora acepta parámetros de despliegue corporativo:
```
RootCause-Setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART
```
