# 🔐 Política de privacidad local de RootCause Demo

RootCause Demo fue planteado para funcionar de forma **local**. Este documento resume qué datos genera, qué nunca hace por defecto y dónde se guardan los archivos.

## 🧭 Enfoque

> RootCause Demo trabaja **en local**: no envía datos a servidores externos salvo acción explícita del usuario.

## 🗂️ Datos que puede generar localmente

- capturas del estado del sistema,
- reportes JSON exportados por el usuario,
- base SQLite local con historial,
- trazas ETL generadas por decisión del usuario,
- archivos de análisis como `summary.txt`, `dumpfile.xml` y `trace-analysis.json`.

## 🚫 Qué no debe hacer por defecto

- no enviar telemetría remota,
- no subir reportes automáticamente,
- no mandar trazas a servidores externos,
- no compartir datos sin acción explícita del usuario.

## 📁 Ubicaciones locales esperadas

- carpeta del usuario para trazas,
- carpeta de instalación para documentos,
- ubicaciones de salida definidas por scripts o exportación manual.

## 🌐 Recomendación para publicación pública

En la web y en el instalador debe indicarse claramente:

- que la app trabaja en local,
- qué archivos puede guardar,
- cómo borrar esos archivos,
- que exportar un reporte es una acción manual del usuario.
