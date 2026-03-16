# Operación

Esta guía explica cómo usar el software en un caso real de lentitud de Windows.

---

## 1) Flujo mínimo recomendado

1. abre la app,
2. deja correr algunos refrescos,
3. mira el semáforo,
4. revisa “Dónde mirar primero”,
5. identifica si el problema parece venir de:
   - proceso,
   - temporal,
   - red,
   - servicio,
   - update,
6. exporta JSON si necesitas respaldo,
7. activa modo de precisión solo si todavía no basta.

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
