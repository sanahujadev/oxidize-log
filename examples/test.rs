use oxidize_log::{Logger, LoggerConfig, LogLevel, SinkConfig};

fn main() {
    // Configuración manual
    let config = LoggerConfig {
        level: LogLevel::Debug,
        colors: true,
        sinks: vec![SinkConfig::Console],
    };

    // Inicializar el logger con la configuración
    let logger = Logger::init(config);

    // Probar un log
    logger.log(LogLevel::Info, "Hola desde el ejemplo con config manual");
}
