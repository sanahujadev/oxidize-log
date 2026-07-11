use oxidize_log::{
    debug, error, fatal, info, trace, warn, ConsoleSink, JsonFormatter, Logger, LoggerBuilder,
    LogLevel,
};
use std::sync::Arc;

fn main() {
    // 1. Demostración del Logger por defecto con COLORES y MACROS
    let logger = Logger::default();

    println!("--- Salida con Colores (SimpleTextFormatter) ---");
    trace!(&logger, "Mensaje de rastro oculto (no se verá porque el default es INFO)");
    debug!(&logger, "Mensaje de debug oculto");
    info!(&logger, "¡Hola desde oxidize-log! El nivel INFO se ve verde");
    warn!(&logger, "Cuidado, esto es una advertencia (amarillo)");
    error!(&logger, "Ocurrió un error grave (rojo)");
    fatal!(&logger, "Falla crítica en el sistema (rojo sobre blanco)");

    // 2. Demostración del Formateador JSON
    let json_sink = Arc::new(ConsoleSink::new(Arc::new(JsonFormatter::default())));
    let json_logger = LoggerBuilder::new()
        .level(LogLevel::Debug) // Bajamos el nivel para ver el debug
        .sink(json_sink)
        .build();

    println!("\n--- Salida con Formato JSON (RFC 8259 estricto) ---");
    debug!(&json_logger, "Procesando request #12345");
    info!(&json_logger, "El usuario {} se ha logueado exitosamente", "zitrojj");
    error!(&json_logger, "JSON escapa caracteres raros: comillas \", saltos \n y nulos \x00");
}
