use oxidize_log::{Logger, LoggerConfig, LogLevel, SinkConfig};

fn main() {
    // Inicializar el logger global usando Logger::default() como se pide en el refactor
    let logger = Logger::default();

    // Probar varios logs usando los helpers
    logger.trace(|| "Trace message".to_string());
    logger.debug(|| "Debug message".to_string());
    logger.info(|| "Hola desde oxidize-log inicializado de forma manual".to_string());
    logger.warn(|| "Warning message".to_string());
    logger.error(|| "Error message".to_string());
    logger.fatal(|| "Fatal message".to_string());

    // Probando inicialización de config manual preservada
    let config = LoggerConfig {
        level: LogLevel::Debug,
        colors: true,
        sinks: vec![SinkConfig::Console],
    };

    let logger_config = Logger::init(config);
    logger_config.debug(|| "Hola desde logger_config".to_string());
}
