use crate::LogLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Dev,
    Staging,
    Prod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SinkConfig {
    Console,
}

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
    use crate::LogLevel;

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

