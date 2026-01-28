use crate::{LogLevel, Environment, LoggerConfig};

pub struct Logger {
    pub config: LoggerConfig,
}

impl Logger {
    pub fn init(config: LoggerConfig) -> Self { Self { config } }

    pub fn init_default() -> Self {
        let config = LoggerConfig::from_env(Environment::Dev);
        Self::init(config) 
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if (level as u8) < (self.config.level as u8) {
            return;
        }

        // Implementación mínima por ahora
        println!("[{:?}] {}", level, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LoggerConfig, Environment, LogLevel};

    #[test]
    fn logger_respects_configured_level() {
        let config = LoggerConfig::from_env(Environment::Prod); // WARN+
        let logger = Logger::init(config);

        assert!(LogLevel::Info < logger.config.level); // INFO < WARN
        assert!(LogLevel::Error >= logger.config.level);
    }

    #[test]
    fn logger_initializes_with_config() {
        let config = LoggerConfig::from_env(Environment::Dev);
        let logger = Logger::init(config.clone());

        assert_eq!(logger.config.level, config.level);
        assert_eq!(logger.config.colors, config.colors);
        assert_eq!(logger.config.sinks, config.sinks);
    }
}
