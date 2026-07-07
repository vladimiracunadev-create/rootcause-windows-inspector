# Security Notes

## Principios

- No matar procesos críticos por defecto.
- No bloquear IPs automáticamente.
- No detener servicios fuera de la lista permitida.
- No borrar TEMP de forma agresiva en esta versión.
- No distribuir binarios opacos dentro del repositorio fuente.
- No prometer precisión forense sin WPR/ETW.

## Riesgos conocidos

- `taskkill` puede requerir privilegios elevados.
- reglas de firewall requieren permisos adecuados.
- detener servicios puede impactar funcionalidades del sistema.
- capturas WPR pueden crecer rápido si se usan mal.
- JSON y ETL pueden contener contexto sensible del equipo.

## Política operativa

La secuencia recomendada es:

1. observar,
2. correlacionar,
3. exportar,
4. intervenir,
5. usar WPR si la duda persiste.

## Distribución

Si distribuyes binarios a terceros, documenta siempre:

- si están firmados o no,
- permisos requeridos,
- límites funcionales,
- formato de instalación,
- hash SHA-256 de los artefactos.

## Licencia

Este software se distribuye bajo **Apache License 2.0**.

Esto implica:

- uso, modificación y redistribución libres con atribución,
- grant explícito de patentes de los contribuidores,
- no se otorgan derechos sobre la marca o nombre del producto.

Ver [`LICENSE`](LICENSE) y [`docs/LICENCIA_Y_DECISION.md`](docs/LICENCIA_Y_DECISION.md) para el razonamiento completo y la ruta futura prevista.

> **Nota:** el repositorio es público bajo licencia Apache 2.0.
