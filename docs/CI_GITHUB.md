# CI/CD en GitHub Actions

Este documento define cómo se valida el proyecto en GitHub para reducir el riesgo de que el repositorio "se vea bien" pero no compile o no se empaquete como se espera.

---

## 1) Qué sí garantiza la CI

La CI deja evidencia automática de que, en un runner Windows limpio:

- el repositorio se descarga correctamente,
- el toolchain de Rust se instala,
- el código pasa formato (`cargo fmt --all -- --check`),
- el código pasa lint con Clippy (`-D warnings`),
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
- publica el GitHub Release con instrucciones completas de instalación,
- sube `RootCause-Portable.zip`, `RootCause-Setup.exe` y `SHA256SUMS.txt`.

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

---

## 7) Patrones Rust que rompen la CI — lecciones aprendidas

### 7.1) `return Ok(format!(...));` dentro de bloques `#[cfg]`

**Problema:** `rustfmt` puede eliminar el `return` de expresiones multilínea pero dejar el `;` final, convirtiendo la expresión en una sentencia que devuelve `()`.

```rust
// ❌ Peligroso: rustfmt puede transformarlo en Ok(format!(...));
return Ok(format!(
    "Mensaje muy largo aquí que supera 100 caracteres..."
));

// ✅ Correcto: expresión de cola sin return ni ;
Ok(format!(
    "Mensaje muy largo aquí que supera 100 caracteres..."
))
```

Regla: dentro de un bloque `#[cfg(target_os = "windows")]` que funciona como única rama del cuerpo de la función, la última expresión debe ser **sin `return` y sin `;`**.

### 7.2) `if` anidados — `clippy::collapsible_if`

Con `-D warnings`, clippy convierte esta advertencia en error. Collapsible `if` significa que dos `if` anidados pueden unirse con `&&`:

```rust
// ❌ Error en CI
if condicion_a {
    if condicion_b {
        hacer_algo();
    }
}

// ✅ Correcto
if condicion_a && condicion_b {
    hacer_algo();
}
```

Para let-chains (Rust 2024), el formato que exige `rustfmt` es:

```rust
// ✅ Formato correcto para let-chains multilínea
if let Some(x) = expresion
    && condicion_extra
{
    hacer_algo();
}
```

### 7.3) `.replace()` consecutivos — `clippy::collapsible_str_replace`

```rust
// ❌ Error en CI
.replace('\n', " ")
.replace('\r', " ")

// ✅ Correcto
.replace(['\n', '\r'], " ")
```

### 7.4) `return` innecesario en cola de función — `clippy::needless_return`

```rust
// ❌ Error en CI
return Command::new("where")
    .arg(command)
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false);

// ✅ Correcto
Command::new("where")
    .arg(command)
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
```

### 7.5) Escapes en strings de tests

En Rust, `"\t"` es el carácter de tabulación, no un backslash seguido de `t`.

```rust
// ❌ Error silencioso: \t es tabulación, el assert siempre falla
assert!(text.contains("windows\temp"));

// ✅ Correcto
assert!(text.contains("windows\\temp"));
```

### 7.6) Orden de condiciones en clasificadores

Cuando una condición más específica puede ser "tapada" por una más general, el orden importa:

```rust
// ❌ "weird-updater.exe" contiene "update" → gana la condición equivocada
if nombre.contains("update") {
    "Actualización"
} else if ruta.contains("\\temp\\") {
    "Temporal"
}

// ✅ La condición más específica (ruta) va primero
if ruta.contains("\\temp\\") {
    "Temporal"
} else if nombre.contains("update") {
    "Actualización"
}
```
