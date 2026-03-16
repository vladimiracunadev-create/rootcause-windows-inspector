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
- [ ] `ci.yml` en verde en GitHub Actions

---

## 2) Documentación

- [ ] README actualizado
- [ ] BUILD_WINDOWS actualizado
- [ ] COMMANDS actualizado
- [ ] REQUIREMENTS actualizado
- [ ] OPERACION actualizado
- [ ] PRECISION_MODE_ETW actualizado
- [ ] PACKAGING_WINDOWS actualizado
- [ ] SECURITY revisado

---

## 3) Funcionalidad

- [ ] la UI arranca
- [ ] refresco funciona
- [ ] exportación JSON funciona
- [ ] bloqueo de IP funciona en entorno controlado
- [ ] finalización de proceso funciona en entorno controlado
- [ ] inicio / stop / cancel de WPR verificados si WPT está instalado
- [ ] `docs/TESTING_WINDOWS.md` ejecutado o marcado parcialmente con evidencia

---

## 4) Empaquetado

- [ ] portable ZIP generado
- [ ] instalador Inno generado si aplica
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

## 7) Entrega final recomendada

- [ ] ZIP portable
- [ ] instalador Inno si corresponde
- [ ] hashes
- [ ] notas de versión
- [ ] fecha de build


## 8) CI/CD

- [ ] `release-windows.yml` validado manualmente
- [ ] artefactos de GitHub Actions descargados y verificados
- [ ] `Cargo.lock` generado y evaluado para commit
