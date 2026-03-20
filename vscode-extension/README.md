# RootCause VS Code Extension

Extension de VS Code para mostrar el estado de RootCause desde el editor.

## Que hace

- muestra severidad, CPU y RAM en la status bar,
- alerta cuando el sistema pasa a `Critical`,
- permite exportar snapshot,
- abre un panel con el snapshot actual.

## Requisito clave

Esta extension **no reemplaza** a RootCause.

Necesita que `rootcause.exe` este:

- instalado via `RootCause-Setup.exe`,
- disponible en el `PATH`,
- o configurado explicitamente en `rootcause.executablePath`.

## Empaquetado

```powershell
npm install
npx @vscode/vsce package --out ..\build\RootCause-VSCode-Extension.vsix
```

## Instalacion manual

```powershell
code --install-extension RootCause-VSCode-Extension.vsix
```
