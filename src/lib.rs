//! oxidize-log
//!
//! Core de logging multiplataforma escrito en Rust.
//! Este módulo inicial define la estructura base del proyecto.

pub mod level;
pub mod logger;
pub mod config;
pub mod sink;

pub use level::LogLevel;
pub use logger::Logger;
pub use config::{Environment, LoggerConfig, SinkConfig};
pub use sink::{Sink, ConsoleSink, build_sinks};
