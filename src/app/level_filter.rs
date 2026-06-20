use crate::domain::level::LogLevel;
use crate::domain::event::Metadata;
use crate::ports::filter::Filter;

pub struct LevelFilter {
    min: LogLevel,
}

impl LevelFilter {
    pub fn new(min: LogLevel) -> Self {
        Self { min }
    }
}

impl Filter for LevelFilter {
    fn enabled(&self, _metadata: Metadata, level: LogLevel) -> bool {
        level >= self.min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_por_nivel_minimo_bloquea_niveles_inferiores() {
        let filter = LevelFilter::new(LogLevel::Info);

        assert!(!filter.enabled(Metadata::UNKNOWN, LogLevel::Trace));
        assert!(!filter.enabled(Metadata::UNKNOWN, LogLevel::Debug));

        assert!(filter.enabled(Metadata::UNKNOWN, LogLevel::Info));
        assert!(filter.enabled(Metadata::UNKNOWN, LogLevel::Warn));
        assert!(filter.enabled(Metadata::UNKNOWN, LogLevel::Error));
        assert!(filter.enabled(Metadata::UNKNOWN, LogLevel::Fatal));
    }
}
