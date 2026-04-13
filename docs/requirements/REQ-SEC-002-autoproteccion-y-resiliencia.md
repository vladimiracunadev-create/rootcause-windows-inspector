# REQ-SEC-002 - Autoproteccion y resiliencia del agente RootCause

## 1. ID del requerimiento

`REQ-SEC-002`

## 2. Nombre

Autoproteccion y resiliencia del agente RootCause.

## 3. Estado

`phase-2-initial`

Justificacion del estado:

- El repositorio ya contempla tray y Windows Service como lineas de evolucion del runtime.
- RootCause guarda historial, evidencia y auditoria local, por lo que la continuidad del agente importa cada vez mas.
- La version actual ya incorpora heartbeat local, deteccion de cierre abrupto previo, visibilidad de salud del agente en GUI/CLI, backoff recomendado y evidencia de integridad de configuracion.
- Aun no existe un supervisor persistente separado tipo servicio ni autoproteccion de nivel sistema; por eso el estado se mantiene como implementacion inicial y no como promesa total.

## 4. Prioridad

Alta.

## 5. Objetivo

Definir una linea de evolucion para que el agente RootCause resista mejor fallos, detenciones inesperadas, corrupcion de configuracion y manipulaciones basicas, y para que deje evidencia cuando su propia ejecucion sea interrumpida o alterada.

## 6. Problema que resuelve

Una herramienta local de diagnostico y observabilidad puede convertirse en objetivo de software malicioso o de un atacante que quiera reducir visibilidad, eliminar trazabilidad o impedir la generacion de evidencia. Incluso sin un ataque sofisticado, tambien puede verse afectada por caidas, corrupcion de estado o errores operativos.

Si RootCause quiere apoyar analisis de causa raiz y, a futuro, senales compatibles con seguridad, necesita contemplar que su propio agente puede ser saboteado, detenido o degradado.

## 7. Alcance

- Watchdog o supervisor para detectar detenciones inesperadas.
- Reinicio automatico controlado cuando aplique.
- Heartbeats o senales de salud del agente.
- Verificacion de integridad de binarios, configuracion y artefactos criticos.
- Proteccion de configuracion frente a cambios no esperados o no autorizados.
- Logging robusto de eventos del propio agente.
- Alertas cuando el agente sea detenido, alterado o entre en modo degradado.
- Mecanismos de recuperacion ante fallo de proceso, corrupcion de almacenamiento o sabotaje basico.

## 8. No alcance

- No implica invulnerabilidad.
- No implica self-defense a nivel kernel ni proteccion absoluta frente a atacante con privilegios altos.
- No implica impedir de forma garantizada la desinstalacion, la depuracion o la sustitucion del binario.
- No implica reemplazar controles del sistema operativo, firma de codigo, Secure Boot o soluciones corporativas de hardening.
- No implica asumir que una herramienta local aislada puede resistir por si sola a un adversario con privilegios de administrador, SYSTEM o kernel.

## 9. Motivacion tecnica

RootCause ya se apoya en componentes persistentes y operativos:

- configuracion local;
- historial SQLite y exportaciones JSON;
- auditoria de acciones;
- skeletons para tray icon y Windows Service;
- CLI y GUI que dependen de continuidad y confianza en el estado local.

Si el producto evoluciona para detectar anomalias compatibles con seguridad, tambien debe poder evidenciar cuando el propio agente deja de correr o es alterado, porque esa interrupcion ya es relevante para el diagnostico.

## 10. Casos o escenarios de uso

- Un proceso malicioso finaliza `rootcause.exe` para evitar que se registren incidentes posteriores.
- Un atacante modifica el archivo de configuracion para desactivar alertas o aumentar umbrales.
- La base de datos local o el registro de auditoria se corrompen tras un cierre abrupto.
- Un supervisor detecta que el agente lleva demasiado tiempo sin heartbeat y genera un evento de recuperacion.
- Durante una actualizacion legitima, el sistema debe diferenciar entre un cambio esperado y una manipulacion sospechosa.

## 11. Criterios de aceptacion

- El sistema puede detectar y registrar una detencion inesperada del agente o de su supervisor.
- Debe existir al menos una estrategia documentada de reinicio automatico controlado o reanudacion.
- Los cambios relevantes en binario, configuracion o artefactos criticos deben poder verificarse o al menos dejar evidencia de integridad.
- La UX o la salida CLI deben indicar cuando el agente esta en modo degradado o recuperado.
- Los eventos de manipulacion o fallo deben persistirse con fecha, contexto y resultado.
- La documentacion debe indicar explicitamente que un atacante con privilegios altos puede comprometer una herramienta local aislada.

## 12. Riesgos

- Bucles de reinicio si la politica de watchdog no tiene backoff o limites.
- Falsos positivos de integridad durante updates legitimos.
- Complejidad operativa mayor en instalacion, servicio o soporte.
- Mayor friccion para usuarios si la proteccion de configuracion es demasiado rigida.
- Sensacion falsa de autoproteccion total si el mensaje comercial no es preciso.

## 13. Dependencias

- Configuracion local y modelos de estado del agente.
- Persistencia de auditoria e incidentes.
- Integracion con Windows Task Scheduler, Service Control Manager o un supervisor equivalente, segun fase.
- Posible mejora futura de firma digital y cadena de release.
- Telemetria local de salud, heartbeats y eventos de arranque/parada.

## 14. Impacto en arquitectura

No exige rehacer la arquitectura completa, pero si introducir piezas nuevas o reforzar las existentes:

- componente supervisor o watchdog separado del proceso principal;
- eventos de salud e integridad dentro del modelo de incidentes;
- proteccion o versionado de configuracion;
- logging mas resistente a interrupciones;
- politicas de recuperacion y backoff;
- integracion futura con la modalidad Windows Service.

## 15. Consideraciones de UX/UI

- Debe existir una forma clara de ver el estado del agente: saludable, degradado, recuperado o potencialmente manipulado.
- Las alertas de autoproteccion no deben sonar apocalipticas; deben explicar impacto y accion sugerida.
- Los reinicios automaticos deben quedar auditados y ser visibles.
- Si una comprobacion de integridad falla, la interfaz debe distinguir entre "cambio esperado" y "cambio pendiente de revisar" cuando el contexto lo permita.

## 16. Consideraciones de seguridad

- Proteger la configuracion critica frente a cambios silenciosos.
- Evitar que el log de autoproteccion se borre o reescriba con facilidad.
- Reducir dependencia de un unico punto de fallo.
- Diseñar actualizaciones legitimas para que no parezcan manipulaciones.
- Mantener el detalle tactico fuera de la landing publica y de mensajes comerciales.

## 17. Limitaciones

- Si un atacante obtiene privilegios altos, una herramienta local aislada puede verse comprometida, detenida o alterada.
- Un watchdog en userland no equivale a proteccion anti-tamper de nivel sistema.
- La verificacion de integridad local puede ser desactivada o falsificada por un adversario suficientemente privilegiado.
- La resiliencia mejora continuidad y trazabilidad, pero no garantiza disponibilidad absoluta.

## 18. Propuesta de implementacion por fases

### Fase 1 - Visibilidad de salud y eventos basicos

- Registrar eventos de arranque, parada, fallo y recuperacion.
- Definir estado del agente y heartbeats minimos.
- Exponer el estado en GUI, CLI y almacenamiento local.

### Fase 2 - Watchdog y reinicio controlado

- Introducir supervisor simple con politica de backoff.
- Detectar detenciones inesperadas y reintentos acotados.
- Registrar cada recuperacion y su resultado.

### Estado implementado hoy

- heartbeat local persistido en archivo de estado del agente;
- deteccion de cierre abrupto previo en el siguiente arranque;
- registro de eventos `agent-start`, `agent-stop`, `agent-recovery` y `config-integrity-change` en auditoria local;
- exposicion visible del estado del agente en GUI, `status --json` y `config show`;
- recomendacion de backoff cuando se observan reinicios/cierres abruptos repetidos dentro de una ventana acotada.

### Fase 3 - Integridad y proteccion de configuracion

- Versionar configuracion critica y validar cambios.
- Incorporar verificacion de integridad de binarios o hashes de referencia cuando sea razonable.
- Alertar ante desviaciones no explicadas.

### Fase 4 - Persistencia robusta y recuperacion

- Reforzar logging y almacenamiento ante cierres abruptos.
- Definir recuperacion desde backup o estado degradado.
- Probar corrupcion de archivos y fallos parciales.

### Fase 5 - Integracion con runtime mas resistente

- Evaluar acoplamiento con Windows Service, tarea programada o supervisor persistente.
- Medir comportamiento bajo sabotaje simple y escenarios de apagado inesperado.
- Ajustar experiencia de actualizacion legitima.

## 19. Roadmap sugerido

- `Post-v1.0`: modelar salud del agente, eventos de parada y recuperacion.
- `Phase 2`: supervisor basico con reinicio y backoff.
- `Phase 3`: integridad de configuracion y binarios criticos.
- `Phase 4`: recuperacion robusta y pruebas de sabotaje o fallo.

## 20. Metricas o senales de validacion

- Tiempo medio de recuperacion tras caida inesperada del agente.
- Porcentaje de detenciones no planificadas detectadas y registradas.
- Tasa de falsos positivos en alertas de integridad durante updates legitimos.
- Cobertura de eventos de salud persistidos correctamente.
- Porcentaje de escenarios de corrupcion basica recuperados sin perdida total de trazabilidad.

## 21. Relacion con la vision de RootCause

La promesa central de RootCause es ayudar a entender que esta degradando el sistema y dejar evidencia util. Esa promesa pierde valor si la propia herramienta puede ser detenida o alterada sin dejar rastro. Por eso la resiliencia del agente no cambia la esencia del producto; la refuerza.

## 22. Nota de posicionamiento comercial honesto

RootCause tambien debe contemplar la resiliencia de su propio agente, ya que una herramienta de diagnostico puede convertirse en objetivo de manipulacion en escenarios reales. Eso no significa invulnerabilidad: si un atacante obtiene privilegios altos, una herramienta local aislada puede verse comprometida.

## 23. Trazabilidad documental

- Registro de requerimientos: [docs/requirements/README.md](README.md)
- Punto de entrada del repo: [README.md](../../README.md)
- Indice documental general: [docs/INDEX.md](../INDEX.md)
- Roadmap tecnico: [docs/ROADMAP.md](../ROADMAP.md)
- Reflejo publico resumido: [landing/index.html](../../landing/index.html)
