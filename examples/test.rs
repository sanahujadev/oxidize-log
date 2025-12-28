use oxidize_log::{Logger, LogLevel};

fn main() {
    let logger = Logger::init_default();
    logger.log(LogLevel::Info, "Hola desde el ejemplo");
}
