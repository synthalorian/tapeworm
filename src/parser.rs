use crate::profile::LogProfile;
use regex::Regex;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Regex compilation error: {0}")]
    RegexError(#[from] regex::Error),
}

/// Standard log levels for color-coding and filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

impl LogLevel {
    /// Parse a log level from a string
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "trace" | "trc" => LogLevel::Trace,
            "debug" | "dbg" => LogLevel::Debug,
            "info" | "inf" | "information" => LogLevel::Info,
            "warn" | "warning" | "wrn" => LogLevel::Warn,
            "error" | "err" => LogLevel::Error,
            "fatal" | "crit" | "critical" | "emerg" | "emergency" => LogLevel::Fatal,
            _ => LogLevel::Unknown,
        }
    }

    /// Detect log level from a message string by looking for level keywords
    pub fn detect_in_message(msg: &str) -> Self {
        let upper = msg.to_uppercase();
        
        // Check for explicit level markers in order of severity
        if upper.contains("FATAL") || upper.contains("CRITICAL") || upper.contains("EMERG") {
            LogLevel::Fatal
        } else if upper.contains("ERROR") || upper.contains(" ERR ") {
            LogLevel::Error
        } else if upper.contains("WARN") || upper.contains("WARNING") {
            LogLevel::Warn
        } else if upper.contains("INFO") || upper.contains(" INF ") {
            LogLevel::Info
        } else if upper.contains("DEBUG") || upper.contains("DBG") {
            LogLevel::Debug
        } else if upper.contains("TRACE") || upper.contains("TRC") {
            LogLevel::Trace
        } else {
            LogLevel::Unknown
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Fatal => "FATAL",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

/// A single parsed log line with extracted fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine {
    pub raw: String,
    pub fields: HashMap<String, String>,
    pub level: Option<LogLevel>,
    pub timestamp: Option<String>,
}

impl ParsedLine {
    /// Create an unparsed line (fallback when no profile matches)
    pub fn unparsed(raw: String) -> Self {
        let level = LogLevel::detect_in_message(&raw);
        let level = if level == LogLevel::Unknown { None } else { Some(level) };
        Self {
            raw,
            fields: HashMap::new(),
            level,
            timestamp: None,
        }
    }
}

/// Log parser that uses regex profiles to extract structured data from log lines
#[derive(Debug)]
pub struct LogParser {
    profile: LogProfile,
    regex: Regex,
}

impl LogParser {
    /// Create a new parser from a profile
    pub fn new(profile: LogProfile) -> Result<Self, ParseError> {
        let regex = Regex::new(&profile.regex)?;
        Ok(Self { profile, regex })
    }

    /// Parse a single log line
    pub fn parse_line(&self, line: &str) -> ParsedLine {
        if let Some(captures) = self.regex.captures(line) {
            let mut fields = HashMap::new();
            
            for field_def in &self.profile.fields {
                if let Some(matched) = captures.name(&field_def.name) {
                    fields.insert(field_def.name.clone(), matched.as_str().to_string());
                }
            }

            // Extract level if configured
            let level = self.profile.level_field.as_ref()
                .and_then(|field_name| fields.get(field_name))
                .map(|value| LogLevel::from_str(value));

            // Extract timestamp if configured
            let timestamp = self.profile.timestamp_field.as_ref()
                .and_then(|field_name| fields.get(field_name))
                .cloned();

            ParsedLine {
                raw: line.to_string(),
                fields,
                level,
                timestamp,
            }
        } else {
            ParsedLine::unparsed(line.to_string())
        }
    }

    /// Parse multiple lines
    pub fn parse_lines(&self, lines: &[String]) -> Vec<ParsedLine> {
        lines.iter().map(|line| self.parse_line(line)).collect()
    }

    /// Get the profile name
    #[allow(dead_code)]
    pub fn profile_name(&self) -> &str {
        &self.profile.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{FieldDef, LogProfile};

    fn make_test_profile() -> LogProfile {
        LogProfile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            regex: r"^(?P<timestamp>\d{4}-\d{2}-\d{2})\s+(?P<level>\w+)\s+(?P<message>.+)$".to_string(),
            fields: vec![
                FieldDef { name: "timestamp".to_string(), field_type: "string".to_string() },
                FieldDef { name: "level".to_string(), field_type: "string".to_string() },
                FieldDef { name: "message".to_string(), field_type: "string".to_string() },
            ],
            level_field: Some("level".to_string()),
            timestamp_field: Some("timestamp".to_string()),
        }
    }

    #[test]
    fn test_parse_line_success() {
        let profile = make_test_profile();
        let parser = LogParser::new(profile).unwrap();
        
        let line = "2024-01-15 ERROR Something went wrong";
        let parsed = parser.parse_line(line);
        
        assert_eq!(parsed.raw, line);
        assert_eq!(parsed.fields.get("timestamp"), Some(&"2024-01-15".to_string()));
        assert_eq!(parsed.fields.get("level"), Some(&"ERROR".to_string()));
        assert_eq!(parsed.fields.get("message"), Some(&"Something went wrong".to_string()));
        assert_eq!(parsed.level, Some(LogLevel::Error));
        assert_eq!(parsed.timestamp, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_parse_line_no_match() {
        let profile = make_test_profile();
        let parser = LogParser::new(profile).unwrap();
        
        let line = "This is just a plain text line without structure";
        let parsed = parser.parse_line(line);
        
        assert_eq!(parsed.raw, line);
        assert!(parsed.fields.is_empty());
        assert!(parsed.level.is_none());
        assert!(parsed.timestamp.is_none());
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("error"), LogLevel::Error);
        assert_eq!(LogLevel::from_str("WARN"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("warning"), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("INFO"), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG"), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("TRACE"), LogLevel::Trace);
        assert_eq!(LogLevel::from_str("FATAL"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("critical"), LogLevel::Fatal);
        assert_eq!(LogLevel::from_str("unknown"), LogLevel::Unknown);
    }

    #[test]
    fn test_log_level_detect_in_message() {
        assert_eq!(LogLevel::detect_in_message("ERROR: failed to connect"), LogLevel::Error);
        assert_eq!(LogLevel::detect_in_message("WARN: low disk space"), LogLevel::Warn);
        assert_eq!(LogLevel::detect_in_message("INFO: server started"), LogLevel::Info);
        assert_eq!(LogLevel::detect_in_message("DEBUG: processing request"), LogLevel::Debug);
        assert_eq!(LogLevel::detect_in_message("TRACE: entering function"), LogLevel::Trace);
        assert_eq!(LogLevel::detect_in_message("FATAL: out of memory"), LogLevel::Fatal);
        assert_eq!(LogLevel::detect_in_message("just a normal message"), LogLevel::Unknown);
    }

    #[test]
    fn test_parse_syslog_line() {
        let profile = LogProfile::syslog();
        let parser = LogParser::new(profile).unwrap();
        
        let line = "Jan 15 10:30:45 myhost kernel: [12345.678901] USB device connected";
        let parsed = parser.parse_line(line);
        
        assert_eq!(parsed.fields.get("timestamp"), Some(&"Jan 15 10:30:45".to_string()));
        assert_eq!(parsed.fields.get("host"), Some(&"myhost".to_string()));
        assert_eq!(parsed.fields.get("process"), Some(&"kernel".to_string()));
        assert_eq!(parsed.fields.get("message"), Some(&"[12345.678901] USB device connected".to_string()));
    }

    #[test]
    fn test_parse_nginx_line() {
        let profile = LogProfile::nginx();
        let parser = LogParser::new(profile).unwrap();
        
        let line = r#"192.168.1.1 - - [15/Jan/2024:10:30:45 +0000] "GET /index.html HTTP/1.1" 200 1234 "-" "Mozilla/5.0""#;
        let parsed = parser.parse_line(line);
        
        assert_eq!(parsed.fields.get("remote_addr"), Some(&"192.168.1.1".to_string()));
        assert_eq!(parsed.fields.get("status"), Some(&"200".to_string()));
        assert_eq!(parsed.fields.get("request"), Some(&"GET /index.html HTTP/1.1".to_string()));
    }

    #[test]
    fn test_parse_lines_batch() {
        let profile = make_test_profile();
        let parser = LogParser::new(profile).unwrap();
        
        let lines = vec![
            "2024-01-15 ERROR Something went wrong".to_string(),
            "2024-01-15 INFO Server started".to_string(),
            "unstructured line".to_string(),
        ];
        
        let parsed = parser.parse_lines(&lines);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].level, Some(LogLevel::Error));
        assert_eq!(parsed[1].level, Some(LogLevel::Info));
        assert_eq!(parsed[2].level, None);
    }

    #[test]
    fn test_invalid_regex() {
        let profile = LogProfile {
            name: "bad".to_string(),
            description: "Bad regex".to_string(),
            regex: r"(?P<unclosed>[\d+".to_string(),
            fields: vec![],
            level_field: None,
            timestamp_field: None,
        };
        
        let result = LogParser::new(profile);
        assert!(result.is_err());
    }
}
