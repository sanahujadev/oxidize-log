use crate::domain::event::LogEvent;
use crate::domain::error::LogError;

pub trait Formatter: Send + Sync {
    fn format(&self, event: &LogEvent) -> Result<String, LogError>;
}
