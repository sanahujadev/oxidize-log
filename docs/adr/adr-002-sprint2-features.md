# ADR-002 — Sprint 2: Captura de metadatos, colores ANSI y formato JSON

> **Architecture Decision Record.** Documento inmutable una vez cerrado.
> Creado por el agente `architect` (Mavis / MiniMax). Nunca se edita — si la decisión cambia, se abre un nuevo ADR
> que referencia y supera a este.
>
> **Estado:** `en revisión` (iteración 0 — propuesta inicial)
> **Fecha de decisión:** 2026-07-11
> **Revisor esperado:** Qwen (Arquitecto Segundo)
> **Sprint asociado:** Sprint 2 — *Features visibles para el usuario (sin cambios arquitectónicos)*

## Documentos relacionados

- [`adr-001-sprint1-hexagonal-refactor.md`](adr-001-sprint1-hexagonal-refactor.md) — `implementado` (Sprint 1, base hexagonal). Este ADR lo **presupone** y no lo modifica.
- [`pdr-001-workspace-split.md`](pdr-001-workspace-split.md) — `en revisión` con propuesta provisional Opción B (split cuando llegue el primer binding). No afectado por este ADR.

## Contexto

Sprint 1 (ADR-001) cerró con la base hexagonal en su sitio: 4 capas (`domain` / `ports` / `app` / `adapters`), 3 traits como puertos (`Sink`, `Formatter`, `Filter`), `Logger` orquestador puro, `LogError` manual, fast path por closure, defaults sensatos, 20 tests TDD + 1 smoke en verde, clippy limpio. **El comportamiento observable del usuario no cambió** entre el prototipo y el refactor: ambos producen la misma línea `[LEVEL] mensaje` por stdout.

Sprint 1 dejó explícitamente fuera las features visibles al usuario que el V0 clasifica como P0/P1 y que no afectaban a la base arquitectónica:

- **R7 (P0)** — Captura de `file!()` / `line!()` / `module_path!()` desde el call site.
- **R10 (P0)** — Colores ANSI en consola según nivel.
- **R6 (P1)** — Formato JSON estructurado para pipelines (ELK, CloudWatch, etc.).

Este ADR cubre esos tres requisitos, y **sólo esos tres**. Cualquier otra feature (R4 structured fields, R8 function name, R9 disable metadata, R11 TTY auto-detect, R12 color themes, R14 file sink, R22 buffering, R23–R30 bindings) es objeto de futuros ADRs/PDRs y queda explícitamente fuera del alcance de Sprint 2.

### Estado de los requisitos en el código actual (post-Sprint 1)

| Req | Estado actual | Lo que falta |
|---|---|---|
| **R7** | `Metadata::UNKNOWN` se usa en los métodos helper (`info`, `debug`, …) | Macros que capturen `file!()` / `line!()` / `module_path!()` automáticamente |
| **R10** | Sin soporte de colores. `SimpleTextFormatter` produce texto plano | Un adaptador de formateo que aplique códigos ANSI al prefijo de nivel |
| **R6** | Sin formato JSON | Un `JsonFormatter` que produzca JSON estructurado |

ADR-001 ya dejó los hooks necesarios: el trait `Formatter`, la inyección de formateadores en cada `Sink`, y la separación entre `Logger` y el formateo. Sprint 2 implementa los formateadores y las macros que aprovechan esos hooks, **sin tocar el core**.

## Decisión

Adoptamos las siguientes tres decisiones arquitectónicas para Sprint 2:

1. **R7 — Macros con captura de metadatos.** Las macros `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!` se introducen como `macro_rules!` en un nuevo módulo `src/macros.rs`. Cada macro recibe `&$Logger` como primer argumento y se expande a `logger.log(Nivel, Metadata::new(module_path!(), file!(), line!()), || format!(...))`. Los métodos helper (`info`, `debug`, …) **se conservan** para uso en tests, en runtime genérico, y para quienes no quieran usar macros.
2. **R10 — Colores ANSI.** Un nuevo adaptador `ColorFormatter<F: Formatter>` que envuelve a otro `Formatter` y aplica códigos ANSI al token de nivel. Se introduce también un struct `ColorScheme` con códigos ANSI por nivel (default: `trace` gris, `debug` cian, `info` verde, `warn` amarillo, `error` rojo, `fatal` rojo bold). El formateador es **agnóstico al sink**: el `ConsoleSink` no se modifica; el usuario decide si quiere colores eligiendo el `Formatter` apropiado. R11 (auto-desactivación) se aborda **parcialmente**: `ColorFormatter` tiene un flag `enabled: bool` y el builder expone `.colors(bool)` que el usuario controla; la auto-detección TTY queda fuera de Sprint 2 (es trivial añadirla después, ver *Consecuencias*).
3. **R6 — Formato JSON.** Un nuevo adaptador `JsonFormatter` que produce un objeto JSON con los campos `timestamp`, `level`, `message`, `module`, `file`, `line`. Se implementa **sin `serde`** (construcción manual del JSON vía `String::with_capacity` + `format!`), porque los datos son simples y `serde` añadiría tres crates (`serde`, `serde_json`, `chrono`) para serializar 6 strings. El timestamp es ISO 8601 UTC generado manualmente con `SystemTime` (sin crate de tiempo).

### Restricciones autoimpuestas

- Sprint 2 **no modifica** `domain/`, `ports/`, ni la estructura del `Logger`. Sólo añade: (a) macros en un módulo nuevo, (b) formateadores como adaptadores, (c) tests. El core sigue intacto.
- **Una sola dependencia nueva**, justificada: `colored` (R10). Para R6 usamos construcción manual de JSON. Cero `serde`, cero `chrono`.
- **No se introducen features fuera de R7 / R10 / R6.** R4, R8, R9, R11-auto, R12, R14, R22, R23–R30 quedan para ADRs futuros.
- Las macros son el camino recomendado, pero los métodos helper **siguen existiendo** y se usan en tests. Coexistencia, no reemplazo.
- `Metadata::UNKNOWN` se mantiene como valor por defecto en los métodos helper; las macros inyectan metadata real. Esto preserva el contrato de los métodos (que el V0 prometió que serían "sin captura").

## Especificaciones Técnicas

### 1. Diseño del Core y Traits (Rust) — adiciones al Sprint 1

#### 1.1 Estructura de módulos nueva

```
src/
├── lib.rs                       ← añade `pub mod macros;` y re-exports
├── macros.rs                    ← NUEVO: macro_rules! para trace!, debug!, …, fatal!
├── domain/                      ← SIN CAMBIOS
├── ports/                       ← SIN CAMBIOS
├── adapters/
│   ├── mod.rs                   ← añade re-exports
│   ├── console.rs               ← SIN CAMBIOS
│   ├── text_format.rs           ← SIN CAMBIOS
│   ├── color_format.rs          ← NUEVO: ColorFormatter<F: Formatter> + ColorScheme
│   └── json_format.rs           ← NUEVO: JsonFormatter
├── app/                         ← SIN CAMBIOS estructurales
│   ├── mod.rs                   ← añade `colors` al builder (R10)
│   ├── logger.rs                ← SIN CAMBIOS
│   ├── config.rs                ← LoggerBuilder: añade `.colors(bool)`
│   └── level_filter.rs          ← SIN CAMBIOS
└── tests/
    └── smoke.rs                 ← ampliada para incluir R7, R10, R6
```

#### 1.2 Dependencias de terceros — la única del sprint

```toml
# Cargo.toml — diff respecto a Sprint 1

[dependencies]

# Descomentamos (R10):
colored = "2"

# Mantenemos comentadas (no se usan en Sprint 2):
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# chrono = "0.4"
```

**Justificación de `colored` (única dep nueva del sprint):**

| Aspecto | Manual (sin dep) | `colored 2.x` |
|---|---|---|
| Líneas de código | ~40 (constantes + lógica wrap) | ~5 |
| TTY detection | Manual (`std::io::IsTerminal`) | Incluida, con override por env var |
| Windows support | No | Sí (`EnableVirtualTerminalProcessing`) |
| Dependencia añadida | 0 | 1 crate, ~25 KB compilado |
| Tests | Manual | Battle-tested por miles de proyectos |

Se elige `colored 2.x` por tres razones:

1. Es la elección estándar de la comunidad Rust para ANSI desde 2018.
2. Ya estaba en el `Cargo.toml` del prototipo como dependencia comentada — el autor del proyecto la tenía en mente.
3. La detección TTY y soporte Windows vienen gratis. El sprint de rendimiento (R35) se beneficia.

> **Plan B**: si Qwen objeta `colored`, la implementación manual es ~40 líneas y un cambio de 1 línea en `Cargo.toml`. Es trivial revertir.

#### 1.3 R7 — Macros con captura de metadatos

```rust
// src/macros.rs

#[macro_export]
macro_rules! trace {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Trace,
            $crate::Metadata::new(
                module_path!(),   // built-in: se resuelve en el call site
                file!(),          // built-in: idem
                line!(),          // built-in: idem
            ),
            || ::std::format!($($arg)+),
        )
    };
}

// Equivalentes para debug!, info!, warn!, error!, fatal!
```

**Decisiones de diseño de las macros:**

1. **API explícita por logger**: `info!(&logger, "msg")` con el `&Logger` como primer argumento. No hay estado global (Sprint 1 lo prohibió en Iteración 2). Esto es coherente con la decisión de no tener `static LOGGER` y permite que un proceso tenga **múltiples loggers**.
2. **Sintaxis estilo `format!`**: `info!(&logger, "x = {}, y = {}", x, y)`. El mensaje se construye con `format!` dentro de la closure, así que el fast path R34 sigue funcionando: si el filtro corta, la closure nunca se invoca.
3. **`Metadata::new` se usa en lugar de `Metadata::UNKNOWN`**: las macros siempre capturan metadata real del call site. Esto cumple R7.
4. **Built-in macros sin `$crate::`**: `module_path!()`, `file!()`, `line!()` son built-ins que el resolver evalúa en el *call site* del usuario, no en el crate que define la macro. Por eso se llaman sin prefijo `$crate::` (que sí usamos para los tipos como `$crate::LogLevel` y `$crate::Metadata::new`, que sí viven en nuestro crate).
5. **Coexistencia con métodos helper**: `logger.info("msg")` (método, `Metadata::UNKNOWN`) e `info!(&logger, "msg")` (macro, metadata real) coexisten. El usuario elige.

**Uso esperado:**

```rust
use oxidize_log::{Logger, info, warn, error};

let logger = Logger::default();

info!(&logger, "Usuario autenticado: {}", user_id);
warn!(&logger, "Cache miss en {}", key);
error!(&logger, "Conexión fallida: {}", e);
```

#### 1.4 R10 — Colores ANSI

```rust
// src/adapters/color_format.rs (esquema)

use colored::*;

pub struct ColorScheme {
    pub trace: Style,    // default: dimmed (gris)
    pub debug: Style,    // default: cyan
    pub info:  Style,    // default: green
    pub warn:  Style,    // default: yellow
    pub error: Style,    // default: red
    pub fatal: Style,    // default: bold red on white
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            trace: Style::default().dimmed(),
            debug: Style::default().cyan(),
            info:  Style::default().green(),
            warn:  Style::default().yellow(),
            error: Style::default().red(),
            fatal: Style::default().red().bold().on_white(),
        }
    }
}

/// Envuelve a otro `Formatter` y aplica el `ColorScheme` al token de nivel.
pub struct ColorFormatter<F: Formatter> {
    inner: F,
    scheme: ColorScheme,
    enabled: bool,   // R11: permite desactivar colores manualmente
}

impl<F: Formatter> ColorFormatter<F> {
    pub fn new(inner: F) -> Self {
        Self { inner, scheme: ColorScheme::default(), enabled: true }
    }
    pub fn with_scheme(inner: F, scheme: ColorScheme) -> Self {
        Self { inner, scheme, enabled: true }
    }
    pub fn disabled(mut self) -> Self { self.enabled = false; self }
}

impl<F: Formatter> Formatter for ColorFormatter<F> {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let formatted = self.inner.format(event)?;

        if !self.enabled {
            return Ok(formatted);
        }

        // El "inner" produce algo como "[INFO] mensaje".
        // Envolvemos "[LEVEL]" con el código ANSI del scheme.
        let level_token = format!("[{}]", event.level.as_str());
        let colored = match event.level {
            LogLevel::Trace => self.scheme.trace.paint(&level_token).to_string(),
            LogLevel::Debug => self.scheme.debug.paint(&level_token).to_string(),
            LogLevel::Info  => self.scheme.info.paint(&level_token).to_string(),
            LogLevel::Warn  => self.scheme.warn.paint(&level_token).to_string(),
            LogLevel::Error => self.scheme.error.paint(&level_token).to_string(),
            LogLevel::Fatal => self.scheme.fatal.paint(&level_token).to_string(),
        };

        // Reemplazar sólo la primera ocurrencia (la del prefijo de nivel).
        Ok(formatted.replacen(&level_token, &colored, 1))
    }
}
```

**Decisiones de diseño de R10:**

1. **`ColorFormatter` envuelve a otro `Formatter` (decorator pattern)**: `ColorFormatter<SimpleTextFormatter>` envuelve a `SimpleTextFormatter`. Esto preserva la separación de responsabilidades y permite combinar con `JsonFormatter` o cualquier otro futuro.
2. **El scheme se aplica al token `[LEVEL]`, no a todo el mensaje**: el formateador interno controla el formato; el `ColorFormatter` sólo añade color al prefijo. Esto evita acoplamiento con el formato del `inner`.
3. **`enabled: bool` cubre R11 parcialmente**: el usuario puede desactivar colores manualmente (`ColorFormatter::disabled()` o vía builder). La auto-detección TTY queda fuera de Sprint 2 (ver *Consecuencias*); cuando llegue, el builder la activará por defecto.
4. **No se modifica `ConsoleSink`**: el sink sigue siendo agnóstico al formato. El usuario decide si quiere colores eligiendo `ColorFormatter(SimpleTextFormatter)` o sólo `SimpleTextFormatter` como formatter del sink.

**Uso esperado:**

```rust
use oxidize_log::{LoggerBuilder, ConsoleSink, SimpleTextFormatter, ColorFormatter};
use std::sync::Arc;

let formatter = Arc::new(ColorFormatter::new(SimpleTextFormatter));
let sink = Arc::new(ConsoleSink::new(formatter));
let logger = LoggerBuilder::new().sink(sink).build();

logger.info("Hola con color");  // → [INFO] (en verde) Hola con color
```

#### 1.5 R6 — Formato JSON

```rust
// src/adapters/json_format.rs (esquema)

use crate::domain::{LogEvent, LogLevel, LogError};
use crate::ports::Formatter;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self { Self }
}

impl Formatter for JsonFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let timestamp = iso8601_utc_now();

        // Construcción manual: ~6 strings, no alocamos intermediarios.
        // Capacidad inicial: 128 bytes es suficiente para logs típicos.
        let mut out = String::with_capacity(128);
        out.push('{');
        write_kv(&mut out, "timestamp", &timestamp);  out.push(',');
        write_kv(&mut out, "level",     event.level.as_str()); out.push(',');
        write_kv_str(&mut out, "message", &event.message); out.push(',');
        write_kv(&mut out, "module",    event.metadata.module); out.push(',');
        write_kv(&mut out, "file",      event.metadata.file); out.push(',');
        write_kv_u32(&mut out, "line",   event.metadata.line);
        out.push('}');
        Ok(out)
    }
}

fn write_kv(out: &mut String, key: &str, value: &str) {
    out.push('"'); out.push_str(key); out.push_str("\":\"");
    out.push_str(&escape_json(value));
    out.push('"');
}

fn write_kv_str(out: &mut String, key: &str, value: &str) { /* alias de write_kv */ }

fn write_kv_u32(out: &mut String, key: &str, value: u32) {
    out.push('"'); out.push_str(key); out.push_str("\":");
    out.push_str(&value.to_string());
}

fn escape_json(s: &str) -> String {
    // Escape: " → \", \ → \\, \n → \n, \r → \r, \t → \t, \x08 → \b, \x0c → \f
    // Implementación: pre-alocar y copiar con escapes cuando haga falta.
    // ~15 líneas de código.
}

fn iso8601_utc_now() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();  // Aceptado: si el reloj está mal, caemos a 0 sin panic.
    let secs = dur.as_secs();
    let nanos = dur.subsec_nanos();
    // Formato ISO 8601: YYYY-MM-DDTHH:MM:SS.NNNZ
    // Conversión secs → Y-M-D-H-M-S sin dep: ~30 líneas con la fórmula
    // de Gregorian calendar (algoritmo de Howard Hinnant, dominio público).
    format_iso8601(secs, nanos)
}
```

**Decisiones de diseño de R6:**

1. **Construcción manual del JSON, sin `serde`**: los 6 campos son strings y un `u32`. `serde` añade `serde_core` + `serde_derive` (proc-macro) + `serde_json`. Para serializar 6 valores, no compensa. Si en el futuro (R4) hay campos dinámicos, se introduce `serde` con `#[derive(Serialize)]`. Esa decisión queda para Sprint 3.
2. **Sin `chrono` ni `time`**: el formato ISO 8601 UTC se genera manualmente. Sí, son ~30 líneas de conversión `secs → Y-M-D-H-M-S`, pero evita un crate de ~200 KB.
3. **Escape JSON explícito**: `escape_json` cubre los 7 caracteres especiales (`"`, `\`, `\n`, `\r`, `\t`, `\b`, `\f`). Suficiente para mensajes típicos. Unicode se pasa tal cual (UTF-8 válido en JSON).
4. **Sin campos structured (`fields: {}`) en Sprint 2**: el schema es estable y minimal. R4 (P1) añadirá fields en Sprint 3 sin breaking change (sólo añade una clave opcional).
5. **Tolerancia a `SystemTime` antes de `UNIX_EPOCH`**: `unwrap_or_default()` (devuelve duration 0) **no es un `panic`**: es la API estándar de `Duration` que convierte el `Result` en `Duration` con un fallback explícito. Aceptamos este único punto no-puro en `iso8601_utc_now` justificado por la garantía contractual del sistema operativo (los relojes modernos nunca van antes de 1970). Marcado con comentario en el código.

**Schema de salida (Sprint 2):**

```json
{
  "timestamp": "2026-07-10T18:47:00.123Z",
  "level": "INFO",
  "message": "Hola mundo",
  "module": "oxidize_log::examples",
  "file": "examples/test.rs",
  "line": 42
}
```

#### 1.6 Configuración — `LoggerBuilder::colors(bool)`

```rust
// src/app/config.rs — diff respecto a Sprint 1

impl LoggerBuilder {
    // ... lo que ya hay ...

    /// Habilita o deshabilita colores (R10). Por defecto: `true` (en consola).
    /// En Sprint 2 es manual; la auto-detección TTY llega en un sprint futuro.
    pub fn colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }
}
```

**Uso esperado:**

```rust
// Logger::default() aplica el scheme de colores por defecto (enabled = true
// si el sink es ConsoleSink y el stdout es TTY, false en otro caso).
// En Sprint 2: enabled = true por defecto. Sprint 3: TTY detection.

// Usuario puede forzar:
LoggerBuilder::new().colors(false).build();  // sin colores
LoggerBuilder::new().colors(true).build();   // con colores
```

### 2. Bindings y Capa de FFI (Multiplataforma)

> Este sprint **no entrega bindings**. La sección es idéntica a la de ADR-001.
>
> PDR-002 (estrategia de bindings: WASM vs N-API, JNI wrapper o directo) queda fuera de este ADR.

### 3. Rendimiento, Concurrencia y Memoria

- **Cero cambios en la sincronización**: las macros son `macro_rules!` que se expanden en tiempo de compilación. El `Logger::log` subyacente es el mismo que en Sprint 1. R19 (`Send + Sync`) y R36 (sin `panic`) se preservan.
- **Cero alocaciones adicionales en fast path**: las macros usan `Metadata::new(...)` que es `Copy` (Sprint 1 lo dejó así). `Metadata::new(module_path!(), file!(), line!())` se evalúa en el call site de la macro, pero el `String` del mensaje (vía closure) sólo se construye si los filtros pasan. **R34 se preserva**.
- **R10 (colores)**: `colored` añade un `String` extra por línea formateada (el código ANSI). El alocamiento es despreciable comparado con el `write` al `Write` del sink.
- **R6 (JSON)**: una sola `String` por log emitido, capacidad inicial 128 bytes (suficiente para logs típicos; redimensiona si no). No hay alocaciones intermedias.

### 4. Sinks e Infraestructura (Adaptadores) — adiciones

#### 4.1 `ColorFormatter<F: Formatter>` (R10)

Definido en §1.4. Es un **decorator**: envuelve a otro `Formatter` y aplica color. Combinable: `ColorFormatter<SimpleTextFormatter>`, `ColorFormatter<JsonFormatter>` (raro pero válido), etc.

#### 4.2 `JsonFormatter` (R6)

Definido en §1.5. Standalone, no envuelve a nadie.

#### 4.3 Sinks entregados en Sprint 2

- `ConsoleSink` — **sin cambios**. Sigue agnóstico al formatter.
- **No** se entrega `FileSink`, `RotatingFileSink`, ni `CloudWatchSink` en Sprint 2. (R14, R15, R16 → sprints futuros.)

## Motivo

**PDR de origen:** decisión directa sin PDR previo. El sprint 2 cubre features que ya estaban priorizadas en el V0; no abren sub-decisiones estructurales que requieran un PDR (los tradeoffs están contenidos en este ADR).

### 1. Por qué macros con `&Logger` y no estado global

El V0 R7 dice "El logger debe poder incluir `file!()`, `line!()` y `module_path!()` mediante macros, sin requerir backtrace." No menciona estado global. Sprint 1 tomó la decisión explícita de no tener estado global (Iteración 2 C2, "no hay `static LOGGER`"). Sprint 2 mantiene esa decisión: las macros son explícitas sobre qué logger usan. Tres beneficios:

1. **Testabilidad**: un test puede usar un logger con un sink que escribe a un `Vec<u8>` y verificar que la metadata se capturó correctamente.
2. **Multi-logger**: un proceso puede tener varios loggers (uno para audit, otro para debug, otro para metrics), cada uno con su propio nivel, sink y formatter.
3. **Cero inicialización oculta**: no hay `Lazy` o `OnceCell` que pueda tener problemas de orden de inicialización en tests.

El trade-off es que el usuario tiene que pasar `&logger` en cada llamada. La ergonomía es buena: `info!(&logger, "msg")` se lee natural. Si en el futuro se quiere azúcar con un logger por defecto, se introduce `oxidize_log::logger()` o similar, pero no se hace en Sprint 2.

### 2. Por qué `ColorFormatter<F: Formatter>` y no `ColoredConsoleSink`

`ColoredConsoleSink` mezclaría dos concerns: serialización a bytes (sink) y aplicación de colores (formato). El principio hexagonal dice: cada adapter hace una cosa. Si el día de mañana queremos escribir logs JSON con colores (raro pero posible: `ColorFormatter<JsonFormatter>`), necesitamos que el formatter sea el responsable del color, no el sink.

Además, el trait `Formatter` ya existe y ya tiene la firma `format(&self, event) -> Result<String, LogError>`. Reusarlo es **cero código nuevo en el core**.

### 3. Por qué `colored` y no implementación manual

Ver tabla en §1.2. El cálculo es:

- `colored`: 1 dep, ~5 líneas de código de uso, TTY/Windows gratis.
- Manual: 0 deps, ~40 líneas, TTY/Windows manual.

Sprint 1 justificó la ausencia de `thiserror` por "no dep, ~30 líneas, mismo efecto". Aquí la diferencia es: `thiserror` no aporta funcionalidad más allá de derivar traits que podemos derivar a mano; `colored` **sí aporta TTY detection, soporte Windows, y battle-testing** que tendríamos que reimplementar. La asimetría justifica la dependencia.

### 4. Por qué JSON manual y no `serde` + `serde_json` + `chrono`

Tres crates para serializar 6 strings. `serde` añade un proc-macro (tiempo de compilación). `serde_json` añade ~150 KB compilado. `chrono` añade ~200 KB. Total: **~350 KB, ~1 s extra de compilación incremental**, para serializar 6 strings que caben en una mano.

La construcción manual con `String::with_capacity(128)` + `push_str` + `escape_json` son ~50 líneas de código. Es la decisión correcta para Sprint 2. Si R4 (campos dinámicos) entra en Sprint 3, se reabre la conversación: en ese momento `serde` compensa, y se introduce con `#[derive(Serialize)]` en una struct `LogRecord`.

Para el timestamp, la fórmula `secs → Y-M-D-H-M-S` son ~30 líneas usando el **algoritmo de Howard Hinnant** (date algorithms, dominio público). Es un patrón conocido y copiable. El código va comentado con la referencia.

### 5. Por qué no incluir R4, R8, R9, R11-auto, R12, R14, R22 en Sprint 2

Regla del Sprint: **tres features, no diez**. Si Sprint 2 intenta cubrir todo lo que el V0 marca como P0/P1, se convierte en un sprint de varios meses y el código pierde coherencia. Cada feature tiene su propio ADR o PDR. El próximo sprint cubre las que estén maduras.

**Excepciones que sí se abordan parcialmente:**

- **R11 (auto-desactivación)**: el flag `enabled: bool` de `ColorFormatter` cubre la parte manual de R11. La auto-detección TTY queda para Sprint 3 (ver *Consecuencias*). Es un `if !is_terminal() { self.enabled = false; }` en el builder, trivial.

**Features que se difieren explícitamente (abren su propio ADR/PDR):**

| Req | Por qué no en Sprint 2 | Próximo sprint candidato |
|---|---|---|
| R4 (campos clave/valor) | Requiere `serde` o cambio de schema JSON | Sprint 3 |
| R8 (nombre de función) | Requiere crate `function_name` o RTTI; trade-off bin size | Sprint 3 |
| R9 (desactivar metadatos) | Cambio de API; no crítico | Sprint 3 |
| R11-auto (TTY detection) | Trivial pero requiere `is_terminal` estable y tests manuales | Sprint 3 |
| R12 (color themes) | Tras R10 y R11-auto; secundario | Sprint 3 |
| R14 (sink archivo) | Otro adaptador + tests; sustantivo pero autocontenido | Sprint 3 |
| R15 (rotación) | Depende de R14 | Sprint 4 |
| R16 (CloudWatch) | AWS SDK, async, complejo | Sprint 5 (P2) |
| R22 (buffering/async) | Cambia la forma del `Logger::log`; ADR dedicado | Sprint 4 |
| R23–R30 (bindings) | Workspace split primero (PDR-001) | Sprint 6+ |

## Consecuencias

### Positivas

- **R7 (P0) cumplido**: las macros capturan `file!`/`line!`/`module_path!` automáticamente. El usuario tiene una API ergonómica: `info!(&logger, "msg")`.
- **R10 (P0) cumplido**: colores ANSI en consola. `ColorFormatter<SimpleTextFormatter>` es combinable con cualquier `Formatter` futuro.
- **R6 (P1) cumplido**: `JsonFormatter` produce JSON estructurado listo para pipelines. Sin deps pesadas.
- **Cero cambios en el core** (`domain/`, `ports/`, `app/Logger`). El refactor de Sprint 1 sigue intacto; Sprint 2 es estrictamente aditivo.
- **Cero alocaciones extra en fast path** (R34 preservado).
- **Una sola dependencia nueva** (`colored`). 25 KB compilado. TTY/Windows gratis.
- **Coexistencia macros + métodos helper**: tests pueden seguir usando `logger.info("msg")` con `Metadata::UNKNOWN`. Producción usa macros con metadata real.
- **Decorator pattern para R10**: `ColorFormatter<F>` es componible. El usuario puede combinar con cualquier `Formatter` futuro.

### Negativas o restricciones introducidas

- `colored` es la única dep nueva del sprint (25 KB). Si en un futuro se quiere 0 deps, hay que reemplazar `colored` con implementación manual (~40 líneas). El cambio es trivial.
- **JSON sin `serde`**: la lógica de escape y formato es manual. ~50 líneas de código que un test de fuzzing debería cubrir. Si en Sprint 3 se añade R4, esto se reabre y se introduce `serde`.
- **R11-auto (TTY detection) queda fuera**: el usuario tiene que llamar `.colors(false)` manualmente si redirige a un pipe. Trade-off explícito; Sprint 3 lo cubre.
- **R4, R8, R9, R12, R14, R15, R16, R22, R23–R30 quedan fuera**. Algunos son P0 (R14, R15) y se abordarán en sprints dedicados. Esto no contradice el V0: el V0 prioriza, no impone plazos.
- **Las macros usan `format!` internamente**: el mensaje siempre se construye como `String`. Si se quiere zero-alloc, hay que cambiar la firma a `&Arguments` o usar macros más elaboradas (como `log!` o `tracing` hacen). Sprint de rendimiento futuro.
- **El `JsonFormatter` no tiene tests de fuzzing**: los tests cubren los casos del Anexo, pero un campo con caracteres Unicode extremos o inputs adversarialmente largos no está cubierto. Recomendación: añadir `cargo-fuzz` o un test de property-based en un sprint posterior.
- **Único `unwrap_or_default()` en producción** (en `iso8601_utc_now`): justificado por garantía contractual del SO; marcado con comentario. No es `unwrap` ni `expect`; es la API estándar de `Duration`. Aceptado.

### Restricciones autoimpuestas (reiteración)

- No se modifica `domain/`, `ports/`, ni el struct `Logger`.
- No se introducen features fuera de R7 / R10 / R6.
- Las macros coexisten con los métodos helper, no los reemplazan.
- Cero `panic!` en producción; cero `unwrap`/`expect` en código de producción (mismo criterio que Sprint 1, con la única excepción documentada de `unwrap_or_default()` en `iso8601_utc_now`).

## Módulos afectados

| Módulo / Archivo | Tipo de cambio |
|---|---|
| `src/lib.rs` | **Modificado** — añade `pub mod macros;` y `pub use adapters::{ColorFormatter, ColorScheme, JsonFormatter};` |
| `src/macros.rs` | **Nuevo** — `macro_rules!` para `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!` |
| `src/domain/` | **Sin cambios** |
| `src/ports/` | **Sin cambios** |
| `src/adapters/console.rs` | **Sin cambios** |
| `src/adapters/text_format.rs` | **Sin cambios** |
| `src/adapters/color_format.rs` | **Nuevo** — `ColorFormatter<F: Formatter>` + `ColorScheme` (R10) |
| `src/adapters/json_format.rs` | **Nuevo** — `JsonFormatter` (R6) |
| `src/app/logger.rs` | **Sin cambios** |
| `src/app/config.rs` | **Modificado** — `LoggerBuilder::colors(bool)` |
| `src/app/level_filter.rs` | **Sin cambios** |
| `tests/smoke.rs` | **Modificado** — smoke tests para R7, R10, R6 |
| `examples/test.rs` | **Modificado** — usa macros (R7) en lugar de métodos helper |
| `Cargo.toml` | **Modificado** — descomenta `colored = "2"` (única dep nueva del sprint) |
| `docs/adr/adr-001-sprint1-hexagonal-refactor.md` | **Sin cambios** (Sprint 1 cerrado) |
| `docs/adr/pdr-001-workspace-split.md` | **Sin cambios** (decisión independiente, Opción B) |

## Criterio de implementación completa

Verificable. Cada item es una orden que se cierra o queda abierta con justificación.

- [ ] `cargo build` sin warnings con `#![deny(warnings)]` en `src/lib.rs`
- [ ] `cargo test` con los **tests TDD del Sprint 2 en verde** (lista cerrada, ver Anexo)
- [ ] `cargo clippy --all-targets -- -D warnings` pasa limpio
- [ ] `grep -rE "\b(unwrap|expect|panic!)\b" src/ --include="*.rs" | grep -vE "mod tests|#\[cfg\(test\)\]"` devuelve vacío (excepto el `unwrap_or_default()` justificado de `iso8601_utc_now`, marcado con comentario)
- [ ] `info!(&logger, "msg")` produce una línea con `module`, `file`, `line` reales (verificable con un test que capture el log y parsee el JSON del `JsonFormatter` o inspeccione el `LogEvent` de un mock)
- [ ] `ColorFormatter::disabled()` produce output sin códigos ANSI (verificable byte a byte)
- [ ] `JsonFormatter` produce JSON válido parseable por `serde_json::from_str` (test de compatibilidad — **única dependencia temporal admitida sólo en tests `[dev-dependencies]`**)
- [ ] `Cargo.toml` tiene exactamente **una dependencia nueva** en `[dependencies]`: `colored = "2"`. `serde`, `serde_json`, `chrono` siguen comentadas. `serde_json` puede aparecer en `[dev-dependencies]` para el test de parseo.
- [ ] `examples/test.rs` actualizado para usar las macros, output visible vía `cargo run --example test`
- [ ] `tests/smoke.rs` ampliado con smoke tests de R7, R10, R6
- [ ] No se introducen features fuera de R7 / R10 / R6. Cualquier tentación se delega al siguiente sprint.

**Tests relevantes en verde**: los tests TDD del Sprint 2 (ver Anexo).

## Todo de implementación por fases (mini sprints)

> Adaptación del template al contexto de un logger en Rust. Las cinco fases canónicas (domain / TDD / app / adapters / controller) se traducen a: *adaptadores → TDD → capa de macros → wiring del builder → tests de integración*. Cada fase termina con tests en verde y un commit con mensaje conventional (`feat(sprint2): ...`).

### Fase 1 — Adaptadores puros (R10, R6)

**Objetivo**: `ColorFormatter<F>`, `ColorScheme` y `JsonFormatter` implementados, con sus propios unit tests, sin tocar nada más.

- [ ] `src/adapters/color_format.rs` con `ColorScheme` (default), `ColorFormatter::new`, `::with_scheme`, `::disabled`, e `impl<F: Formatter> Formatter for ColorFormatter<F>`.
- [ ] `src/adapters/json_format.rs` con `JsonFormatter`, helpers `write_kv`/`write_kv_u32`, `escape_json` y `iso8601_utc_now`.
- [ ] `src/adapters/mod.rs` re-exporta los nuevos tipos.
- [ ] Unit tests en cada archivo (Bloque B y Bloque C del Anexo, sin smoke).

### Fase 2 — TDD bloque por bloque

**Objetivo**: Cada test del Anexo escrito antes (o en paralelo) a la implementación. Cobertura: 12 tests unit + 2 smoke = 14.

- [ ] Bloque A (R7 macros) — 4 tests, 1 por macro comportamiento (captura, fast-path, multi-args, coexistencia).
- [ ] Bloque B (R10 colores) — 4 tests, 1 por comportamiento (ANSI codes, scheme default, disabled, integration con sink).
- [ ] Bloque C (R6 JSON) — 4 tests, 1 por invariante (válido, schema completo, regex timestamp, escape).
- [ ] Bloque D (smoke) — 2 tests en `tests/smoke.rs`.

### Fase 3 — Capa de aplicación (extensión, no ruptura)

**Objetivo**: `LoggerBuilder::colors(bool)` y `src/macros.rs` conectados al core, sin modificar el `Logger`.

- [ ] `src/app/config.rs`: campo `colors_enabled: bool` en `LoggerBuilder`; método `pub fn colors(mut self, enabled: bool) -> Self`; default `true` (auto-detección queda para Sprint 3).
- [ ] `src/macros.rs`: las 6 macros con la sintaxis exacta de §1.3.
- [ ] `src/lib.rs`: `pub mod macros;` y re-exports actualizados.

### Fase 4 — Adaptadores de frontera (wiring del builder)

**Objetivo**: `Logger::default()` puede opcionalmente aplicar `ColorFormatter<SimpleTextFormatter>` por encima del `ConsoleSink` cuando el builder tiene `colors_enabled = true`. **Decisión a confirmar en iteración 1**: si el `Default` cambia, hay que actualizar el test 19 de Sprint 1 (que verifica el output sin colores vía `Vec<u8>`).

- [ ] Si el wiring cambia el default: actualizar el test 19 para que use `LoggerBuilder::new().colors(false).sink(...)` explícitamente, manteniendo la verificación byte a byte.
- [ ] Si el wiring no cambia el default: documentar y seguir.
- [ ] `examples/test.rs` actualizado a macros (`info!(&logger, "...")`), output visible con `cargo run --example test`.

### Fase 5 — Integración, smoke, criterios de cierre

**Objetivo**: Sprint cerrado. 14/14 tests en verde. Clippy limpio. Cero `unwrap`/`expect`/`panic!` en prod (excepto el justificado).

- [ ] `cargo test --all` verde.
- [ ] `cargo clippy --all-targets -- -D warnings` limpio.
- [ ] `grep` del criterio de `unwrap`/`expect`/`panic!` ejecutado y verificado.
- [ ] `Cargo.toml` revisado: `colored` es la única `[dependencies]` nueva. `serde_json` puede aparecer en `[dev-dependencies]`.
- [ ] Cierre: estado del ADR pasa a `implementado`, commit con tag `sprint-2-closed`.

## Anexo — Lista cerrada de tests TDD del Sprint 2

> Esta lista es **el** criterio de cierre. Si un test no está, el sprint no está cerrado.
> Numeración y trazabilidad a V0 entre paréntesis.

### Bloque A — R7: Macros con captura de metadatos (4 tests)

1. `trace_macro_captures_file_line_module` (R7) — `trace!(&logger, "msg")` produce un `LogEvent` con `metadata.module` igual a `module_path!()`, `metadata.file` igual a `file!()`, `metadata.line` igual a `line!()`. Verificable con un `MockSink` que capture el evento.
2. `info_macro_skips_evaluation_when_filtered` (R7, R34) — con `LevelFilter::Warn`, `info!(&logger, "x = {}", expensive())` **no llama a `expensive()`**. El fast path se preserva a través de la macro.
3. `error_macro_does_not_panic_on_complex_args` (R7, R36) — `error!(&logger, "x = {}, y = {}", x, y)` funciona con múltiples argumentos, similar a `format!`.
4. `macros_coexist_with_helper_methods` (R7) — `logger.info("msg")` (helper) usa `Metadata::UNKNOWN`; `info!(&logger, "msg")` (macro) usa metadata real. Ambos coexisten en el mismo proceso sin colisión.

### Bloque B — R10: Colores ANSI (4 tests)

1. `color_formatter_wraps_level_with_ansi_codes` (R10) — `ColorFormatter::new(SimpleTextFormatter).format(&event_info)` produce un `String` que contiene `\x1b[32m` (green), `[INFO]`, y `\x1b[0m` (reset).
2. `color_scheme_default_maps_levels_to_colors` (R10) — `ColorScheme::default()` aplica los códigos correctos: `trace` gris, `debug` cian, `info` verde, `warn` amarillo, `error` rojo, `fatal` rojo bold.
3. `color_formatter_disabled_produces_no_ansi` (R10, R11) — `ColorFormatter::new(SimpleTextFormatter).disabled().format(&event)` produce el mismo output que `SimpleTextFormatter` (sin códigos ANSI).
4. `color_formatter_with_console_sink_writes_to_terminal` (R10, R13) — integración: un `Logger` con `ConsoleSink(ColorFormatter(SimpleTextFormatter))` redirigido a un `Vec<u8>` produce bytes que contienen códigos ANSI.

### Bloque C — R6: Formato JSON (4 tests)

1. `json_formatter_produces_valid_json_object` (R6) — `JsonFormatter::format(&event)` produce un `String` parseable por `serde_json::from_str::<serde_json::Value>`. Test de compatibilidad.
2. `json_formatter_includes_all_required_fields` (R6) — el JSON contiene exactamente: `timestamp`, `level`, `message`, `module`, `file`, `line`. Verificable con `serde_json::Value` o parse manual.
3. `json_formatter_iso8601_utc_timestamp_format` (R6) — `timestamp` matchea la regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$`.
4. `json_formatter_escapes_special_characters` (R6) — un mensaje con `"`, `\`, `\n`, `\t` produce JSON que sigue siendo parseable y los caracteres especiales están escapados.

### Bloque D — Smoke / integración (2 tests)

1. `smoke_macros_capture_real_metadata` (R7) — test de integración en `tests/smoke.rs`: usa `info!(&logger, "msg")` desde un módulo específico y verifica que el `module` capturado es ese módulo.
2. `smoke_color_and_json_combined` (R10, R6) — test de integración: un logger con `ConsoleSink(JsonFormatter)` redirigido a `Vec<u8>` produce JSON parseable con los 6 campos correctos.

### Compatibilidad con Sprint 1

| Test del Sprint 1 | Estado en Sprint 2 |
|---|---|
| Los 20 unit tests + 1 smoke test | **Siguen pasando sin cambios** (Sprint 2 es aditivo) |
| `cargo clippy` limpio | **Sigue limpio** (sin nuevos warnings) |
| `Cargo.toml` sin deps nuevas (Sprint 1) | **Añade 1 dep** (`colored = "2"`) — única del sprint |
| Test 19 (smoke sin colores) | **A confirmar en Fase 4**: si el `Default` ahora aplica `ColorFormatter`, el test debe usar `.colors(false)` explícitamente |

## Historial de revisión

> Bloque presente desde ADR-001. Se rellena en cada revisión de Qwen.

### Iteración 0 — 2026-07-11 (propuesta inicial)

**Veredicto esperado de Qwen**: pendiente. Revisión entrante.

**Puntos abiertos que se espera que Qwen revise:**

1. **Dependencia `colored`**: única dep nueva del sprint. ¿OK o implementar manual?
2. **JSON sin `serde`**: construcción manual de ~50 líneas. ¿OK o usar `serde_json`?
3. **Macros con `&Logger` (no estado global)**: decisión heredada de Sprint 1, ¿se mantiene?
4. **R11-auto (TTY detection) fuera de Sprint 2**: ¿OK como está, o entra como objetivo de Sprint 2?
5. **Features fuera de scope (R4, R8, R9, R12, R14, etc.)**: ¿el recorte es el correcto, o falta alguna?
6. **Schema JSON**: el orden de campos y nombres (`timestamp`, `level`, `message`, `module`, `file`, `line`). ¿Estándar de la industria o debemos ajustarlo?
7. **Wiring del `Default` (Fase 4)**: si cambiamos el default para que `Logger::default()` aplique `ColorFormatter`, ¿se prefiere mantener el default sin colores por compatibilidad con el test 19 de Sprint 1, o se acepta actualizar el test 19?
8. **`unwrap_or_default()` en `iso8601_utc_now`**: la única " impureza" del criterio "cero `unwrap`/`expect`". ¿Se acepta, o se sustituye por match explícito?

**Resultado**: pendiente de revisión.

## Historial de superación

**Estado:** en revisión
**Superado por:** N/A
**Fecha:** N/A
**Motivo:** N/A

## Resumen de features resueltos

- **R7 (P0)** — Las macros `trace!` / `debug!` / `info!` / `warn!` / `error!` / `fatal!` capturan `file!`/`line!`/`module_path!` del call site. API ergonómica con `format!`-style args. El fast path de Sprint 1 (R34) se preserva a través de las macros.
- **R10 (P0)** — `ColorFormatter<F: Formatter>` aplica códigos ANSI al prefijo `[LEVEL]` mediante un `ColorScheme` configurable. Decorator pattern que no obliga a tocar el `ConsoleSink`. `enabled: bool` cubre la parte manual de R11.
- **R6 (P1)** — `JsonFormatter` produce JSON estructurado con 6 campos (`timestamp`, `level`, `message`, `module`, `file`, `line`) y timestamp ISO 8601 UTC. Validado con `serde_json` parse en test de compatibilidad.
- **Configuración**: `LoggerBuilder::colors(bool)` para activar/desactivar colores globalmente.

## Resumen de cosas que se deciden NO RESOLVER en este ADR

- **R4** — campos clave/valor en el log (`fields: {}`). Requiere `serde` o cambio de schema JSON. **Sprint 3**.
- **R8** — nombre de función (`function_name` crate o RTTI). Trade-off bin size vs info. **Sprint 3**.
- **R9** — desactivar metadatos completos. Cambio de API; no crítico. **Sprint 3**.
- **R11-auto** — auto-detección TTY. Trivial con `std::io::IsTerminal`; necesita tests manuales. **Sprint 3**.
- **R12** — color themes (`minimal`, `high contrast`, etc.). Tras R10 y R11-auto. **Sprint 3**.
- **R14** — `FileSink` simple (P0). Adaptador autocontenido. **Sprint 3**.
- **R15** — `RotatingFileSink` (P1). Depende de R14. **Sprint 4**.
- **R16** — `CloudWatchSink` (P2). AWS SDK + async. **Sprint 5**.
- **R17** — múltiples sinks simultáneos vía builder API pública (parcialmente listo desde Sprint 1; falta pulido). **Sprint 3**.
- **R18** — filtros por sink. Depende de R17. **Sprint 3**.
- **R21** — multi-proceso escribiendo al mismo archivo. P2. **Sprint 5+**.
- **R22** — buffering/async. Cambia la firma de `Logger::log`. ADR dedicado. **Sprint 4**.
- **R23–R30** — bindings JS/Java. Bloqueados por PDR-001 (workspace split). **Sprint 6+**.
- **Sprint de rendimiento (R34+R35)** — zero-alloc con `core::fmt::Arguments`, benchmarks. ADR dedicado.
- **`no_std`** — `LogError` manual ya lo permite; falta feature flag y polyfill. ADR dedicado.
