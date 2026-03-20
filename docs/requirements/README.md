# Requerimientos estrategicos de seguridad y resiliencia

Este registro formaliza dos lineas de evolucion del producto que ya no deben quedar como ideas sueltas ni backlog oculto.

RootCause sigue orientado a observabilidad, diagnostico, rendimiento, salud del sistema, correlacion de eventos y analisis de causa raiz en el endpoint. Estos requerimientos amplian ese alcance sin cambiar su esencia.

## Objetivo de este registro

- Dejar visibles los requerimientos REQ-SEC-001 y REQ-SEC-002 dentro del repositorio.
- Mantener trazabilidad entre README principal, indice documental, roadmap tecnico y landing publica.
- Fijar un lenguaje honesto: RootCause no se presenta aqui como antivirus completo ni como EDR empresarial.

## Criterio de estado y priorizacion

Ambos requerimientos quedan con estado `planned` y prioridad alta porque:

- el repositorio ya tiene capacidades base de heuristicas, correlacion, incidentes, auditoria local e historial persistente;
- el roadmap vigente de `v1.0` sigue centrado en distribucion, autostart, tray y operacion formal;
- por coherencia con el estado real del producto, estas lineas se documentan como evolucion posterior a `v1.0`, no como capacidad ya implementada.

## Registro activo

| ID | Nombre | Estado | Prioridad | Relacion con RootCause |
|---|---|---|---|---|
| [REQ-SEC-001](REQ-SEC-001-deteccion-comportamiento-anomalo.md) | Deteccion de comportamiento anomalo y posible actividad maliciosa | `planned` | Alta estrategica | Extiende la observabilidad del endpoint hacia senales compatibles con actividad no autorizada, sin reemplazar AV/EDR. |
| [REQ-SEC-002](REQ-SEC-002-autoproteccion-y-resiliencia.md) | Autoproteccion y resiliencia del agente RootCause | `planned` | Alta | Refuerza continuidad operativa, integridad y alerta ante manipulacion del propio agente, sin prometer invulnerabilidad. |

## Mensajes obligatorios que este registro preserva

- RootCause puede evolucionar para detectar señales compatibles con actividad maliciosa o no autorizada, sin reemplazar una solucion antivirus o EDR especializada.
- RootCause tambien debe contemplar la resiliencia de su propio agente, ya que una herramienta de diagnostico puede convertirse en objetivo de manipulacion en escenarios reales.
- La documentacion de estos requerimientos queda visible y enlazada, no como una nota aislada, sino como parte explicita del roadmap tecnico del producto.

## Trazabilidad documental

- Punto de entrada principal: [README del repositorio](../../README.md)
- Indice general de documentacion: [docs/INDEX.md](../INDEX.md)
- Roadmap tecnico vivo: [docs/ROADMAP.md](../ROADMAP.md)
- Referencia publica resumida: [landing/index.html](../../landing/index.html)
- Guia de publicacion de la landing: [landing/README.md](../../landing/README.md)

## Regla de mantenimiento

- Si cambia el estado de cualquiera de estos requerimientos, actualizar este registro y `docs/ROADMAP.md`.
- Si cambia el posicionamiento comercial del producto, revisar primero ambos requerimientos y luego el `README.md`.
- La landing publica solo debe reflejar estas lineas a nivel estrategico y sin detallar heuristicas, reglas internas o mecanismos de evasion.
