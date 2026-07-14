# RootCause Windows Inspector - Landing Page

Pagina web publica del producto [RootCause Windows Inspector](https://github.com/vladimiracunadev-create/rootcause-windows-inspector).

Ver online: https://vladimiracunadev-create.github.io/rootcause-windows-inspector

**Version actual:** v0.18.0

---

## Estructura

```text
landing/                    <- subdirectorio servido por GitHub Pages
|-- index.html              <- pagina principal (hero, features, editions, install, cli, download)
|-- assets/
|   |-- style.css           <- estilos
|   `-- favicon.svg         <- icono
`-- README.md               <- este archivo

# El workflow de deploy vive en ../.github/workflows/deploy-landing.yml
```

## Taxonomia de producto que debe respetar la landing

- **nucleo publicado hoy:** GUI Desktop y CLI-only
- **integraciones publicadas hoy:** PowerShell Module y VS Code Extension
- **produccion (desde v0.16):** Tray icon
- **experimental / skeleton:** Windows Service
- **perfil alternativo:** RootCause Demo

La landing no debe volver a mezclar:
- `Portable ZIP` con `CLI-only`
- integraciones con motores standalone
- demo con edicion principal

## Secciones de la landing

| Seccion | Ancla | Descripcion |
|---|---|---|
| Hero | - | Titulo, badges, boton de descarga |
| Caracteristicas | `#features` | Features del producto principal |
| Ediciones | `#editions` | Modalidades reales + estado |
| Pestañas | - | Tabla de las 11 secciones de la GUI (barra lateral, estilo Windows 11) |
| Requisitos | `#requirements` | Minimos, recomendados, modo precision |
| Instalacion | `#install` | GUI, portable, CLI-only, PowerShell y VS Code |
| CLI | `#cli` | Referencia de comandos + demo terminal |
| Atajos | - | Tabla de atajos |
| Descargar | `#download` | Artefactos reales publicados por release |

## Releases y descargas

Los botones de descarga apuntan a artefactos directos en:

```text
https://github.com/vladimiracunadev-create/rootcause-windows-inspector/releases/latest/download/
```

Artefactos esperados del release principal:
- `RootCause-Setup.exe`
- `RootCause-Portable.zip`
- `RootCause-CLI-Portable.zip`
- `RootCause.psm1`
- `RootCause-VSCode-Extension.vsix`
- `SHA256SUMS.txt`

## Reglas de instalacion que deben quedar claras

- `RootCause-Setup.exe`: instala la app principal GUI + CLI
- `RootCause-Portable.zip`: portable del build principal GUI + CLI
- `RootCause-CLI-Portable.zip`: runtime CLI-only
- `RootCause.psm1`: integracion; requiere `rootcause.exe`
- `RootCause-VSCode-Extension.vsix`: integracion; requiere `rootcause.exe`

## Evolucion visible en la landing (version actual)

- mantener un bloque breve sobre seguridad y resiliencia del agente
- reflejar las features actuales: gestion de espacio de Docker, idioma ES/EN, modos de tema Claro/Oscuro/Windows, rediseno Windows 11 (barra lateral, iconos de linea), icono de bandeja de produccion
- mencionar heartbeat local, integridad basica de configuracion y deteccion de cierre abrupto previo
- Tab Autostart con baseline en SQLite y deteccion de cambios de persistencia (`persistence-change`): entradas NUEVA / MODIFICADA / ELIMINADA, aceptables con `rootcause autostart --accept`
- deteccion de cambios en servicios de Windows sobre un motor generico de baseline reutilizable (generaliza el patron de autostart): servicio NUEVO / MODIFICADO / ELIMINADO por StartMode + ruta del binario, alertas `service-change`, aceptables con `rootcause services --accept` (listado con `rootcause services` / `rootcause services --json`)
- no presentar RootCause como antivirus, EDR completo ni proteccion perfecta
- conservar el look & feel existente del repo publico

## Notas

- Este repo es **publico** para usar GitHub Pages gratis.
- El codigo fuente del producto vive en el repo `rootcause-windows-inspector`.
- Telemetria: cero.

