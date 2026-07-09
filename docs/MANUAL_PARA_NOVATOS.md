# 📘 Manual para novatos: uso del proyecto sin saber de software

> Guía en lenguaje simple para entender **qué hace RootCause** y **cómo se usa sin perderse**, sin necesidad de saber programar.
>
> ¿Quieres el detalle claro de **qué es cada cosa** (cada pestaña, cada término, con glosario)? Ese está en el **[Manual de usuario](MANUAL_USUARIO.md)**. Este documento es la versión introductoria.

## 1. Para quién es este manual

Este documento está pensado para una persona que:

- no programa,
- no conoce Rust,
- no conoce GitHub Actions,
- no conoce ETW, WPR o WPA,
- y solo quiere entender **qué hace este proyecto** y **cómo se usa sin perderse**.

---

## 2. Qué es este software, dicho en simple

Este software intenta descubrir por qué un computador con Windows se pone lento.

Ejemplos de problemas que quiere detectar:

- un programa que usa mucho disco,
- un programa que gasta demasiada memoria,
- una actualización de Windows corriendo en segundo plano,
- una carpeta temporal que creció demasiado,
- una conexión rara a internet,
- un proceso que parece fuera de lugar.

> No está pensado para "arreglar todo automáticamente".
> Está pensado para **mostrarte el problema principal con claridad**.

---

## 3. Qué significa "repositorio"

Un repositorio es una carpeta organizada con:

- código fuente,
- documentación,
- scripts,
- archivos de configuración.

En este caso, el repositorio es la casa completa del proyecto.

> No es solo el programa: también incluye las instrucciones para construirlo, probarlo y entenderlo.

---

## 4. Cómo obtener el programa (ya no necesitas compilarlo)

El `.exe` es el archivo final ejecutable de Windows. **Ya hay versiones publicadas listas para usar** — no necesitas saber programar ni compilar nada:

- **Descarga directa:** entra a la [página del producto](https://vladimiracunadev-create.github.io/rootcause-windows-inspector/) o a las [Releases en GitHub](https://github.com/vladimiracunadev-create/rootcause-windows-inspector/releases/latest) y baja **`RootCause-Portable.zip`** (o el `rootcause.exe` suelto). Lo extraes y lo abres. Eso es todo.
- **La primera vez** Windows puede mostrar un aviso de "editor desconocido" (SmartScreen), porque el programa aún no tiene firma digital. Es esperable: pulsa *Más información → Ejecutar de todas formas*.

El **código fuente** también está en el repositorio (junto con la documentación) por si quieres revisarlo o **construir tú mismo** el ejecutable — la guía está en `docs/BUILD_WINDOWS.md`. Pero para solo usarlo, con la descarga basta.

> En versiones muy antiguas el repositorio no incluía binario y había que compilarlo; **eso ya cambió**: hoy cada versión se publica con su `.exe` verificado.

---

## 5. Qué partes importantes verás

| Parte | Qué es |
|---|---|
| `README.md` | La puerta de entrada. Explica qué hace el proyecto, qué incluye, qué no incluye y por dónde empezar. |
| Carpeta `docs/` | La biblioteca del proyecto. Allí están los manuales y explicaciones. |
| Carpeta `src/` | Donde vive el programa. |
| Carpeta `scripts/` | Ayudas automáticas para compilar, validar o empaquetar. |
| Carpeta `packaging/` | Lo necesario para el instalador de Windows. |

---

## 6. Qué hace el programa cuando funciona

Cuando el programa corre, debería ayudarte a ver cosas como estas:

- cuál proceso está cargando más el equipo,
- si la memoria está demasiado alta,
- si el disco está bajo mucha presión,
- si hay muchas conexiones de red,
- si hay una carpeta temporal que creció mucho,
- si Windows Update está involucrado,
- si existe una alerta importante.

La idea es que la interfaz sea clara y use colores tipo semáforo:

| Color | Significado |
|---|---|
| 🟢 Verde | estable |
| 🟡 Amarillo | atención |
| 🔴 Rojo | problema importante |

---

## 7. Qué significa "modo de precisión"

A veces un problema dura poco o no se ve claramente en la observación normal. Para eso existe el modo de precisión.

En simple:

1. el programa empieza a grabar una traza,
2. ocurre el problema,
3. el programa detiene la grabación,
4. se genera evidencia,
5. luego esa evidencia se resume o se analiza con más detalle.

> Piensa en esto como una "caja negra" temporal del sistema.

---

## 8. Qué cosas podrías hacer como usuario

Esta tabla resume, según el síntoma, qué conviene mirar primero:

| Problema | Qué mirar |
|---|---|
| **A. El disco se pone al 100%** | procesos dominantes, temporales, servicios relacionados a Windows, y si hace falta activas el modo de precisión. |
| **B. Internet se pone lento** | conexiones activas, IP remotas, proceso asociado, y si parece sospechoso, revisas si bloquearlo tiene sentido. |
| **C. Windows se vuelve lento después de un rato** | memoria, rutas temporales, estado de servicios, y usas captura ETL cuando el problema aparezca. |

---

## 9. Qué no debes esperar

No debes esperar que el software:

- arregle todos los problemas solo,
- reemplace completamente herramientas profundas de Microsoft,
- tome decisiones perfectas siempre,
- permita matar cualquier proceso del sistema sin riesgo.

> Es una herramienta de diagnóstico y apoyo a la decisión.

---

## 10. Cómo se construye el ejecutable en palabras simples

En una máquina Windows con el entorno listo, se usa una consola para ejecutar comandos.

Los pasos generales son:

1. instalar Rust,
2. instalar herramientas necesarias,
3. abrir la carpeta del proyecto,
4. ejecutar el comando de compilación,
5. revisar si generó el ejecutable.

La guía detallada está en:

- `docs/BUILD_WINDOWS.md`
- `docs/COMMANDS.md`

---

## 11. Qué significa CI o GitHub Actions, explicado sin tecnicismos

Cuando el proyecto se sube a GitHub, puede quedar configurado para que GitHub lo revise automáticamente.

Eso permite verificar si:

- el código está ordenado,
- compila,
- pasa pruebas,
- y está en mejor estado para ser distribuido.

> Eso ayuda a no depender solo de "creo que funciona".

---

## 12. Qué significa empaquetado e instalador

| Camino | Qué es |
|---|---|
| **Ejecutable portable** | El programa listo para abrir directamente. |
| **Instalador** | El paquete que guía la instalación en Windows, crea accesos y deja el programa en una carpeta ordenada. |

> El proyecto contempla ambos caminos.

---

## 13. Qué mirar primero si no sabes nada

Orden recomendado:

1. `README.md`
2. `docs/MANUAL_USUARIO.md` (qué es cada cosa, en claro — con glosario)
3. `docs/RECLUTADORES.md`
4. `docs/REPOSITORIO_ANALISIS.md`
5. `docs/RUST_PARA_ROOTCAUSE.md` si luego quieres entender más.

---

## 14. Resumen final

Este proyecto es una herramienta para Windows que busca encontrar la causa principal de problemas de rendimiento.

Su valor está en:

- observar,
- explicar,
- dejar evidencia,
- y ayudar a actuar con más criterio.

> Aunque contiene código, también contiene manuales para que una persona no técnica pueda entender qué resuelve y por qué tiene sentido.
