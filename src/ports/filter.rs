use crate::domain::event::Metadata;
use crate::domain::level::LogLevel;

pub trait Filter: Send + Sync {
    fn enabled(&self, metadata: Metadata, level: LogLevel) -> bool;
}
