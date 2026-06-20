use oxidize_log::Logger;
use std::thread;

#[test]
fn smoke_test_default_no_panica_con_multiples_logs() {
    let logger = Logger::default();

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
}
