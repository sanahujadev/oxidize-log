# ADR-001 — Refactor del prototipo a arquitectura hexagonal (Sprint 1)

> Architecture Decision Record. Documento inmutable una vez cerrado.
> Creado por el agente `architect` (MiniMax). Nunca se edita — si la decisión cambia, se abre un nuevo ADR
> que referencia y supera a este.
> Estado: `en revisión`
> Fecha de decisión: 2026-06-09
> Revisor esperado: Qwen (Arquitecto Segundo)
> Sprint asociado: Sprint 1 — Hexagonal Refactor (sin features nuevas)

---

## Contexto

El proyecto `oxidize-log` parte de un prototipo mínimo organizado como crate único con cinco ficheros planos (`src/lib.rs`, `src/level.rs`, `src/config.rs`, `src/logger.rs`, `src/sink.rs`) más un ejemplo (`examples/test.rs`) y un documento de diseño V0 con 39 requisitos (R1–R39) priorizados en P0/P1/P2 y tipados como F/NF.

El prototipo cumple R3 (los seis niveles existen) y la mecánica básica (un logger, un sink de consola, configuración por entorno), pero el diagnóstico técnico cruzándolo contra el V0 revela que la base actual bloquea o contradice varios requisitos P0 no funcionales y, con ella, la mayor parte de las features P1/P2:

| Requisito V0 | Estado en el prototipo | Bloqueo que produce |
| :--- | :--- | :--- |
| **R1** (workspace logger-core + bindings) | Crate único, sin `[workspace]` | Imposible empezar R23–R30 sin reorganizar primero. |
| **R2** (core único, sin duplicación) | Cumplido de facto, pero frágil: todo en un módulo | Si los bindings se enganchan ahora, acoplarán al detalle del prototipo. |
| **R19** (Send + Sync por defecto) | `trait Sink` no declara `Send + Sync` | `Box<dyn Sink>` no cruza threads; un test paralelo ya no compilaría. |
| **R20** (atomicidad de línea en archivo) | No hay sink de archivo aún, pero no hay `Result` de vuelta en `Sink::log` | No hay canal para reportar errores de I/O. |
| **R34** (fast path barato) | `Logger::log(level, &str)` recibe el mensaje ya construido | Si el nivel está desactivado, ya se pagó la alocación del `String` en el call site. |
| **R36** (sin panic!, errores tipados) | `Sink::log` devuelve `()`; no hay tipo `LogError` | Imposible degradar con gracia — un sink que falle no tiene a quién reportar. |
| **R39** (Default sensato) | `LoggerConfig` no implementa `Default`; sólo `from_env(Environment)` | El usuario no puede hacer `Logger::default().info("hola")`. |
| **R5 / R6** (formatos texto y JSON) | Formato `println!("[{:?}] {}", level, message)` hardcodeado dentro de `ConsoleSink` | Cualquier formato nuevo obliga a modificar el sink, no a inyectar un adaptador. |
| **R33** (niveles por módulo) | Un único filtro global por `level < config.level` | No hay trait `Filter` que permita reglas por metadata. |

A esto se suman defectos de diseño que el Sprint 1 debe corregir aunque no aparezcan explícitamente en el V0:
1. **OCP Violado**: `SinkConfig` es un enum cerrado con una sola variant (`Console`). Cualquier sink futuro (`R14` archivo, `R15` rotación, `R16` CloudWatch) obliga a tocar el enum, el `build_sinks` y el match del `Logger`.
2. **Olor en Tests**: `trait Sink` exige `fn as_any(&self) -> &dyn Any` solo para hacer `downcast_ref::<MockSink>()` en los tests. En hexagonal se testea contra el trait, no contra el tipo concreto.
3. **Encapsulación Rota**: `Logger.config` es `pub`, por lo que los consumidores leen/escriben la configuración interna.
4. **Acoplamiento de Fabricación**: `Logger::init` recibe `LoggerConfig` y fabrica los sinks internamente. La hexagonal dice: el orquestador recibe los adaptadores ya construidos; la fabricación es del builder.
5. **Falsos Positivos**: El test `console_sink_prints` no tiene ningún `assert!`. Pasa aunque `ConsoleSink::log` no haga nada.
6. **Comparación Frágil**: El test `levels_are_ordered` usa `(Level::Trace as u8) < (Level::Debug as u8)`. Funciona, pero no testea el `Ord` derivado y enmascara reordenamientos accidentales de los variants.

**Conclusión del diagnóstico**: El prototipo cumple la prueba de escritorio del logging básico, pero la base arquitectónica no soporta los requisitos P0 NF del V0. Hay que refactorizar antes de añadir features, en un sprint dedicado que no entregue funcionalidad nueva y cuyo criterio de cierre sea la invariante "el comportamiento observable del usuario no cambia, pero la base es hexagonal y testeable".

---

## Decisión

Adoptamos una arquitectura hexagonal en cuatro capas (`domain`, `ports`, `app`, `adapters`) para el núcleo de `oxidize-log`, con tres traits como puertos (`Sink`, `Formatter`, `Filter`), un `Logger` orquestador puro que sólo conoce el dominio y los puertos, errores tipados (`LogError`), fast path basado en closure para cumplir `R34`, y un `Default` sensato que cumpla `R39` sin depender de un enum `Environment` externo.

El Sprint 1 es un refactor sin features nuevas: el usuario final sigue viendo logs por consola, con el mismo aspecto, pero el código interno queda preparado para implementar `R7`, `R10`, `R14`, `R15`, `R37`, etc., en sprints posteriores sin nuevas reescrituras estructurales.

---

## Restricciones autoimpuestas

1. **Una sola decisión arquitectónica por ADR**: Las decisiones tácticas (forma exacta de `LogError`, shape de `LogEvent`, firmas finales de traits) son especificaciones derivadas de esta decisión. Si en revisión Qwen detecta que alguna merece su propio PDR, se abre.
2. **No se introducen nuevas dependencias** sin justificación explícita en la sección Motivo.
3. **No se añade funcionalidad visible** al usuario en este sprint (sin colores, sin JSON, sin archivo, sin macros). Cualquier tentación se delega al siguiente sprint.
4. **No se migra a workspace** `[workspace]` en este sprint. Razón en Motivo. Quedará como `PDR-001` (a abrir al cierre de este sprint, listo para revisión de Qwen).

---

## Especificaciones Técnicas

### 1. Diseño del Core y Traits (Rust)

#### 1.1 Estructura de módulos destino
```text
src/
├── lib.rs                       ← re-exports públicos (cara bindings-friendly)
├── domain/                      ← Rust puro, sin I/O, sin red, sin fs
│   ├── mod.rs
│   ├── level.rs                 ← LogLevel + Display + FromStr + as_str
│   ├── event.rs                 ← LogEvent, Metadata
│   └── error.rs                 ← LogError (thiserror o manual)
├── ports/                       ← Traits. Sólo dependen de `domain` y `core`
│   ├── mod.rs
│   ├── sink.rs                  ← trait Sink
│   ├── formatter.rs             ← trait Formatter
│   └── filter.rs                ← trait Filter
├── adapters/                    ← Implementaciones. Importan lo que necesiten
│   ├── mod.rs
│   ├── console.rs               ← ConsoleSink (delega formato al Formatter)
│   └── text_format.rs           ← SimpleTextFormatter (lo que hoy hace println!)
├── app/                         ← Orquestador y configuración
│   ├── mod.rs
│   ├── logger.rs                ← Logger { filtros, sinks, formatter }
│   └── config.rs                ← Builder + Default sensato
└── tests/                       ← (Pendiente: tests de integración cross-módulo)
```

#### 1.2 Value Objects (capa domain)
```rust
// domain/level.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogLevel { Trace, Debug, Info, Warn, Error, Fatal }

impl LogLevel {
    pub fn as_str(&self) -> &'static str { /* "TRACE".."FATAL" */ }
}

impl core::fmt::Display for LogLevel { /* usa as_str */ }
impl core::str::FromStr for LogLevel { /* case-insensitive, devuelve LogError::InvalidLevel */ }
```

```rust
// domain/event.rs
#[derive(Debug, Clone)]
pub struct Metadata {
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
    // function: Option<&'static str> queda para R8, no se incluye en Sprint 1
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,           // ver §3 sobre fast-path
    pub metadata: Metadata,
}
```

```rust
// domain/error.rs
#[derive(Debug)]
pub enum LogError {
    InvalidLevel(String),
    InvalidMetadata,
    Write { sink: &'static str, source: Box<dyn std::error::Error + Send + Sync> },
    Format(&'static str),
    Config(&'static str),
}

impl core::fmt::Display for LogError { /* ... */ }
impl std::error::Error for LogError { /* source() */ }
```
> [!NOTE]
> **Decisión táctica explícita:** usar `thiserror 1.x` para derivar `Display` y `Error` salvo que Qwen objete. Si se rechaza, se implementa a mano con `impl Display` + `impl Error` (ver Motivo §2).

#### 1.3 Puertos (capa ports)
```rust
// ports/sink.rs
pub trait Sink: Send + Sync {
    /// El `Sink` recibe el evento YA formateado. No conoce el `Formatter`.
    /// Devuelve `Result` para cumplir R36.
    fn write(&self, event: &LogEvent, bytes: &[u8]) -> Result<(), LogError>;

    /// Hook opcional para que un sink pida flush. Default = no-op.
    fn flush(&self) -> Result<(), LogError> { Ok(()) }
}
```

```rust
// ports/formatter.rs
pub trait Formatter: Send + Sync {
    /// Escribe la representación del evento en el buffer provisto.
    /// El formateador NO aloca un `String` por evento: el `Logger` le pasa
    /// un buffer reutilizable y el sink lo vuelca.
    fn format(&self, event: &LogEvent, buf: &mut String) -> Result<(), LogError>;
}
```

```rust
// ports/filter.rs
pub trait Filter: Send + Sync {
    /// Decide si el evento debe procesarse. Es el ÚNICO punto donde
    /// se aplica el fast-path. Si devuelve `false`, el `Logger` ni
    /// construye el mensaje (gracias a la closure, ver §3).
    fn enabled(&self, metadata: &Metadata, level: LogLevel) -> bool;
}
```
Reglas de uso:
- El `Logger` no llama a `Formatter::format` si todos los `Filter` han devuelto `false`.
- El `Logger` no llama a `Sink::write` si el `Formatter` ha fallado (decisión a confirmar en revisión).
- `Sink` recibe `bytes: &[u8]`, no `String`, para evitar acoplar el sink a la codificación del formateador.

#### 1.4 Abstracción: genéricos estáticos vs dyn
Decisión: `Arc<dyn Trait>` con coerción, no genéricos.

| Aspecto | Genéricos (`Logger<S: Sink, F: Formatter, ...>`) | Trait objects (`Arc<dyn Sink>`) |
| :--- | :--- | :--- |
| **Cero overhead en hot path** | ✅ monomorfización | ❌ un indirect call por dispatch |
| **Componer varios sinks** | ❌ requiere tuplas o `Vec<Box<dyn Sink>>` | ✅ natural |
| **Componer varios filtros** | ❌ idem | ✅ natural |
| **Default sensato sin tipos concretos** | ❌ `Default` no puede devolver genéricos libres | ✅ `Default` devuelve `Arc<dyn …>` |
| **API bindings-friendly (FFI/WASM)** | ❌ los generics no cruzan FFI limpio | ✅ `Arc<dyn …>` se traduce trivialmente |
| **Testabilidad** | ⚠️ requiere un tipo por combinación | ✅ un mock implementa el trait y se inyecta |

La composición y la facilidad de binding ganan. El coste del *indirect call* es del orden de nanosegundos y se documenta como restricción a vigilar en el sprint de rendimiento (`R34`+`R35`). Si en un futuro se necesita cero overhead en un caso específico, se ofrecerá una API estática opcional al lado de la API dinámica, no como reemplazo.

#### 1.5 Logger orquestador (capa app)
```rust
// app/logger.rs
pub struct Logger {
    filters: Vec<Arc<dyn Filter>>,
    sinks: Vec<Arc<dyn Sink>>,
    formatter: Arc<dyn Formatter>,
}

impl Logger {
    pub fn log<F>(&self, level: LogLevel, metadata: Metadata, message: F)
    where
        F: FnOnce() -> String,         // cierre: NO se evalúa si filtros cortan
    { /* ver §3 */ }
}

impl Default for Logger {
    fn default() -> Self {
        // R39: consola + SimpleTextFormatter + LevelFilter::Info
        Self::builder().build()
    }
}
```
El `Logger` no implementa `Clone` directamente: se clona por dentro como `Arc` (cada campo ya es `Arc<dyn _>`, así que un `#[derive(Clone)]` barato funciona y mantiene las semánticas de compartición de `R19`).

#### 1.6 Builder (capa app)
```rust
// app/config.rs
pub struct LoggerBuilder { /* campos privados */ }

impl LoggerBuilder {
    pub fn new() -> Self { /* default sensato */ }
    pub fn filter(mut self, f: Arc<dyn Filter>) -> Self { ... }
    pub fn sink(mut self, s: Arc<dyn Sink>) -> Self { ... }
    pub fn formatter(mut self, f: Arc<dyn Formatter>) -> Self { ... }
    pub fn level(mut self, level: LogLevel) -> Self { ... }   // atajo: añade LevelFilter
    pub fn build(self) -> Logger { ... }
}
```
`LoggerConfig` (struct plana) se mantiene internamente como representación serializable (para futuro `R32`), pero no es la API pública de configuración. La API pública es el builder.

#### 1.7 API pública del core (cara de los bindings)
`src/lib.rs` expone únicamente tipos que cruzan FFI limpiamente:
```rust
pub use domain::{LogLevel, LogEvent, Metadata, LogError};
pub use ports::{Sink, Formatter, Filter};
pub use adapters::{ConsoleSink, SimpleTextFormatter, LevelFilter};
pub use app::{Logger, LoggerBuilder};
pub use app::macros::*;   // R37, en el siguiente sprint — placeholder en Sprint 1
```
No se re-exporta `SinkConfig` (eliminado), `Environment` (movido a `app::config` privado o a un crate de ejemplos), ni nada que contenga generics en su firma pública.

---

### 2. Bindings y Capa de FFI (Multiplataforma)

Este sprint no entrega bindings. La sección deja constancia de las implicaciones del refactor sobre los futuros `R23`–`R30`.
- **Tecnología**: PDR pendiente (WASM con `wasm-bindgen` para JS, JNI con `jni` para Java). La elección se aborda en `PDR-002` tras cerrar este sprint.
- **Pasaje de datos**: El refactor deja la API en `Arc<dyn …>` y value objects `Clone + Send + Sync`, lo que es traducible a FFI plana (`extern "C"`) o a `wasm-bindgen` sin conversiones exóticas. Los strings se exponen como `*const c_char` en FFI; los enums como `u8`; los `LogError` como código + mensaje.
- **Sin dependencias prematuras**: No se introduce `napi-rs`, `wasm-bindgen`, `jni` ni `abi_stable` en este sprint.
- **Implicación concreta**: El `Default for Logger` y la ausencia de genéricos en la API pública son precondiciones para que los bindings del futuro no necesiten wrappers adaptados al tipo concreto.

---

### 3. Rendimiento, Concurrencia y Memoria

#### 3.1 Sincronización
- Los traits exigen `Send + Sync` en la declaración (no implícitos). Esto cierra `R19`.
- La sincronización fina (`Mutex`, `RwLock`, lock-free) es responsabilidad del adaptador, no del dominio. `ConsoleSink` no necesita lock (un `Write` que sea `Send + Sync` ya es thread-safe vía `Stdout` internamente). `FileSink` (sprint futuro) llevará `Mutex<BufWriter<File>>`.
- El `Logger` no tiene estado mutable compartido fuera de sus `Arc<dyn _>`. Inserciones y lecturas son mediante `&self`.

#### 3.2 Memoria y no_std
- El núcleo no será `no_std` en Sprint 1 (mantiene `std` por simplicidad de tests, I/O, y `Box<dyn Error>`).
- Sí se mantiene la disciplina: `domain/` no usa `Vec`, `String`, ni nada que no esté también en `alloc` o `core`. Si en un futuro se quiere `no_std` + `alloc`, la migración es viable.
- Las alocaciones de `String` están en el cierre del fast path (ver §3.3), no en el dominio.

#### 3.3 Fast path (R34)
Decisión: el método `log` toma un `impl FnOnce() -> String`, no un `&str` ni un `String`.

```rust
pub fn log<F>(&self, level: LogLevel, metadata: Metadata, message: F)
where
    F: FnOnce() -> String,
{
    // 1. Filtros: si todos rechazan, no se evalúa `message`
    if !self.filters.iter().all(|f| f.enabled(&metadata, level)) {
        return;
    }
    // 2. Construir el mensaje (aquí sí se evalúa la closure)
    let message = message();
    let event = LogEvent { level, message, metadata };

    // 3. Formatear en un buffer reutilizable
    let mut buf = String::with_capacity(128);
    if self.formatter.format(&event, &mut buf).is_err() {
        // R36: degradar con gracia, no panic
        return;
    }
    // 4. Emitir a todos los sinks
    for sink in &self.sinks {
        let _ = sink.write(&event, buf.as_bytes()); // errores se ignoran o se reportan
    }
}
```
El test `TDD-14` del Sprint 1 (`logger_log_evalua_mensaje_via_closure`) verifica con un `Cell<u32>` que la closure no se invoca cuando el nivel está desactivado. Esa es la única garantía formal de `R34` en este sprint.

*Limitación explícita:* cuando el log sí se emite, se aloca un `String` (el del mensaje) y otro `String` (el del formateador). Reducir esto a zero-alloc con `core::fmt::Arguments` es objeto de un sprint de rendimiento futuro (`R34`+`R35`), no de este ADR.

#### 3.4 Sin panic! (R36)
- Ningún `unwrap()`, `expect()` o `panic!()` en `src/domain/`, `src/ports/`, `src/app/`. Verificable mediante chequeos estáticos.
- En `src/adapters/`, los unwrap están prohibidos salvo justificación inline.
- Errores de I/O en adaptadores se convierten en `LogError::Write`. Errores de configuración en `LogError::Config`.

---

### 4. Sinks e Infraestructura (Adaptadores)

#### 4.1 I/O y Buffering
- `ConsoleSink` recibe un `Arc<dyn Formatter>` y delega. Usa `std::io::Stdout`. No bufferiza por sí mismo.
- Síncrono en este sprint. `R22` (buffering/async) se aborda en un sprint dedicado.
- `SimpleTextFormatter` produce por defecto un formato compatible con el actual, más la metadata cuando esté disponible.

#### 4.2 Dependencias de terceros

| Dependencia | Versión objetivo | Uso | Justificación |
| :--- | :--- | :--- | :--- |
| **thiserror** | 1.x | Derivar Display + Error para LogError | Estándar de la comunidad Rust. Alternativa manual viable si se rechaza. |

No se añaden en este sprint: `serde`, `serde_json`, `chrono`, `time`, `tracing`, `log`, `aws-sdk-*`, `napi-rs`, `wasm-bindgen`, `jni`, `parking_lot`, `crossbeam`, `once_cell`.

#### 4.3 Sinks entregados en Sprint 1
- `ConsoleSink` — único sink del sprint. Es el adaptador de `R13`.
- No se entrega `FileSink`, `RotatingFileSink`, ni `CloudWatchSink` en Sprint 1.

---

## Motivo

1. **Por qué hexagonal y no quedarse en el prototipo plano**:
   El prototipo plano no escala frente a los 39 requisitos cruzados. La hexagonal reduce los cambios a un trait nuevo en `ports/` + un adaptador nuevo en `adapters/`, sin tocar el `Logger` ni el dominio. Es una decisión de productividad a medio plazo.
2. **Por qué thiserror y no LogError manual**:
   Es estándar y reduce código repetitivo mediante macros declarativos simples. No arrastra dependencias transitivas pesadas. Si se prefiere manual para evitar la dependencia, se modifica este ADR y se implementa con `impl Display` y `impl Error` tradicionales.
3. **Por qué Arc<dyn …> y no genéricos**:
   - FFI-friendliness para los bindings futuros.
   - Posibilidad de proveer un `Default` sensato para `Logger`.
   - Composición natural para múltiples sinks y filtros en colecciones como `Vec`.
   El coste de indirect call (~1-2 ns) es despreciable.
4. **Por qué no migrar a workspace en este sprint**:
   Para evitar añadir ruido y fricción extra de paths en un refactor que ya es de gran envergadura. Se dividirá cuando se integren los primeros bindings.
5. **Por qué FnOnce() -> String y no Arguments ni &str**:
   `&str` obliga a construir el String antes de la llamada, violando el fast-path. `Arguments` acopla el dominio a `core::fmt`. `FnOnce` permite evaluación perezosa testeable fácilmente y pospone la alocación al momento en que el filtro aprueba el log.
6. **Por qué SimpleTextFormatter y no saltar directo a JSON**:
   Fiel a la restricción del Sprint 1 de no añadir nuevas funcionalidades al usuario final y asegurar que los tests de salida preexistentes sigan pasando.

---

## Consecuencias

**Positivas:**
- `R1`, `R2`, `R19`, `R20`, `R34`, `R36`, `R39` quedan desbloqueados a nivel de diseño.
- API FFI-friendly desde el día 1.
- Testabilidad mejorada mediante mocks sobre traits sin acoples a tipos concretos.
- Mayor robustez (sin panic!).

**Negativas o restricciones introducidas:**
- Refactorización de gran volumen (~13 archivos nuevos).
- Workspace no migrado temporalmente.
- String en el fast-path activo: se sigue alocando al loguear.
- Overhead menor por dynamic dispatch en llamadas de logging.
- `thiserror` añadida como dependencia.

---

## Módulos afectados

| Módulo / Archivo | Tipo de cambio |
| :--- | :--- |
| `src/lib.rs` | Modificado — re-exports actualizados |
| `src/level.rs` | Eliminado — movido a `src/domain/level.rs` |
| `src/config.rs` | Eliminado — LoggerConfig se reemplaza por LoggerBuilder |
| `src/logger.rs` | Eliminado — reemplazado por `src/app/logger.rs` |
| `src/sink.rs` | Eliminado — movido y dividido |
| `src/domain/` | Nuevo — nivel, event, error |
| `src/ports/` | Nuevo — traits sink, formatter, filter |
| `src/adapters/` | Nuevo — console sink y formatter simple |
| `src/app/` | Nuevo — logger y builder |
| `examples/test.rs` | Modificado — usa la nueva API |
| `Cargo.toml` | Modificado — añade `thiserror` |

---

## Criterio de implementación completa

- [ ] `cargo build` sin warnings con `#![deny(warnings)]` en `src/lib.rs`.
- [ ] `cargo test` con todos los tests unitarios y de integración en verde.
- [ ] `cargo clippy --all-targets -- -D warnings` sin warnings.
- [ ] Comprobación de que no hay `unwrap()`, `expect()`, ni `panic!()` accidentales en el dominio.
- [ ] `Logger::default().info("hola")` imprime en consola con formato `[INFO] hola`.
- [ ] El test de evaluación perezosa demuestra que la closure no corre si el filtro rechaza el log.
