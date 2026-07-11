use oxidize_log::{trace, info, error, LoggerBuilder, JsonFormatter, ConsoleSink, LogLevel};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::Write;

#[derive(Clone)]
struct VecWriter {
    data: Arc<Mutex<Vec<u8>>>,
}

impl Write for VecWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.data.lock().unwrap().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn trace_macro_captures_file_line_module() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new()
        .level(LogLevel::Trace)
        .sink(sink)
        .build();

    let line_num = line!() + 1;
    trace!(&logger, "test");

    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    let parsed: serde_json::Value = serde_json::from_str(output).expect("Must be valid JSON");
    
    assert_eq!(parsed.get("module").and_then(|v| v.as_str()), Some("macros_test"));
    assert_eq!(parsed.get("file").and_then(|v| v.as_str()), Some("tests/macros_test.rs"));
    assert_eq!(parsed.get("line").and_then(|v| v.as_u64()), Some(line_num as u64));
}

#[test]
fn info_macro_skips_evaluation_when_filtered() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new()
        .level(LogLevel::Warn)
        .sink(sink)
        .build();

    let el_booleano = Arc::new(AtomicBool::new(false));
    let el_booleano_clone = el_booleano.clone();

    info!(&logger, "test {}", { el_booleano_clone.store(true, Ordering::SeqCst); 5 });

    assert!(!el_booleano.load(Ordering::SeqCst));
}

#[test]
fn error_macro_does_not_panic_on_complex_args() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new()
        .sink(sink)
        .build();

    error!(&logger, "A: {}, B: {:?}, C: {:#?}", 1, "dos", vec![3, 4]);

    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    let parsed: serde_json::Value = serde_json::from_str(output).expect("Must be valid JSON");
    assert_eq!(
        parsed.get("message").and_then(|v| v.as_str()),
        Some("A: 1, B: \"dos\", C: [\n    3,\n    4,\n]")
    );
}

#[test]
fn macros_coexist_with_helper_methods() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new()
        .sink(sink)
        .build();

    logger.info(|| "viejo".to_string());
    info!(&logger, "nuevo");

    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    let lines: Vec<&str> = output.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 2);

    let parsed_viejo: serde_json::Value = serde_json::from_str(lines[0]).expect("Must be valid JSON");
    let parsed_nuevo: serde_json::Value = serde_json::from_str(lines[1]).expect("Must be valid JSON");

    assert_eq!(parsed_viejo.get("module").and_then(|v| v.as_str()), Some("<unknown>"));
    assert_eq!(parsed_viejo.get("file").and_then(|v| v.as_str()), Some("<unknown>"));

    assert_eq!(parsed_nuevo.get("module").and_then(|v| v.as_str()), Some("macros_test"));
    assert_eq!(parsed_nuevo.get("file").and_then(|v| v.as_str()), Some("tests/macros_test.rs"));
}
