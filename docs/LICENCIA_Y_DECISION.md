# Licencia y decisión de distribución

Este documento registra la decisión de licencia tomada para este proyecto, el razonamiento detrás de ella y la ruta futura prevista.

---

## Licencia actual

**Apache License 2.0**

Aplicada desde la versión v0.5.0 en adelante.

El texto completo está en [`LICENSE`](../LICENSE).

---

## Por qué Apache 2.0 y no MIT

Ambas licencias son open source permisivas, pero Apache 2.0 añade una protección importante que MIT no tiene:

| Aspecto                        | MIT           | Apache 2.0         |
|-------------------------------|---------------|--------------------|
| Redistribución libre          | ✅            | ✅                 |
| Uso comercial                 | ✅            | ✅                 |
| Modificación sin restricciones| ✅            | ✅                 |
| Grant de patentes explícito   | ❌            | ✅                 |
| Protección contra claims de patentes de terceros | ❌ | ✅ |
| Requiere preservar avisos     | ✅ (mínimo)   | ✅ (más explícito) |

El **grant de patentes** de Apache 2.0 es la diferencia clave: si un contribuidor tiene patentes relacionadas con el código que aporta, Apache 2.0 le otorga automáticamente al usuario una licencia para usarlas. Con MIT eso no queda cubierto.

Para un software Windows de diagnóstico y observación que podría tener utilidad corporativa, Apache 2.0 es más defensiva y más reconocida en contextos empresariales que MIT.

---

## Estado del repositorio

**El repositorio es actualmente privado.**

La visibilidad pública queda a criterio exclusivo del autor. Esta decisión de licencia aplica para cuando se haga visible o se distribuyan artefactos, pero no obliga a hacer el repositorio público.

---

## Qué cubre Apache 2.0

- cualquier persona puede usar, estudiar, modificar y redistribuir el software,
- puede usarse en productos comerciales,
- debe conservarse el aviso de copyright y la atribución,
- se incluye un grant explícito de patentes de los contribuidores,
- si se inicia un litigio de patentes contra el proyecto, se pierde automáticamente el grant de patentes recibido.

---

## Qué NO cubre

- No protege la **marca** ni el nombre "RootCause" (eso requiere registro de marca separado).
- No obliga a los usuarios o empresas a publicar sus modificaciones (no es copyleft).
- No garantiza que terceros no usen el nombre o el código de formas que el autor no anticipe.

---

## Ruta futura prevista — v1.0

En la etapa v1.0, cuando el proyecto entre en distribución formal, se evaluará una de estas dos rutas:

### Opción A — Dual license (recomendada si el proyecto es viable comercialmente)
- **GPL v3** para uso open source (fuerza reciprocidad: quien modifique debe publicar los cambios),
- **Licencia comercial privada** para empresas que quieran usar el software sin publicar sus modificaciones.

Esta es la ruta de proyectos como Qt, Redis (antes SSPL), MySQL, etc.

### Opción B — Mantener Apache 2.0
- Adecuada si el objetivo es máxima adopción libre, incluso corporativa.
- Sin barreras de entrada, pero sin monetización forzada.

La decisión entre A y B depende de si el autor quiere explotar comercialmente el software o simplemente distribuirlo de forma confiable.

---

## Recomendación operativa actual

Para la fase actual (repositorio privado, producto en maduración):

1. **Apache 2.0 como licencia base** — protege sin restringir,
2. **No publicar el repositorio** hasta tener criterio claro de nombre y marca,
3. **Registrar la marca "RootCause"** antes de publicación formal si se decide ese nombre,
4. **Evaluar dual license** cuando exista tracción real de usuarios o interés corporativo.

---

## Historial

| Versión | Licencia    | Fecha      | Notas                             |
|---------|-------------|------------|-----------------------------------|
| v0.1–v0.4 | MIT       | 2026       | Licencia inicial, sin decisión consciente |
| v0.5.0  | Apache 2.0  | 2026-03-17 | Decisión consciente documentada   |
