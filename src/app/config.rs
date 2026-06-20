use std::sync::{Arc, Mutex};
use crate::domain::level::LogLevel;
use crate::ports::filter::Filter;
use crate::ports::sink::Sink;
use crate::app::logger::Logger;
use crate::app::level_filter::LevelFilter;
use crate::adapters::console::ConsoleSink;
use crate::adapters::text_format::SimpleTextFormatter;

pub struct LoggerBuilder {
    filters: Vec<Arc<dyn Filter>>,
    sinks: Vec<Arc<dyn Sink>>,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        Self {
            filters: vec![Arc::new(LevelFilter::new(LogLevel::Info))],
            sinks: vec![Arc::new(ConsoleSink::new(Arc::new(SimpleTextFormatter)))],
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

    pub fn level(mut self, level: LogLevel) -> Self {
        self.filters.push(Arc::new(LevelFilter::new(level)));
        self
    }

    pub fn build(self) -> Logger {
        Logger {
            filters: self.filters,
            sinks: self.sinks,
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn from_config(config: LoggerConfig) -> Self {
        let mut builder = LoggerBuilder::new().level(config.level);
        // Reseteamos los sinks del default porque from_config decide
        builder.sinks.clear();
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
