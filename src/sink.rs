use std::any::Any;
use crate::{LogLevel, SinkConfig};

pub trait Sink {
    fn log(&self, level: LogLevel, message: &str);
    fn as_any(&self) -> &dyn Any;
}

pub struct ConsoleSink;

impl ConsoleSink {
    pub fn new() -> Self {
        Self
    }
}

impl Sink for ConsoleSink {
    fn log(&self, level: LogLevel, message: &str) {
        println!("[{:?}] {}", level, message);
    }

    fn as_any(&self) -> &dyn Any { self }
}

pub fn build_sinks(configs: &[SinkConfig]) -> Vec<Box<dyn Sink>> {
    let mut sinks: Vec<Box<dyn Sink>> = Vec::new();

    for cfg in configs {
        match cfg {
            SinkConfig::Console => sinks.push(Box::new(ConsoleSink::new())),
        }
    }

    sinks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LogLevel;

    #[test]
    fn console_sink_prints() {
        let sink = ConsoleSink::new();
        sink.log(LogLevel::Info, "test message");
        // No assertion: just ensures no panic
    }

    #[test]
    fn build_sinks_creates_console_sink() {
        let sinks = build_sinks(&[SinkConfig::Console]);
        assert_eq!(sinks.len(), 1);
    }
}
