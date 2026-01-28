use crate::{LoggerConfig, LogLevel, Sink, build_sinks};

pub struct Logger {
    pub config: LoggerConfig,
    sinks: Vec<Box<dyn Sink>>,
}

impl Logger {
    pub fn init(config: LoggerConfig) -> Self {
        let sinks = build_sinks(&config.sinks);
        Self { config, sinks }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if level < self.config.level {
            return;
        }

        for sink in &self.sinks {
            sink.log(level, message);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use crate::{LogLevel, LoggerConfig};

    // MockSink: captura los logs en memoria
    struct MockSink {
        pub calls: std::cell::RefCell<Vec<(LogLevel, String)>>,
    }

    impl MockSink {
        fn new() -> Self {
            Self {
                calls: std::cell::RefCell::new(Vec::new()),
            }
        }
    }

    impl Sink for MockSink {
        fn log(&self, level: LogLevel, message: &str) {
            self.calls
                .borrow_mut()
                .push((level, message.to_string()));
        }

        fn as_any(&self) -> &dyn Any { self }
    }

        #[test]
    fn logger_logs_equal_or_higher_levels() {
        let mock = MockSink::new();

        let config = LoggerConfig {
            level: LogLevel::Info,
            colors: false,
            sinks: vec![],
        };

        let logger = Logger {
            config,
            sinks: vec![Box::new(mock)],
        };

        logger.log(LogLevel::Info, "hello");
        logger.log(LogLevel::Error, "boom");

        let mock = logger.sinks[0]
            .as_any()
            .downcast_ref::<MockSink>()
            .unwrap();

        let calls = mock.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].1, "hello");
        assert_eq!(calls[1].1, "boom");
    }
}
