# Mini manual de Rust orientado a este repositorio

## 1. Objetivo de este documento

Este manual está pensado para ti si nunca has trabajado en Rust o casi no lo conoces, pero quieres entender **qué está pasando en este repositorio** sin estudiar todo el lenguaje de forma abstracta.

No intenta enseñarte Rust completo.

Intenta enseñarte el Rust que más te sirve para leer y mantener **RootCause**.

---

## 2. Qué es Rust, explicado simple

Rust es un lenguaje compilado, nativo y rápido, diseñado para construir software con:

- buen rendimiento,
- control del sistema,
- seguridad de memoria,
- menos errores peligrosos que en C/C++.

En este proyecto se eligió Rust porque queremos un ejecutable Windows:

- ligero,
- serio,
- mantenible,
- con acceso a sistema,
- y sin depender de runtimes pesados.

---

## 3. Cómo se ve un proyecto Rust

En este repositorio, la raíz importante es esta:

```text
Cargo.toml
src/
  main.rs
  app.rs
  models.rs
  services/
```

### `Cargo.toml`
Es el archivo de configuración del proyecto.

Cumple funciones como:

- nombre del proyecto,
- versión,
- edición del lenguaje,
- dependencias,
- configuración de compilación.

Piensa en él como el “centro de control” del proyecto Rust.

### `src/main.rs`
Es el punto de entrada.

Aquí parte el programa.

### `src/app.rs`
Contiene la lógica principal de la interfaz.

### `src/models.rs`
Contiene estructuras compartidas del dominio.

### `src/services/`
Contiene módulos con responsabilidades separadas.

---

## 4. Palabras clave que verás mucho

### `fn`
Declara una función.

Ejemplo:

```rust
fn main() {
    println!("Hola");
}
```

### `struct`
Declara una estructura de datos.

Ejemplo:

```rust
struct ProcessSnapshot {
    pid: u32,
    name: String,
}
```

Piensa en una `struct` como una ficha con campos.

### `enum`
Permite representar variantes posibles.

Ejemplo:

```rust
enum Severity {
    Green,
    Yellow,
    Red,
}
```

En este proyecto sirve muy bien para semáforos, estados o categorías.

### `impl`
Define métodos asociados a una estructura.

Ejemplo:

```rust
impl ProcessSnapshot {
    fn is_heavy(&self) -> bool {
        self.pid > 0
    }
}
```

### `mod`
Declara módulos.

Ejemplo:

```rust
mod services;
```

### `use`
Importa elementos de otros módulos.

Ejemplo:

```rust
use crate::models::Severity;
```

---

## 5. Lo más importante de Rust para leer este repo

### 5.1 Propiedad (`ownership`)
Rust controla quién “posee” un valor.

Esto evita muchos errores de memoria.

No necesitas dominar toda la teoría al comienzo. Para este repo basta con entender:

- un valor tiene dueño,
- a veces se mueve,
- a veces se presta,
- y Rust no deja usos inseguros.

### 5.2 Referencias (`&`)
Permiten usar un valor sin tomar posesión.

Ejemplo:

```rust
fn print_name(name: &String) {
    println!("{}", name);
}
```

Cuando veas `&`, normalmente significa “lo estoy usando prestado”.

### 5.3 Mutabilidad (`mut`)
Por defecto, Rust trata los valores como inmutables.

Si quieres cambiarlos, debes decirlo explícitamente.

Ejemplo:

```rust
let mut total = 0;
total += 1;
```

Eso ayuda mucho a leer el código, porque te deja claro qué cambia y qué no.

### 5.4 Manejo de errores con `Result`
Rust prefiere que los errores sean explícitos.

Ejemplo típico:

```rust
fn load_data() -> Result<String, String> {
    Ok("ok".to_string())
}
```

En este proyecto es importante porque muchas operaciones pueden fallar:

- leer procesos,
- invocar PowerShell,
- consultar netstat,
- abrir archivos,
- leer directorios,
- ejecutar herramientas WPR/WPA.

### 5.5 Valores opcionales con `Option`
Representa algo que puede existir o no.

Ejemplo:

```rust
let maybe_ip: Option<String> = None;
```

Es útil cuando una ruta, un proceso o un dato no siempre están presentes.

---

## 6. Cómo leer este repositorio sin perderte

### Paso 1: abre `Cargo.toml`
Mira:

- nombre,
- dependencias,
- versión,
- edición.

### Paso 2: abre `src/main.rs`
Mira cómo arranca la aplicación.

### Paso 3: abre `src/app.rs`
Aquí verás la interfaz y el flujo principal.

### Paso 4: abre `src/models.rs`
Aquí entiendes los tipos de datos centrales.

### Paso 5: abre `src/services/`
Aquí entiendes cómo el programa obtiene y procesa información del sistema.

---

## 7. Qué hace cada módulo de este proyecto

### `main.rs`
- inicia la app,
- prepara el runtime de interfaz,
- conecta el arranque general.

### `app.rs`
- pinta la interfaz,
- administra estado visual,
- responde a botones y acciones,
- muestra hallazgos.

### `models.rs`
- define estructuras como snapshots, severidades, hallazgos y resultados.

### `services/inspector.rs`
- coordina inspección principal,
- junta información de varias fuentes.

### `services/temp_scan.rs`
- revisa carpetas temporales,
- calcula tamaños,
- detecta crecimiento o rutas sospechosas.

### `services/network.rs`
- procesa conexiones,
- relaciona procesos con IP.

### `services/windows.rs`
- consulta estado de servicios,
- lee eventos,
- interactúa con herramientas Windows.

### `services/persistence.rs`
- guarda información en SQLite.

### `services/etl.rs`
- gestiona la lógica de trazas ETL/WPR,
- resume información del análisis de trazas.

---

## 8. Patrones comunes que verás

### A. `derive`
Rust permite generar comportamientos comunes automáticamente.

Ejemplo:

```rust
#[derive(Debug, Clone)]
struct Sample {
    id: u32,
}
```

Esto evita escribir mucho código repetido.

### B. `match`
Sirve para evaluar variantes.

Ejemplo:

```rust
match severity {
    Severity::Green => "Todo bien",
    Severity::Yellow => "Atención",
    Severity::Red => "Crítico",
}
```

Es muy útil para semáforos, decisiones y estados.

### C. Iteradores
Rust trabaja mucho con listas de forma funcional.

Ejemplo:

```rust
let heavy: Vec<_> = processes
    .iter()
    .filter(|p| p.pid > 100)
    .collect();
```

No necesitas memorizarlo todo. Solo entender que muchas transformaciones se encadenan.

---

## 9. Comandos Rust que te importan en este repo

### Compilar en debug
```powershell
cargo build
```

### Compilar release
```powershell
cargo build --release
```

### Ejecutar la app
```powershell
cargo run
```

### Ejecutar tests
```powershell
cargo test
```

### Formatear código
```powershell
cargo fmt
```

### Linter
```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

---

## 10. Qué debes aprender primero para mantener este proyecto

Orden recomendado:

1. estructura de módulos,
2. `struct`, `enum`, `fn`, `impl`,
3. `Option` y `Result`,
4. referencias `&` y `mut`,
5. `match`,
6. iteradores,
7. manejo básico de errores,
8. lectura de dependencias en `Cargo.toml`.

Con eso ya puedes leer gran parte del repositorio sin sentirte perdido.

---

## 11. Errores mentales comunes al venir de otros lenguajes

### “¿Dónde están mis clases?”
Rust no gira alrededor de clases tradicionales como Java o C#.

Piensa en:

- `struct` para datos,
- `impl` para métodos,
- `enum` para estados,
- módulos para organización.

### “¿Por qué me obliga a marcar `mut`?”
Porque Rust intenta que los cambios sean explícitos.

### “¿Por qué no deja usar una variable después?”
Porque probablemente fue movida o prestada de forma incompatible. Eso es parte del sistema que previene errores.

### “¿Por qué tanto `Result` y `Option`?”
Porque el lenguaje intenta que manejes casos reales en vez de asumir que todo saldrá bien.

---

## 12. Cómo practicar con este mismo repositorio

### Ejercicio 1
Abre `models.rs` e identifica:

- qué `struct` representan procesos,
- qué `enum` representan severidad,
- qué tipos modelan hallazgos.

### Ejercicio 2
Abre `temp_scan.rs` y sigue el flujo:

- entrada,
- lectura de directorios,
- cálculo,
- salida.

### Ejercicio 3
Ejecuta:

```powershell
cargo test
```

Y luego revisa qué test protege qué parte.

### Ejercicio 4
Ejecuta:

```powershell
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

Así verás cómo Rust te obliga a mantener orden y calidad.

---

## 13. Conclusión

Para trabajar en este repositorio no necesitas “saber todo Rust”.

Necesitas dominar primero estas ideas:

- cómo se organiza un proyecto con Cargo,
- cómo leer módulos,
- cómo entender `struct`, `enum`, `Result` y `Option`,
- cómo seguir el flujo desde `main.rs` hasta `services/`.

Una vez entiendas eso, este proyecto deja de verse como “un lenguaje raro” y pasa a verse como un sistema ordenado y bastante legible.

