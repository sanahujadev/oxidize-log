/// Macro para registrar mensajes en nivel TRACE.
#[macro_export]
macro_rules! trace {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Trace,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}

/// Macro para registrar mensajes en nivel DEBUG.
#[macro_export]
macro_rules! debug {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Debug,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}

/// Macro para registrar mensajes en nivel INFO.
#[macro_export]
macro_rules! info {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Info,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}

/// Macro para registrar mensajes en nivel WARN.
#[macro_export]
macro_rules! warn {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Warn,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}

/// Macro para registrar mensajes en nivel ERROR.
#[macro_export]
macro_rules! error {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Error,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}

/// Macro para registrar mensajes en nivel FATAL.
#[macro_export]
macro_rules! fatal {
    ($logger:expr, $($arg:tt)+) => {
        $logger.log(
            $crate::LogLevel::Fatal,
            $crate::Metadata::new(
                module_path!(),
                file!(),
                line!(),
            ),
            || ::std::format!($($arg)+),
        )
    };
}
