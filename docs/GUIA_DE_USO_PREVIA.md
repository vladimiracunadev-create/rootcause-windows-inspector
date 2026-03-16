# Guía de uso previa de RootCause Demo

Lee esto antes de ejecutar la demo.

## 1. Qué hace

RootCause Demo observa el equipo y trata de responder preguntas como:

- qué proceso está forzando el disco,
- qué carpeta temporal está creciendo,
- si Windows Update o Delivery Optimization están activos,
- qué conexión externa aparece vinculada a un proceso,
- si conviene capturar una traza de precisión.

## 2. Qué no hace

- no acelera automáticamente Windows,
- no reemplaza antivirus,
- no es una suite de limpieza genérica,
- no corrige problemas de hardware físico,
- no garantiza identificar todos los archivos exactos en todos los escenarios.

## 3. Cómo usarlo de forma segura

1. Abre RootCause Demo.
2. Espera al menos 2 o 3 ciclos de captura.
3. Revisa el semáforo general.
4. Mira primero el panel de procesos dominantes.
5. Revisa temporales y conexiones activas.
6. Exporta un JSON si quieres guardar evidencia.
7. Solo usa acciones de cierre o bloqueo cuando entiendas qué ocurre.

## 4. Cuándo usar modo de precisión

Usa WPR/ETW si:

- el problema ocurre por ventanas cortas,
- el disco sube al 100 % sin explicación clara,
- sospechas Windows Update o instaladores en segundo plano,
- necesitas evidencia más detallada para revisar después.

## 5. Cuándo no usar acciones agresivas

Evita terminar procesos o detener servicios si:

- no reconoces el ejecutable,
- el equipo está instalando una actualización,
- estás trabajando con archivos sin guardar,
- se trata de un equipo corporativo administrado.
