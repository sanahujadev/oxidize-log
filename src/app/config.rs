use std::sync::{Arc, Mutex};
use crate::domain::level::LogLevel;
use crate::ports::filter::Filter;
use crate::ports::sink::Sink;
use crate::app::logger::Logger;
use crate::app::level_filter::LevelFilter;
use crate::adapters::console::ConsoleSink;
use crate::adapters::text_format::SimpleTextFormatter;

pub struct LoggerBuilder {
    /// El filtro de nivel mínimo. Si es None al llamar a `build()`,
    /// se resuelve a `LevelFilter::new(LogLevel::Info)` (R39).
    /// Reemplazable: la última llamada a `.level()` gana.
    level_filter: Option<Arc<dyn Filter>>,
    /// Filtros adicionales del usuario (custom). Se evalúan después
    /// del `level_filter` con `.all()`, así que basta con que uno
    /// rechace para que el log no se emita.
    extra_filters: Vec<Arc<dyn Filter>>,
    /// Sinks configurados. Si está vacío al llamar a `build()`,
    /// se inyecta `ConsoleSink::new(SimpleTextFormatter)` por defecto (R39).
    /// Cualquier `.sink()` añade, y se respeta la lista del usuario.
    sinks: Vec<Arc<dyn Sink>>,
}

impl LoggerBuilder {
    /// Estado vacío: defaults se resuelven en `build()`.
    pub fn new() -> Self {
        Self {
            level_filter: None,
            extra_filters: vec![],
            sinks: vec![],
        }
    }

    /// Fija el nivel mínimo. **Reemplaza** cualquier valor anterior.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.level_filter = Some(Arc::new(LevelFilter::new(level)));
        self
    }

    /// Añade un filtro custom (se acumula).
    pub fn filter(mut self, f: Arc<dyn Filter>) -> Self {
        self.extra_filters.push(f);
        self
    }

    /// Añade un sink (se acumula).
    pub fn sink(mut self, s: Arc<dyn Sink>) -> Self {
        self.sinks.push(s);
        self
    }

    pub fn build(self) -> Logger {
        // Resolver level_filter (R39: Info por defecto)
        let level_filter = self.level_filter
            .unwrap_or_else(|| Arc::new(LevelFilter::new(LogLevel::Info)));

        // Componer la lista de filtros: level primero, extras después.
        // `Logger::log` evalúa con `.all()`, así que el orden no afecta
        // la decisión final, pero ponemos el nivel primero porque es
        // el caso más común de fast-path.
        let mut filters = Vec::with_capacity(1 + self.extra_filters.len());
        filters.push(level_filter);
        filters.extend(self.extra_filters);

        // Resolver sinks (R39: ConsoleSink por defecto si la lista está vacía)
        let sinks = if self.sinks.is_empty() {
            vec![Arc::new(ConsoleSink::new(Arc::new(SimpleTextFormatter))) as Arc<dyn Sink>]
        } else {
            self.sinks
        };

        Logger {
            filters,
            sinks,
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    /// Fachada de compatibilidad (Qwen C3, Opción A).
    /// Ahora coherente con el nuevo modelo: empezamos con `level` fijado
    /// y sinks vacíos; `build()` resuelve los defaults si hace falta.
    pub fn from_config(config: LoggerConfig) -> Self {
        let mut builder = Self {
            level_filter: Some(Arc::new(LevelFilter::new(config.level))),
            extra_filters: vec![],
            sinks: vec![],
        };
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
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Environment { Dev, Staging, Prod }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkConfig { Console }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggerConfig {
    pub level: LogLevel,
    pub colors: bool,
    pub sinks: Vec<SinkConfig>,
}

impl LoggerConfig {
    pub fn from_env(env: Environment) -> Self {
        match env {
            Environment::Dev => Self {
                level: LogLevel::Debug,
                colors: true,
                sinks: vec![SinkConfig::Console],
            },
            Environment::Staging => Self {
                level: LogLevel::Info,
                colors: false,
                sinks: vec![SinkConfig::Console],
            },
            Environment::Prod => Self {
                level: LogLevel::Warn,
                colors: false,
                sinks: vec![SinkConfig::Console],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn config_for_dev() {
        let cfg = LoggerConfig::from_env(Environment::Dev);
        assert_eq!(cfg.level, LogLevel::Debug);
        assert!(cfg.colors);
        assert_eq!(cfg.sinks, vec![SinkConfig::Console]);
    }

    #[test]
    fn config_for_staging() {
        let cfg = LoggerConfig::from_env(Environment::Staging);
        assert_eq!(cfg.level, LogLevel::Info);
        assert!(!cfg.colors);
        assert_eq!(cfg.sinks, vec![SinkConfig::Console]);
    }

    #[test]
    fn config_for_prod() {
        let cfg = LoggerConfig::from_env(Environment::Prod);
        assert_eq!(cfg.level, LogLevel::Warn);
        assert!(!cfg.colors);
        assert_eq!(cfg.sinks, vec![SinkConfig::Console]);
    }
}
