use crate::domain::event::LogEvent;
use crate::domain::error::LogError;
use crate::ports::formatter::Formatter;
use std::time::{SystemTime, Duration};

pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }

    pub(crate) fn format_iso8601_from_duration(dur: Duration) -> String {
        let secs = dur.as_secs();
        let days_since_1970 = secs / 86400;

        let z = (days_since_1970 as i64) + 719468;
        let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
        let doe = (z - era * 146097) as u32;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = (yoe as i32) + (era as i32) * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if m <= 2 { y + 1 } else { y };

        let secs_in_day = secs % 86400;
        let hour = secs_in_day / 3600;
        let minute = (secs_in_day % 3600) / 60;
        let second = secs_in_day % 60;
        let millis = dur.subsec_millis();

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            y, m, d, hour, minute, second, millis
        )
    }

    pub(crate) fn iso8601_utc_now_from_time(time: SystemTime) -> Result<String, LogError> {
        let dur = time.duration_since(SystemTime::UNIX_EPOCH).map_err(|e| {
            LogError::Format {
                formatter: "JsonFormatter",
                source: Box::new(e),
            }
        })?;
        Ok(Self::format_iso8601_from_duration(dur))
    }

    pub(crate) fn iso8601_utc_now() -> Result<String, LogError> {
        Self::iso8601_utc_now_from_time(SystemTime::now())
    }

    pub(crate) fn escape_json(s: &str) -> String {
        let mut escaped = String::with_capacity(s.len() + 10);
        for c in s.chars() {
            match c {
                '"' => escaped.push_str("\\\""),
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\r' => escaped.push_str("\\r"),
                '\t' => escaped.push_str("\\t"),
                '\x08' => escaped.push_str("\\b"),
                '\x0c' => escaped.push_str("\\f"),
                c if (c as u32) <= 0x1f => {
                    use std::fmt::Write as _;
                    let _ = write!(escaped, "\\u{:04x}", c as u32);
                }
                _ => escaped.push(c),
            }
        }
        escaped
    }

    fn write_kv_u32(buf: &mut String, key: &str, val: u32) {
        buf.push('"');
        buf.push_str(key);
        buf.push_str("\":");
        // TODO(R35): to_string() can be optimized in the future.
        buf.push_str(&val.to_string());
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, event: &LogEvent) -> Result<String, LogError> {
        let ts = Self::iso8601_utc_now()?;

        let mut buf = String::with_capacity(160);
        buf.push('{');

        // "timestamp"
        buf.push_str("\"timestamp\":\"");
        buf.push_str(&ts);
        buf.push_str("\",");

        // "level"
        buf.push_str("\"level\":\"");
        buf.push_str(event.level.as_str());
        buf.push_str("\",");

        // "message"
        buf.push_str("\"message\":\"");
        buf.push_str(&Self::escape_json(&event.message));
        buf.push_str("\",");

        // "module"
        buf.push_str("\"module\":\"");
        buf.push_str(&Self::escape_json(event.metadata.module));
        buf.push_str("\",");

        // "file"
        buf.push_str("\"file\":\"");
        buf.push_str(&Self::escape_json(event.metadata.file));
        buf.push_str("\",");

        // "line"
        Self::write_kv_u32(&mut buf, "line", event.metadata.line);

        buf.push_str("}\n");
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::expect_used)]
    use super::*;
    use crate::domain::level::LogLevel;
    use crate::domain::event::Metadata;

    #[test]
    fn json_formatter_produces_valid_json_object() {
        let formatter = JsonFormatter::new();
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::new("my_module", "file.rs", 42),
        };
        let result = formatter.format(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn json_formatter_includes_all_required_fields_in_stable_order() {
        let formatter = JsonFormatter::new();
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::new("my_module", "file.rs", 42),
        };
        let result = formatter.format(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = parsed.as_object().unwrap();
        let keys: Vec<&str> = obj.keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["timestamp", "level", "message", "module", "file", "line"]);
    }

    #[test]
    fn json_formatter_iso8601_utc_timestamp_format() {
        let formatter = JsonFormatter::new();
        let event = LogEvent {
            level: LogLevel::Info,
            message: "Hello world".to_string(),
            metadata: Metadata::new("my_module", "file.rs", 42),
        };
        let result = formatter.format(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let ts = parsed.get("timestamp").unwrap().as_str().unwrap();

        assert_eq!(ts.len(), 24);
        assert_eq!(&ts[4..5], "-");
        assert_eq!(&ts[7..8], "-");
        assert_eq!(&ts[10..11], "T");
        assert_eq!(&ts[13..14], ":");
        assert_eq!(&ts[16..17], ":");
        assert_eq!(&ts[19..20], ".");
        assert_eq!(&ts[23..24], "Z");

        assert!(ts[0..4].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[5..7].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[8..10].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[11..13].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[14..16].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[17..19].chars().all(|c| c.is_ascii_digit()));
        assert!(ts[20..23].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn json_formatter_escapes_all_control_chars_rfc8259() {
        let input = "quote: \", backslash: \\, tab: \t, newline: \n, carriage: \r, backspace: \x08, formfeed: \x0c, null: \x00, control: \x1f";
        let escaped = JsonFormatter::escape_json(input);

        assert!(escaped.contains("\\\""));
        assert!(escaped.contains("\\\\"));
        assert!(escaped.contains("\\t"));
        assert!(escaped.contains("\\n"));
        assert!(escaped.contains("\\r"));
        assert!(escaped.contains("\\b"));
        assert!(escaped.contains("\\f"));
        assert!(escaped.contains("\\u0000"));
        assert!(escaped.contains("\\u001f"));
    }

    #[test]
    fn format_iso8601_from_duration_formats_correctly() {
        let ts_epoch = JsonFormatter::format_iso8601_from_duration(Duration::from_secs(0));
        assert_eq!(ts_epoch, "1970-01-01T00:00:00.000Z");

        let ts_known = JsonFormatter::format_iso8601_from_duration(Duration::new(1710000000, 123_000_000));
        assert_eq!(ts_known, "2024-03-09T16:00:00.123Z");
    }

    #[test]
    fn json_formatter_propagates_clock_error() {
        let before_epoch = SystemTime::UNIX_EPOCH - Duration::from_secs(10);
        let result = JsonFormatter::iso8601_utc_now_from_time(before_epoch);
        assert!(result.is_err());
        match result {
            Err(LogError::Format { formatter, .. }) => {
                assert_eq!(formatter, "JsonFormatter");
            }
            _ => panic!("Expected LogError::Format"),
        }
    }
}
