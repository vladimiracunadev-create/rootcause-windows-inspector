# CI/CD en GitHub Actions

Este documento define cómo se valida el proyecto en GitHub para reducir el riesgo de que el repositorio “se vea bien” pero no compile o no se empaquete como se espera.

---

## 1) Qué sí garantiza la CI

La CI deja evidencia automática de que, en un runner Windows limpio:

- el repositorio se descarga correctamente,
- el toolchain de Rust se instala,
- el código pasa formato,
- el código pasa lint con Clippy,
- los tests unitarios existentes pasan,
- el binario release compila,
- el ejecutable y la documentación pueden publicarse como artefactos,
- el ZIP portable y el instalador pueden construirse en el flujo de empaquetado.

---

## 2) Qué NO garantiza por sí sola

La CI no reemplaza estas validaciones:

- prueba manual de la interfaz en una sesión Windows real,
- prueba con permisos de usuario estándar y administrador,
- prueba de WPR/WPA/tracerpt en un equipo que tenga WPT instalado,
- validación contra antivirus o políticas corporativas,
- firma digital del binario o del instalador,
- análisis funcional prolongado con síntomas reales de disco al 100 %.

---

## 3) Flujos incluidos

### `ci.yml`
Valida en cada push / pull request:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features -- --nocapture`
- `cargo check --all-targets`
- `cargo build --release --verbose`

Además sube como artefacto el `.exe` release y documentación útil para revisión.

### `release-windows.yml`
Pensado para etiquetar releases o ejecutar manualmente:

- corre los quality gates,
- compila release,
- genera ZIP portable,
- instala Inno Setup,
- compila el instalador,
- genera hashes SHA-256,
- publica la carpeta `build/` como artefacto.

---

## 4) Dónde están los archivos

```text
.github/workflows/ci.yml
.github/workflows/release-windows.yml
```

---

## 5) Recomendación profesional

Para este tipo de software Windows, la ruta seria es:

1. validar en CI cada commit,
2. validar manualmente en un Windows real,
3. empaquetar solo cuando la rama principal esté verde,
4. guardar hashes y notas de build,
5. firmar binarios cuando el proyecto entre en etapa de distribución formal.

---

## 6) Paso adicional recomendado

Cuando hagas el primer build local exitoso, se recomienda:

```powershell
cargo generate-lockfile
```

y luego **commitear `Cargo.lock`** para fijar dependencias de forma más reproducible en builds futuros.
