# 🔒 Política de seguridad

RootCause Windows Inspector es una herramienta de diagnóstico que puede terminar procesos, tocar reglas de firewall y detener servicios. Esta política resume cómo reportar vulnerabilidades, qué versiones reciben correcciones y los principios de seguridad que rigen el producto.

## 📣 Cómo reportar una vulnerabilidad

Si encuentras un problema de seguridad, repórtalo de forma responsable:

1. **No** abras un issue público con los detalles del exploit.
2. Escribe a `vladimir.acuna.dev@gmail.com` con el asunto `SECURITY: RootCause`.
3. Incluye pasos de reproducción, versión afectada y hash SHA-256 del artefacto si aplica.
4. Recibirás acuse de recibo y coordinaremos la divulgación una vez publicada la corrección.

## 🗂️ Versiones soportadas

| Versión | Estado | Correcciones de seguridad |
|---|---|---|
| `v0.13.x` | ✅ Activa | Sí |
| `< v0.13` | ⚠️ Obsoleta | Solo actualizando a la versión activa |

> **Nota:** el repositorio es público bajo licencia Apache 2.0.

## 🧭 Principios

- No matar procesos críticos por defecto.
- No bloquear IPs automáticamente.
- No detener servicios fuera de la lista permitida.
- Borrar TEMP solo de forma conservadora y explícita: únicamente tu carpeta
  `%TEMP%`, únicamente lo no modificado en las últimas 24 h, saltando lo que esté
  en uso (bloqueado). Nunca `C:\Windows\Temp`, el sistema ni Windows Update.
- No distribuir binarios opacos dentro del repositorio fuente.
- No prometer precisión forense sin WPR/ETW.

## ⚠️ Riesgos conocidos

| Riesgo | Detalle |
|---|---|
| `taskkill` | Puede requerir privilegios elevados. |
| Reglas de firewall | Requieren permisos adecuados. |
| Detener servicios | Puede impactar funcionalidades del sistema. |
| Limpiar `%TEMP%` | Solo tu `%TEMP%`, solo lo no usado en 24 h y no bloqueado; aun así, borra archivos. Windows impide borrar lo que tiene un handle abierto, por lo que "no en uso" es intrínsecamente seguro. |
| Capturas WPR | Pueden crecer rápido si se usan mal. |
| JSON y ETL | Pueden contener contexto sensible del equipo. |

## 🔁 Política operativa

La secuencia recomendada es:

1. observar,
2. correlacionar,
3. exportar,
4. intervenir,
5. usar WPR si la duda persiste.

## 📦 Distribución

Si distribuyes binarios a terceros, documenta siempre:

- si están firmados o no,
- permisos requeridos,
- límites funcionales,
- formato de instalación,
- hash SHA-256 de los artefactos.

> **Firma digital:** los artefactos actuales **no** están firmados digitalmente. Windows SmartScreen puede mostrar una advertencia al ejecutarlos. Verifica siempre el hash SHA-256 contra `SHA256SUMS.txt` antes de instalar.

## 📄 Licencia

Este software se distribuye bajo **Apache License 2.0**.

Esto implica:

- uso, modificación y redistribución libres con atribución,
- grant explícito de patentes de los contribuidores,
- no se otorgan derechos sobre la marca o nombre del producto.

Ver [`LICENSE`](LICENSE) y [`docs/LICENCIA_Y_DECISION.md`](docs/LICENCIA_Y_DECISION.md) para el razonamiento completo y la ruta futura prevista.
