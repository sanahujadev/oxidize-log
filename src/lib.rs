//! oxidize-log
//!
//! Core de logging multiplataforma escrito en Rust.
//! Este módulo inicial define la estructura base del proyecto.

pub mod level;
pub mod logger;

pub use level::LogLevel;
pub use logger::Logger;
