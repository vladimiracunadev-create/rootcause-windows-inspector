# Arquitectura y escalabilidad del proyecto

## 1. Objetivo de esta arquitectura

La arquitectura de RootCause busca equilibrar cinco cosas al mismo tiempo:

- claridad para el usuario,
- bajo consumo de recursos,
- modularidad del código,
- facilidad de build y despliegue,
- y capacidad de crecer sin destruir lo ya construido.

---

## 2. Arquitectura actual, vista general

```text
┌────────────────────────────────────────────────────┐
│                    Interfaz UI                     │
│ semáforo, tablas, acciones, estado, resumen ETL   │
└────────────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────┐
│                Capa de orquestación                │
│ coordinador de muestras, refresco y decisiones    │
└────────────────────────────────────────────────────┘
                        │
        ┌───────────────┼────────────────┬───────────────┐
        ▼               ▼                ▼               ▼
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ Procesos/I-O │ │ Temp y caché │ │ Red / IP     │ │ Windows/ETW  │
└──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘
        │               │                │               │
        └───────────────┴────────────────┴───────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────┐
│                 Modelos de dominio                 │
│ hallazgos, snapshots, severidades, acciones       │
└────────────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────┐
│                Persistencia / evidencia            │
│ SQLite, JSON, ETL, XML exportado, summary.txt     │
└────────────────────────────────────────────────────┘
```

---

## 3. Principios arquitectónicos

### 3.1 Diagnóstico antes que automatismo
La aplicación prioriza explicar el problema antes de ejecutar acciones.

### 3.2 Bajo acoplamiento
Cada servicio debe poder evolucionar sin romper completamente la UI o la persistencia.

### 3.3 Bajo impacto operativo
La herramienta no debe convertirse en el nuevo problema de consumo.

### 3.4 Escalamiento progresivo
La precisión más profunda se activa cuando hace falta, no siempre.

### 3.5 Trazabilidad
La evidencia debe quedar guardada y ser exportable.

---

## 4. Módulos principales

### `main.rs`
Punto de entrada. Detecta args CLI → `cli::run()` o GUI.

### `meta.rs`
Constantes del producto: `VERSION`, `DISPLAY_NAME`, `AUTHOR`, `EMAIL`, `GITHUB`, `GITLAB`, `LICENSE`. Único lugar de verdad.

### `cli.rs`
Interfaz de línea de comandos completa: `--help`, `--version`, `status`, `snapshot`, `history [N]`, `export`, `wpr start/stop/cancel/analyze`, `kill`, `block-ip`, `stop-service`, `--gui`.

### `app.rs`
Capa de presentación y estado visual.

### `models.rs`
Tipos compartidos y contratos internos.

### `services/inspector.rs`
Orquestación de observación general.

### `services/temp_scan.rs`
Análisis de temporales y cachés.

### `services/network.rs`
Correlación de conexiones y procesos.

### `services/windows.rs`
Servicios, eventos y automatización Windows.

### `services/persistence.rs`
Persistencia local y soporte a evidencia.

### `services/etl.rs`
Captura y resumen del modo de precisión.

---

## 5. Por qué esta arquitectura escala bien

### A. Porque separa responsabilidades
No mezcla UI, acceso al sistema y persistencia en un mismo archivo gigante.

### B. Porque admite reemplazo por capas
Por ejemplo:

- `network.rs` puede volverse más sofisticado,
- `etl.rs` puede pasar de heurísticas simples a análisis más ricos,
- `persistence.rs` puede pasar de SQLite a otra estrategia si hiciera falta.

### C. Porque el flujo es entendible
Eso baja el costo de onboarding y mantenimiento.

### D. Porque la documentación acompaña la estructura
Una arquitectura que no se explica bien escala peor, aunque el código sea bueno.

---

## 6. Escenarios de escalamiento

## Escenario 1: más precisión sin reescribir la UI
Objetivo:

- mejorar análisis de ETL,
- correlacionar ventanas temporales,
- refinar reglas.

Impacto:

- principalmente `services/etl.rs`,
- quizá nuevos modelos,
- cambios menores en `app.rs`.

## Escenario 2: más fuentes de observación
Objetivo:

- sumar más servicios,
- detectar más rutas,
- añadir nuevas reglas de sospecha.

Impacto:

- nuevos módulos en `services/`,
- nuevos tipos en `models.rs`,
- paneles extra en UI.

## Escenario 3: producto más corporativo
Objetivo:

- firma digital,
- instalador más robusto,
- actualización controlada,
- observabilidad opcional.

Impacto:

- empaquetado,
- pipeline,
- políticas de seguridad,
- ciclo de release.

## Escenario 4: versión pro / empresarial
Objetivo:

- políticas de bloqueo más avanzadas,
- perfiles de captura,
- reportes exportables,
- quizá backend opcional.

Impacto:

- podría aparecer una nueva capa de sincronización o servicio.

---

## 7. Qué conviene no hacer si el proyecto crece

### No mezclar todo en `app.rs`
La UI debe seguir siendo una capa de presentación, no el cerebro de negocio completo.

### No meter lógica Windows profunda dispersa
Todo acceso a sistema debería concentrarse en servicios bien nombrados.

### No usar acciones destructivas sin modelo de permisos
Matar procesos o bloquear IP debe mantenerse bajo control.

### No depender siempre del modo de precisión
La app debe seguir siendo útil incluso sin ETW profundo.

---

## 8. Ruta técnica de escalamiento recomendada

### Fase 1: consolidación
- compilar estable en Windows,
- cerrar warnings,
- ampliar tests,
- endurecer CI.

### Fase 2: robustez funcional
- más cobertura de casos reales,
- mejores heurísticas,
- mejor resumen ETL,
- validación de consumo.

### Fase 3: distribución profesional
- instalador pulido,
- firma digital,
- branding consistente,
- release reproducible.

### Fase 4: capacidades avanzadas
- correlación temporal avanzada,
- reportes más ricos,
- perfiles de captura,
- más automatización controlada.

---

## 9. Relación entre arquitectura y reclutamiento

Esta arquitectura comunica algo importante a un reclutador técnico:

- el proyecto no es improvisado,
- hay intención de separación de responsabilidades,
- existe visión de producto,
- y el crecimiento futuro ya fue pensado.

Eso es valioso porque muestra criterio de diseño, no solo capacidad de “hacer que algo corra”.

---

## 10. Conclusión

La arquitectura actual permite que RootCause empiece como una herramienta Windows útil y evolucione hacia una solución más robusta sin necesitar una reescritura completa.

Su mayor fortaleza arquitectónica está en esto:

> **observación liviana para el día a día, precisión más profunda cuando el caso lo exige y una estructura modular capaz de crecer.**

