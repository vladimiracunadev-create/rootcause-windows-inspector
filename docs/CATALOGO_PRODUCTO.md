# Catalogo Del Producto

Documento fuente de verdad para distinguir:

- que es una **edicion** del producto,
- que es un **artefacto** de distribucion,
- que es un **adaptador** sobre el binario principal,
- y que partes siguen siendo **skeleton / futuro**.

---

## 1) Definiciones

### Edicion
Forma funcional en que RootCause se ejecuta o se consume.

### Artefacto
Archivo concreto publicado en releases (`.exe`, `.zip`, `.psm1`, `.vsix`, hashes).

### Adaptador
Integracion que reutiliza `rootcause.exe` como motor. No reemplaza el nucleo del producto.

### Perfil de distribucion
Presentacion comercial o publica del mismo motor.

Ejemplos:
- `RootCause Windows Inspector` -> perfil principal.
- `RootCause Demo` -> perfil alternativo de evaluacion publica.

---

## 2) Matriz canonica de modalidades

| Modalidad | Tipo | Estado | Se publica en `release-windows` | Requiere `rootcause.exe` | Notas |
|---|---|---|---|---|---|
| GUI Desktop | Nucleo principal | Produccion | Si | No | Build por defecto con `gui` |
| CLI-only | Nucleo alternativo | Produccion | Si | No | Build con `--no-default-features` |
| PowerShell Module | Adaptador | Produccion | Si | Si | Wrapper sobre comandos CLI |
| VS Code Extension | Adaptador | Produccion | Si | Si | Wrapper sobre `status --json`, `export`, `snapshot` |
| Tray icon | Extension de runtime | Produccion | Si | Si | Incluido en la edicion GUI (color por severidad + menu) |
| Windows Service | Extension de runtime | Skeleton | No | Si | Arquitectura documentada, no GA |
| RootCause Demo | Perfil de distribucion | Opcional | No por defecto | No | Instalador alternativo con branding y mensajes demo |

---

## 3) Artefactos oficiales del release principal

| Archivo | Que entrega | Estado |
|---|---|---|
| `RootCause-Setup.exe` | Instalador principal GUI + CLI | Publicado |
| `RootCause-Portable.zip` | Portable del build principal GUI + CLI | Publicado |
| `RootCause-CLI-Portable.zip` | Portable CLI-only | Publicado |
| `RootCause.psm1` | Modulo PowerShell | Publicado |
| `RootCause-VSCode-Extension.vsix` | Extension VS Code | Publicado |
| `SHA256SUMS.txt` | Integridad para todos los artefactos | Publicado |

---

## 4) Instalacion por modalidad

### GUI principal
- `RootCause-Setup.exe`: instalacion recomendada.
- `RootCause-Portable.zip`: extraer y ejecutar.

### CLI-only
- descargar `RootCause-CLI-Portable.zip`,
- extraer,
- ejecutar `rootcause.exe` desde consola.

### PowerShell Module
- descargar `RootCause.psm1`,
- instalar primero RootCause GUI o CLI-only,
- dejar `rootcause.exe` en PATH o junto al modulo,
- importar con `Import-Module .\RootCause.psm1`.

### VS Code Extension
- instalar primero RootCause GUI o CLI-only,
- instalar `RootCause-VSCode-Extension.vsix`,
- verificar `rootcause` en PATH o configurar `rootcause.executablePath`.

### No publicables como produccion todavia
- `Windows Service`

No debe venderse como descarga final hasta salir de skeleton.

---

## 5) Reglas de comunicacion

### Lo que si se puede afirmar

- RootCause tiene multiples **modalidades reales**.
- El runtime principal es la app Windows en GUI y CLI.
- PowerShell y VS Code son **adaptadores**, no motores alternativos.
- Windows Service **no** esta listo para venderse como produccion (Tray icon sí, desde v0.16.0).

### Lo que no se debe volver a mezclar

- `Portable ZIP` no implica automaticamente `CLI-only`.
- `RootCause Demo` no es una edicion distinta del motor: es un perfil de distribucion.
- Un boton de descarga no debe prometer un artefacto que el workflow no publica.

---

## 6) Regla operativa para documentacion y landing

Siempre usar esta estructura:

1. **Nucleo publicado hoy**
2. **Integraciones publicadas hoy**
3. **Experimental / skeleton**
4. **Perfil demo opcional**

Si una superficie publica contradice esta matriz, este documento tiene prioridad.
