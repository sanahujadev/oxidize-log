use std::fmt;

#[derive(Debug)]
pub enum LogError {
    InvalidLevel(String),
    InvalidMetadata,
    Write { sink: &'static str, source: Box<dyn std::error::Error + Send + Sync> },
    Format { formatter: &'static str, source: Box<dyn std::error::Error + Send + Sync> },
    Config(&'static str),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLevel(s)      => write!(f, "invalid log level: {s}"),
            Self::InvalidMetadata      => write!(f, "invalid log metadata"),
            Self::Write { sink, source } => write!(f, "write error in sink {sink}: {source}"),
            Self::Format { formatter, source } => write!(f, "format error in {formatter}: {source}"),
            Self::Config(msg)          => write!(f, "logger configuration error: {msg}"),
        }
    }
}

impl std::error::Error for LogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Write { source, .. } | Self::Format { source, .. } => {
                Some(&**source as &(dyn std::error::Error + 'static))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn logerror_es_representable_y_no_panicea() {
        let err1 = LogError::InvalidLevel("FOO".into());
        assert_eq!(err1.to_string(), "invalid log level: FOO");
        assert!(std::error::Error::source(&err1).is_none());

        let io_err = std::io::Error::other("io failed");
        let err2 = LogError::Write { sink: "MockSink", source: Box::new(io_err) };
        assert_eq!(err2.to_string(), "write error in sink MockSink: io failed");
        assert!(std::error::Error::source(&err2).is_some());

        let fmt_err = std::fmt::Error;
        let err3 = LogError::Format { formatter: "MockFormatter", source: Box::new(fmt_err) };
        assert_eq!(err3.to_string(), "format error in MockFormatter: an error occurred when formatting an argument");
        assert!(std::error::Error::source(&err3).is_some());
    }
}
