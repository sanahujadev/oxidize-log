#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    
    fn levels_are_ordered() {
        assert!((LogLevel::Trace as u8) < (LogLevel::Debug as u8));
        assert!((LogLevel::Debug as u8) < (LogLevel::Info as u8));
        assert!((LogLevel::Info as u8) < (LogLevel::Warn as u8));
        assert!((LogLevel::Warn as u8) < (LogLevel::Error as u8));
        assert!((LogLevel::Error as u8) < (LogLevel::Fatal as u8));
    } 
}