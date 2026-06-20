use std::sync::{Arc, Mutex};
use std::io::Write;
use crate::domain::event::LogEvent;
use crate::domain::error::LogError;
use crate::ports::sink::Sink;
use crate::ports::formatter::Formatter;

pub struct ConsoleSink {
    formatter: Arc<dyn Formatter>,
    writer: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
}

impl ConsoleSink {
    pub fn new(formatter: Arc<dyn Formatter>) -> Self {
        Self {
            formatter,
            writer: Arc::new(Mutex::new(Box::new(std::io::stdout()))),
        }
    }

    pub fn with_writer(
        formatter: Arc<dyn Formatter>,
        writer: Arc<Mutex<Box<dyn Write + Send + Sync>>>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::level::LogLevel;
    use crate::domain::event::Metadata;

    struct MockFormatter {
        content: String,
    }

    impl Formatter for MockFormatter {
        fn format(&self, _event: &LogEvent) -> Result<String, LogError> {
            Ok(self.content.clone())
        }
    }

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("mock write error"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    // Helper wrapper so we can inspect what was written in test
    #[derive(Clone)]
    struct VecWriter {
        data: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for VecWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.data.lock().unwrap().write(buf)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn console_sink_escribe_con_formatter_inyectado() {
        let formatter = Arc::new(MockFormatter { content: "formatted log\n".to_string() });
        let written_data = Arc::new(Mutex::new(Vec::new()));
        let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));

        let sink = ConsoleSink::with_writer(formatter, writer);

        let event = LogEvent {
            level: LogLevel::Info,
            message: "hello".to_string(),
            metadata: Metadata::UNKNOWN,
        };

        sink.write(&event).unwrap();

        let data = written_data.lock().unwrap();
        assert_eq!(std::str::from_utf8(&data).unwrap(), "formatted log\n");
    }

    #[test]
    fn console_sink_no_panica_si_writer_falla() {
        let formatter = Arc::new(MockFormatter { content: "msg".to_string() });
        let writer = Arc::new(Mutex::new(Box::new(FailingWriter) as Box<dyn Write + Send + Sync>));

        let sink = ConsoleSink::with_writer(formatter, writer);

        let event = LogEvent {
            level: LogLevel::Info,
            message: "hello".to_string(),
            metadata: Metadata::UNKNOWN,
        };

        let result = sink.write(&event);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("write error in sink ConsoleSink: mock write error"));
    }
}
