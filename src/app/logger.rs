use std::sync::{Arc, Mutex};
use crate::domain::level::LogLevel;
use crate::domain::event::{LogEvent, Metadata};
use crate::domain::error::LogError;
use crate::ports::filter::Filter;
use crate::ports::sink::Sink;
use crate::app::config::LoggerBuilder;

#[derive(Clone)]
pub struct Logger {
    pub(crate) filters: Vec<Arc<dyn Filter>>,
    pub(crate) sinks: Vec<Arc<dyn Sink>>,
    pub(crate) last_error: Arc<Mutex<Option<LogError>>>,
}

impl Logger {
    pub fn log<F>(&self, level: LogLevel, metadata: Metadata, message: F)
    where
        F: FnOnce() -> String,
    {
        if !self.filters.iter().all(|f| f.enabled(metadata, level)) {
            return;
        }

        let event = LogEvent {
            level,
            message: message(),
            metadata,
        };

        for sink in &self.sinks {
            if let Err(e) = sink.write(&event) {
                self.record_error(e);
            }
        }
    }

    fn record_error(&self, err: LogError) {
        match self.last_error.lock() {
            Ok(mut slot) => *slot = Some(err),
            Err(poisoned) => {
                let mut slot = poisoned.into_inner();
                *slot = Some(LogError::Config("previous sink error poisoned the lock"));
            }
        }
    }

    pub fn trace<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Trace, Metadata::UNKNOWN, m); }
    pub fn debug<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Debug, Metadata::UNKNOWN, m); }
    pub fn info <F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Info,  Metadata::UNKNOWN, m); }
    pub fn warn <F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Warn,  Metadata::UNKNOWN, m); }
    pub fn error<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Error, Metadata::UNKNOWN, m); }
    pub fn fatal<F: FnOnce() -> String>(&self, m: F) { self.log(LogLevel::Fatal, Metadata::UNKNOWN, m); }

    pub fn init(config: crate::app::config::LoggerConfig) -> Self {
        LoggerBuilder::from_config(config).build()
    }
}

impl Default for Logger {
    fn default() -> Self {
        LoggerBuilder::new().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use crate::app::level_filter::LevelFilter;

    struct MockSink {
        calls: Mutex<Vec<LogEvent>>,
        should_fail: bool,
    }

    impl MockSink {
        fn new(should_fail: bool) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                should_fail,
            }
        }
    }

    impl Sink for MockSink {
        fn write(&self, event: &LogEvent) -> Result<(), LogError> {
            if self.should_fail {
                return Err(LogError::Write { sink: "MockSink", source: "mock error".into() });
            }
            self.calls.lock().unwrap().push(event.clone());
            Ok(())
        }
    }

    struct MockFilter {
        allow_module: &'static str,
    }

    impl Filter for MockFilter {
        fn enabled(&self, metadata: Metadata, _level: LogLevel) -> bool {
            metadata.module == self.allow_module
        }
    }

    #[test]
    fn sink_no_se_invoca_en_fast_path() {
        let sink = Arc::new(MockSink::new(false));
        let filter = Arc::new(LevelFilter::new(LogLevel::Info));
        let logger = Logger {
            filters: vec![filter],
            sinks: vec![sink.clone()],
            last_error: Arc::new(Mutex::new(None)),
        };

        logger.debug(|| "this should not run".to_string());
        assert_eq!(sink.calls.lock().unwrap().len(), 0);
    }

    #[test]
    fn filter_personalizado_puede_decidir_por_metadata() {
        let filter = Arc::new(MockFilter { allow_module: "my_module" });
        let sink = Arc::new(MockSink::new(false));
        let logger = Logger {
            filters: vec![filter],
            sinks: vec![sink.clone()],
            last_error: Arc::new(Mutex::new(None)),
        };

        let meta_allowed = Metadata::new("my_module", "file.rs", 10);
        logger.log(LogLevel::Info, meta_allowed, || "allowed".to_string());

        let meta_blocked = Metadata::new("other_module", "file.rs", 10);
        logger.log(LogLevel::Info, meta_blocked, || "blocked".to_string());

        let calls = sink.calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].message, "allowed");
    }

    #[test]
    fn logger_default_usa_consola_texto_y_info() {
        let logger = Logger::default();
        // Just checking that it works and does not panic
        logger.info(|| "hello from default logger".to_string());
    }

    // Helper wrapper so we can inspect what was written in test 19
    #[derive(Clone)]
    struct VecWriter {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl std::io::Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn logger_default_emite_a_writer_capturado() {
        let formatter = Arc::new(crate::adapters::text_format::SimpleTextFormatter);
        let written_data = Arc::new(Mutex::new(Vec::new()));
        let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn std::io::Write + Send + Sync>));

        let sink = Arc::new(crate::adapters::console::ConsoleSink::with_writer(formatter, writer));

        let logger = LoggerBuilder::new().sink(sink).build();

        logger.info(|| "hola".to_string());

        let data = written_data.lock().unwrap();
        assert_eq!(std::str::from_utf8(&data).unwrap(), "[INFO] hola\n");
    }

    #[test]
    fn logger_log_evalua_mensaje_via_closure() {
        let filter = Arc::new(LevelFilter::new(LogLevel::Info));
        let sink = Arc::new(MockSink::new(false));
        let logger = Logger {
            filters: vec![filter],
            sinks: vec![sink.clone()],
            last_error: Arc::new(Mutex::new(None)),
        };

        let counter = Cell::new(0);

        logger.debug(|| {
            counter.set(counter.get() + 1);
            "should not run".to_string()
        });

        assert_eq!(counter.get(), 0);

        logger.info(|| {
            counter.set(counter.get() + 1);
            "should run".to_string()
        });

        assert_eq!(counter.get(), 1);
        assert_eq!(sink.calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn logger_redirige_a_multiples_sinks() {
        let sink1 = Arc::new(MockSink::new(false));
        let sink2 = Arc::new(MockSink::new(false));
        let logger = Logger {
            filters: vec![],
            sinks: vec![sink1.clone(), sink2.clone()],
            last_error: Arc::new(Mutex::new(None)),
        };

        logger.info(|| "multisink".to_string());

        assert_eq!(sink1.calls.lock().unwrap().len(), 1);
        assert_eq!(sink2.calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn logger_propag_error_de_sink_sin_panicar() {
        let fail_sink = Arc::new(MockSink::new(true));
        let ok_sink = Arc::new(MockSink::new(false));
        let logger = Logger {
            filters: vec![],
            sinks: vec![fail_sink.clone(), ok_sink.clone()],
            last_error: Arc::new(Mutex::new(None)),
        };

        logger.info(|| "will fail in first sink".to_string());

        // El primer sink falla pero NO entra en panic.
        // El segundo sink DEBE recibir el mensaje.
        assert_eq!(ok_sink.calls.lock().unwrap().len(), 1);

        let last_err = logger.last_error.lock().unwrap();
        assert!(last_err.is_some());
        let err_str = last_err.as_ref().unwrap().to_string();
        assert!(err_str.contains("write error in sink MockSink: mock error"));
    }
}
