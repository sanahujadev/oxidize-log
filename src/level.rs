#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        assert!((LogLevel::Debug as u8) < (LogLevel::Info as u8));
        assert!((LogLevel::Info as u8) < (LogLevel::Warn as u8));
    } 
}