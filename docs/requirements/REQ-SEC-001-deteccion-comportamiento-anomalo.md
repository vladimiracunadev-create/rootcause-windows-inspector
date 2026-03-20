# REQ-SEC-001 - Deteccion de comportamiento anomalo y posible actividad maliciosa

## 1. ID del requerimiento

`REQ-SEC-001`

## 2. Nombre

Deteccion de comportamiento anomalo y posible actividad maliciosa.

## 3. Estado

`planned`

Justificacion del estado:

- RootCause ya tiene base tecnica para correlacionar procesos, red, historial, reglas e incidentes.
- El roadmap vigente sigue priorizando distribucion formal y consolidacion operativa antes de ampliar alcance de seguridad.
- Por honestidad tecnica, este requerimiento se documenta como evolucion planificada y no como capacidad disponible hoy.

## 4. Prioridad

Alta estrategica.

## 5. Objetivo

Extender RootCause para detectar y priorizar senales compatibles con actividad no autorizada o potencialmente maliciosa dentro del endpoint, usando heuristicas y correlacion de evidencia tecnica, sin abandonar su funcion principal de observabilidad, diagnostico y causa raiz.

## 6. Problema que resuelve

Una degradacion de rendimiento, una inestabilidad recurrente o una actividad de red inesperada no siempre tienen origen puramente operacional. En escenarios reales, un proceso no autorizado, una persistencia sospechosa o un patron de ejecucion anomalo pueden manifestarse primero como lentitud, picos de I/O, uso inusual de memoria o trafico saliente poco explicable.

Hoy RootCause ayuda a ver sintomas, procesos dominantes y evidencia local. Este requerimiento formaliza una evolucion para convertir esas senales en hipotesis tecnicas mas claras sobre posible actividad maliciosa o no autorizada.

## 7. Alcance

- Deteccion heuristica de procesos sospechosos a partir de contexto, ruta, comportamiento y severidad.
- Identificacion de consumo anomalo de CPU, RAM, disco o red que no calza con lineas base simples del equipo.
- Deteccion de conexiones salientes inusuales por proceso, destino o patron temporal.
- Observacion de persistencia sospechosa en mecanismos conocidos de inicio automatico o continuidad.
- Senalizacion de rutas de ejecucion sospechosas, por ejemplo perfiles de usuario, temporales o ubicaciones no esperadas.
- Correlacion de multiples senales para elevar severidad, consolidar evidencia y proponer una hipotesis de causa raiz.
- Presentacion de evidencia tecnica y sugerencias de mitigacion no destructivas.
- Relacion explicita entre hallazgos de seguridad y sintomas operativos como lentitud, degradacion o inestabilidad.

## 8. No alcance

- No implica motor antivirus por firmas.
- No implica cuarentena, limpieza automatica o remediacion agresiva por defecto.
- No implica EDR completo, consola centralizada, telemetria remota obligatoria ni respuesta a incidentes empresarial.
- No implica deteccion perfecta de malware, ransomware, gusanos o herramientas fileless.
- No implica analisis de memoria profunda, inspeccion de kernel ni monitoreo de red equivalente a NDR.

## 9. Motivacion tecnica

El producto ya dispone de piezas que hacen razonable esta evolucion:

- inventario de procesos con severidad y puntaje;
- correlacion de conexiones de red con PID;
- historial persistente e incidentes resumidos;
- motor ligero de reglas y evidencia asociada;
- exportacion JSON y captura ETW/WPR para ampliar contexto.

La oportunidad tecnica no es convertir RootCause en otra categoria de producto, sino enriquecer su motor de observabilidad para que tambien identifique patrones compatibles con abuso, ejecucion no autorizada o degradacion inducida.

## 10. Casos o escenarios de uso

- Un equipo presenta lentitud sostenida y RootCause observa un proceso desde `%AppData%` con I/O alto, red saliente constante y persistencia sospechosa.
- Un ejecutable poco esperado opera desde `%TEMP%`, consume CPU de forma irregular y abre conexiones externas no habituales.
- Un caso de degradacion del disco muestra escrituras masivas no explicadas; RootCause deberia poder sugerir una hipotesis de cifrado masivo no autorizado o actividad destructiva similar, sin declararlo como confirmacion.
- Un servicio o tarea programada reaparece despues de ser detenido y su rastro coincide con rutas de ejecucion no estandar.
- Un usuario reporta "internet lenta" y el producto detecta correlacion entre proceso desconocido, trafico saliente persistente y crecimiento de temporales.

## 11. Criterios de aceptacion

- El producto puede emitir incidentes etiquetados como "anomalia compatible con actividad no autorizada" sin afirmar infeccion confirmada.
- Cada incidente de esta categoria debe incluir severidad, evidencia tecnica y explicacion breve de por que se genero.
- La correlacion de senales debe elevar la severidad solo cuando existan al menos dos fuentes de evidencia compatibles.
- La salida debe mantenerse disponible en GUI, CLI, exportaciones y almacenamiento historico.
- El sistema debe distinguir entre "anomalia operacional" y "anomalia con componente potencial de seguridad" cuando el contexto lo permita.
- Las sugerencias de mitigacion deben ser controladas, reversibles cuando aplique y claramente separadas de una accion automatica.
- La documentacion y el producto deben indicar explicitamente que esta capacidad no reemplaza una solucion antivirus o EDR especializada.

## 12. Riesgos

- Falsos positivos en software legitimo con comportamiento atipico.
- Expectativas comerciales incorrectas si se comunica como seguridad completa.
- Sobreajuste de heuristicas a unos pocos casos y baja generalizacion.
- Mayor carga local si se agregan demasiadas comprobaciones sin control de frecuencia.
- Riesgo de que usuarios tomen acciones destructivas sobre procesos legitimos si la UX no explica bien el contexto.

## 13. Dependencias

- Motor de reglas y correlacion ya existente.
- Persistencia local de incidentes e historial.
- Configuracion operativa para umbrales y politicas.
- Recoleccion de procesos, red, servicios, temporales y, cuando aplique, ETW/WPR.
- Futuras capacidades de autostart, tray o service pueden ampliar cobertura, pero no son requisito para documentar ni iniciar esta linea.

## 14. Impacto en arquitectura

No requiere reescritura completa de la arquitectura actual, pero si una evolucion controlada de varios componentes:

- ampliacion del catalogo de senales y scoring en el motor de reglas;
- nuevos campos de evidencia, confianza y tipo de incidente en modelos persistidos;
- mayor parametrizacion de umbrales y exclusiones en configuracion;
- vistas especificas en UI y CLI para "hallazgos compatibles con seguridad";
- pruebas de regresion con datasets y escenarios representativos.

## 15. Consideraciones de UX/UI

- La interfaz debe explicar por que un hallazgo fue marcado y con que evidencia.
- La etiqueta visual debe evitar lenguaje alarmista o concluyente.
- La severidad debe convivir con un nivel de confianza o contexto para reducir interpretaciones binarias.
- Las sugerencias deben priorizar acciones de verificacion, aislamiento o captura de evidencia antes que medidas irreversibles.
- El usuario debe poder distinguir con claridad un problema de rendimiento puro de una anomalia que tambien merece revision de seguridad.

## 16. Consideraciones de seguridad

- Evitar que el detalle publico de heuristicas facilite evasiones triviales.
- Mantener el procesamiento local por defecto y no introducir dependencia de nube obligatoria.
- Registrar evidencia suficiente para auditoria posterior.
- No ejecutar respuesta automatica agresiva sin confirmacion explicita.
- Considerar exclusiones y listas de confianza locales para reducir ruido, pero sin borrar trazabilidad.

## 17. Limitaciones

- Un comportamiento malicioso puede parecer normal o pasar desapercibido si opera con bajo perfil.
- El producto depende del nivel de visibilidad que permitan Windows y los privilegios disponibles.
- Sin sensores adicionales, RootCause no cubrira todas las tecnicas de persistencia o movimiento lateral.
- La clasificacion seguira siendo heuristica y contextual, no confirmacion definitiva.
- Un atacante con tecnicas avanzadas o privilegios altos puede degradar o evadir una herramienta local.

## 18. Propuesta de implementacion por fases

### Fase 1 - Base semantica y modelo de evidencia

- Definir taxonomia de senales anomalas y categorias de incidente.
- Separar incidentes operacionales de incidentes con posible componente de seguridad.
- Establecer puntajes, severidades y mensajes explicativos iniciales.

### Fase 2 - Nuevas fuentes y correlacion minima viable

- Incorporar verificacion de rutas de ejecucion sospechosas.
- Incorporar observacion de persistencia sospechosa.
- Correlacionar procesos, red y anomalias de consumo con evidencia historica simple.

### Fase 3 - UX, CLI y exportacion

- Mostrar hallazgos en GUI y CLI con evidencia, contexto y acciones sugeridas.
- Persistir incidentes enriquecidos y exportarlos de forma trazable.
- Agregar filtros por categoria y severidad.

### Fase 4 - Validacion y afinado

- Medir falsos positivos en software legitimo frecuente.
- Probar escenarios de alto I/O, conexiones salientes anormales y patrones de ejecucion no autorizada.
- Ajustar umbrales y explicaciones.

### Fase 5 - Patrones avanzados opcionales

- Evaluar deteccion de escrituras masivas compatibles con cifrado no autorizado.
- Evaluar patrones compatibles con gusanos o abuso de recursos a gran escala.
- Mantener siempre el posicionamiento de "compatibilidad con senales", no de deteccion perfecta.

## 19. Roadmap sugerido

- `Post-v1.0`: formalizar catalogo de senales, scoring y nomenclatura de incidentes.
- `Phase 2`: correlacionar procesos, red, persistencia y rutas sospechosas en el endpoint.
- `Phase 3`: enriquecer interfaz, exportaciones y sugerencias de mitigacion.
- `Phase 4`: validar escenarios, medir ruido y documentar limites operativos.

## 20. Metricas o senales de validacion

- Tiempo medio hasta identificar el proceso o artefacto sospechoso dominante.
- Porcentaje de incidentes con evidencia tecnica suficiente para auditoria humana.
- Tasa de falsos positivos observada en software legitimo comun.
- Impacto de CPU y memoria de la propia deteccion adicional.
- Cantidad de hallazgos que explican simultaneamente degradacion operativa y posible actividad no autorizada.

## 21. Relacion con la vision de RootCause

Este requerimiento encaja con la vision del producto porque parte del mismo principio: no mostrar solo sintomas, sino ayudar a explicar la causa dominante. En algunos casos esa causa no sera un problema de configuracion o rendimiento, sino un comportamiento anomalo con posible implicancia de seguridad. RootCause puede evolucionar para detectar esas senales sin dejar de ser una herramienta de observabilidad y diagnostico del endpoint.

## 22. Nota de posicionamiento comercial honesto

RootCause puede evolucionar para detectar señales compatibles con actividad maliciosa o no autorizada, sin reemplazar una solucion antivirus o EDR especializada.

## 23. Trazabilidad documental

- Registro de requerimientos: [docs/requirements/README.md](README.md)
- Punto de entrada del repo: [README.md](../../README.md)
- Indice documental general: [docs/INDEX.md](../INDEX.md)
- Roadmap tecnico: [docs/ROADMAP.md](../ROADMAP.md)
- Reflejo publico resumido: [landing/index.html](../../landing/index.html)
