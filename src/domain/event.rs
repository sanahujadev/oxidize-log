use crate::domain::level::LogLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metadata {
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
}

impl Metadata {
    pub const UNKNOWN: Metadata = Metadata {
        module: "<unknown>",
        file: "<unknown>",
        line: 0,
    };

    pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
        Self { module, file, line }
    }
}

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub level: LogLevel,
    pub message: String,
    pub metadata: Metadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logevent_construye_con_nivel_mensaje_y_metadata() {
        let metadata = Metadata::new("test_module", "test.rs", 42);
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello".to_string(),
            metadata,
        };
        assert_eq!(event.level, LogLevel::Info);
        assert_eq!(event.message, "Hello");
        assert_eq!(event.metadata, metadata);

        // Copy is implemented for Metadata
        let metadata_copy = metadata;
        assert_eq!(metadata, metadata_copy);
    }

    #[test]
    fn metadata_origen_devuelve_file_line_module() {
        let meta = Metadata::new("test_module", "test.rs", 42);
        assert_eq!(meta.module, "test_module");
        assert_eq!(meta.file, "test.rs");
        assert_eq!(meta.line, 42);

        let unknown = Metadata::UNKNOWN;
        assert_eq!(unknown.module, "<unknown>");
        assert_eq!(unknown.file, "<unknown>");
        assert_eq!(unknown.line, 0);
    }
}
