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
}

