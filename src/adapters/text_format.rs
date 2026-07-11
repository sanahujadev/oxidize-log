use crate::domain::event::LogEvent;
use crate::domain::error::LogError;
use crate::ports::formatter::Formatter;

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
            Self::Plain => "",
            Self::Dimmed => "\x1b[2m",
            Self::Bold => "\x1b[1m",
            Self::Red => "\x1b[31m",
            Self::Green => "\x1b[32m",
            Self::Yellow => "\x1b[33m",
            Self::Blue => "\x1b[34m",
            Self::Magenta => "\x1b[35m",
            Self::Cyan => "\x1b[36m",
            Self::White => "\x1b[37m",
            Self::BoldRed => "\x1b[1;31m",
            Self::BoldRedOnWhite => "\x1b[1;31;47m",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorScheme {
    pub trace: AnsiStyle,
    pub debug: AnsiStyle,
    pub info: AnsiStyle,
    pub warn: AnsiStyle,
    pub error: AnsiStyle,
    pub fatal: AnsiStyle,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            trace: AnsiStyle::Dimmed,
            debug: AnsiStyle::Cyan,
            info: AnsiStyle::Green,
            warn: AnsiStyle::Yellow,
            error: AnsiStyle::Red,
            fatal: AnsiStyle::BoldRedOnWhite,
        }
    }
}

use crate::domain::level::LogLevel;

pub struct SimpleTextFormatter {
    colors: Option<ColorScheme>,
}

impl SimpleTextFormatter {
    pub fn new() -> Self {
        Self { colors: None }
    }
    pub fn with_colors(scheme: ColorScheme) -> Self {
        Self { colors: Some(scheme) }
    }
}

impl Default for SimpleTextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for SimpleTextFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        match &self.colors {
            None => Ok(format!("[{}] {}\n", event.level, event.message)),
            Some(scheme) => {
                let style = match event.level {
                    LogLevel::Trace => scheme.trace,
                    LogLevel::Debug => scheme.debug,
                    LogLevel::Info => scheme.info,
                    LogLevel::Warn => scheme.warn,
                    LogLevel::Error => scheme.error,
                    LogLevel::Fatal => scheme.fatal,
                };
                Ok(format!("{}[{}]{} {}\n", style.as_ansi(), event.level, AnsiStyle::RESET, event.message))
            }
        }
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
        let formatter = SimpleTextFormatter::new();
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::UNKNOWN,
        };
        let result = formatter.format(&event).unwrap();
        assert_eq!(result, "[INFO] Hello world\n");
    }

    #[test]
    fn ansi_style_as_ansi_returns_correct_codes() {
        assert_eq!(AnsiStyle::Plain.as_ansi(), "");
        assert_eq!(AnsiStyle::Dimmed.as_ansi(), "\x1b[2m");
        assert_eq!(AnsiStyle::Bold.as_ansi(), "\x1b[1m");
        assert_eq!(AnsiStyle::Red.as_ansi(), "\x1b[31m");
        assert_eq!(AnsiStyle::Green.as_ansi(), "\x1b[32m");
        assert_eq!(AnsiStyle::Yellow.as_ansi(), "\x1b[33m");
        assert_eq!(AnsiStyle::Blue.as_ansi(), "\x1b[34m");
        assert_eq!(AnsiStyle::Magenta.as_ansi(), "\x1b[35m");
        assert_eq!(AnsiStyle::Cyan.as_ansi(), "\x1b[36m");
        assert_eq!(AnsiStyle::White.as_ansi(), "\x1b[37m");
        assert_eq!(AnsiStyle::BoldRed.as_ansi(), "\x1b[1;31m");
        assert_eq!(AnsiStyle::BoldRedOnWhite.as_ansi(), "\x1b[1;31;47m");
        assert_eq!(AnsiStyle::RESET, "\x1b[0m");
    }

    #[test]
    fn color_scheme_default_maps_levels_correctly() {
        let scheme = ColorScheme::default();
        assert_eq!(scheme.trace, AnsiStyle::Dimmed);
        assert_eq!(scheme.debug, AnsiStyle::Cyan);
        assert_eq!(scheme.info, AnsiStyle::Green);
        assert_eq!(scheme.warn, AnsiStyle::Yellow);
        assert_eq!(scheme.error, AnsiStyle::Red);
        assert_eq!(scheme.fatal, AnsiStyle::BoldRedOnWhite);
    }

    #[test]
    fn simple_text_formatter_new_produces_no_ansi() {
        let formatter = SimpleTextFormatter::new();
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::UNKNOWN,
        };
        let result = formatter.format(&event).unwrap();
        assert_eq!(result, "[INFO] Hello world\n");
        assert!(!result.contains("\x1b"));
    }

    #[test]
    fn simple_text_formatter_with_colors_wraps_level_with_ansi_codes() {
        let scheme = ColorScheme::default();
        let formatter = SimpleTextFormatter::with_colors(scheme);
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::UNKNOWN,
        };
        let result = formatter.format(&event).unwrap();
        let expected = format!("{}[INFO]{} Hello world\n", AnsiStyle::Green.as_ansi(), AnsiStyle::RESET);
        assert_eq!(result, expected);
        assert!(result.contains("\x1b[32m"));
        assert!(result.contains("\x1b[0m"));
    }

    #[test]
    fn simple_text_formatter_with_console_sink_writes_colored_bytes() {
        use std::sync::{Arc, Mutex};
        use std::io::Write;
        use crate::adapters::console::ConsoleSink;
        use crate::ports::sink::Sink;

        #[derive(Clone)]
        struct TestVecWriter {
            data: Arc<Mutex<Vec<u8>>>,
        }
        impl Write for TestVecWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.data.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let scheme = ColorScheme::default();
        let formatter = Arc::new(SimpleTextFormatter::with_colors(scheme));
        let written_data = Arc::new(Mutex::new(Vec::new()));
        let writer = Arc::new(Mutex::new(Box::new(TestVecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));

        let sink = ConsoleSink::with_writer(formatter, writer);
        let event = LogEvent {
            level: LogLevel::Info,
            message: "test log".to_string(),
            metadata: Metadata::UNKNOWN,
        };

        sink.write(&event).unwrap();

        let data = written_data.lock().unwrap();
        let result_str = std::str::from_utf8(&data).unwrap();
        let expected = format!("{}[INFO]{} test log\n", AnsiStyle::Green.as_ansi(), AnsiStyle::RESET);
        assert_eq!(result_str, expected);
    }
}

