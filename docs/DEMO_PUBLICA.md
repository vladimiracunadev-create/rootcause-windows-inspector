# RootCause Demo: distribución pública controlada

Este documento define cómo publicar **RootCause Demo** como perfil público de evaluación del producto (el repositorio es público bajo Apache 2.0; la demo es un perfil de distribución con branding y mensajes orientados a evaluación).

> `RootCause Demo` no es otro motor del producto: es un **perfil de distribución** sobre la edición GUI principal, con branding y mensajes más explícitos para evaluación.

## Objetivo de la demo

La demo pública debe servir para:

- mostrar valor real del producto,
- permitir instalación y prueba guiada,
- capturar retroalimentación sobre problemas reales de Windows,
- demostrar un enfoque profesional y orientado a resultados,
- dejar claro que la demo **diagnostica antes de actuar**.

## Qué se publica

Se recomienda publicar solo estos artefactos:

- `RootCause-Demo-Setup.exe` como instalador principal,
- opcionalmente `RootCause-Demo-Portable.zip` como variante sin instalación,
- `SHA256SUMS.txt` para integridad,
- página web de descarga con explicación simple,
- mini manual de uso previo.

## Qué NO se publica

- el repositorio fuente,
- scripts internos de desarrollo que no aporten al usuario final,
- trazas ETL de ejemplo con datos privados,
- credenciales,
- automatizaciones de build internas.

## Mensaje recomendado para la web

> RootCause Demo es una utilidad experimental para Windows orientada a detectar causas reales de lentitud: procesos pesados, temporales anómalos, uso de disco, memoria, servicios de Windows y conexiones sospechosas. No promete acelerar mágicamente el sistema; ayuda a identificar la causa dominante y actuar con criterio.

## Flujo recomendado de descarga pública

1. El usuario entra a la página de descarga.
2. Lee qué hace la demo y qué no hace.
3. Revisa requisitos mínimos y advertencias.
4. Descarga el instalador o la versión portable.
5. Revisa la guía de uso previo.
6. Ejecuta RootCause Demo en modo observación.
7. Solo después decide si quiere terminar procesos, detener servicios permitidos o capturar una traza ETW.

## Mensajes clave que deben ser visibles antes de descargar

- Esta es una **demo funcional**, no una versión comercial final.
- Algunas acciones avanzadas pueden requerir permisos de administrador.
- La herramienta no reemplaza antivirus ni soporte técnico.
- La herramienta no corrige todo con un clic mágico.
- Las acciones destructivas deben confirmar el riesgo.

## Artefactos recomendados en la web

- título corto,
- texto de 3 a 5 líneas,
- botón de descarga,
- resumen de requisitos,
- advertencia breve,
- hash SHA-256,
- fecha de build,
- número de versión.
