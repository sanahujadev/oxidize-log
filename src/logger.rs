use crate::LogLevel;

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    pub fn init_default() -> Self {
        Self::new(LogLevel::Info)
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if (level as u8) < (self.level as u8) {
            return;
        }

        // Implementación mínima por ahora
        println!("[{:?}] {}", level, message);
    }
}
