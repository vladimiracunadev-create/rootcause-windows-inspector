# Build en Windows

Este documento define la ruta oficial para construir el ejecutable de RootCause en un equipo Windows, desde preparación de entorno hasta verificación del artefacto final.

---

## 1) Política de build

Este repositorio **no distribuye** el `.exe` precompilado.

La construcción del binario se hace **localmente en Windows**, de forma reproducible y auditable.

Salida esperada:

```text
target\release\rootcause.exe
```

---

## 2) Requisitos previos

### Obligatorios
- Windows 10 x64 o Windows 11 x64
- PowerShell 5.1+ o PowerShell 7+
- Rust estable vía Rustup
- toolchain MSVC operativo
- espacio libre suficiente para `target/`, ETL y empaquetado

### Recomendados
- Visual Studio Build Tools o Visual Studio con **Desktop development with C++**
- Git
- Inno Setup si vas a generar instalador
- Windows Performance Toolkit si vas a usar modo de precisión

Detalle ampliado en [`REQUIREMENTS.md`](REQUIREMENTS.md).

---

## 3) Verificación del entorno

### PowerShell

```powershell
.\scripts\verify-environment.ps1
```

### Batch

```bat
scripts\verify-environment.bat
```

La validación debe confirmar, idealmente:

- `rustup`
- `cargo`
- `rustc`
- `cl.exe` o linker equivalente MSVC
- `powershell`
- `wpr` si usarás captura ETW
- `tracerpt` si usarás resumen ETL local
- `wpa` si usarás análisis profundo
- `ISCC.exe` si vas a generar instalador

---

## 4) Instalación de Rust

### Ruta recomendada
Instala Rust con Rustup y deja el toolchain estable por defecto.

### Comprobación

```powershell
rustup --version
cargo --version
rustc --version
```

### Notas prácticas
- si `cargo` no aparece, cierra y abre la terminal,
- si el linker falla, abre **Developer PowerShell for VS**,
- si tu entorno corporativo restringe PATH, verifica primero `where cargo`.

---

## 5) Construcción mínima

Desde la raíz del repositorio:

```powershell
cargo build --release
```

Resultado esperado:

```text
target\release\rootcause.exe
```

---

## 6) Construcción recomendada

Usa el script del proyecto:

```powershell
.\scripts\build-release.ps1
```

O bien:

```bat
scripts\build-release.bat
```

Ventajas:

- da mensajes más claros,
- valida mejor el artefacto esperado,
- sirve como ruta estandarizada del proyecto,
- se alinea con la documentación y la fase de empaquetado.

---

## 7) Quality gates antes del release

```powershell
.\scripts\quality-gates.ps1
```

Este script debe cubrir, según tu instalación:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo build --release`

### Orden recomendado manual

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build --release
```

---

## 8) Limpieza de artefactos

```powershell
.\scripts\clean.ps1
```

Esto elimina:

- artefactos Cargo,
- carpetas `build/` generadas por empaquetado,
- residuos de empaquetados anteriores si el script está configurado así.

Usa esta limpieza antes de un build reproducible o antes de validar hashes finales.

---

## 9) Ubicación del binario generado

La salida release se espera en:

```text
target\release\rootcause.exe
```

No cambies esta convención sin actualizar al mismo tiempo:

- `Cargo.toml`
- scripts
- documentación
- plantilla de instalador
- checklist de release

---

## 10) Build de desarrollo

Para probar la aplicación sin empaquetado:

```powershell
cargo run
```

Usa esta ruta cuando quieras validar:

- interfaz,
- detección de procesos,
- escaneo de temporales,
- lectura de eventos,
- control WPR,
- resumen ETL.

---

## 11) Flujo recomendado del mantenedor

```powershell
.\scripts\verify-environment.ps1
cargo check
cargo test
.\scripts\quality-gates.ps1
.\scripts\build-release.ps1
.\scripts\package-portable.ps1
.\scripts\package-inno.ps1
.\scripts\package-powershell-module.ps1
.\scripts\package-vscode-extension.ps1
.\scripts\hash-artifacts.ps1
```

---

## 12) Problemas comunes

### `cargo` no existe
Instala Rustup y reinicia terminal.

### `link.exe` o `cl.exe` no existen
Instala Build Tools con C++.

### El release falla pero debug no
Revisa:

- flags de release,
- antivirus,
- políticas corporativas,
- espacio disponible,
- permisos de escritura sobre `target/`.

### PowerShell bloquea scripts
Usa:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
```

### La app abre pero no captura ETW
Revisa:

- permisos,
- presencia real de `wpr.exe`,
- políticas corporativas,
- instalación de Windows Performance Toolkit.

### La app captura ETW pero no resume ETL
Revisa:

- presencia real de `tracerpt.exe`,
- que exista un `.etl` reciente,
- espacio suficiente en la carpeta `traces\analysis`.

---

## 13) Convención del nombre del binario

Nombre actual del ejecutable:

```text
rootcause.exe
```

Si en el futuro cambias este nombre, debes actualizar al mismo tiempo:

- `Cargo.toml`
- scripts de empaquetado
- Inno Setup
- documentación
- checklist de release

---

## 14) Validación posterior al build

Después de compilar, verifica al menos:

- que el binario exista,
- que abra sin dependencias faltantes,
- que muestre la UI,
- que actualice snapshot,
- que exporte JSON,
- que inicie y detenga WPR si está disponible,
- que pueda resumir el último ETL si `tracerpt` está disponible.

---

## 15) Resumen operativo

La ruta profesional recomendada es:

1. verificar entorno,
2. validar toolchain,
3. correr quality gates,
4. compilar release,
5. validar binario,
6. empaquetar,
7. generar hashes,
8. documentar versión liberada.


## 14) Integración con GitHub Actions

Este repositorio incluye:

- `.github/workflows/ci.yml`
- `.github/workflows/release-windows.yml`

La idea es que el build local y el build de CI se parezcan lo más posible.

### Réplica local rápida

```powershell
.\scripts\ci-local.ps1
```

### Recomendación

- usa CI para validar cada commit,
- usa build local para validar tu entorno real,
- usa release workflow solo cuando la rama principal esté en verde.

## 15) Sobre `Cargo.lock`

Hoy el repositorio puede construirse sin `Cargo.lock`, pero para una distribución más reproducible se recomienda generar el lockfile en el primer build local exitoso y commitearlo:

```powershell
cargo generate-lockfile
```


## Marca e icono en el binario

El proyecto usa `build.rs` para incrustar en Windows el icono `assets/rootcause.ico` y metadatos mínimos de producto como `ProductName` y `FileDescription`. Esto ayuda a que `rootcause.exe` se vea correctamente en el Explorador, los accesos directos y las búsquedas del sistema.
