# Manual para novatos: uso del proyecto sin saber de software

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

No está pensado para “arreglar todo automáticamente”.

Está pensado para **mostrarte el problema principal con claridad**.

---

## 3. Qué significa “repositorio”

Un repositorio es una carpeta organizada con:

- código fuente,
- documentación,
- scripts,
- archivos de configuración.

En este caso, el repositorio es la casa completa del proyecto.

No es solo el programa: también incluye las instrucciones para construirlo, probarlo y entenderlo.

---

## 4. Qué significa que no venga el `.exe`

El `.exe` es el archivo final ejecutable de Windows.

Este repositorio no lo trae incluido.

Eso no significa que falte el proyecto.

Significa que se entrega:

- el código fuente,
- la documentación,
- y la ruta para construir el ejecutable en una máquina Windows.

Esto es normal cuando se quiere:

- revisar el código,
- generar un ejecutable alineado con el sistema real,
- evitar subir binarios opacos,
- y mantener un proceso más profesional.

---

## 5. Qué partes importantes verás

### `README.md`
Es la puerta de entrada.

Te explica:

- qué hace el proyecto,
- qué incluye,
- qué no incluye,
- y por dónde empezar.

### Carpeta `docs/`
Es la biblioteca del proyecto.

Allí están los manuales y explicaciones.

### Carpeta `src/`
Es donde vive el programa.

### Carpeta `scripts/`
Son ayudas automáticas para compilar, validar o empaquetar.

### Carpeta `packaging/`
Contiene lo necesario para el instalador de Windows.

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

- verde: estable,
- amarillo: atención,
- rojo: problema importante.

---

## 7. Qué significa “modo de precisión”

A veces un problema dura poco o no se ve claramente en la observación normal.

Para eso existe el modo de precisión.

En simple:

1. el programa empieza a grabar una traza,
2. ocurre el problema,
3. el programa detiene la grabación,
4. se genera evidencia,
5. luego esa evidencia se resume o se analiza con más detalle.

Piensa en esto como una “caja negra” temporal del sistema.

---

## 8. Qué cosas podrías hacer como usuario

### Caso A: el disco se pone al 100%
Abres el programa y miras:

- procesos dominantes,
- temporales,
- servicios relacionados a Windows,
- y si hace falta activas el modo de precisión.

### Caso B: internet se pone lento
Miras:

- conexiones activas,
- IP remotas,
- proceso asociado,
- y si parece sospechoso, revisas si bloquearlo tiene sentido.

### Caso C: Windows se vuelve lento después de un rato
Miras:

- memoria,
- rutas temporales,
- estado de servicios,
- y usas captura ETL cuando el problema aparezca.

---

## 9. Qué no debes esperar

No debes esperar que el software:

- arregle todos los problemas solo,
- reemplace completamente herramientas profundas de Microsoft,
- tome decisiones perfectas siempre,
- permita matar cualquier proceso del sistema sin riesgo.

Es una herramienta de diagnóstico y apoyo a la decisión.

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

Eso ayuda a no depender solo de “creo que funciona”.

---

## 12. Qué significa empaquetado e instalador

### Ejecutable portable
Es el programa listo para abrir directamente.

### Instalador
Es el paquete que guía la instalación en Windows, crea accesos y deja el programa en una carpeta ordenada.

El proyecto contempla ambos caminos.

---

## 13. Qué mirar primero si no sabes nada

Orden recomendado:

1. `README.md`
2. `docs/RECLUTADORES.md`
3. `docs/REPOSITORIO_ANALISIS.md`
4. `docs/MANUAL_PARA_NOVATOS.md`
5. `docs/RUST_PARA_SIGNALWATCH.md` si luego quieres entender más.

---

## 14. Resumen final

Este proyecto es una herramienta para Windows que busca encontrar la causa principal de problemas de rendimiento.

Su valor está en:

- observar,
- explicar,
- dejar evidencia,
- y ayudar a actuar con más criterio.

Aunque contiene código, también contiene manuales para que una persona no técnica pueda entender qué resuelve y por qué tiene sentido.

