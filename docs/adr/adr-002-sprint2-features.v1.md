# ADR-002 — Sprint 2: Captura de metadatos, colores ANSI y formato JSON

> **Architecture Decision Record.** Documento inmutable una vez cerrado.
> Creado por el agente `architect` (Mavis / MiniMax). Nunca se edita — si la decisión cambia, se abre un nuevo ADR
> que referencia y supera a este.
>
> **Estado:** `en revisión` (iteración 1 — incorporando revisión de Qwen)
> **Fecha de decisión:** 2026-07-11
> **Revisor:** Qwen (Arquitecto Segundo) — `Aprobado con condiciones` (pendiente resolver C1, C2, C3)
> **Sprint asociado:** Sprint 2 — *Features visibles para el usuario (sin cambios arquitectónicos)*

## Documentos relacionados

- [`adr-001-sprint1-hexagonal-refactor.md`](adr-001-sprint1-hexagonal-refactor.md) — `implementado` (Sprint 1, base hexagonal). Este ADR lo **presupone** y no lo modifica.
- [`adr-002-sprint2-features.v0.md`](adr-002-sprint2-features.v0.md) — iteración 0 (propuesta inicial, superada por esta versión).
- [`pdr-001-workspace-split.md`](pdr-001-workspace-split.md) — `en revisión`. No afectado por este ADR.

## Historial de revisión

> Bloque presente desde ADR-001. Cada iteración se conserva y se referencia desde aquí.

### Iteración 1 — 2026-07-11 (esta versión)

**Disparador**: revisión de Qwen sobre iteración 0 + feedback del Arquitecto Principal sobre el decorator `ColorFormatter<F>`.

**Cambios incorporados respecto a v0:**

| ID | Tipo | Cambio | Sección |
|---|---|---|---|
| **Arq** | 🔴 Corrección arquitectónica | **Eliminado** `ColorFormatter<F>` decorator. Colores internalizados en `SimpleTextFormatter` con `Option<ColorScheme>`. La leaky abstraction del `replacen` sobre el string opaco se resuelve de raíz. | §1.4, §1.5, Motivo §6 |
| **C1** | 🔴 Crítico | Especificada la integración `LoggerBuilder::colors(bool)` ↔ `SimpleTextFormatter`: aplica sólo al sink por defecto construido en `.build()`. Si el usuario inyectó un sink custom, `.colors()` es no-op documentado. | §1.6 |
| **C2** | 🔴 Crítico | `escape_json` cubre **todos** los caracteres de control U+0000–U+001F (RFC 8259), no sólo los 7 nombrados. Formato `\u{:04x}` para los no nombrados. | §1.5 |
| **C3** | 🔴 Crítico | `iso8601_utc_now` ahora devuelve `Result<String, LogError>`. `JsonFormatter::format` propaga el error con `?`. Sin `unwrap_or_default`. | §1.5 |
| **I1** | 🟡 Importante | `ColorScheme` y `AnsiStyle` derivan `Debug, Clone, Copy, PartialEq, Eq`. | §1.4 |
| **I2** | 🟡 Importante | Tests de JSON especifican el uso de `serde_json::from_str` desde `[dev-dependencies]`. Listado de qué tests lo usan. | Anexo Bloque C |
| **I3** | 🟡 Importante | `JsonFormatter` implementa `Default` además de `new()`. | §1.5 |
| **I4** | 🟡 Importante | Orden de campos JSON documentado como estable: `timestamp, level, message, module, file, line`. Razón: legibilidad humana + consistencia con `tracing-subscriber`. | §1.5 |
| **I5** | 🟡 Importante | Macros: documentado que `info!(&logger, "lit")` aloca un `String` aunque el literal sea estático. El "fast path" sólo evita la alocación cuando el nivel está desactivado. | §1.3 |
| **I6** | 🟡 Importante | Limitación Windows legacy (pre-TH2 sin soporte ANSI) documentada como trade-off aceptable. Windows 10+ y terminales modernos funcionan. | §1.4 |
| **I7** | 🟡 Importante | Constructores de `SimpleTextFormatter` especificados explícitamente: `new()` (sin colores), `with_colors(scheme)`, `Default = new()`. | §1.4 |
| **M1** | 🟢 Menor | Convención minúsculas para macros (`info!`, `trace!`) documentada (vs. `INFO`, `TRACE` mayúsculas para los niveles). | §1.3 |
| **M2** | 🟢 Menor | Falta de fuzzing para `escape_json` documentada como TODO para sprint futuro (proptest / property-based). | Consecuencias |
| **M3** | 🟢 Menor | `SystemTime` no es monótono (afectado por NTP / cambios de reloj). Documentado como limitación general de loggers. | §1.5 |
| **M4** | 🟢 Menor | Por qué se rechazó el decorator pattern documentado explícitamente en *Motivo* §6. | Motivo |
| **M5** | 🟢 Menor | Representación de estilos: `enum AnsiStyle` con `as_ansi() -> &'static str` (tipado fuerte, sin `String` allocation). | §1.4 |
| **Cargo** | 🟡 Deps | **Eliminada** la propuesta de `colored = "2"` de la v0. Sprint 2 termina con **cero dependencias nuevas**. | §1.2, Módulos afectados |

**Resultado**: pendiente de cierre. C1, C2, C3 ya incorporados a la especificación. Qwen dará el veredicto final tras la primera tanda de tests en verde.

### Iteración 0 — 2026-07-11 (propuesta inicial)

> Preservada en [`adr-002-sprint2-features.v0.md`](adr-002-sprint2-features.v0.md).

**Disparador**: features R7, R10, R6 priorizadas en V0 y diferidas explícitamente por ADR-001.

**Puntos clave de la v0 que esta versión modifica:**

- ❌ `ColorFormatter<F>` decorator con `replacen` — **eliminado** (leaky abstraction).
- ❌ Dependencia `colored = "2"` — **eliminada** (cero deps en Sprint 2).
- ❌ `escape_json` con 7 caracteres — **ampliado** a RFC 8259 completo.
- ❌ `iso8601_utc_now` con `unwrap_or_default` — **cambiado** a `Result` propagado.
- ⚠️ Integración builder↔formatter sub-especificada — **ahora explícita** (C1).

---

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
| **R10** | Sin soporte de colores. `SimpleTextFormatter` produce texto plano | `SimpleTextFormatter` con `Option<ColorScheme>` para aplicar ANSI |
| **R6** | Sin formato JSON | `JsonFormatter` con escape RFC 8259 y timestamp ISO 8601 UTC |

ADR-001 ya dejó los hooks necesarios: el trait `Formatter`, la inyección de formateadores en cada `Sink`, y la separación entre `Logger` y el formateo. Sprint 2 implementa los formateadores y las macros que aprovechan esos hooks, **sin tocar el core**.

## Decisión

Adoptamos las siguientes decisiones arquitectónicas para Sprint 2:

1. **R7 — Macros con captura de metadatos.** Las macros `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!` se introducen como `macro_rules!` en un nuevo módulo `src/macros.rs`. Cada macro recibe `&$Logger` como primer argumento y se expande a `logger.log(Nivel, Metadata::new(module_path!(), file!(), line!()), || format!(...))`. Los métodos helper (`info`, `debug`, …) **se conservan** para uso en tests, en runtime genérico, y para quienes no quieran usar macros.
2. **R10 — Colores ANSI internalizados.** Se elimina el decorator `ColorFormatter<F>` propuesto en v0 (ver *Motivo* §6). En su lugar, `SimpleTextFormatter` admite un `Option<ColorScheme>` opcional en su constructor: cuando es `Some`, aplica códigos ANSI al prefijo `[LEVEL]` durante el `format`; cuando es `None`, produce el mismo output que antes (compatible con el test 19 de Sprint 1). Se introducen los tipos `AnsiStyle` (enum de estilos) y `ColorScheme` (struct con un `AnsiStyle` por nivel), ambos con códigos ANSI hardcodeados (cero dependencias externas). R11 (auto-desactivación) se aborda **parcialmente**: `LoggerBuilder::colors(bool)` controla el default; la auto-detección TTY queda fuera de Sprint 2.
3. **R6 — Formato JSON.** Un nuevo adaptador `JsonFormatter` que produce un objeto JSON con los campos `timestamp`, `level`, `message`, `module`, `file`, `line` en **orden estable**. Escape conforme a RFC 8259 (todos los caracteres de control U+0000–U+001F). Timestamp ISO 8601 UTC generado manualmente con `SystemTime`. **Sin `serde`, sin `chrono`, sin `time`** — implementación manual. La función `iso8601_utc_now` propaga errores (no usa `unwrap_or_default`): un reloj de sistema antes de UNIX_EPOCH produce un `LogError::Config`, no un timestamp incorrecto silencioso.

### Restricciones autoimpuestas

- Sprint 2 **no modifica** `domain/`, `ports/`, ni la estructura del `Logger`. Sólo añade: (a) macros en un módulo nuevo, (b) lógica de colores en `SimpleTextFormatter`, (c) `JsonFormatter`, (d) tests. El core sigue intacto.
- **Cero dependencias nuevas**. La propuesta de `colored` en v0 queda **descartada**. Implementación manual de códigos ANSI (~30 líneas).
- **No se introducen features fuera de R7 / R10 / R6.** R4, R8, R9, R11-auto, R12, R14, R22, R23–R30 quedan para ADRs futuros.
- Las macros son el camino recomendado, pero los métodos helper **siguen existiendo** y se usan en tests. Coexistencia, no reemplazo.
- `Metadata::UNKNOWN` se mantiene como valor por defecto en los métodos helper; las macros inyectan metadata real. Esto preserva el contrato de los métodos.
- `SimpleTextFormatter::default()` y `SimpleTextFormatter::new()` **no aplican colores**, manteniendo la compatibilidad con el test 19 de Sprint 1. Los colores se activan sólo cuando el `LoggerBuilder` los solicita explícitamente.

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
│   ├── mod.rs                   ← añade re-exports de ColorScheme y AnsiStyle
│   ├── console.rs               ← SIN CAMBIOS
│   ├── text_format.rs           ← MODIFICADO: ahora con Option<ColorScheme>
│   └── json_format.rs           ← NUEVO: JsonFormatter
├── app/                         ← SIN CAMBIOS estructurales
│   ├── mod.rs                   ← añade `colors` al builder (R10)
│   ├── logger.rs                ← SIN CAMBIOS
│   ├── config.rs                ← LoggerBuilder: añade `.colors(bool)`
│   └── level_filter.rs          ← SIN CAMBIOS
└── tests/
    └── smoke.rs                 ← ampliada para incluir R7, R10, R6
```

> **Eliminado de v0**: `src/adapters/color_format.rs` (no hay `ColorFormatter<F>` decorator en v1). Los colores viven dentro de `text_format.rs`.

#### 1.2 Dependencias de terceros — **cero nuevas** en Sprint 2

```toml
# Cargo.toml — diff respecto a Sprint 1

[dependencies]
# (sin cambios — Sprint 1 dejó el crate sin dependencias externas)

# Mantenemos comentadas (no se usan en Sprint 2):
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# chrono = "0.4"
# colored = "2"   # propuesta de v0, descartada en v1

[dev-dependencies]
# Único añadido del sprint: para tests de conformidad JSON.
serde_json = "1"
```

**Justificación de cero deps en runtime** (después de la corrección arquitectónica de v1):

- **ANSI colors**: ~30 líneas hardcodeadas con códigos `\x1b[..m`. Comparado con `colored` (~25 KB compilado, un crate): la diferencia de LOC es despreciable, **y la leaky abstraction del decorator ya no aplica** porque los colores viven dentro del formatter que controla el formato. Por tanto, el trade-off que justificó `colored` en v0 desaparece.
- **JSON escape**: ~20 líneas con un `match` exhaustivo. Comparado con `serde_json` (~150 KB + `serde` proc-macro): no compensa para 6 campos.
- **ISO 8601**: ~30 líneas con el algoritmo de Howard Hinnant (date algorithms, dominio público). Comparado con `chrono` (~200 KB): no compensa.

> **Único añadido en `[dev-dependencies]`**: `serde_json = "1"` para validar conformidad JSON en tests (I2). No afecta al grafo de dependencias en runtime.

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

1. **API explícita por logger**: `info!(&logger, "msg")` con el `&Logger` como primer argumento. No hay estado global (Sprint 1 lo prohibió en Iteración 2 C2). Tres beneficios: testabilidad, multi-logger, cero inicialización oculta.
2. **Sintaxis estilo `format!`**: `info!(&logger, "x = {}, y = {}", x, y)`. El mensaje se construye con `format!` dentro de la closure, así que el fast path R34 sigue funcionando: si el filtro corta, la closure nunca se invoca.
3. **`Metadata::new` se usa en lugar de `Metadata::UNKNOWN`**: las macros siempre capturan metadata real del call site. Esto cumple R7.
4. **Built-in macros sin `$crate::`**: `module_path!()`, `file!()`, `line!()` son built-ins que el resolver evalúa en el *call site* del usuario, no en el crate que define la macro. Por eso se llaman sin prefijo `$crate::` (que sí usamos para los tipos como `$crate::LogLevel` y `$crate::Metadata::new`, que sí viven en nuestro crate).
5. **Coexistencia con métodos helper**: `logger.info("msg")` (método, `Metadata::UNKNOWN`) e `info!(&logger, "msg")` (macro, metadata real) coexisten. El usuario elige.
6. **Convención minúsculas (M1)**: las macros siguen la convención de la comunidad Rust (`log::info!`, `tracing::info!`). Los niveles (`INFO`, `TRACE`, …) van en mayúsculas, también por convención. La asimetría es intencional y se documenta en `src/macros.rs` con un comentario de cabecera.
7. **Alocación de literales (I5)**: `info!(&logger, "lit")` **aloca un `String`** para el mensaje, aunque el literal sea estático. El fast path R34 sólo evita la alocación cuando el nivel está desactivado por el filtro. Si el log se emite, hay una alocación. La zero-alloc con `core::fmt::Arguments` queda para el sprint de rendimiento (R34+R35).

**Uso esperado:**

```rust
use oxidize_log::{Logger, info, warn, error};

let logger = Logger::default();

info!(&logger, "Usuario autenticado: {}", user_id);
warn!(&logger, "Cache miss en {}", key);
error!(&logger, "Conexión fallida: {}", e);
```

#### 1.4 R10 — Colores ANSI internalizados en `SimpleTextFormatter`

```rust
// src/adapters/text_format.rs (esquema — sección de colores, aditiva)

/// Estilos ANSI como enum tipado (M5). Representación: &'static str sin alocación.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiStyle {
    Plain,
    Dimmed,
    Bold,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BoldRed,
    BoldRedOnWhite,
}

impl AnsiStyle {
    pub const RESET: &'static str = "\x1b[0m";

    pub fn as_ansi(&self) -> &'static str {
        match self {
            Self::Plain            => "",
            Self::Dimmed           => "\x1b[2m",
            Self::Bold             => "\x1b[1m",
            Self::Red              => "\x1b[31m",
            Self::Green            => "\x1b[32m",
            Self::Yellow           => "\x1b[33m",
            Self::Blue             => "\x1b[34m",
            Self::Magenta          => "\x1b[35m",
            Self::Cyan             => "\x1b[36m",
            Self::White            => "\x1b[37m",
            Self::BoldRed          => "\x1b[1;31m",
            Self::BoldRedOnWhite   => "\x1b[1;31;47m",
        }
    }
}

/// Scheme: un `AnsiStyle` por nivel. `Copy` para ergonomía en builders y tests (I1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScheme {
    pub trace: AnsiStyle,
    pub debug: AnsiStyle,
    pub info:  AnsiStyle,
    pub warn:  AnsiStyle,
    pub error: AnsiStyle,
    pub fatal: AnsiStyle,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            trace: AnsiStyle::Dimmed,
            debug: AnsiStyle::Cyan,
            info:  AnsiStyle::Green,
            warn:  AnsiStyle::Yellow,
            error: AnsiStyle::Red,
            fatal: AnsiStyle::BoldRedOnWhite,
        }
    }
}

/// `SimpleTextFormatter` ahora lleva un `Option<ColorScheme>` opcional.
pub struct SimpleTextFormatter {
    colors: Option<ColorScheme>,
}

impl SimpleTextFormatter {
    /// Sin colores. Output: `[LEVEL] mensaje\n`. Compatible con el test 19 de Sprint 1.
    pub fn new() -> Self { Self { colors: None } }

    /// Con colores. Aplica el scheme al prefijo `[LEVEL]`.
    pub fn with_colors(scheme: ColorScheme) -> Self { Self { colors: Some(scheme) } }
}

impl Default for SimpleTextFormatter {
    /// Default = sin colores. Mantiene el output del prototipo y de Sprint 1.
    fn default() -> Self { Self::new() }
}

impl Formatter for SimpleTextFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let level_token = format!("[{}]", event.level.as_str());
        match &self.colors {
            None => Ok(format!("{} {}\n", level_token, event.message)),
            Some(scheme) => {
                let style = match event.level {
                    LogLevel::Trace => scheme.trace,
                    LogLevel::Debug => scheme.debug,
                    LogLevel::Info  => scheme.info,
                    LogLevel::Warn  => scheme.warn,
                    LogLevel::Error => scheme.error,
                    LogLevel::Fatal => scheme.fatal,
                };
                // M3: Una sola alocación para el output final coloreado.
                Ok(format!("{}{}{} {}\n", style.as_ansi(), level_token, AnsiStyle::RESET, event.message))
            }
        }
    }
}
```

**Decisiones de diseño de R10 (revisadas en v1):**

1. **Colores internalizados en `SimpleTextFormatter`**, no en un decorator. La v0 propuso `ColorFormatter<F>` que envolvía a otro formatter y usaba `String::replacen` para sustituir `[INFO]` por su versión coloreada. Esto es una **leaky abstraction** (asumía la estructura interna del output del inner) y producía combinaciones inválidas (`ColorFormatter<JsonFormatter>` daba JSON corrupto). Se elimina. (M4, ver *Motivo* §6.)
2. **`Option<ColorScheme>` en `SimpleTextFormatter`**: presencia de colores es decisión del formatter, no de un envoltorio externo. Cero indirección, cero `replacen`, cero alocaciones mágicas.
3. **`AnsiStyle` como `enum` con `as_ansi() -> &'static str`** (M5): tipado fuerte, sin `String`, sin crate externo. El `RESET` es constante pública.
4. **`ColorScheme: Copy`** (I1): permite pasar el scheme por valor en builders, tests y constructores sin fricción.
5. **Constructores explícitos** (I7): `SimpleTextFormatter::new()` (sin colores) y `SimpleTextFormatter::with_colors(scheme)`. `Default = new()`, lo que **preserva la compatibilidad con el test 19 de Sprint 1** sin cambios.
6. **Limitación Windows legacy (I6)**: los códigos ANSI no funcionan en `cmd.exe` de Windows pre-10-TH2 sin habilitar `ENABLE_VIRTUAL_TERMINAL_PROCESSING`. Windows 10+ y terminales modernos (Windows Terminal, PowerShell Core) los soportan nativamente. Documentado en el `README` y en este ADR. Trade-off aceptable: el target del proyecto son pipelines modernos (ELK, CloudWatch), no terminals legacy.
7. **No se modifica `ConsoleSink`**: el sink sigue agnóstico al formatter. El usuario decide los colores eligiendo el `SimpleTextFormatter` que inyecta.

**Uso esperado:**

```rust
use oxidize_log::{LoggerBuilder, ConsoleSink, SimpleTextFormatter, ColorScheme};
use std::sync::Arc;

// Sin colores (compatible con Sprint 1):
let plain = Arc::new(SimpleTextFormatter::new());

// Con colores:
let colored = Arc::new(SimpleTextFormatter::with_colors(ColorScheme::default()));

// O con un scheme custom:
let custom = Arc::new(SimpleTextFormatter::with_colors(ColorScheme {
    info: AnsiStyle::Cyan,
    warn: AnsiStyle::BoldRed,
    ..ColorScheme::default()
}));

let sink = Arc::new(ConsoleSink::new(colored));
let logger = LoggerBuilder::new().sink(sink).build();
```

#### 1.5 R6 — Formato JSON

```rust
// src/adapters/json_format.rs (esquema)

use crate::domain::{LogEvent, LogError};
use crate::ports::Formatter;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self { Self }
}

impl Default for JsonFormatter {
    fn default() -> Self { Self::new() }  // I3: unit struct idiomático
}

impl Formatter for JsonFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let timestamp = iso8601_utc_now()?;  // C3: propaga error

        // I4: orden de campos estable: timestamp, level, message, module, file, line.
        // Razón: legibilidad humana y consistencia con tracing-subscriber.
        let mut out = String::with_capacity(160);
        out.push('{');
        write_kv(&mut out, "timestamp", &timestamp);    out.push(',');
        write_kv(&mut out, "level",     event.level.as_str()); out.push(',');
        write_kv(&mut out, "message",   &event.message); out.push(',');
        write_kv(&mut out, "module",    event.metadata.module); out.push(',');
        write_kv(&mut out, "file",      event.metadata.file);   out.push(',');
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

fn write_kv_u32(out: &mut String, key: &str, value: u32) {
    out.push('"'); out.push_str(key); out.push_str("\":");
    // M2: to_string() aloca en el heap. Micro-ineficiencia documentada
    // como aceptable para Sprint 2, a optimizar en R35 (sprint de rendimiento).
    out.push_str(&value.to_string());
}

/// Escape JSON conforme a RFC 8259 §7 (C2).
/// Cubre los 7 escapes con nombre + cualquier carácter de control U+0000–U+001F
/// en formato `\uXXXX`. Unicode no-ASCII pasa tal cual (UTF-8 válido en JSON).
fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"'  => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c <= '\u{001F}' => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// ISO 8601 UTC. C3: propaga error en lugar de degradar a 1970-01-01 silenciosamente.
fn iso8601_utc_now() -> Result<String, LogError> {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| LogError::Config("system clock is before UNIX epoch"))?;
    Ok(format_iso8601_from_duration(dur))
}

/// Función pura extraída para testabilidad (M1).
/// Internamente usa el algoritmo de Howard Hinnant (date algorithms, dominio público).
/// Convierte secs desde UNIX_EPOCH a "YYYY-MM-DDTHH:MM:SS.nnnZ".
/// ~30 líneas; ver Howard Hinnant: "chrono-Compatible Low-Level Date Algorithms".
fn format_iso8601_from_duration(dur: Duration) -> String { /* ... */ }
```

**Decisiones de diseño de R6:**

1. **Construcción manual del JSON, sin `serde`**: 6 campos son strings y un `u32`. `serde` añadiría proc-macro + 2 crates. No compensa.
2. **Escape JSON conforme a RFC 8259 §7** (C2): cubre todos los caracteres de control U+0000–U+001F, no sólo los 7 con nombre. Esto garantiza conformidad con parsers estrictos.
3. **`iso8601_utc_now` propaga `Result`** (C3): `SystemTime` antes de UNIX_EPOCH produce `LogError::Config` en lugar de un timestamp "1970-01-01" silenciosamente incorrecto. La firma cambia a `Result<String, LogError>`, coherente con `Formatter::format`.
4. **`Default` para `JsonFormatter`** (I3): unit struct idiomático.
5. **Orden de campos estable y documentado** (I4): `timestamp, level, message, module, file, line`. JSON no requiere orden, pero humanos sí. Consistencia con `tracing-subscriber` para que pipelines de log aggregation no necesiten reordenar.
6. **`SystemTime` no es monótono** (M3): puede ser afectado por NTP o cambios manuales de reloj. Para un logger esto significa que timestamps pueden no estar en orden cronológico estricto. Limitación general de los loggers, no específica de este ADR. Documentado en el código y en el README.
7. **Sin campos structured (`fields: {}`)** en Sprint 2: schema minimal. R4 los añadirá en Sprint 3 sin breaking change.
8. **Fuzzing ausente (M2)**: `escape_json` no tiene tests de property-based. Recomendación: añadir `proptest` o `cargo-fuzz` en un sprint futuro. Documentado como TODO en `Consecuencias`.

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

pub struct LoggerBuilder {
    level_filter: Option<Arc<dyn Filter>>,
    extra_filters: Vec<Arc<dyn Filter>>,
    sinks: Vec<Arc<dyn Sink>>,
    colors_enabled: bool,   // NUEVO (R10)
}

impl LoggerBuilder {
    /// Habilita o deshabilita colores ANSI en el **formatter por defecto**
    /// construido por `.build()`. Default: `true`.
    ///
    /// **Importante** (C1): este método sólo afecta al `SimpleTextFormatter`
    /// que el builder construye cuando no se ha inyectado ningún sink vía
    /// `.sink()`. Si el usuario ya inyectó un sink custom, el builder **no
    /// lo modifica** — el usuario es responsable de los colores en su propio
    /// formatter. Esto se documenta explícitamente para evitar ambigüedad.
    pub fn colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    pub fn build(self) -> Logger {
        let level_filter = self.level_filter
            .unwrap_or_else(|| Arc::new(LevelFilter::new(LogLevel::Info)));

        let mut filters = Vec::with_capacity(1 + self.extra_filters.len());
        filters.push(level_filter);
        filters.extend(self.extra_filters);

        let sinks = if self.sinks.is_empty() {
            // C1: el builder construye el sink por defecto aquí. Si el usuario
            // inyectó un sink custom, no entramos en esta rama.
            let formatter: Arc<dyn Formatter> = if self.colors_enabled {
                Arc::new(SimpleTextFormatter::with_colors(ColorScheme::default()))
            } else {
                Arc::new(SimpleTextFormatter::new())
            };
            vec![Arc::new(ConsoleSink::new(formatter))]
        } else {
            self.sinks   // el usuario es responsable de su propio formatter
        };

        Logger { filters, sinks, last_error: Arc::new(Mutex::new(None)) }
    }
}
```

**Uso esperado:**

```rust
// Caso 1: default con colores (true por defecto):
let logger = LoggerBuilder::new().build();   // ConsoleSink + SimpleTextFormatter::with_colors(default)

// Caso 2: sin colores:
let logger = LoggerBuilder::new().colors(false).build();  // SimpleTextFormatter::new()

// Caso 3: sink custom — `.colors()` no tiene efecto sobre él:
let my_formatter = Arc::new(SimpleTextFormatter::with_colors(custom_scheme));
let my_sink = Arc::new(ConsoleSink::new(my_formatter));
let logger = LoggerBuilder::new()
    .colors(false)            // no-op: ya hay un sink custom
    .sink(my_sink)
    .build();
// → los colores de my_sink son los de custom_scheme, no afectados por .colors(false)
```

> **Nota sobre el test 19 de Sprint 1**: ese test construye un `Logger` con `ConsoleSink::with_writer` (sink custom con `Vec<u8>`) y verifica `[INFO] hola\n` sin códigos ANSI. Como inyecta un sink custom, **no le afecta** el cambio de default del `Builder`. Sigue pasando sin cambios. La rama `self.sinks.is_empty()` del builder **nunca se evalúa** en ese test.

### 2. Bindings y Capa de FFI (Multiplataforma)

> Este sprint **no entrega bindings**. La sección es idéntica a la de ADR-001.
>
> PDR-002 (estrategia de bindings: WASM vs N-API, JNI wrapper o directo) queda fuera de este ADR.

### 3. Rendimiento, Concurrencia y Memoria

- **Cero cambios en la sincronización**: las macros son `macro_rules!` que se expanden en tiempo de compilación. El `Logger::log` subyacente es el mismo que en Sprint 1. R19 (`Send + Sync`) y R36 (sin `panic`) se preservan.
- **Cero alocaciones adicionales en fast path**: las macros usan `Metadata::new(...)` que es `Copy` (Sprint 1 lo dejó así). `Metadata::new(module_path!(), file!(), line!())` se evalúa en el call site de la macro, pero el `String` del mensaje (vía closure) sólo se construye si los filtros pasan. **R34 se preserva**.
- **R10 (colores)**: cada log emitido con colores aloca **un `String` extra** (el prefijo con códigos ANSI). El alocamiento es despreciable comparado con el `write` al `Write` del sink. Sin colores: cero alocaciones extra respecto a Sprint 1.
- **R6 (JSON)**: una sola `String` por log emitido, capacidad inicial 160 bytes (suficiente para logs típicos; redimensiona si no). `escape_json` aloca otro `String` por valor de retorno (~15–30 bytes para mensajes típicos). Aceptable.

### 4. Sinks e Infraestructura (Adaptadores) — adiciones

#### 4.1 `SimpleTextFormatter` extendido (R10)

Definido en §1.4. Ahora con `Option<ColorScheme>` opcional. No se introduce ningún adaptador nuevo para colores — la funcionalidad vive dentro de `SimpleTextFormatter`.

#### 4.2 `JsonFormatter` (R6)

Definido en §1.5. Standalone. Implementa `Default` además de `new()`.

#### 4.3 Sinks entregados en Sprint 2

- `ConsoleSink` — **sin cambios**. Sigue agnóstico al formatter.
- **No** se entrega `FileSink`, `RotatingFileSink`, ni `CloudWatchSink` en Sprint 2. (R14, R15, R16 → sprints futuros.)

## Motivo

**PDR de origen:** decisión directa sin PDR previo. Sprint 2 cubre features ya priorizadas en el V0; los tradeoffs están contenidos en este ADR.

### 1. Por qué macros con `&Logger` y no estado global

Sprint 1 tomó la decisión explícita de no tener estado global (Iteración 2 C2, "no hay `static LOGGER`"). Sprint 2 mantiene esa decisión: las macros son explícitas sobre qué logger usan. Tres beneficios:

1. **Testabilidad**: un test puede usar un logger con un sink que escribe a un `Vec<u8>` y verificar que la metadata se capturó correctamente.
2. **Multi-logger**: un proceso puede tener varios loggers (uno para audit, otro para debug, otro para metrics), cada uno con su propio nivel, sink y formatter.
3. **Cero inicialización oculta**: no hay `Lazy` o `OnceCell` que pueda tener problemas de orden de inicialización en tests.

El trade-off es que el usuario tiene que pasar `&logger` en cada llamada. La ergonomía es buena: `info!(&logger, "msg")` se lee natural. Si en el futuro se quiere azúcar con un logger por defecto, se introduce `oxidize_log::logger()` o similar, pero no se hace en Sprint 2.

### 2. Por qué `SimpleTextFormatter` con `Option<ColorScheme>` y no un `Formatter` separado para colores

(Inicialmente propuesta en v0 como `ColorFormatter<F>`; revisada en v1. Ver §6 para la justificación del rechazo.)

Un formatter separado para colores tendría que producir un output estructuralmente compatible con el formatter "interno" (los colores se aplican a un token que el inner debe producir). Eso obliga a:

- O bien un contrato de string (que es lo que la v0 asumía con `replacen`).
- O bien reformatear el `LogEvent` desde cero, duplicando lógica de formato.

Ambos son leaky abstractions. La solución de v1 — colores dentro del formatter que controla el formato — evita la duplicación y la dependencia de un string opaco. Costo: el decorator pattern deja de ser posible para colores. Trade-off aceptado (M4).

### 3. Por qué cero dependencias (sin `colored`, sin `serde`, sin `chrono`)

La v0 proponía `colored = "2"` para R10. **Esta decisión se revisa en v1**:

- Sin decorator, los colores son ~30 líneas hardcodeadas. La justificación de `colored` (TTY/Windows gratis) **deja de compensar**: el coste de implementación manual es despreciable, y la cobertura TTY/Windows puede documentarse como limitación aceptable (I6).
- `serde` + `serde_json` + `chrono` para 6 strings: ~350 KB, ~1 s de compilación incremental, y un proc-macro. No compensa.
- ISO 8601 con el algoritmo de Howard Hinnant (date algorithms, dominio público): ~30 líneas, sin dep.

Resultado: Sprint 2 termina con **cero `[dependencies]` nuevas**. La única adición es `serde_json` en `[dev-dependencies]` para validar conformidad JSON en tests (I2).

### 4. Por qué JSON manual con escape RFC 8259 completo

`escape_json` cubre todos los caracteres de control U+0000–U+001F (C2). Esto es lo que exige RFC 8259 §7 ("All Unicode characters may be placed within the quotation marks, except for the characters that MUST be escaped: quotation mark, reverse solidus, and the control characters U+0000 through U+001F."). Una implementación parcial (sólo los 7 con nombre) produciría JSON inválido para parsers estrictos si el mensaje contiene un `\x00`, `\x01`, etc.

El coste extra: ~3 líneas (`c if c <= '\u{001F}' => format!("\\u{:04x}", c as u32)`). El beneficio: conformidad total con el estándar.

### 5. Por qué `iso8601_utc_now` devuelve `Result` y no degrada con `unwrap_or_default`

`SystemTime::now().duration_since(UNIX_EPOCH)` devuelve `Err` sólo si el reloj del sistema está **antes** de 1970-01-01. Esto no ocurre en máquinas modernas, **pero puede ocurrir** en:

- Sistemas con reloj mal configurado (BIOS con fecha 1969, VMs con drift).
- Dispositivos embebidos sin RTC válida al arrancar.
- Tests que mockean el reloj (futuro).

`unwrap_or_default()` (propuesto en v0) degrada silenciosamente a `1970-01-01T00:00:00Z`. Esto es **peor que un error explícito**: el usuario ve timestamps "1970" en producción y no sabe por qué. La solución correcta es propagar el error como `LogError::Config` (C3) y dejar que el `Logger` lo degrade como cualquier otro error (vía `last_error` y log a `stderr`).

### 6. Por qué NO `ColorFormatter<F>` decorator (corrección arquitectónica de v1)

**Disparador**: feedback del Arquitecto Principal + revisión de Qwen. La v0 proponía:

```rust
// v0 — RECHAZADO
impl<F: Formatter> Formatter for ColorFormatter<F> {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let formatted = self.inner.format(event)?;
        let level_token = format!("[{}]", event.level.as_str());
        let colored = match event.level { /* paint */ };
        Ok(formatted.replacen(&level_token, &colored, 1))   // ← leaky
    }
}
```

**Problemas identificados**:

1. **Leaky abstraction (M4)**: `ColorFormatter` asume que el output del inner comienza con `[LEVEL] algo`. Esto es cierto para `SimpleTextFormatter`, pero **no es un contrato del trait `Formatter`**. Si alguien implementa `Formatter` con un formato distinto (e.g. `"2026-07-10 INFO mensaje"`), `ColorFormatter::format` falla silenciosamente o produce output corrupto.
2. **`ColorFormatter<JsonFormatter>` es inválido**: aplicar colores al JSON lo rompe. `{"timestamp":"...","[INFO]"` no es JSON válido. El decorator **no es componible** con cualquier formatter.
3. **`String::replacen` es un parche sintáctico**: reemplaza la primera ocurrencia de `[INFO]` por su versión coloreada. Si el mensaje contiene `[INFO]` (falso positivo, raro pero posible), se rompe. Si el inner produce `[ INFO ]` con espacios, no se reemplaza.
4. **Doble alocación inútil**: el inner formatea, el decorator aloca un `String` extra para la versión coloreada del token, después hace `replacen` que aloca otro `String` para el output final. Tres alocaciones por línea.

**Alternativa adoptada en v1**: los colores son un **detalle de implementación del formatter**, no un wrapper. `SimpleTextFormatter` ya controla la estructura de su output; añadirle un `Option<ColorScheme>` es una decisión que vive donde corresponde. Componibilidad compleja (middleware/pipeline de formateo) llega en un ADR-003 futuro si se necesita; por ahora, un formatter = un formato.

### 7. Por qué no incluir R4, R8, R9, R11-auto, R12, R14, R22 en Sprint 2

Regla del Sprint: **tres features, no diez**. Si Sprint 2 intenta cubrir todo lo que el V0 marca como P0/P1, se convierte en un sprint de varios meses y el código pierde coherencia. Cada feature tiene su propio ADR o PDR.

**Excepciones que sí se abordan parcialmente:**

- **R11 (auto-desactivación)**: el flag `colors_enabled: bool` del `LoggerBuilder` cubre la parte manual. La auto-detección TTY queda para Sprint 3 (es un `if !is_terminal() { self.colors_enabled = false; }` en el builder, trivial).

**Features que se difieren explícitamente (abren su propio ADR/PDR):**

| Req | Por qué no en Sprint 2 | Próximo sprint candidato |
|---|---|---|
| R4 (campos clave/valor) | Requiere `serde` o cambio de schema JSON | Sprint 3 |
| R8 (nombre de función) | Requiere crate `function_name` o RTTI; trade-off bin size | Sprint 3 |
| R9 (desactivar metadatos) | Cambio de API; no crítico | Sprint 3 |
| R11-auto (TTY detection) | Trivial con `std::io::IsTerminal`; necesita tests manuales | Sprint 3 |
| R12 (color themes) | Tras R10 y R11-auto; secundario | Sprint 3 |
| R14 (sink archivo) | Otro adaptador + tests; sustantivo pero autocontenido | Sprint 3 |
| R15 (rotación) | Depende de R14 | Sprint 4 |
| R16 (CloudWatch) | AWS SDK, async, complejo | Sprint 5 (P2) |
| R22 (buffering/async) | Cambia la forma del `Logger::log`; ADR dedicado | Sprint 4 |
| R23–R30 (bindings) | Workspace split primero (PDR-001) | Sprint 6+ |

## Consecuencias

### Positivas

- **R7 (P0) cumplido**: las macros capturan `file!`/`line!`/`module_path!` automáticamente. El usuario tiene una API ergonómica: `info!(&logger, "msg")`.
- **R10 (P0) cumplido**: colores ANSI en consola. `SimpleTextFormatter::with_colors(scheme)` produce el output coloreado, con un `ColorScheme` configurable y `Copy` para ergonomía.
- **R6 (P1) cumplido**: `JsonFormatter` produce JSON conforme a RFC 8259, con orden de campos estable y timestamp ISO 8601 UTC.
- **Cero cambios en el core** (`domain/`, `ports/`, `app/Logger`). El refactor de Sprint 1 sigue intacto; Sprint 2 es estrictamente aditivo.
- **Cero alocaciones extra en fast path** (R34 preservado). R10 sólo añade una alocación por línea **emitida** (no en fast path).
- **Cero dependencias de runtime nuevas**. Sprint 2 no introduce ni un solo crate en `[dependencies]`. La única adición es `serde_json` en `[dev-dependencies]` para tests.
- **Leaky abstraction eliminada**: los colores viven dentro del formatter que controla el formato. Componibilidad restaurada con cualquier otro formatter.
- **Coexistencia macros + métodos helper**: tests pueden seguir usando `logger.info("msg")` con `Metadata::UNKNOWN`. Producción usa macros con metadata real.
- **Test 19 de Sprint 1 sin cambios**: `SimpleTextFormatter::new()` y `Default = new()` preservan el output del prototipo. La rama del builder que aplica colores sólo se evalúa cuando **no** se inyecta un sink custom.

### Negativas o restricciones introducidas

- **Sin decorator para colores**: si en el futuro se quiere componer colores con otro formatter (e.g. `ColorDecorator<JsonFormatter>`), hay que reintroducir la abstracción. ADR-003 cuando se necesite.
- **JSON sin `serde`**: la lógica de escape y formato es manual. ~50 líneas de código que un test de fuzzing debería cubrir (M2: TODO para sprint futuro).
- **`iso8601_utc_now` cambia a `Result`**: cualquier consumidor de `JsonFormatter::format` que no espere el error path necesita `?` o `match`. En la práctica, el `Sink` ya espera `Result`, así que el impacto es mínimo.
- **Limitación Windows legacy (I6)**: `cmd.exe` pre-10-TH2 no muestra colores ANSI sin habilitar `ENABLE_VIRTUAL_TERMINAL_PROCESSING`. Trade-off aceptable; documentado.
- **R4, R8, R9, R11-auto, R12, R14, R15, R16, R22, R23–R30 quedan fuera**. Algunos son P0 (R14, R15) y se abordarán en sprints dedicados. El V0 prioriza, no impone plazos.
- **Macros alocan `String` para literales** (I5): `info!(&logger, "lit")` aloca aunque el literal sea estático. Zero-alloc con `Arguments` queda para el sprint de rendimiento.
- **`SystemTime` no monótono** (M3): timestamps pueden no estar en orden cronológico estricto. Limitación general de loggers, no específica de este ADR.
- **Fuzzing ausente para `escape_json`** (M2): la conformidad RFC 8259 está validada con casos del Anexo, pero property-based / `cargo-fuzz` queda como TODO. Un input con caracteres Unicode extremos podría exponer un edge case.

### Restricciones autoimpuestas (reiteración)

- No se modifica `domain/`, `ports/`, ni el struct `Logger`.
- No se introducen features fuera de R7 / R10 / R6.
- Las macros coexisten con los métodos helper, no los reemplazan.
- Cero `panic!` en producción; cero `unwrap`/`expect` en código de producción. La única excepción es `LogError::Config` propagado correctamente desde `iso8601_utc_now` (C3).
- Cero `[dependencies]` nuevas. `serde_json` en `[dev-dependencies]` es la única adición.

## Módulos afectados

| Módulo / Archivo | Tipo de cambio |
|---|---|
| `src/lib.rs` | **Modificado** — añade `pub mod macros;` y `pub use adapters::{SimpleTextFormatter, ColorScheme, AnsiStyle, JsonFormatter};` |
| `src/macros.rs` | **Nuevo** — `macro_rules!` para `trace!`, `debug!`, `info!`, `warn!`, `error!`, `fatal!` |
| `src/domain/` | **Sin cambios** |
| `src/ports/` | **Sin cambios** |
| `src/adapters/console.rs` | **Sin cambios** |
| `src/adapters/text_format.rs` | **Modificado** — añade `AnsiStyle`, `ColorScheme`, y `Option<ColorScheme>` interno. Nuevos constructores `new()` y `with_colors()`. `Default = new()`. |
| ~~`src/adapters/color_format.rs`~~ | **Eliminado de v0** — el decorator `ColorFormatter<F>` no se implementa. |
| `src/adapters/json_format.rs` | **Nuevo** — `JsonFormatter` con `escape_json` RFC 8259 y `iso8601_utc_now -> Result` |
| `src/adapters/mod.rs` | **Modificado** — re-exports actualizados (sin `ColorFormatter`) |
| `src/app/logger.rs` | **Sin cambios** |
| `src/app/config.rs` | **Modificado** — `LoggerBuilder::colors(bool)` con semántica explícita (C1) |
| `src/app/level_filter.rs` | **Sin cambios** |
| `tests/smoke.rs` | **Modificado** — smoke tests para R7, R10, R6 |
| `examples/test.rs` | **Modificado** — usa macros (R7) en lugar de métodos helper |
| `Cargo.toml` (`[dependencies]`) | **Sin cambios** — **cero deps nuevas** |
| `Cargo.toml` (`[dev-dependencies]`) | **Modificado** — añade `serde_json = "1"` para tests de conformidad |
| `docs/adr/adr-001-sprint1-hexagonal-refactor.md` | **Sin cambios** (Sprint 1 cerrado) |
| `docs/adr/pdr-001-workspace-split.md` | **Sin cambios** (decisión independiente, Opción B) |

## Criterio de implementación completa

Verificable. Cada item es una orden que se cierra o queda abierta con justificación.

- [ ] `cargo build` sin warnings con `#![deny(warnings)]` en `src/lib.rs`
- [ ] `cargo test` con los **tests TDD del Sprint 2 en verde** (lista cerrada, ver Anexo)
- [ ] `cargo clippy --all-targets -- -D warnings` pasa limpio
- [ ] `grep -rE "\b(unwrap|expect|panic!)\b" src/ --include="*.rs" | grep -vE "mod tests|#\[cfg\(test\)\]"` devuelve vacío. **Sin excepciones**: `iso8601_utc_now` propaga con `?` (C3), no hay `unwrap_or_default` en prod.
- [ ] `info!(&logger, "msg")` produce una línea con `module`, `file`, `line` reales (verificable con un test que capture el log y parsee el JSON del `JsonFormatter` o inspeccione el `LogEvent` de un mock)
- [ ] `SimpleTextFormatter::with_colors(ColorScheme::default()).format(&event_info)` produce output con `\x1b[32m` (green), `[INFO]`, y `\x1b[0m` (reset)
- [ ] `SimpleTextFormatter::new().format(&event)` produce output **idéntico** al del prototipo / Sprint 1 (test 19 sigue verde sin cambios)
- [ ] `JsonFormatter` produce JSON válido parseable por `serde_json::from_str` (test de conformidad)
- [ ] `JsonFormatter` escapa correctamente todos los caracteres de control U+0000–U+001F, no sólo los 7 con nombre (C2)
- [ ] `JsonFormatter::format` propaga `LogError::Config` cuando el reloj del sistema está antes de UNIX_EPOCH (C3). Testabilidad (M1): la conversión de fechas se extrae a una función pura `format_iso8601_from_duration` que se testea con duraciones específicas, obviando la necesidad de inyectar un trait `Clock` en el Sprint 2.
- [ ] `Cargo.toml` tiene **cero** entradas nuevas en `[dependencies]`. `serde_json` aparece en `[dev-dependencies]`. `serde`, `chrono`, `colored` siguen comentadas.
- [ ] `examples/test.rs` actualizado para usar las macros, output visible vía `cargo run --example test`
- [ ] `tests/smoke.rs` ampliado con smoke tests de R7, R10, R6
- [ ] No se introducen features fuera de R7 / R10 / R6. Cualquier tentación se delega al siguiente sprint.

**Tests relevantes en verde**: los tests TDD del Sprint 2 (ver Anexo).

## Todo de implementación por fases (mini sprints)

> Adaptación del template al contexto de un logger en Rust. Las cinco fases canónicas (domain / TDD / app / adapters / controller) se traducen a: *tipos de colores → adaptadores → TDD → capa de macros y builder → tests de integración*. Cada fase termina con tests en verde y un commit con mensaje conventional (`feat(sprint2): ...`).

### Fase 1 — Tipos de colores (R10 infraestructura)

**Objetivo**: `AnsiStyle` y `ColorScheme` definidos, copiables, testeados como unidades puras.

- [ ] `AnsiStyle` enum con `as_ansi() -> &'static str` (M5).
- [ ] `ColorScheme` struct con `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` (I1) y `Default` con los 6 niveles mapeados.
- [ ] Tests unitarios: `ansi_style_as_ansi_returns_correct_codes`, `color_scheme_default_maps_levels_correctly` (sin todavía un formatter, sólo la pura materialización de los códigos).

### Fase 2 — Adaptadores (R10 aplicación, R6 base)

**Objetivo**: `SimpleTextFormatter` extendido con colores; `JsonFormatter` con escape RFC 8259 completo y `Result` propagado.

- [ ] `src/adapters/text_format.rs`: campo `colors: Option<ColorScheme>`, constructores `new()` y `with_colors(scheme)`, `Default = new()` (I7), `Formatter` impl con dos ramas (con/sin colores).
- [ ] `src/adapters/json_format.rs`: `JsonFormatter` con `Default` (I3), `escape_json` RFC 8259 completo (C2), `iso8601_utc_now -> Result` (C3), `Formatter` impl con orden de campos estable (I4).
- [ ] `src/adapters/mod.rs` re-exports actualizados.
- [ ] Tests unitarios: bloques B y C del Anexo (sin smoke).

### Fase 3 — TDD bloque por bloque

**Objetivo**: Cada test del Anexo escrito antes (o en paralelo) a la implementación. Cobertura: 12 tests unit + 2 smoke = 14.

- [ ] Bloque A (R7 macros) — 4 tests, 1 por macro comportamiento (captura, fast-path, multi-args, coexistencia).
- [ ] Bloque B (R10 colores) — 4 tests, 1 por comportamiento (códigos ANSI en `[INFO]`, scheme default, sin colores en `new()`, integración con `ConsoleSink`).
- [ ] Bloque C (R6 JSON) — 5 tests, incluyendo conformidad RFC 8259 (test 4 ampliado) y propagación de error de clock (test 5 nuevo, si se hace testeable).
- [ ] Bloque D (smoke) — 2 tests en `tests/smoke.rs`.

### Fase 4 — Capa de aplicación (builder + macros)

**Objetivo**: `LoggerBuilder::colors(bool)` y `src/macros.rs` conectados al core, sin modificar el `Logger`.

- [ ] `src/app/config.rs`: campo `colors_enabled: bool` en `LoggerBuilder`; método `pub fn colors(mut self, enabled: bool) -> Self`; rama de `.build()` que aplica `SimpleTextFormatter::with_colors(default)` cuando `colors_enabled = true` y no hay sinks custom (C1).
- [ ] Documentación inline del método `.colors()` explicando la semántica con sinks custom.
- [ ] `src/macros.rs`: las 6 macros con la sintaxis exacta de §1.3.
- [ ] `src/lib.rs`: `pub mod macros;` y re-exports actualizados.

### Fase 5 — Integración, smoke, criterios de cierre

**Objetivo**: Sprint cerrado. 14/14 tests en verde. Clippy limpio. Cero `unwrap`/`expect`/`panic!` en prod.

- [ ] `cargo test --all` verde.
- [ ] `cargo clippy --all-targets -- -D warnings` limpio.
- [ ] `grep` del criterio de `unwrap`/`expect`/`panic!` ejecutado y verificado **vacío**.
- [ ] `Cargo.toml` revisado: `[dependencies]` intacto, `serde_json` en `[dev-dependencies]`.
- [ ] Test 19 de Sprint 1 sigue verde sin cambios (verificación explícita).
- [ ] Cierre: estado del ADR pasa a `implementado`, commit con tag `sprint-2-closed`.

## Anexo — Lista cerrada de tests TDD del Sprint 2

> Esta lista es **el** criterio de cierre. Si un test no está, el sprint no está cerrado.
> Numeración y trazabilidad a V0 entre paréntesis.

### Bloque A — R7: Macros con captura de metadatos (4 tests)

1. `trace_macro_captures_file_line_module` (R7) — `trace!(&logger, "msg")` produce un `LogEvent` con `metadata.module` igual a `module_path!()`, `metadata.file` igual a `file!()`, `metadata.line` igual a `line!()`. Verificable con un `MockSink` que capture el evento.
2. `info_macro_skips_evaluation_when_filtered` (R7, R34) — con `LevelFilter::Warn`, `info!(&logger, "x = {}", expensive())` **no llama a `expensive()`**. El fast path se preserva a través de la macro.
3. `error_macro_does_not_panic_on_complex_args` (R7, R36) — `error!(&logger, "x = {}, y = {}", x, y)` funciona con múltiples argumentos, similar a `format!`.
4. `macros_coexist_with_helper_methods` (R7) — `logger.info("msg")` (helper) usa `Metadata::UNKNOWN`; `info!(&logger, "msg")` (macro) usa metadata real. Ambos coexisten en el mismo proceso sin colisión.

### Bloque B — R10: Colores ANSI internalizados (4 tests)

1. `simple_text_formatter_with_colors_wraps_level_with_ansi_codes` (R10) — `SimpleTextFormatter::with_colors(ColorScheme::default()).format(&event_info)` produce un `String` que contiene `\x1b[32m` (green), `[INFO]`, y `\x1b[0m` (reset).
2. `color_scheme_default_maps_levels_to_colors` (R10) — `ColorScheme::default()` aplica los códigos correctos: `trace` gris, `debug` cian, `info` verde, `warn` amarillo, `error` rojo, `fatal` rojo bold.
3. `simple_text_formatter_new_produces_no_ansi` (R10, R11) — `SimpleTextFormatter::new().format(&event)` produce el mismo output que el prototipo / Sprint 1 (sin códigos ANSI). **Esto protege el test 19 de Sprint 1.**
4. `simple_text_formatter_with_console_sink_writes_colored_bytes` (R10, R13) — integración: un `Logger` con `ConsoleSink(SimpleTextFormatter::with_colors(...))` redirigido a un `Vec<u8>` produce bytes que contienen códigos ANSI.

### Bloque C — R6: Formato JSON conforme a RFC 8259 (5 tests)

1. `json_formatter_produces_valid_json_object` (R6, I2) — `JsonFormatter::format(&event)` produce un `String` parseable por `serde_json::from_str::<serde_json::Value>` desde `[dev-dependencies]`.
2. `json_formatter_includes_all_required_fields_in_stable_order` (R6, I2, I4) — el JSON contiene exactamente `timestamp, level, message, module, file, line` en ese orden. Verificable iterando sobre `serde_json::Value::as_object()`.
3. `json_formatter_iso8601_utc_timestamp_format` (R6) — `timestamp` matchea la regex `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$`.
4. `json_formatter_escapes_all_control_chars_rfc8259` (R6, C2) — un mensaje con `"`, `\`, `\n`, `\r`, `\t`, `\b`, `\f`, **más** un `\x00`, `\x01`, `\x1f` produce JSON que sigue siendo parseable por `serde_json::from_str` y los caracteres especiales están escapados (`\"`, `\\`, `\n`, `\r`, `\t`, `\b`, `\f`, `\u0000`, `\u0001`, `\u001f`).
5. `json_formatter_propagates_clock_error` (R6, C3) — `JsonFormatter::format` devuelve `Err(LogError::Config(_))` cuando el reloj del sistema está antes de UNIX_EPOCH. Testeable indirectamente; el formato en sí se testea unitariamente pasándole una `Duration` a la función pura `format_iso8601_from_duration` (M1).

### Bloque D — Smoke / integración (2 tests)

1. `smoke_macros_capture_real_metadata` (R7) — test de integración en `tests/smoke.rs`: usa `info!(&logger, "msg")` desde un módulo específico y verifica que el `module` capturado es ese módulo.
2. `smoke_color_and_json_combined` (R10, R6) — test de integración: un logger con `ConsoleSink(JsonFormatter)` redirigido a `Vec<u8>` produce JSON parseable con los 6 campos correctos.

### Compatibilidad con Sprint 1

| Test del Sprint 1 | Estado en Sprint 2 |
|---|---|
| Los 20 unit tests + 1 smoke test | **Siguen pasando sin cambios** (Sprint 2 es estrictamente aditivo) |
| Test 19 (`logger_default_emite_a_writer_capturado`) | **Sigue verde sin cambios** — `SimpleTextFormatter::new()` / `Default = new()` preserva el output sin códigos ANSI, y el test usa `ConsoleSink::with_writer` (sink custom) que el builder no modifica |
| `cargo clippy` limpio | **Sigue limpio** (sin nuevos warnings) |
| `Cargo.toml [dependencies]` sin deps nuevas (Sprint 1) | **Sigue sin deps nuevas** en `[dependencies]` — la única adición del sprint es `serde_json` en `[dev-dependencies]` |

## Historial de superación

**Estado:** en revisión
**Superado por:** N/A
**Fecha:** N/A
**Motivo:** N/A

## Resumen de features resueltos

- **R7 (P0)** — Las macros `trace!` / `debug!` / `info!` / `warn!` / `error!` / `fatal!` capturan `file!`/`line!`/`module_path!` del call site. API ergonómica con `format!`-style args. El fast path de Sprint 1 (R34) se preserva a través de las macros.
- **R10 (P0)** — `SimpleTextFormatter::with_colors(ColorScheme)` aplica códigos ANSI al prefijo `[LEVEL]` con un scheme configurable (6 estilos tipados en `AnsiStyle`, hardcodeados, sin deps). `ColorScheme: Copy` para ergonomía. R11 (auto-desactivación) cubierto parcialmente con `LoggerBuilder::colors(bool)`.
- **R6 (P1)** — `JsonFormatter` produce JSON conforme a RFC 8259 con 6 campos en orden estable (`timestamp, level, message, module, file, line`) y timestamp ISO 8601 UTC. Escape exhaustivo de U+0000–U+001F. Errores de clock se propagan como `LogError::Config` (no degradación silenciosa).
- **Configuración**: `LoggerBuilder::colors(bool)` con semántica explícita sobre sinks custom.

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
- **Fuzzing para `escape_json`** (M2) — `proptest` o `cargo-fuzz` con inputs adversariales. ADR dedicado cuando se introduzca.
