use crate::domain::event::LogEvent;
use crate::domain::error::LogError;
use crate::ports::formatter::Formatter;

pub struct SimpleTextFormatter;

impl Formatter for SimpleTextFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        Ok(format!("[{}] {}\n", event.level, event.message))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;
    use crate::domain::level::LogLevel;
    use crate::domain::event::Metadata;

    #[test]
    fn formatter_texto_simple_produce_formato_esperado() {
        let formatter = SimpleTextFormatter;
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::UNKNOWN,
        };
        let result = formatter.format(&event).unwrap();
        assert_eq!(result, "[INFO] Hello world\n");
    }
}
