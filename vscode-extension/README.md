# RootCause VS Code Extension

Extensión de VS Code para mostrar el estado de RootCause desde el editor, sin salir de tu flujo de trabajo.

## ✨ Características

- 📊 Muestra severidad, CPU y RAM en la status bar.
- 🚨 Alerta cuando el sistema pasa a `Critical`.
- 📤 Permite exportar un snapshot.
- 🪟 Abre un panel con el snapshot actual.

## 📋 Requisitos

> Esta extensión **no reemplaza** a RootCause: es un visor que se apoya en el ejecutable.

Necesita que `rootcause.exe` esté disponible de alguna de estas formas:

| Requisito | Detalle |
|---|---|
| Instalado | Vía `RootCause-Setup.exe`. |
| En el `PATH` | Accesible como `rootcause` desde la consola. |
| Configurado | Ruta explícita en `rootcause.executablePath`. |

## 🚀 Uso

1. Instala la extensión (ver más abajo).
2. Asegúrate de que `rootcause.exe` cumpla uno de los requisitos anteriores.
3. Observa la severidad, CPU y RAM en la status bar.
4. Abre el panel para ver el snapshot actual o expórtalo cuando lo necesites.

## 📦 Empaquetado

```powershell
npm install
npx @vscode/vsce package --out ..\build\RootCause-VSCode-Extension.vsix
```

## 🔧 Instalación manual

```powershell
code --install-extension RootCause-VSCode-Extension.vsix
```
