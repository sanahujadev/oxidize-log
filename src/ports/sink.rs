use crate::domain::event::LogEvent;
use crate::domain::error::LogError;

pub trait Sink: Send + Sync {
    fn write(&self, event: &LogEvent) -> Result<(), LogError>;
    fn flush(&self) -> Result<(), LogError> { Ok(()) }
}
