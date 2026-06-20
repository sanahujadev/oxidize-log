#![deny(warnings)]
pub mod domain;
pub mod ports;
pub mod adapters;
pub mod app;

pub use domain::{LogLevel, LogEvent, Metadata, LogError};
pub use ports::{Sink, Formatter, Filter};
pub use adapters::{ConsoleSink, SimpleTextFormatter};
pub use app::{Logger, LoggerBuilder, LoggerConfig, Environment, SinkConfig, LevelFilter};
