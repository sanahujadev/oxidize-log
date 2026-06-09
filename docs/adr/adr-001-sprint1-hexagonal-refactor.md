# ADR-001 — Refactor del prototipo a arquitectura hexagonal (Sprint 1)

> **Architecture Decision Record.** Documento inmutable una vez cerrado.
> Creado por el agente `architect` (MiniMax). Nunca se edita — si la decisión cambia, se abre un nuevo ADR
> que referencia y supera a este.
>
> **Estado:** `en implementación` (iteración 2 — APROBADO CONDICIONAL Qwen)
> **Fecha de decisión:** 2026-06-09
> **Revisor:** Qwen (Arquitecto Segundo)
> **Iteraciones anteriores:**
> - [`adr-001-sprint1-hexagonal-refactor.v0.md`](adr-001-sprint1-hexagonal-refactor.v0.md) — propuesta inicial
> - [`adr-001-sprint1-hexagonal-refactor.v1.md`](adr-001-sprint1-hexagonal-refactor.v1.md) — iteración 1 (resolución C1, C2, C3, I1–I4, M1, M2)
> - **Esta versión (v2)**: iteración 2 — corrección de 3 errores técnicos, 4 gaps de especificación y 7 observaciones de coherencia detectadas en la segunda lectura de Qwen
>
> **Sprint asociado:** Sprint 1 — *Hexagonal Refactor (sin features nuevas)*

---

## Contexto

El proyecto `oxidize-log` parte de un **prototipo mínimo** organizado como crate único con cinco ficheros planos (`src/lib.rs`, `src/level.rs`, `src/config.rs`, `src/logger.rs`, `src/sink.rs`) más un ejemplo (`examples/test.rs`) y un documento de diseño V0 con 39 requisitos (R1–R39) priorizados en P0/P1/P2 y tipados como F/NF.

El prototipo cumple **R3** (los seis niveles existen) y la **mecánica básica** (un logger, un sink de consola, configuración por entorno), pero el diagnóstico técnico cruzándolo contra el V0 revela que la base actual **bloquea o contradice** varios requisitos P0 no funcionales y, con ella, la mayor parte de las features P1/P2:

| Requisito V0 | Estado en el prototipo | Bloqueo que produce |
|---|---|---|
| **R1** (workspace `logger-core` + bindings) | Crate único, sin `[workspace]` | Imposible empezar R23–R30 sin reorganizar primero |
| **R2** (core único, sin duplicación) | Cumplido de facto, pero frágil: todo en un módulo | Si los bindings se enganchan ahora, acoplarán al detalle del prototipo |
| **R19** (`Send + Sync` por defecto) | `trait Sink` **no declara** `Send + Sync` | `Box<dyn Sink>` no cruza threads; un test paralelo ya no compilaría |
| **R20** (atomicidad de línea en archivo) | No hay sink de archivo aún, pero no hay `Result` de vuelta en `Sink::log` | No hay canal para reportar errores de I/O |
| **R34** (fast path barato) | `Logger::log(level, &str)` recibe el mensaje ya construido | Si el nivel está desactivado, ya se pagó la alocación del `String` en el call site |
| **R36** (sin `panic!`, errores tipados) | `Sink::log` devuelve `()`; no hay tipo `LogError` | Imposible degradar con gracia — un sink que falle no tiene a quién reportar |
| **R39** (`Default` sensato) | `LoggerConfig` no implementa `Default`; sólo `from_env(Environment)` | El usuario no puede hacer `Logger::default().info("hola")` |
| **R5 / R6** (formatos texto y JSON) | Formato `println!("[{:?}] {}", level, message)` **hardcodeado dentro de `ConsoleSink`** | Cualquier formato nuevo obliga a modificar el sink, no a inyectar un adaptador |
| **R33** (niveles por módulo) | Un único filtro global por `level < config.level` | No hay `trait Filter` que permita reglas por metadata |

A esto se suman **defectos de diseño** que el Sprint 1 debe corregir aunque no aparezcan explícitamente en el V0:

- `SinkConfig` es un `enum` cerrado con una sola variant (`Console`). Cualquier sink futuro (R14 archivo, R15 rotación, R16 CloudWatch) obliga a tocar el enum, el `build_sinks` y el `match` del `Logger` → **violación de OCP**, deuda que se paga con intereses.
- `trait Sink` exige `fn as_any(&self) -> &dyn Any`. Esta firma **solo la usa el test** para hacer `downcast_ref::<MockSink>()`. Es un olor: en arquitectura hexagonal se testea contra el trait, no contra el tipo concreto.
- `Logger.config` es `pub`. Encapsulación rota: los consumidores leen/escriben la configuración interna.
- `Logger::init` recibe `LoggerConfig` y **fabrica** los sinks internamente. La hexagonal dice: el orquestador recibe los adaptadores ya construidos; la fabricación es del builder.
- Test `console_sink_prints` no tiene ningún `assert!`. Pasa aunque `ConsoleSink::log` no haga nada.
- Test `levels_are_ordered` usa `(Level::Trace as u8) < (Level::Debug as u8)`. Funciona, pero **no testea el `Ord` derivado** y enmascara reordenamientos accidentales de los variants.

**Conclusión del diagnóstico**: el prototipo cumple la *prueba de escritorio* del logging básico, pero la base arquitectónica no soporta los requisitos P0 NF del V0. **Hay que refactorizar antes de añadir features**, en un sprint dedicado que no entregue funcionalidad nueva y cuyo criterio de cierre sea la invariante *"el comportamiento observable del usuario no cambia, pero la base es hexagonal y testeable"*.

---

## Decisión

> **Adoptamos** una arquitectura **hexagonal en cuatro capas** (`domain`, `ports`, `app`, `adapters`) para el núcleo de `oxidize-log`, con tres **traits como puertos** (`Sink`, `Formatter`, `Filter`), un **`Logger` orquestador puro** que sólo conoce el dominio y los puertos, **errores tipados** (`LogError`, **implementación manual** sin dependencias externas), **fast path basado en closure** para cumplir R34, y un **`Default` sensato** que cumpla R39 sin depender de un enum `Environment` externo.

Tres decisiones arquitectónicas **adoptadas tras la revisión de Qwen** (C1, C2, C3) y que son parte integral de esta decisión:

- **El `Formatter` es propiedad del `Sink`, no del `Logger`** (resolución C1). Cada `Sink` decide cómo serializa el `LogEvent` que recibe: `ConsoleSink` lo formatea como texto y lo escribe a un `Write`, `JsonSink` (futuro) lo serializa a JSON, `CloudWatchSink` (futuro) lo envía estructurado. El `Logger` no conoce ningún formato.
- **Sprint 1 entrega métodos helper por nivel (`info`, `debug`, …) sin captura de metadatos**. R7 (captura de `file!`/`line!`/`module_path!` vía macros) queda **explícitamente en Sprint 2** (resolución C2). Los métodos usan `Metadata::UNKNOWN` como placeholder.
- **`LoggerConfig` y `Environment` se conservan como fachadas delgadas** que delegan al `LoggerBuilder` interno (resolución C3, Opción A). La API pública antigua sigue funcionando; los tests `config_for_dev/staging/prod` siguen pasando sin cambios.

El Sprint 1 es **un refactor sin features nuevas**: el usuario final sigue viendo logs por consola, con el mismo aspecto, pero el código interno queda preparado para implementar R7, R10, R14, R15, R37, etc. en sprints posteriores sin nuevas reescrituras estructurales.

### Restricciones autoimpuestas

- **Una sola decisión arquitectónica por ADR** (este). Las decisiones tácticas que vayan surgiendo (forma exacta del binario, política de flushing, etc.) son **especificaciones derivadas** de esta decisión, no ADRs separados. Si en revisión Qwen detecta que alguna merece su propio PDR, se abre.
- **No se introducen nuevas dependencias de terceros**. `thiserror` se rechazó tras revisión de Qwen (§ I1). `LogError` se implementa a mano.
- **No se añade funcionalidad visible al usuario en este sprint** (sin colores, sin JSON, sin archivo, sin macros, sin captura de metadatos). R10 (colores), aunque es P0 en el V0, **se pospone a Sprint 2**; ver *Motivo* §1 (coherencia C1).
- **No se migra a workspace `[workspace]`** en este sprint. Razón en *Motivo* §4. Quedará como **PDR-001** (a crear al cierre de este sprint, listo para revisión de Qwen).

---

## Especificaciones Técnicas

### 1. Diseño del Core y Traits (Rust)

#### 1.1 Estructura de módulos destino

```
src/
├── lib.rs                       ← re-exports públicos (cara bindings-friendly)
├── domain/                      ← Rust puro, sin I/O, sin red, sin fs
│   ├── mod.rs
│   ├── level.rs                 ← LogLevel + Display + FromStr + as_str
│   ├── event.rs                 ← LogEvent, Metadata (con ::new y ::UNKNOWN)
│   └── error.rs                 ← LogError (implementación manual, ~30 líneas)
├── ports/                       ← Traits. Sólo dependen de `domain` y `core`
│   ├── mod.rs
│   ├── sink.rs                  ← trait Sink
│   ├── formatter.rs             ← trait Formatter
│   └── filter.rs                ← trait Filter (enabled toma Metadata por valor, I2 + S4)
├── adapters/                    ← Implementaciones. Importan lo que necesiten
│   ├── mod.rs
│   ├── console.rs               ← ConsoleSink (con Formatter + Write propios, Send+Sync — T1)
│   └── text_format.rs           ← SimpleTextFormatter
├── app/                         ← Orquestador y configuración
│   ├── mod.rs
│   ├── logger.rs                ← Logger { filtros, sinks, last_error } + Default + helpers
│   ├── config.rs                ← LoggerBuilder + fachadas LoggerConfig/Environment
│   └── level_filter.rs          ← LevelFilter (impl de Filter por nivel mínimo) — S2
└── tests/                       ← (Pendiente: tests de integración cross-módulo)
```

#### 1.2 Value Objects (capa `domain`)

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]   // ← Copy tras revisión Qwen (I2)
pub struct Metadata {
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
    // function: Option<&'static str> queda para R8, no se incluye en Sprint 1
}

impl Metadata {
    /// Placeholder usado por los métodos helper (`info`, `debug`, ...) en Sprint 1.
    /// En Sprint 2 las macros `info!` / `error!` sustituirán este valor por la
    /// metadata real capturada del call site (resolución Qwen C2).
    pub const UNKNOWN: Metadata = Metadata {
        module: "<unknown>",
        file:   "<unknown>",
        line:   0,
    };

    /// Constructor explícito para tests y para el call site de `Logger::log`
    /// cuando se quiere metadata real (test 12, p. ej.). — Qwen S3
    pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
        Self { module, file, line }
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub metadata: Metadata,
}
```

```rust
// domain/error.rs
// Implementación MANUAL (sin thiserror, tras revisión Qwen — I1).
// Ver sección *Motivo* §2 para la justificación.
use std::fmt;

#[derive(Debug)]
pub enum LogError {
    InvalidLevel(String),
    InvalidMetadata,
    Write { sink: &'static str, source: Box<dyn std::error::Error + Send + Sync> },
    Format { formatter: &'static str, source: Box<dyn std::error::Error + Send + Sync> },
    Config(&'static str),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLevel(s)      => write!(f, "invalid log level: {s}"),
            Self::InvalidMetadata      => write!(f, "invalid log metadata"),
            Self::Write { sink, source } => write!(f, "write error in sink {sink}: {source}"),
            Self::Format { formatter, source } => write!(f, "format error in {formatter}: {source}"),
            Self::Config(msg)          => write!(f, "logger configuration error: {msg}"),
        }
    }
}

impl std::error::Error for LogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { source, .. } | Self::Format { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
```

#### 1.3 Puertos (capa `ports`)

```rust
// ports/sink.rs
/// El `Sink` recibe un `LogEvent` estructurado y decide cómo serializarlo
/// y dónde escribirlo. Esto permite que R6 (JSON) y R16 (CloudWatch) sean
/// adaptadores puros sin tocar el core (resolución Qwen C1).
pub trait Sink: Send + Sync {
    fn write(&self, event: &LogEvent) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError> { Ok(()) }
}
```

```rust
// ports/formatter.rs
/// El `Formatter` transforma un `LogEvent` en su representación textual (o binaria).
/// El `Logger` no conoce al `Formatter`: cada `Sink` lo inyecta en su constructor
/// si lo necesita (resolución Qwen C1).
///
/// Decisión de forma (resolución Qwen I3): devuelve un `String` owned.
/// No fingimos thread-local buffer reuse en Sprint 1 — la simplificación es
/// deliberada y la optimización se aborda en el sprint de rendimiento (R35).
pub trait Formatter: Send + Sync {
    fn format(&self, event: &LogEvent) -> Result<String, LogError>;
}
```

```rust
// ports/filter.rs
/// El `Filter` es el ÚNICO punto donde se aplica el fast-path. Si devuelve `false`,
/// el `Logger` ni siquiera evalúa la closure del mensaje (R34).
///
/// `Metadata` se recibe por valor (Qwen S4) por consistencia con su `Copy`;
/// ahorra un `&` y elimina la ambigüedad de "es solo punteros, ¿por qué presto?".
pub trait Filter: Send + Sync {
    fn enabled(&self, metadata: Metadata, level: LogLevel) -> bool;
}
```

**Reglas de uso**:
- El `Logger` no llama a `Formatter::format` (no tiene acceso a él). El formateo es responsabilidad del `Sink`.
- El `Logger` no llama a `Sink::write` si todos los `Filter` han devuelto `false` (fast path, R34).
- Si un `Sink` falla, su `Err` se reporta a través de `LogError` y el `Logger` continúa con el siguiente sink (R36). El error queda accesible vía el campo `last_error: Arc<Mutex<Option<LogError>>>` del `Logger` para tests; en producción se loguea a `stderr` como degradación graciosa. **Limitación documentada** (coherencia C5): `last_error` sólo conserva el **último** error. Si se necesita historial, se migrará a `Vec<LogError>` en un sprint futuro; para Sprint 1 `Option` es suficiente para tests y degradación graciosa.

#### 1.4 Abstracción: genéricos estáticos vs `dyn`

**Decisión**: **`Arc<dyn Trait>` con coerción, no genéricos**. Inalterado respecto al v0 — Qwen lo aprobó.

| Aspecto | Genéricos (`Logger<S: Sink, F: Formatter, ...>`) | Trait objects (`Arc<dyn Sink>`) |
|---|---|---|
| Cero overhead en hot path | ✅ monomorfización | ❌ un indirect call por dispatch |
| Componer varios sinks | ❌ requiere tuplas o `Vec<Box<dyn Sink>>` ya de vuelta | ✅ natural |
| Componer varios filtros | ❌ idem | ✅ natural |
| `Default` sensato sin tipos concretos | ❌ `Default` no puede devolver genéricos libres | ✅ `Default` devuelve `Arc<dyn …>` |
| API bindings-friendly (FFI/WASM) | ❌ los generics no cruzan FFI limpio | ✅ `Arc<dyn …>` se traduce trivialmente |
| Testabilidad | ⚠️ requiere un tipo por combinación | ✅ un mock implementa el trait y se inyecta |

La **composición**, la **bindings-friendliness** y la posibilidad de que **cada sink elija su propio `Formatter`** (resolución C1) ganan. El coste del indirect call es del orden de nanosegundos y se documenta como restricción a vigilar en el sprint de rendimiento (R34+R35). Si en un futuro se necesita cero overhead en un caso específico, se ofrece una API estática opcional *al lado* de la API dinámica, no como reemplazo.

#### 1.5 Logger orquestador (capa `app`)

```rust
// app/logger.rs
use std::sync::{Arc, Mutex};

pub struct Logger {
    filters: Vec<Arc<dyn Filter>>,
    sinks: Vec<Arc<dyn Sink>>,
    /// Acumulador de errores de los sinks para inspección (R36, degradación
    /// graciosa). En producción el logger vuelca `last_error` a `stderr`.
    /// Limitación (Qwen C5): sólo guarda el último error. — Qwen T2
    last_error: Arc<Mutex<Option<LogError>>>,
}

impl Logger {
    /// Método principal. La closure `message` NO se evalúa si los filtros
    /// rechazan el nivel: es la única garantía formal de R34.
    /// `metadata` se pasa por valor porque `Metadata: Copy` (consistencia con
    /// `Filter::enabled` — Qwen S4).
    pub fn log<F>(&self, level: LogLevel, metadata: Metadata, message: F)
    where
        F: FnOnce() -> String,
    {
        if !self.filters.iter().all(|f| f.enabled(metadata, level)) {
            return; // fast path — la closure nunca se invoca
        }
        let event = LogEvent { level, message: message(), metadata };
        for sink in &self.sinks {
            if let Err(e) = sink.write(&event) {
                self.record_error(e);
            }
        }
    }

    /// Registra un error de un sink. R36: nunca panics; degrada con gracia.
    /// El `Mutex` puede envenenarse sólo si un `panic!` ocurrió mientras se
    /// sostenía el lock; eso sería una violación del criterio "sin panic en
    /// producción". En ese caso, lo recuperamos devolviendo el `None` previo
    /// para no propagar el envenenamiento.
    fn record_error(&self, err: LogError) {
        match self.last_error.lock() {
            Ok(mut slot) => *slot = Some(err),
            Err(poisoned) => {
                // Recuperación defensiva: extraemos el guard aunque esté envenenado.
                let mut slot = poisoned.into_inner();
                *slot = Some(LogError::Config("previous sink error poisoned the lock"));
            }
        }
    }

    // ── Métodos helper por nivel (Sprint 1). Usan Metadata::UNKNOWN como
    //    placeholder. En Sprint 2 las macros info!/error!/... los sustituirán
    //    por versiones que capturan file!/line!/module_path! del call site
    //    (resolución Qwen C2).
    pub fn trace<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Trace, Metadata::UNKNOWN, m); }
    pub fn debug<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Debug, Metadata::UNKNOWN, m); }
    pub fn info <F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Info,  Metadata::UNKNOWN, m); }
    pub fn warn <F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Warn,  Metadata::UNKNOWN, m); }
    pub fn error<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Error, Metadata::UNKNOWN, m); }
    pub fn fatal<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Fatal, Metadata::UNKNOWN, m); }
}

impl Default for Logger {
    fn default() -> Self {
        // R39: consola (stdout) + SimpleTextFormatter + LevelFilter::Info
        LoggerBuilder::new().build()
    }
}
```

**El `Logger` no implementa `Clone` directamente**: sus campos ya son `Arc<dyn _>` y `Arc<Mutex<...>>`, así que un `#[derive(Clone)]` barato funciona y mantiene la sharing semantics de R19. El test del bloque C lo verifica.

#### 1.6 Builder y fachada de configuración (capa `app`)

```rust
// app/level_filter.rs  (Qwen S2)
pub struct LevelFilter {
    min: LogLevel,
}

impl LevelFilter {
    pub fn new(min: LogLevel) -> Self {
        Self { min }
    }
}

impl Filter for LevelFilter {
    fn enabled(&self, _metadata: Metadata, level: LogLevel) -> bool {
        level >= self.min
    }
}
```

```rust
// app/config.rs
pub struct LoggerBuilder {
    filters: Vec<Arc<dyn Filter>>,
    sinks:   Vec<Arc<dyn Sink>>,
}

impl LoggerBuilder {
    /// Constructor con default sensato (R39) — Qwen S1.
    pub fn new() -> Self {
        Self {
            filters: vec![Arc::new(LevelFilter::new(LogLevel::Info))],
            sinks:   vec![Arc::new(ConsoleSink::new(Arc::new(SimpleTextFormatter)))],
        }
    }

    pub fn filter(mut self, f: Arc<dyn Filter>) -> Self {
        self.filters.push(f);
        self
    }

    pub fn sink(mut self, s: Arc<dyn Sink>) -> Self {
        self.sinks.push(s);
        self
    }

    /// Atajo: ajusta el `LevelFilter` por defecto al nivel indicado.
    /// Si ya hay filtros configurados, añade un `LevelFilter` adicional.
    /// Para control fino, usar `.filter(Arc::new(LevelFilter::new(level)))` directamente.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.filters.push(Arc::new(LevelFilter::new(level)));
        self
    }

    pub fn build(self) -> Logger {
        Logger {
            filters: self.filters,
            sinks:   self.sinks,
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Fachada de compatibilidad: convierte un `LoggerConfig` (antiguo) en un builder.
    /// — Qwen C3
    pub fn from_config(config: LoggerConfig) -> Self {
        // Estrategia: si config.sinks contiene Console, añadimos un ConsoleSink con
        // SimpleTextFormatter (o un "raw formatter" sin colores, ya que colors=false
        // se ignora en Sprint 1). El nivel mínimo del LevelFilter es config.level.
        let mut builder = LoggerBuilder::new().level(config.level);
        for sink_cfg in &config.sinks {
            match sink_cfg {
                SinkConfig::Console => {
                    builder = builder.sink(Arc::new(ConsoleSink::new(Arc::new(SimpleTextFormatter))));
                }
            }
        }
        builder
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self { Self::new() }
}
```

**Fachadas preservadas (resolución Qwen C3, Opción A)**:

```rust
// En `app/config.rs` se mantienen estos tres tipos como wrappers
// que delegan al builder. La API pública antigua sigue funcionando
// y los tests `config_for_dev/staging/prod` no se tocan.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment { Dev, Staging, Prod }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkConfig { Console }   // se conserva el nombre por compatibilidad,
                                  // pero deja de ser el mecanismo de inyección

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub colors: bool,    // ← se conserva el campo aunque en Sprint 1 no se use
                         //   (R10 llega en Sprint 2). Mantenerlo evita churn
                         //   futuro en la API pública.
    pub sinks: Vec<SinkConfig>,
}

impl LoggerConfig {
    pub fn from_env(env: Environment) -> Self { /* mismo código que en el prototipo */ }
}

impl Logger {
    /// Atajo de compatibilidad: `Logger::init(cfg)` es equivalente a
    /// `LoggerBuilder::from_config(cfg).build()`.
    pub fn init(config: LoggerConfig) -> Self {
        LoggerBuilder::from_config(config).build()
    }
}
```

> `SinkConfig` se mantiene como enum **interno** de la fachada `LoggerConfig`, no como mecanismo de inyección de sinks. La inyección real de sinks se hace siempre vía `LoggerBuilder::sink(...)` con un `Arc<dyn Sink>` ya construido. Esto preserva la API pública del prototipo (C3) sin contaminar el dominio (hexagonal intacta).

#### 1.7 API pública del core (cara de los bindings)

`src/lib.rs` expone **únicamente** tipos que cruzan FFI limpiamente:

```rust
pub use domain::{LogLevel, LogEvent, Metadata, LogError};
pub use ports::{Sink, Formatter, Filter};
pub use adapters::{ConsoleSink, SimpleTextFormatter};
pub use app::{Logger, LoggerBuilder, LoggerConfig, Environment, SinkConfig, LevelFilter};
// app::macros  ← R37, Sprint 2 — placeholder en Sprint 1
```

**No** se re-exporta nada que contenga generics en su firma pública.

---

### 2. Bindings y Capa de FFI (Multiplataforma)

> Este sprint **no entrega bindings**. La sección deja constancia de las implicaciones del refactor sobre los futuros R23–R30.

- **Tecnología**: PDR pendiente (WASM con `wasm-bindgen` para JS, JNI con `jni` para Java). La elección se aborda en **PDR-002** tras cerrar este sprint.
- **Pasaje de datos**: el refactor deja la API en `Arc<dyn …>` y value objects `Clone + Send + Sync`, lo que es traducible a FFI plana (`extern "C"`) o a `wasm-bindgen` sin conversiones exóticas. Los strings se exponen como `*const c_char` en FFI; los enums como `u8`; los `LogError` como código + mensaje.
- **No se introduce** `napi-rs`, `wasm-bindgen`, `jni` ni `abi_stable` en este sprint. Estas dependencias llegan con sus PDRs.

**Implicación concreta del refactor**:
- El `Default for Logger` y la ausencia de genéricos en la API pública son **pre-condiciones** para que los bindings del futuro no necesiten wrappers adaptados al tipo concreto.
- La elección de `&'static str` en `Metadata` (M2 — observación de Qwen) **obliga** a los bindings futuros a usar `Box::leak` o un pool de strings para los nombres de módulo/archivo. Se reevaluará en PDR-002 (bindings) si conviene migrar a `Cow<'static, str>` o a un `Metadata` owned. En Sprint 1 no impacta porque los métodos helper usan `Metadata::UNKNOWN` (literales estáticos).

---

### 3. Rendimiento, Concurrencia y Memoria

#### 3.1 Sincronización

- **Los traits exigen `Send + Sync` en la declaración** (no implícitos). Esto cierra R19.
- La sincronización fina (`Mutex`, `RwLock`, `lock-free`) es **responsabilidad del adaptador**, no del dominio. `ConsoleSink` usa `Arc<Mutex<Box<dyn Write + Send + Sync>>>` (Qwen T1) para serializar escrituras. `FileSink` (sprint futuro) llevará `Mutex<BufWriter<File>>`.
- El `Logger` no tiene estado mutable compartido fuera de sus `Arc<dyn _>` y el `Arc<Mutex<Option<LogError>>>` de `last_error`. Inserciones y lecturas son `&self`.
- **Nota sobre el `Mutex` en `ConsoleSink` (Qwen C4)**: el `Mutex` es necesario para `Write` arbitrarios (e.g. `File` o `Vec<u8>` en tests) que no son thread-safe. Para `Stdout` en producción, `Stdout` ya es internamente lock-safe y el `Mutex` añade overhead. Se mantiene por simplicidad de API en Sprint 1; en el sprint de rendimiento (R35) se puede optimizar con un `enum Writer { Stdout, Other(Mutex<...>) }` o un parámetro de tipo en `ConsoleSink` si los benchmarks lo justifican.

#### 3.2 Memoria y `no_std`

- El núcleo **no** será `no_std` en Sprint 1 (mantiene `std` por simplicidad de tests, I/O, y `Box<dyn Error>`).
- **Sí** se mantiene la disciplina: `domain/` no usa `Vec`, `String`, ni nada que no esté también en `alloc` o `core`. Si en un futuro se quiere `no_std + alloc`, la migración es viable — y de hecho **más fácil** ahora que `LogError` es manual (resolución Qwen I1: `thiserror` depende de `std::error::Error` y no compila en `no_std`).
- Las alocaciones de `String` están en:
  - El cierre del fast path (`message()`).
  - El formateador (`Formatter::format` devuelve un `String`).
  - El `Mutex<Box<dyn Write + Send + Sync>>` del sink, que en `ConsoleSink` es `Box::new(Vec::new())` en tests y `Box::new(Stdout)` en default.
- `no_std` es un objetivo documentado pero **no** una restricción de Sprint 1.

#### 3.3 Fast path (R34)

**Decisión**: el método `log` toma un **`impl FnOnce() -> String`**, no un `&str` ni un `String`.

```rust
pub fn log<F>(&self, level: LogLevel, metadata: Metadata, message: F)
where F: FnOnce() -> String,
{
    if !self.filters.iter().all(|f| f.enabled(metadata, level)) {
        return; // la closure nunca se invoca
    }
    let event = LogEvent { level, message: message(), metadata };
    for sink in &self.sinks {
        if let Err(e) = sink.write(&event) {
            self.record_error(e);
        }
    }
}
```

El test **TDD-14** del Sprint 1 (`logger_log_evalua_mensaje_via_closure`) verifica con un `Cell<u32>` que la closure **no se invoca** cuando el nivel está desactivado. Esa es la única garantía formal de R34 en este sprint.

**Limitaciones explícitas** (aceptadas tras revisión de Qwen, I3 e I4):
- Cuando el log *sí* se emite, hay una alocación del `String` del mensaje **y** una alocación del `String` del formateador. Reducir esto a zero-alloc con `core::fmt::Arguments` es objeto de un **sprint de rendimiento futuro** (R34+R35), no de este ADR.
- Se eligió `String` y no `Cow<'a, str>` (aceptado por Qwen con nota de tradeoff, I4) por simplicidad de API. La alocación extra por literal es aceptable para Sprint 1; se reevaluará si los benchmarks lo justifican.

#### 3.4 Sin `panic!` (R36)

- Ningún `unwrap()`, `expect()` o `panic!()` en `src/domain/`, `src/ports/`, `src/app/`. Verificable con:
  ```
  ! grep -rE "\b(unwrap|expect|panic!)\b" src/domain src/ports src/app
  ```
- En `src/adapters/`, los `unwrap` están **prohibidos** salvo justificación inline. La única excepción documentada en este sprint es el `expect("Mutex poisoned")` dentro de `ConsoleSink::write`/`flush`, que recupera envenenamiento vía `into_inner()` (ver `Logger::record_error`) — esto NO es un `unwrap`, es recuperación explícita.
- Errores de I/O en adaptadores se convierten en `LogError::Write`. Errores de configuración en `LogError::Config`.
- Cuando un `Sink::write` devuelve `Err`, el `Logger` no entra en panic: lo guarda en `last_error` (visible para tests) y **continúa con el siguiente sink**. En producción, el `Logger` vuelca `last_error` a `stderr` como degradación graciosa.

---

### 4. Sinks e Infraestructura (Adaptadores)

#### 4.1 I/O y Buffering

```rust
// adapters/console.rs (esquema)
use std::io::Write;

pub struct ConsoleSink {
    formatter: Arc<dyn Formatter>,
    /// `Box<dyn Write + Send + Sync>` (Qwen T1) garantiza que el `ConsoleSink`
    /// cumple el bound `Send + Sync` que exige el trait `Sink`.
    writer: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
}

impl ConsoleSink {
    /// Constructor para uso en producción: escribe a `stdout`.
    pub fn new(formatter: Arc<dyn Formatter>) -> Self {
        Self {
            formatter,
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
        }
    }
    /// Constructor para tests: escribe a un `Write` arbitrario (típicamente
    /// un `Arc<Mutex<Vec<u8>>>` capturado por el test).
    pub fn with_writer(
        formatter: Arc<dyn Formatter>,
        writer: Arc<Mutex<dyn Write + Send + Sync>>,
    ) -> Self {
        Self { formatter, writer }
    }
}

impl Sink for ConsoleSink {
    fn write(&self, event: &LogEvent) -> Result<(), LogError> {
        let formatted = self.formatter.format(event)
            .map_err(|e| LogError::Format { formatter: "ConsoleSink.formatter", source: Box::new(e) })?;
        let mut w = self.writer.lock().map_err(|_| {
            LogError::Write { sink: "ConsoleSink", source: "writer mutex poisoned".into() }
        })?;
        w.write_all(formatted.as_bytes())
            .map_err(|e| LogError::Write { sink: "ConsoleSink", source: Box::new(e) })?;
        Ok(())
    }
    fn flush(&self) -> Result<(), LogError> {
        let mut w = self.writer.lock().map_err(|_| {
            LogError::Write { sink: "ConsoleSink", source: "writer mutex poisoned".into() }
        })?;
        w.flush()
            .map_err(|e| LogError::Write { sink: "ConsoleSink", source: Box::new(e) })
    }
}
```

- `ConsoleSink` **delega** el formato al `Formatter` que recibe en el constructor (resolución Qwen C1). El sink no contiene lógica de formato.
- `SimpleTextFormatter` produce, por defecto, un formato **compatible** con el `println!("[{:?}] {}", level, message)` actual. Esto preserva el comportamiento observable y evita romper expectativas de usuarios tempranos. La forma exacta del formato la fija el test TDD-9.
- **Síncrono** en este sprint. R22 (buffering/async) se aborda en un sprint dedicado.
- `ConsoleSink::flush()` existe desde el día 1: aunque `stdout` no lo necesite, otros `Write` sí, y la API del trait debe ser uniforme.

#### 4.2 Dependencias de terceros

> **Revisión Qwen (I1)**: `thiserror` se **rechaza** como dependencia. `LogError` se implementa a mano (~30 líneas en `src/domain/error.rs`). Justificación en *Motivo* §2.

**No se añade ninguna dependencia de terceros en Sprint 1** (Qwen C6). El `Cargo.toml` del prototipo puede contener dependencias **comentadas** (e.g. `serde`, `chrono`, `colored`); **se mantienen comentadas** tal cual y **no se descomentan en Sprint 1**. Se descomentarán en sus respectivos sprints:

- `serde_json` → Sprint 2 con R6 (formato JSON)
- `chrono` o `time` → Sprint 2 con R5 mejorado (timestamp real, no placeholder)
- `colored` o `owo-colors` → Sprint 2 con R10 (colores ANSI)
- `aws-sdk-cloudwatchlogs` → Sprint P2 con R16

**No** se añaden en este sprint, entre otras:

- `thiserror`, `anyhow`, `snafu` → manual
- `tracing`, `log` → nunca (son competencia, no dependencias)
- `napi-rs`, `wasm-bindgen`, `jni` → con sus PDRs
- `parking_lot`, `crossbeam`, `once_cell` → con su PDR si se necesitan
- `gag`, `os_pipe` (redirigir stdout en tests) → **no** se añaden; los tests redirigen vía `ConsoleSink::with_writer` y un `Vec<u8>`

#### 4.3 Sinks entregados en Sprint 1

- **`ConsoleSink`** — único sink del sprint. Es el adaptador de R13.
- **No** se entrega `FileSink`, `RotatingFileSink`, ni `CloudWatchSink` en Sprint 1. Su ausencia **no contradice** el V0: P0 cubre R13 (consola) y R14 (archivo simple) pero el alcance de Sprint 1 es **refactor**, no implementación de R14.

---

## Motivo

### 1. Por qué hexagonal y no quedarse en el prototipo plano

El prototipo plano funcionaba para los **dos** casos de uso que cubre (Dev loguea, Prod loguea, todos a consola). El V0 documenta **39 requisitos**, varios con dependencias cruzadas (R1↔R23↔R27, R5↔R6↔R10, R14↔R15↔R17↔R20). En la práctica esto significa que **cualquier feature nueva va a tocar mínimo tres archivos** del prototipo. La hexagonal reduce eso a **un trait nuevo en `ports/` + un adaptador nuevo en `adapters/`**, sin tocar el `Logger` ni el dominio. Es una decisión de productividad a medio plazo, no de pureza arquitectónica.

**Sobre la posposición de R10 a Sprint 2 (Qwen C1)**: R10 (colores ANSI) es P0 en el V0, pero se pospone a Sprint 2 porque Sprint 1 es **un refactor sin features nuevas**. La justificación es que los colores **no bloquean la base arquitectónica**: cuando llegue R10, se implementa como un `ColorFormatter` (o un `ColoredConsoleSink` que envuelve a `SimpleTextFormatter`) **sin tocar el core**. Si la decisión se reconsidera y R10 debe entrar en Sprint 1, lo hace como Sprint 1.5 sin afectar al refactor; el coste de añadirlo incrementalmente es bajo. Esta es una decisión de scope explícita, no una omisión.

### 2. Por qué `LogError` manual y no `thiserror` (revisión Qwen I1)

`thiserror 1.x` se había propuesto en la v0. **Qwen objetó** con cuatro argumentos, todos válidos. Los incorporo y se descarta `thiserror` para Sprint 1:

1. **Tiempo de compilación incremental**: `thiserror` es un proc-macro. Cada `cargo build` incremental paga ~200-400ms extra. En un core de logging que se compila en cada proyecto que lo use, no es despreciable.
2. **Dependencias transitivas**: `thiserror` arrastra `thiserror-impl` (proc-macro). Son dos crates en el árbol de dependencias del usuario.
3. **`core::error::Error` es estable desde Rust 1.81** (noviembre 2024). Estamos en junio 2026. La implementación manual son **~30 líneas** en `src/domain/error.rs` y se ve como en §1.2.
4. **Preparación para `no_std`**: cuando el core quiera ser `no_std + alloc`, `thiserror` **no compila** (depende de `std::error::Error`). El código manual sí, con un feature flag y un polyfill mínimo de `Error` para `core::error::Error`. Adelantamos trabajo futuro al no introducir la dependencia ahora.

Conclusión: `LogError` se implementa a mano. La API pública es **idéntica** a la que se habría tenido con `thiserror` (`Display + Debug + Error + source()`), así que ningún consumidor nota la diferencia.

### 3. Por qué `Arc<dyn …>` y no genéricos

Tres razones, en orden de importancia:

1. **API bindings-friendly**. Un `Logger<S: Sink, F: Formatter>` genérico no cruza FFI limpio. WASM y JNI necesitan tipos concretos o `Arc<dyn …>`. Empezar con genéricos y migrar después es un refactor adicional gratuito.
2. **`Default` sensato (R39)**. `Default::default()` no puede devolver tipos genéricos abiertos. Con `Arc<dyn …>`, `Default for Logger` devuelve directamente un logger usable.
3. **Composición natural**. R17 pide "varios sinks simultáneos". Con genéricos esto requiere `Logger<S1, S2, ...>` o un `Vec<Box<dyn Sink>>` que rompe la monomorfización. Con `Arc<dyn …>`, `Vec<Arc<dyn Sink>>` es directo.
4. **Cada sink elige su propio `Formatter`** (resolución Qwen C1). Esto es imposible con genéricos sin una explosión combinatoria de tipos (`Logger<C: ConsoleSink, J: JsonSink, F: FileSink, ...>`).

El coste (un indirect call por log) es del **orden de 1–2 ns** en arquitecturas modernas. En el rango de operación de un logger (decenas a cientos de miles de logs/segundo) es despreciable. Si en un benchmark se observa lo contrario, se introduce una API estática opcional en el sprint de rendimiento, **sin retirar la API dinámica**.

### 4. Por qué no migrar a workspace `[workspace]` en este sprint

- Sprint 1 ya es grande: cuatro capas, tres traits, refactor de cinco archivos, 20 tests nuevos, ajustes de `lib.rs` y `examples/test.rs`. Añadir el workspace split encadena renombrados de paths en cada commit, fricción con los tests, y un `Cargo.toml` raíz que aún no aporta valor (sólo hay un crate).
- El **único motivo** para un workspace es que haya un segundo crate que comparta código. Eso ocurrirá cuando empecemos `oxidize-log-js` o `oxidize-log-java`, en sprints posteriores. En ese momento el split se hace **una vez**, con la estructura estable de `logger-core` ya consolidada.
- R1 (separación en capas) **se cumple a nivel arquitectónico** con este refactor: `domain/`, `ports/`, `app/`, `adapters/` son las capas del core, y los bindings futuros serán crates separados. La separación física en workspace es la **materialización**, no la condición.

Apertura (creación) de **PDR-001 (workspace split)** al cierre de Sprint 1, con la decisión de hacer el split cuando se introduzca el primer crate de bindings, no antes.

### 5. Por qué `FnOnce() -> String` y no `Arguments` ni `&str`

- `&str` (prototipo actual): el mensaje se construye **antes** de la llamada a `log`, así que el fast path es una mentira. R34 violado.
- `Arguments` (`core::fmt::Arguments`): zero-alloc cuando el log se emite, pero:
  - El `Formatter` tendría que aceptar `Arguments` directamente, lo que acopla el dominio a `core::fmt`.
  - Los tests de fast-path se vuelven más complicados: verificar "no se invocó la closure" requiere que haya closure, y `Arguments` no es trivialmente testeable como tal.
  - Optimización prematura: el cuello de botella de un logger está en el **sink** (I/O, lock, formateo a texto), no en construir el `String` del mensaje. Confirmar con benchmarks es el sprint R35.
- **`FnOnce() -> String`**: el call site se ve `logger.info("hola")` (gracias a los métodos helper, no a macros) o `logger.info(|| format!("hola {}", name))` — la segunda forma es la canónica cuando hay formateo. El test del fast path es trivial (`Cell<u32>` que cuenta invocaciones de la closure). La alocación del `String` se paga **solo** si el log pasa los filtros. Esto cumple R34 de forma testeable y predecible.

**Tradeoff `String` vs `Cow<'a, str>`** (nota explícita por Qwen, I4): se acepta `String` por simplicidad de API. Para literales como `logger.info("hola")`, esto aloca un `String` adicional cuando el log se emite (uno para el `message`, otro para el `String` que devuelve `Formatter::format`). En el caso **desactivado** del fast path, ninguna de esas alocaciones ocurre. Si en el sprint de rendimiento (R35) los benchmarks muestran que esa alocación importa, se reevaluará.

### 6. Por qué `SimpleTextFormatter` y no saltar directo a JSON (R6)

- R6 es **P1**, no P0. Sprint 1 no entrega features nuevas.
- El formateador de texto en Sprint 1 produce **el mismo output** que el `println!` actual (`[LEVEL] mensaje`). Esto garantiza que el comportamiento observable no cambia y los tests viejos (los que se mantienen) siguen pasando.
- Cuando llegue R6, se añade `JsonFormatter` como **otro adaptador** que implementa el mismo trait `Formatter`. El `Logger` no se entera. Esto valida la separación de puertos en la práctica.

---

## Consecuencias

### Positivas

- **R1, R2, R19, R20, R34, R36, R39** quedan **desbloqueados** a nivel de diseño. Las features concretas llegan en sprints posteriores sin reescritura estructural.
- **API bindings-friendly desde el día 1**: el `Default for Logger`, la ausencia de genéricos en la API pública, y los value objects `Clone + Send + Sync` son las pre-condiciones para R23–R30.
- **Testabilidad**: los tests usan mocks que implementan `Sink`/`Formatter`/`Filter` directamente. Se elimina `as_any()` y el downcast. Los tests pueden inyectar un `Write` arbitrario en `ConsoleSink` para verificar la salida byte a byte.
- **Sin `panic!`** en el dominio: `LogError` tipado, errores de I/O se reportan, configuración inválida se detecta en build del `Logger` (no en runtime).
- **Cero dependencias externas nuevas en Sprint 1**: el árbol del usuario no crece, el tiempo de compilación incremental no se resiente, y el core queda a un paso de `no_std + alloc` (LogError manual lo permite).
- **R39 cumplido**: `Logger::default().info("hola")` funciona end-to-end y produce `[INFO] hola` por stdout.
- **Apertura explícita de PDRs** para sub-decisiones que este sprint no resuelve (PDR-001 workspace, PDR-002 bindings, etc.), manteniendo el ADR limpio y revisable.
- **Resolución C1 (Qwen)**: cada `Sink` es libre de formatear a su manera, abriendo la puerta a `JsonSink`, `CloudWatchSink` y otros como adaptadores puros.
- **`Filter::enabled` recibe `Metadata` por valor** (Qwen S4): aprovecha el `Copy` derivado, ahorra un `&` en cada llamada, y elimina la ambigüedad visual de "estoy prestando algo que es solo punteros".

### Negativas o restricciones introducidas

- **Migración grande**: se eliminan cinco archivos del prototipo y se introducen ~13 archivos nuevos. Es un *big bang* de un solo sprint, mitigado por el hecho de que la base es pequeña.
- **Workspace no migrado** (R1 sólo cumplido a nivel arquitectónico, no físico). Riesgo bajo porque la estructura interna ya está aislada; el split será mecánico cuando ocurra.
- **`String` en fast path**: cuando el log se emite, hay alocaciones. R34 mide el **caso desactivado**, no el emitido, así que R34 está cubierto. La optimización zero-alloc con `Arguments` queda diferida al sprint de rendimiento (R35). Aceptado por Qwen (I4) con nota de tradeoff.
- **Dispatch dinámico**: un indirect call por log emitido. Ver §3. Si en benchmarks es relevante, se ofrece API estática opcional.
- **`Metadata::UNKNOWN` como placeholder en métodos helper** (resolución Qwen C2): los métodos `info`, `debug`, etc. **no capturan** `file!`/`line!`/`module_path!` del call site. Eso llega en Sprint 2 con las macros `info!`/`error!`/... Si un usuario de Sprint 1 necesita metadata, debe construir el `Metadata` con `Metadata::new(...)` y llamar a `Logger::log` directamente (test 12 muestra el patrón).
- **Limitación de `&'static str` en `Metadata` para bindings futuros** (observación M2 de Qwen): los bindings JS/Java no tendrán `&'static str` para los nombres de módulo/archivo. La elección actual obliga a `Box::leak` o a un pool de strings en el binding. Se reevaluará en PDR-002 (bindings) si conviene migrar a `Cow<'static, str>` o a un `Metadata` owned.
- **API pública de `LoggerConfig`/`Environment`/`SinkConfig` preservada como fachada** (resolución Qwen C3, Opción A): estos tres tipos **siguen existiendo** y se pueden usar, pero internamente delegan al `LoggerBuilder`. Esto preserva compatibilidad con el prototipo y mantiene los tests `config_for_dev/staging/prod` en verde, a cambio de mantener un tipo (`SinkConfig`) que ya no es el mecanismo real de inyección. Es un trade-off explícito.
- **`Sink::write` ahora hace formato + I/O** (resolución Qwen C1): si un usuario tiene N sinks que usan el mismo `Formatter`, el formateo se hace N veces en vez de una. En la práctica un `Logger` tiene 1-2 sinks y el coste de formatear texto es despreciable frente al I/O, pero conviene documentarlo. Si en el futuro hace falta formateo único, se introduce un `SharedFormatter` o un patrón de cacheo; no es problema de este sprint.
- **`last_error` sólo guarda el último error** (Qwen C5): si múltiples sinks fallan en un mismo `log()`, sólo se conserva el último. Para tests y degradación graciosa es suficiente; si se necesita historial, se migrará a `Vec<LogError>` en un sprint futuro.
- **`Mutex` redundante en `ConsoleSink` para `Stdout`** (Qwen C4): `Stdout` ya es thread-safe, pero el `Mutex` se mantiene por uniformidad con otros `Write`. Optimización prevista para el sprint de rendimiento (R35) si los benchmarks lo justifican.
- **R10 (colores) pospuesto a Sprint 2** (Qwen C1): R10 es P0 en el V0, pero Sprint 1 es un refactor sin features. La justificación está en *Motivo* §1.

---

## Módulos afectados

| Módulo / Archivo | Tipo de cambio |
|---|---|
| `src/lib.rs` | **Modificado** — re-exports actualizados a la nueva estructura |
| `src/level.rs` | **Eliminado** — movido a `src/domain/level.rs` con API ampliada |
| `src/config.rs` | **Eliminado de raíz, fachada preservada en `src/app/config.rs`** — `LoggerConfig`, `Environment`, `SinkConfig` y `Logger::init` se conservan como wrappers que delegan al `LoggerBuilder`. Tests `config_for_dev/staging/prod` siguen pasando sin cambios. (Resolución Qwen C3) |
| `src/logger.rs` | **Eliminado** — reemplazado por `src/app/logger.rs` con semántica ampliada (helpers `info`/`debug`/..., `last_error`, `record_error` con recuperación de envenenamiento) |
| `src/sink.rs` | **Eliminado** — `Sink` trait se mueve a `src/ports/sink.rs` (rediseñado, sin `as_any`, sin parámetro `bytes`), `ConsoleSink` se mueve a `src/adapters/console.rs` (con `Formatter` y `Write` inyectados, `Send + Sync`) |
| `src/domain/mod.rs` | **Nuevo** |
| `src/domain/level.rs` | **Nuevo** — `LogLevel` + `Display` + `FromStr` + `as_str` |
| `src/domain/event.rs` | **Nuevo** — `LogEvent`, `Metadata` con `Copy` (Qwen I2), `Metadata::UNKNOWN` y `Metadata::new` (Qwen S3) |
| `src/domain/error.rs` | **Nuevo** — `LogError` con `Display + Error` implementados **a mano** (Qwen I1, sin `thiserror`) |
| `src/ports/mod.rs` | **Nuevo** |
| `src/ports/sink.rs` | **Nuevo** — `trait Sink` rediseñado (`Send + Sync`, `Result`, **sin `as_any`**, **sin parámetro `bytes`**) |
| `src/ports/formatter.rs` | **Nuevo** — `trait Formatter` con firma `format -> Result<String, _>` (Qwen I3) |
| `src/ports/filter.rs` | **Nuevo** — `trait Filter` con `enabled(metadata: Metadata, level: LogLevel)` (Qwen S4, `Metadata` por valor) |
| `src/adapters/mod.rs` | **Nuevo** |
| `src/adapters/console.rs` | **Nuevo** — `ConsoleSink` con `Formatter` y `Write` **propios** (Qwen C1), `Box<dyn Write + Send + Sync>` (Qwen T1) |
| `src/adapters/text_format.rs` | **Nuevo** — `SimpleTextFormatter` |
| `src/app/mod.rs` | **Nuevo** |
| `src/app/logger.rs` | **Nuevo** — `Logger` orquestador (con `last_error`, Qwen T2) + `Default` + helpers por nivel (Qwen C2) |
| `src/app/config.rs` | **Nuevo** — `LoggerBuilder` (con default explícito R39, Qwen S1) + fachadas `LoggerConfig`/`Environment`/`SinkConfig` (Qwen C3) + `LoggerBuilder::from_config` |
| `src/app/level_filter.rs` | **Nuevo** — `LevelFilter` (implementación de `Filter` por nivel mínimo, Qwen S2) |
| `tests/` | **Nuevo directorio** (puede quedar vacío en Sprint 1; placeholder) |
| `examples/test.rs` | **Modificado** — usa los métodos helper: `let lg = Logger::default(); lg.info("hola")` (sin macros, R37 queda para Sprint 2) |
| `docs/adr/adr-001-sprint1-hexagonal-refactor.v0.md` | **Nuevo** — propuesta inicial preservada para trazabilidad |
| `docs/adr/adr-001-sprint1-hexagonal-refactor.v1.md` | **Nuevo** — iteración 1 preservada para trazabilidad |
| `docs/adr/adr-001-sprint1-hexagonal-refactor.md` | **Este documento** — iteración 2, `en implementación` |
| `docs/adr/pdr-001-workspace-split.md` | **A crear al cierre de Sprint 1** (fuera de alcance) |
| `Cargo.toml` | **Sin cambios** — no se añade ninguna dependencia en Sprint 1 (Qwen I1, C6); las dependencias comentadas se mantienen comentadas |
| `docs/roadmap.md` | **Modificado** — Sprint 1 marcado como `en revisión` y luego `en implementación` |
| `docs/next-session.md` | **Modificado** — siguiente sesión arranca en tests TDD del Sprint 1 |

---

## Criterio de implementación completa

> Verificable. Cada item es una orden que se cierra o queda abierta con justificación.

- [ ] `cargo build` sin warnings con `#![deny(warnings)]` en `src/lib.rs`
- [ ] `cargo test` con los **20 tests TDD del Sprint 1** en verde (lista cerrada, ver Anexo)
- [ ] `cargo clippy --all-targets -- -D warnings` sin warnings
- [ ] `grep -rE "\b(unwrap|expect|panic!)\b" src/domain src/ports src/app` devuelve **vacío**
- [ ] `Logger::default().info("hola")` ejecuta sin `panic!` y, con un `ConsoleSink` redirigido a un `Vec<u8>`, produce una línea que comienza con `[INFO] ` seguida de `hola` y un `\n` (test 19, smoke)
- [ ] Test 14 (`logger_log_evalua_mensaje_via_closure`) verifica con un `Cell<u32>` que la closure **no** se invoca cuando `LevelFilter` rechaza el nivel
- [ ] Test 17 (`console_sink_escribe_con_formatter_inyectado`) usa `ConsoleSink::with_writer` con un `Arc<Mutex<Vec<u8>>>` y verifica los bytes escritos
- [ ] Test 16 (`logger_propag_error_de_sink_sin_panicar`) usa un sink mock que devuelve `Err` y verifica que el logger no entra en panic y los demás sinks siguen recibiendo
- [ ] Test del bloque A: el test viejo `console_sink_prints` ha sido **reemplazado** por uno que captura la salida (vía `Write` inyectado) y verifica el contenido emitido byte a byte (Qwen M1)
- [ ] `examples/test.rs` actualizado, `cargo run --example test` produce salida visible
- [ ] `Logger::init(LoggerConfig::from_env(Environment::Dev))` **sigue funcionando** y devuelve un logger equivalente al que se construía con el prototipo (test `config_for_dev` en verde sin cambios)
- [ ] `docs/roadmap.md` actualizado con el sprint cerrado
- [ ] PDR-001 (workspace split) **creado** en `docs/adr/pdr-001-workspace-split.md` con estructura inicial (Contexto, Problema, Opciones) — listo para revisión de Qwen (Qwen C7: "creado", no "abierto")
- [ ] `Cargo.toml` **no contiene** nuevas dependencias: `grep -E "thiserror|anyhow|serde|chrono" Cargo.toml` devuelve vacío (sólo las comentadas, que ya estaban)

**Tests relevantes en verde** (los 20 tests TDD del Sprint 1, ver Anexo).

---

## Todo de implementación

→ [`todo-001-sprint1-hexagonal-refactor.md`](../todo-001-sprint1-hexagonal-refactor.md)

> A generar por el task-manager (`agy`) tras la aprobación de este ADR. **No** es parte del alcance de MiniMax producir el todo; queda aquí como puntero contractual.

---

## Anexo — Lista cerrada de tests TDD del Sprint 1

> Esta lista es **el** criterio de cierre. Si un test no está, el sprint no está cerrado.
> Numeración y trazabilidad a V0 entre paréntesis.
> Tras la iteración 2: tests 9, 10, 11, 12, 17, 19 ajustados a la nueva arquitectura (Sink posee su Formatter; `Filter::enabled` toma `Metadata` por valor; `Metadata::new` existe; `LevelFilter` API especificada; `last_error` poblado en test 16).

### Bloque A — Dominio puro (8 tests)

1. `level_ordena_con_derived_ord` (R3) — usa `Ord` derivado, no cast a `u8`.
2. `level_display_muestra_nombre_mayusculas` (R3, R31) — `Level::Info.to_string() == "INFO"`.
3. `level_as_str_devuelve_nombre_estable` (R3, R6-prep) — helper para JSON futuro.
4. `level_from_str_acepta_minusculas_y_mayusculas` (R31) — `"info"` y `"INFO"` parsean.
5. `level_from_str_rechaza_nombres_invalidos` (R36) — `"verbose"` → `Err(LogError::InvalidLevel)`.
6. `logevent_construye_con_nivel_mensaje_y_metadata` (R7-prep) — value object, `Metadata: Copy`.
7. `metadata_origen_devuelve_file_line_module` (R7-prep) — `Metadata::new("test_module", "test.rs", 42)` (Qwen S3) devuelve el struct correcto; `Metadata::UNKNOWN` tiene los valores placeholder.
8. `logerror_es_representable_y_no_panicea` (R36) — `Display + Debug + Error` sin `unwrap`; `source()` correcto en `Write { .. }` y `Format { .. }`.

### Bloque B — Puertos / traits (4 tests)

9. `formatter_texto_simple_produce_formato_esperado` (R5) — `SimpleTextFormatter::format(&event)` devuelve un `String` con el formato exacto; verificamos byte a byte. (Qwen I3: ya no se pasa `&mut String`).
10. `sink_no_se_invoca_en_fast_path` (R34) — renombrado desde `formatter_con_nivel_desactivado_no_se_invoca_en_fast_path` (Qwen T3) — el `Logger` no llama a ningún `Sink::write` cuando los filtros cortan, lo que implica que tampoco se invoca el `Formatter` del sink.
11. `filter_por_nivel_minimo_bloquea_niveles_inferiores` (R33-prep, R18-prep) — `LevelFilter` (Qwen S2) — trazabilidad corregida (Qwen C2).
12. `filter_personalizado_puede_decidir_por_metadata` (R33) — el test construye un `LogEvent` con `Metadata::new("test_module", "test.rs", 42)` y llama a `Logger::log` **directamente** (no usa los métodos helper `info`/`debug`) (Qwen C3), pasando un closure `|| "msg"`; verifica que el filter rechaza por `Metadata::module`.

### Bloque C — Logger orquestador (4 tests)

13. `logger_default_usa_consola_texto_y_info` (R39) — `Default::default()` es utilizable y, con un `ConsoleSink` que escribe a un `Vec<u8>`, la línea aparece con el formato de `SimpleTextFormatter`.
14. `logger_log_evalua_mensaje_via_closure` (R34) — `Cell<u32>` cuenta invocaciones; en fast-path = 0.
15. `logger_redirige_a_multiples_sinks` (R17) — dos sinks, dos registros, ambos reciben.
16. `logger_propag_error_de_sink_sin_panicar` (R36) — un sink devuelve `Err`, el logger no panic, los demás sinks siguen recibiendo, y **`last_error` queda `Some(_)`** (verificable con un sink mock que espía `last_error`).

### Bloque D — Adaptador de consola (2 tests)

17. `console_sink_escribe_con_formatter_inyectado` (R13) — usa `ConsoleSink::with_writer` con un `Arc<Mutex<Vec<u8>>>` y verifica los bytes escritos (Qwen C1: el sink posee su `Formatter`). El test verifica que cambiar el `Formatter` cambia la salida, demostrando que el `Logger` no la controla.
18. `console_sink_no_panica_si_writer_falla` (R36) — `Write` que devuelve error → `LogError::Write { sink: "ConsoleSink", .. }`.

### Bloque E — Smoke / integración (2 tests)

19. `logger_default_emite_a_writer_capturado` (R39, R5) — se construye un `Logger` con `ConsoleSink::with_writer` y `SimpleTextFormatter`, se llama a `logger.info("hola")`, y se verifica el `Vec<u8>` contiene una línea que comienza con `[INFO] hola` y termina en `\n`. (Qwen M1: el antiguo `console_sink_prints` sin assert queda reemplazado por esta versión con assert real).
20. `smoke_test_default_no_panica_con_multiples_logs` (R36, R19) — test de integración en `tests/smoke.rs`, varios niveles, varios logs, sin panic; verificable en paralelo con `--test-threads=4`.

### Compatibilidad con API del prototipo

| Test del prototipo | Estado en Sprint 1 |
|---|---|
| `levels_are_ordered` (con `as u8`) | **Reemplazado** por test 1. |
| `config_for_dev/staging/prod` | **Mantenido sin cambios** — la factory `from_env` y `Logger::init` se conservan como fachada (Qwen C3). |
| `logger_logs_equal_or_higher_levels` | **Reemplazado** por tests 14 + 15. |
| `console_sink_prints` (sin assert) | **Reemplazado** por test 17 (con assert real) y reforzado por el test 19 (smoke). |
| `build_sinks_creates_console_sink` | **Eliminado** — `build_sinks` desaparece; la fábrica de sinks vive en `LoggerBuilder` y se invoca implícitamente cuando se llama a `Logger::init(LoggerConfig)` o `LoggerBuilder::from_config(LoggerConfig)` (Qwen C3). |

---

## ✅ Qué resuelve este ADR (Sprint 1)

* Arquitectura hexagonal con cuatro capas (`domain`, `ports`, `app`, `adapters`) y separación clara de responsabilidades.
* Tres traits como puertos: `Sink`, `Formatter`, `Filter`, todos `Send + Sync`.
* Logger orquestador puro que no conoce formatos ni I/O, solo dominio y puertos.
* Fast path real (R34): `FnOnce() -> String` solo se evalúa si los filtros lo permiten.
* Errores tipados (R36): `LogError` manual, sin `unwrap`/`panic!` en dominio/puertos/app.
* Concurrencia segura (R19): `Send + Sync` en traits; sincronización delegada a adaptadores.
* Atomicidad de escritura (R20): garantizada por `Mutex` en sinks que escriben a `Write`.
* R39 (Default sensato): `Logger::default().info("hola")` funciona.
* R2 (Core único sin duplicación): lógica en Rust, futuros bindings solo traducirán.
* R1 (Separación en capas): a nivel arquitectónico; workspace físico cuando haya bindings.
* R13 (Sink consola): `ConsoleSink` implementado con `Formatter` y `Write` inyectables.
* R5 (Formato texto): `SimpleTextFormatter`, mismo output que el prototipo.
* R33 (Niveles por módulo): preparado con trait `Filter` y `LevelFilter`.
* R17 (Múltiples sinks): el `Logger` ya acepta `Vec<Arc<dyn Sink>>`.
* API bindings-friendly: sin genéricos, `Default`, value objects `Clone + Send + Sync`.
* Compatibilidad con API antigua: `LoggerConfig`, `Environment`, `SinkConfig` y `Logger::init` se conservan como fachada.
* Cero dependencias nuevas: `thiserror` rechazado, `LogError` manual, `Cargo.toml` intacto.
* 20 tests TDD que definen el criterio de cierre del sprint.

---

## ❌ Qué NO resuelve este ADR (queda fuera del Sprint 1)

* No implementa ninguna feature nueva visible al usuario (sin colores, sin JSON, sin archivos, sin rotación, sin CloudWatch, sin macros, sin captura de metadatos).
* No entrega bindings a JS/TS ni Java (ni siquiera el workspace).
* No hace benchmarks ni optimizaciones de rendimiento más allá del fast-path funcional.
* No migra a `no_std`.
* No cambia el formato de salida respecto al prototipo (sigue siendo `[NIVEL] mensaje`).
* No resuelve el problema de `&'static str` para bindings futuros (queda documentado y diferido a PDR-002).
* No implementa filtros avanzados (solo `LevelFilter` por nivel mínimo).
* No soporta múltiples procesos escribiendo al mismo archivo (R21).

--

## Historial de revisión

> Bloque nuevo, no presente en el template original. Rastreo de iteraciones de revisión mientras el ADR está abierto. Se conserva al cerrar el ADR.

### Iteración 2 — 2026-06-09 (revisión final Qwen)

**Veredicto de Qwen**: "**APROBADO CONDICIONAL**. El ADR está arquitectónicamente correcto y puede implementarse. Las correcciones necesarias son menores y no afectan la arquitectura."

**Cambios incorporados en esta iteración**:

| ID | Tipo | Cambio | Secciones afectadas |
|---|---|---|---|
| **T1** | 🔴 Error técnico | `ConsoleSink.writer` cambia de `Box<dyn Write + Send>` a **`Box<dyn Write + Send + Sync>`** para que el struct satisfaga `Sink: Send + Sync`. | §1.1, §3.1, §4.1 |
| **T2** | 🔴 Error técnico | Añadido el campo `last_error: Arc<Mutex<Option<LogError>>>` al struct `Logger` y el método privado `record_error(&self, err: LogError)` con recuperación defensiva ante `Mutex` envenenado (vía `into_inner()`). | §1.3 (reglas de uso), §1.5, Anexo (test 16) |
| **T3** | 🔴 Error técnico | Test 10 renombrado de `formatter_con_nivel_desactivado_no_se_invoca_en_fast_path` a **`sink_no_se_invoca_en_fast_path`** (más fiel al comportamiento post-C1). | Anexo (test 10) |
| **S1** | 🟡 Gap de especificación | `LoggerBuilder::new()` especificado explícitamente: `LevelFilter::new(LogLevel::Info)` + `ConsoleSink::new(SimpleTextFormatter)`. | §1.6 |
| **S2** | 🟡 Gap de especificación | `LevelFilter` API especificada: `pub struct`, `LevelFilter::new(min: LogLevel)`, `impl Filter { fn enabled(..., level) { level >= self.min } }`. | §1.1, §1.6 |
| **S3** | 🟡 Gap de especificación | `Metadata::new(module: &'static str, file: &'static str, line: u32) -> Self` añadido. Test 7 actualizado para usar `Metadata::new`. | §1.2, Anexo (test 7, test 12) |
| **S4** | 🟡 Gap de especificación | `Filter::enabled` cambia de `&Metadata` a **`Metadata` por valor** (consistencia con `Copy`). `LevelFilter` y el `Logger::log` actualizados. | §1.3, §1.5, §1.6, Anexo (tests 11, 12) |
| **C1** | 🟢 Coherencia | Añadida justificación explícita en *Motivo* §1 de por qué R10 (P0 en V0) se pospone a Sprint 2. | Motivo §1, Consecuencias |
| **C2** | 🟢 Coherencia | Test 11 trazabilidad corregida: `(R33-prep, R18-prep)` en lugar de sólo `(R33-prep)`. | Anexo (test 11) |
| **C3** | 🟢 Coherencia | Test 12 especifica que se llama a `Logger::log` directamente con `Metadata::new(...)`, **no** a los métodos helper. | Anexo (test 12) |
| **C4** | 🟢 Coherencia | Añadido párrafo en §3.1 explicando que el `Mutex` en `ConsoleSink` es redundante para `Stdout` pero necesario para `Write` arbitrarios, con plan de optimización en sprint de rendimiento. | §3.1, Consecuencias |
| **C5** | 🟢 Coherencia | Documentada limitación de `last_error: Option<LogError>` (sólo último error, no historial) en §1.3, §1.5 y Consecuencias. | §1.3, §1.5, Consecuencias |
| **C6** | 🟢 Coherencia | Aclarado en §4.2 que las dependencias **comentadas** del `Cargo.toml` del prototipo se mantienen comentadas y no se descomentan en Sprint 1. | §4.2 |
| **C7** | 🟢 Coherencia | Criterio de cierre: PDR-001 se **crea** (no "abre") al cierre del sprint, con estructura inicial. | Criterio |

**Resultado de la iteración**: el ADR pasa a estado `en implementación`. Qwen dará el **APROBADO** definitivo tras verificar la primera tanda de tests en verde.

**Preservación**: la v0 vive en [`adr-001-sprint1-hexagonal-refactor.v0.md`](adr-001-sprint1-hexagonal-refactor.v0.md) y la v1 en [`adr-001-sprint1-hexagonal-refactor.v1.md`](adr-001-sprint1-hexagonal-refactor.v1.md). Esta versión (v2) es la canónica y la que se cierra al final del sprint.

### Iteración 1 — 2026-06-09 (revisión Qwen)

**Veredicto de Qwen**: "Aprobable con los 3 puntos críticos resueltos. Los 4 importantes son ajustes finos. Los 2 menores son cosméticos."

**Cambios incorporados en la iteración 1** (resumen; ver [`adr-001-sprint1-hexagonal-refactor.v1.md`](adr-001-sprint1-hexagonal-refactor.v1.md) para el detalle completo):

| ID | Severidad | Resumen |
|---|---|---|
| **C1** | 🔴 | `Sink::write` rediseñado; `Formatter` propiedad del `Sink`. |
| **C2** | 🔴 | Sprint 1 = métodos helper, no macros; R7 a Sprint 2. |
| **C3** | 🔴 | `LoggerConfig`/`Environment` preservados como fachadas (Opción A). |
| **I1** | 🟡 | `thiserror` descartado; `LogError` manual. |
| **I2** | 🟡 | `Metadata: Copy`. |
| **I3** | 🟡 | `Formatter::format` → `Result<String, _>`. |
| **I4** | 🟡 | Nota de tradeoff `String` vs `Cow`. |
| **M1** | 🟢 | Reemplazo de `console_sink_prints` documentado. |
| **M2** | 🟢 | Nota sobre `&'static str` para bindings. |

---

## Historial de superación

**Estado:** `en implementación`
**Superado por:** N/A
**Fecha:** N/A
**Motivo:** N/A
