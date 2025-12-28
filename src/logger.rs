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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn logger_filters_lower_levels() {
        let logger = Logger::new(LogLevel::Warn);
        // No imprime nada, pero podemos testear la lógica interna 
        assert!((LogLevel::Info as u8) < (logger.level as u8));
    }
}