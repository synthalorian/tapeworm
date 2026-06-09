use crate::parser::{LogLevel, ParsedLine};
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::HashMap;

/// Time window size for bucketing log entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    OneHour,
}

impl TimeWindow {
    pub fn duration_seconds(&self) -> i64 {
        match self {
            TimeWindow::OneMinute => 60,
            TimeWindow::FiveMinutes => 300,
            TimeWindow::FifteenMinutes => 900,
            TimeWindow::OneHour => 3600,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TimeWindow::OneMinute => "1m",
            TimeWindow::FiveMinutes => "5m",
            TimeWindow::FifteenMinutes => "15m",
            TimeWindow::OneHour => "1h",
        }
    }
}

/// Try to parse a timestamp string into a DateTime<Utc>
/// Supports common log timestamp formats
pub fn parse_timestamp(ts: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339/ISO8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        return Some(dt.with_timezone(&Utc));
    }

    // Common log formats
    let formats = [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%:z",
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%d/%b/%Y:%H:%M:%S %z",    // nginx format: 15/Jan/2024:10:30:45 +0000
        "%b %d %H:%M:%S",          // syslog format: Jan 15 10:30:45
        "%b %d %Y %H:%M:%S",       // syslog with year
        "%Y-%m-%dT%H:%M:%S%.3fZ",  // common JSON format
    ];

    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(ts, format) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(ts, format) {
            return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
        }
    }

    None
}

/// Aggregation result for a collection of parsed lines
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AggregationResult {
    pub total_count: usize,
    pub by_level: HashMap<LogLevel, usize>,
    pub by_field: HashMap<String, HashMap<String, usize>>,
    pub time_buckets: HashMap<String, usize>,
}

/// Configuration for what fields to group by
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupByConfig {
    pub field_name: String,
    pub top_n: Option<usize>,
}

impl GroupByConfig {
    pub fn new(field_name: String) -> Self {
        Self {
            field_name,
            top_n: None,
        }
    }

    pub fn with_top_n(mut self, n: usize) -> Self {
        self.top_n = Some(n);
        self
    }
}

/// Engine that computes aggregations over parsed log lines
#[derive(Debug, Clone)]
pub struct AggregationEngine {
    pub time_window: TimeWindow,
    pub group_by_configs: Vec<GroupByConfig>,
}

impl Default for AggregationEngine {
    fn default() -> Self {
        Self {
            time_window: TimeWindow::OneMinute,
            group_by_configs: vec![],
        }
    }
}

impl AggregationEngine {
    pub fn new(time_window: TimeWindow) -> Self {
        Self {
            time_window,
            group_by_configs: vec![],
        }
    }

    pub fn with_group_by(mut self, configs: Vec<GroupByConfig>) -> Self {
        self.group_by_configs = configs;
        self
    }

    /// Add a group by configuration
    pub fn add_group_by(&mut self, config: GroupByConfig) {
        self.group_by_configs.push(config);
    }

    /// Remove all group by configurations
    pub fn clear_group_by(&mut self) {
        self.group_by_configs.clear();
    }

    /// Aggregate a collection of parsed lines
    pub fn aggregate(&self, lines: &[ParsedLine]) -> AggregationResult {
        let mut result = AggregationResult {
            total_count: lines.len(),
            ..Default::default()
        };

        for line in lines {
            // Count by level
            if let Some(level) = line.level {
                *result.by_level.entry(level).or_insert(0) += 1;
            }

            // Group by configured fields
            for config in &self.group_by_configs {
                if let Some(value) = line.fields.get(&config.field_name) {
                    let field_map = result
                        .by_field
                        .entry(config.field_name.clone())
                        .or_insert_with(HashMap::new);
                    *field_map.entry(value.clone()).or_insert(0) += 1;
                }
            }

            // Time bucketing
            if let Some(ref ts) = line.timestamp {
                if let Some(dt) = parse_timestamp(ts) {
                    let bucket = self.bucket_key(dt);
                    *result.time_buckets.entry(bucket).or_insert(0) += 1;
                }
            }
        }

        result
    }

    /// Compute bucket key for a given datetime
    fn bucket_key(&self, dt: DateTime<Utc>) -> String {
        let seconds = self.time_window.duration_seconds();
        let timestamp = dt.timestamp();
        let bucket_start = (timestamp / seconds) * seconds;
        let bucket_dt = DateTime::from_timestamp(bucket_start, 0).unwrap_or(dt);
        bucket_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Cycle to the next time window size
    pub fn cycle_time_window(&mut self) {
        self.time_window = match self.time_window {
            TimeWindow::OneMinute => TimeWindow::FiveMinutes,
            TimeWindow::FiveMinutes => TimeWindow::FifteenMinutes,
            TimeWindow::FifteenMinutes => TimeWindow::OneHour,
            TimeWindow::OneHour => TimeWindow::OneMinute,
        };
    }

    /// Get top N values for a grouped field, sorted by count descending
    pub fn top_values_for_field(
        &self,
        result: &AggregationResult,
        field_name: &str,
        n: usize,
    ) -> Vec<(String, usize)> {
        let mut values: Vec<(String, usize)> = result
            .by_field
            .get(field_name)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
            .unwrap_or_default();

        values.sort_by(|a, b| b.1.cmp(&a.1));
        values.truncate(n);
        values
    }

    /// Get time buckets sorted chronologically
    pub fn sorted_time_buckets(&self, result: &AggregationResult) -> Vec<(String, usize)> {
        let mut buckets: Vec<(String, usize)> = result
            .time_buckets
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        buckets.sort_by(|a, b| a.0.cmp(&b.0));
        buckets
    }

    /// Get level counts sorted by severity
    pub fn sorted_level_counts(&self, result: &AggregationResult) -> Vec<(LogLevel, usize)> {
        let severity_order = [
            LogLevel::Fatal,
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
            LogLevel::Trace,
            LogLevel::Unknown,
        ];

        severity_order
            .iter()
            .filter_map(|level| {
                result
                    .by_level
                    .get(level)
                    .map(|count| (*level, *count))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LogLevel;
    use chrono::{Datelike, Timelike};

    fn make_parsed_line(level: Option<LogLevel>, timestamp: Option<&str>, fields: Vec<(&str, &str)>) -> ParsedLine {
        ParsedLine {
            raw: "test line".to_string(),
            fields: fields.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            level,
            timestamp: timestamp.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_time_window_durations() {
        assert_eq!(TimeWindow::OneMinute.duration_seconds(), 60);
        assert_eq!(TimeWindow::FiveMinutes.duration_seconds(), 300);
        assert_eq!(TimeWindow::FifteenMinutes.duration_seconds(), 900);
        assert_eq!(TimeWindow::OneHour.duration_seconds(), 3600);
    }

    #[test]
    fn test_parse_timestamp_iso8601() {
        let ts = parse_timestamp("2024-01-15T10:30:45Z");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 10);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_parse_timestamp_nginx() {
        let ts = parse_timestamp("15/Jan/2024:10:30:45 +0000");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_parse_timestamp_syslog() {
        let _ = parse_timestamp("Jan 15 10:30:45");
    }

    #[test]
    fn test_parse_timestamp_invalid() {
        assert!(parse_timestamp("not a timestamp").is_none());
        assert!(parse_timestamp("").is_none());
    }

    #[test]
    fn test_aggregate_count() {
        let engine = AggregationEngine::new(TimeWindow::OneMinute);
        let lines = vec![
            make_parsed_line(Some(LogLevel::Error), None, vec![]),
            make_parsed_line(Some(LogLevel::Info), None, vec![]),
            make_parsed_line(Some(LogLevel::Error), None, vec![]),
        ];

        let result = engine.aggregate(&lines);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.by_level.get(&LogLevel::Error), Some(&2));
        assert_eq!(result.by_level.get(&LogLevel::Info), Some(&1));
    }

    #[test]
    fn test_aggregate_group_by() {
        let mut engine = AggregationEngine::new(TimeWindow::OneMinute);
        engine.add_group_by(GroupByConfig::new("status".to_string()));

        let lines = vec![
            make_parsed_line(Some(LogLevel::Error), None, vec![("status", "500")]),
            make_parsed_line(Some(LogLevel::Info), None, vec![("status", "200")]),
            make_parsed_line(Some(LogLevel::Error), None, vec![("status", "500")]),
            make_parsed_line(Some(LogLevel::Info), None, vec![("status", "404")]),
        ];

        let result = engine.aggregate(&lines);
        let status_counts = result.by_field.get("status").unwrap();
        assert_eq!(status_counts.get("500"), Some(&2));
        assert_eq!(status_counts.get("200"), Some(&1));
        assert_eq!(status_counts.get("404"), Some(&1));
    }

    #[test]
    fn test_aggregate_time_buckets() {
        let engine = AggregationEngine::new(TimeWindow::OneMinute);
        let lines = vec![
            make_parsed_line(None, Some("2024-01-15T10:30:45Z"), vec![]),
            make_parsed_line(None, Some("2024-01-15T10:30:50Z"), vec![]),
            make_parsed_line(None, Some("2024-01-15T10:31:15Z"), vec![]),
        ];

        let result = engine.aggregate(&lines);
        // Two lines in 10:30 bucket, one in 10:31 bucket
        assert_eq!(result.time_buckets.len(), 2);
        
        let buckets = engine.sorted_time_buckets(&result);
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].1, 2); // First bucket has 2 entries
        assert_eq!(buckets[1].1, 1); // Second bucket has 1 entry
    }

    #[test]
    fn test_aggregate_time_buckets_hour_window() {
        let engine = AggregationEngine::new(TimeWindow::OneHour);
        let lines = vec![
            make_parsed_line(None, Some("2024-01-15T10:30:45Z"), vec![]),
            make_parsed_line(None, Some("2024-01-15T10:45:00Z"), vec![]),
            make_parsed_line(None, Some("2024-01-15T11:15:00Z"), vec![]),
        ];

        let result = engine.aggregate(&lines);
        assert_eq!(result.time_buckets.len(), 2);
        
        let buckets = engine.sorted_time_buckets(&result);
        assert_eq!(buckets[0].1, 2); // 10:00 hour
        assert_eq!(buckets[1].1, 1); // 11:00 hour
    }

    #[test]
    fn test_cycle_time_window() {
        let mut engine = AggregationEngine::new(TimeWindow::OneMinute);
        assert_eq!(engine.time_window, TimeWindow::OneMinute);
        
        engine.cycle_time_window();
        assert_eq!(engine.time_window, TimeWindow::FiveMinutes);
        
        engine.cycle_time_window();
        assert_eq!(engine.time_window, TimeWindow::FifteenMinutes);
        
        engine.cycle_time_window();
        assert_eq!(engine.time_window, TimeWindow::OneHour);
        
        engine.cycle_time_window();
        assert_eq!(engine.time_window, TimeWindow::OneMinute);
    }

    #[test]
    fn test_top_values_for_field() {
        let mut engine = AggregationEngine::new(TimeWindow::OneMinute);
        engine.add_group_by(GroupByConfig::new("status".to_string()));

        let lines = vec![
            make_parsed_line(None, None, vec![("status", "500")]),
            make_parsed_line(None, None, vec![("status", "500")]),
            make_parsed_line(None, None, vec![("status", "500")]),
            make_parsed_line(None, None, vec![("status", "200")]),
            make_parsed_line(None, None, vec![("status", "200")]),
            make_parsed_line(None, None, vec![("status", "404")]),
        ];

        let result = engine.aggregate(&lines);
        let top = engine.top_values_for_field(&result, "status", 2);
        
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("500".to_string(), 3));
        assert_eq!(top[1], ("200".to_string(), 2));
    }

    #[test]
    fn test_sorted_level_counts() {
        let engine = AggregationEngine::new(TimeWindow::OneMinute);
        let lines = vec![
            make_parsed_line(Some(LogLevel::Info), None, vec![]),
            make_parsed_line(Some(LogLevel::Error), None, vec![]),
            make_parsed_line(Some(LogLevel::Warn), None, vec![]),
            make_parsed_line(Some(LogLevel::Error), None, vec![]),
        ];

        let result = engine.aggregate(&lines);
        let levels = engine.sorted_level_counts(&result);
        
        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].0, LogLevel::Error);
        assert_eq!(levels[0].1, 2);
        assert_eq!(levels[1].0, LogLevel::Warn);
        assert_eq!(levels[1].1, 1);
        assert_eq!(levels[2].0, LogLevel::Info);
        assert_eq!(levels[2].1, 1);
    }

    #[test]
    fn test_empty_lines() {
        let engine = AggregationEngine::new(TimeWindow::OneMinute);
        let result = engine.aggregate(&[]);
        
        assert_eq!(result.total_count, 0);
        assert!(result.by_level.is_empty());
        assert!(result.by_field.is_empty());
        assert!(result.time_buckets.is_empty());
    }

    #[test]
    fn test_multiple_group_by_fields() {
        let mut engine = AggregationEngine::new(TimeWindow::OneMinute);
        engine.add_group_by(GroupByConfig::new("status".to_string()));
        engine.add_group_by(GroupByConfig::new("method".to_string()));

        let lines = vec![
            make_parsed_line(None, None, vec![("status", "200"), ("method", "GET")]),
            make_parsed_line(None, None, vec![("status", "200"), ("method", "POST")]),
            make_parsed_line(None, None, vec![("status", "500"), ("method", "GET")]),
        ];

        let result = engine.aggregate(&lines);
        
        assert_eq!(result.by_field.len(), 2);
        assert!(result.by_field.contains_key("status"));
        assert!(result.by_field.contains_key("method"));
    }

    #[test]
    fn test_group_by_missing_field() {
        let mut engine = AggregationEngine::new(TimeWindow::OneMinute);
        engine.add_group_by(GroupByConfig::new("nonexistent".to_string()));

        let lines = vec![
            make_parsed_line(None, None, vec![("status", "200")]),
        ];

        let result = engine.aggregate(&lines);
        assert!(result.by_field.get("nonexistent").is_none());
    }

    #[test]
    fn test_timestamp_without_time_window() {
        // Lines without timestamps should still be counted in total
        let engine = AggregationEngine::new(TimeWindow::OneMinute);
        let lines = vec![
            make_parsed_line(Some(LogLevel::Error), None, vec![]),
            make_parsed_line(Some(LogLevel::Info), None, vec![]),
        ];

        let result = engine.aggregate(&lines);
        assert_eq!(result.total_count, 2);
        assert!(result.time_buckets.is_empty());
    }

    #[test]
    fn test_parse_timestamp_with_offset() {
        let ts = parse_timestamp("2024-01-15T10:30:45+05:30");
        assert!(ts.is_some());
        let dt = ts.unwrap();
        assert_eq!(dt.hour(), 5); // Converted to UTC: 10:30 +0530 -> 05:00 UTC
        assert_eq!(dt.minute(), 0);
    }
}
