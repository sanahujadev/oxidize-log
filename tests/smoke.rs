use oxidize_log::{LoggerBuilder, ConsoleSink, SimpleTextFormatter, JsonFormatter};
use std::sync::{Arc, Mutex};
use std::io::Write;
use std::thread;

// Helper wrapper so we can inspect what was written in test
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
fn smoke_test_default_no_panica_con_multiples_logs() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(SimpleTextFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new().sink(sink).build();

    // We launch a few threads to ensure Send + Sync functionality and no panics
    let mut handles = vec![];
    for i in 0..4 {
        let l = logger.clone();
        handles.push(thread::spawn(move || {
            l.trace(|| format!("trace {}", i));
            l.debug(|| format!("debug {}", i));
            l.info(|| format!("info {}", i));
            l.warn(|| format!("warn {}", i));
            l.error(|| format!("error {}", i));
            l.fatal(|| format!("fatal {}", i));
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // As in R34, by default (Info level), trace and debug shouldn't be printed
    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    // 4 threads × 4 niveles (info/warn/error/fatal) = 16 líneas que pasan el filtro
    // 4 threads × 2 niveles (trace/debug) = 8 líneas filtradas por LevelFilter::Info
    assert_eq!(output.matches("[INFO]").count(),  4);
    assert_eq!(output.matches("[WARN]").count(),  4);
    assert_eq!(output.matches("[ERROR]").count(), 4);
    assert_eq!(output.matches("[FATAL]").count(), 4);
    assert_eq!(output.matches("[TRACE]").count(), 0);
    assert_eq!(output.matches("[DEBUG]").count(), 0);

    // Cada nivel aparece 4 veces (una por thread)
    assert_eq!(output.matches("info 0").count() + output.matches("info 1").count()
             + output.matches("info 2").count() + output.matches("info 3").count(), 4);
    assert_eq!(output.matches("warn 0").count() + output.matches("warn 1").count()
             + output.matches("warn 2").count() + output.matches("warn 3").count(), 4);
    assert_eq!(output.matches("error 0").count() + output.matches("error 1").count()
             + output.matches("error 2").count() + output.matches("error 3").count(), 4);
    assert_eq!(output.matches("fatal 0").count() + output.matches("fatal 1").count()
             + output.matches("fatal 2").count() + output.matches("fatal 3").count(), 4);
}

#[test]
fn smoke_color_and_json_combined() {
    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new().sink(sink).build();
    logger.info(|| "integration test message".to_string());

    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    let parsed: serde_json::Value = serde_json::from_str(output).expect("Must be valid JSON");
    assert!(parsed.is_object());
    assert_eq!(parsed.get("message").and_then(|v| v.as_str()), Some("integration test message"));
}

#[test]
fn smoke_macros_capture_real_metadata() {
    use oxidize_log::info;
    use oxidize_log::LogLevel;

    let written_data = Arc::new(Mutex::new(Vec::new()));
    let writer = Arc::new(Mutex::new(Box::new(VecWriter { data: written_data.clone() }) as Box<dyn Write + Send + Sync>));
    let formatter = Arc::new(JsonFormatter::new());
    let sink = Arc::new(ConsoleSink::with_writer(formatter, writer));

    let logger = LoggerBuilder::new()
        .level(LogLevel::Info)
        .sink(sink)
        .build();

    info!(&logger, "smoke test message");

    let data = written_data.lock().expect("Mutex poisoned");
    let output = std::str::from_utf8(&data).expect("UTF-8 válido");

    let parsed: serde_json::Value = serde_json::from_str(output).expect("Must be valid JSON");
    assert_eq!(parsed.get("module").and_then(|v| v.as_str()), Some("smoke"));
    assert_eq!(parsed.get("file").and_then(|v| v.as_str()), Some("tests/smoke.rs"));
}
