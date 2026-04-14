# Release Checklist

Checklist profesional antes de distribuir artefactos.

---

## 1) Código

- [ ] `cargo check --all-targets`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features -- --nocapture`
- [ ] `cargo build --release --verbose`
- [ ] `scripts\ci-local.ps1` ejecutado localmente
- [ ] `scripts\release-product.ps1 -VerifyEnvironment` ejecutado si esta version se va a empaquetar/publicar
- [ ] `ci.yml` en verde en GitHub Actions

---

## 2) Documentación

- [ ] README actualizado (ASCII art, tabs, estructura src)
- [ ] BUILD_WINDOWS actualizado
- [ ] COMMANDS actualizado
- [ ] REQUIREMENTS actualizado
- [ ] OPERACION actualizado
- [ ] PRECISION_MODE_ETW actualizado
- [ ] PACKAGING_WINDOWS actualizado
- [ ] ARCHITECTURE.md actualizado con módulos nuevos
- [ ] ROADMAP actualizado con los ítems completados en esta versión
- [ ] RECLUTADORES.md actualizado con funciones nuevas
- [ ] SECURITY revisado
- [ ] Landing page: versión actualizada en index.html

---

## 3) Funcionalidad

- [ ] la UI arranca
- [ ] refresco funciona
- [ ] exportación JSON funciona (GUI y CLI: `rootcause export`)
- [ ] CLI: `rootcause --help`, `status`, `snapshot`, `history` verificados
- [ ] atajos de teclado verificados (F5, Ctrl+E, Ctrl+1…8)
- [ ] tab Acerca muestra versión, autor y hardware del equipo correctamente
- [ ] sección Características del equipo visible en tab Overview
- [ ] bloqueo de IP funciona en entorno controlado
- [ ] finalización de proceso funciona en entorno controlado
- [ ] inicio / stop / cancel de WPR verificados si WPT está instalado
- [ ] `docs/TESTING_WINDOWS.md` ejecutado o marcado parcialmente con evidencia

---

## 4) Empaquetado

- [ ] portable ZIP principal generado
- [ ] portable CLI-only generado
- [ ] instalador Inno generado si aplica
- [ ] módulo PowerShell publicado si aplica
- [ ] extensión VS Code empaquetada si aplica
- [ ] carpeta `build/` revisada
- [ ] nombre de versión validado
- [ ] publisher validado

---

## 5) Integridad

- [ ] hashes SHA-256 generados
- [ ] artefactos revisados visualmente
- [ ] README y docs incluidos en el paquete si corresponde

---

## 6) Transparencia y seguridad

- [ ] se deja claro que el repo no incluye `.exe`
- [ ] se deja claro si el binario está firmado o no
- [ ] se dejan claros los permisos requeridos
- [ ] se dejan claros los límites del modo de precisión actual

---

## 7) GitHub Release

- [ ] `release-windows.yml` ejecutado y en verde
- [ ] `scripts\release-product.ps1 -VerifyEnvironment -Publish` ejecutado para evitar pasos manuales omitidos
- [ ] el release de GitHub tiene `body:` con instrucciones de instalación, no solo links de commits
- [ ] se incluyen las secciones: instalación, verificación de hash, requisitos, funciones
- [ ] los artefactos adjuntos coinciden con el catálogo actual: Setup, portable GUI, portable CLI, módulo PowerShell, VSIX y hashes
- [ ] el tag apunta al commit correcto (verificar con `git log --oneline`)

---

## 8) Entrega final recomendada

- [ ] ZIP portable GUI
- [ ] ZIP portable CLI-only
- [ ] instalador Inno si corresponde
- [ ] módulo PowerShell si corresponde
- [ ] VSIX si corresponde
- [ ] hashes
- [ ] notas de versión
- [ ] fecha de build

---

## 9) CI/CD

- [ ] `release-windows.yml` validado manualmente
- [ ] artefactos de GitHub Actions descargados y verificados
- [ ] `Cargo.lock` generado y evaluado para commit
- [ ] sin advertencias nuevas de clippy que puedan convertirse en errores futuros
