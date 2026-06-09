use crate::aggregate::AggregationResult;
use crate::parser::LogLevel;
use std::collections::HashMap;

/// Severity of an anomaly
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AnomalySeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalySeverity::Low => "LOW",
            AnomalySeverity::Medium => "MED",
            AnomalySeverity::High => "HIGH",
            AnomalySeverity::Critical => "CRIT",
        }
    }
}

/// Type of anomaly detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    /// Log rate significantly above baseline
    RateSpike,
    /// Error/fatal level count significantly above baseline
    ErrorSpike,
    /// New pattern (field value) seen for the first time
    NewPattern,
    /// Distribution of log levels changed significantly
    LevelShift,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalyType::RateSpike => "RATE SPIKE",
            AnomalyType::ErrorSpike => "ERROR SPIKE",
            AnomalyType::NewPattern => "NEW PATTERN",
            AnomalyType::LevelShift => "LEVEL SHIFT",
        }
    }
}

/// A single detected anomaly
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub bucket: String,
    pub description: String,
    pub severity: AnomalySeverity,
    pub value: usize,
    pub expected: usize,
}

/// Historical data for a single time bucket
#[derive(Debug, Clone, Default)]
pub struct BucketHistory {
    pub bucket: String,
    pub count: usize,
    pub error_count: usize,
    pub level_counts: HashMap<LogLevel, usize>,
    pub field_values: HashMap<String, HashMap<String, usize>>,
}

/// Configuration for anomaly detection
#[derive(Debug, Clone)]
pub struct AnomalyConfig {
    /// Number of standard deviations above mean to trigger rate spike
    pub rate_spike_threshold: f64,
    /// Number of standard deviations above mean to trigger error spike
    pub error_spike_threshold: f64,
    /// Minimum number of historical buckets before detecting anomalies
    pub min_baseline_buckets: usize,
    /// Maximum history size (number of buckets to keep)
    pub max_history_size: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            rate_spike_threshold: 2.5,
            error_spike_threshold: 2.0,
            min_baseline_buckets: 3,
            max_history_size: 30,
        }
    }
}

/// Engine for detecting anomalies in log aggregation data
#[derive(Debug, Clone, Default)]
pub struct AnomalyEngine {
    pub config: AnomalyConfig,
    pub history: Vec<BucketHistory>,
}

impl AnomalyEngine {
    pub fn new(config: AnomalyConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
        }
    }

    /// Detect anomalies from the current aggregation result
    pub fn detect(&mut self, aggregation: &AggregationResult) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Build current bucket state from aggregation
        let current = self.build_current_history(aggregation);

        // Detect rate spikes for each time bucket
        for (bucket, count) in &aggregation.time_buckets {
            if let Some(anomaly) = self.check_rate_spike(bucket, *count) {
                anomalies.push(anomaly);
            }
        }

        // Detect error spikes
        let current_error_count: usize = aggregation
            .by_level
            .iter()
            .filter(|(level, _)| matches!(level, LogLevel::Error | LogLevel::Fatal))
            .map(|(_, count)| *count)
            .sum();

        if let Some(anomaly) = self.check_error_spike(&current.bucket, current_error_count) {
            anomalies.push(anomaly);
        }

        // Detect new patterns
        for anomaly in self.detect_new_patterns(&current) {
            anomalies.push(anomaly);
        }

        // Detect level shifts
        if let Some(anomaly) = self.detect_level_shift(&current) {
            anomalies.push(anomaly);
        }

        // Update history
        self.update_history(current);

        anomalies
    }

    fn build_current_history(&self, aggregation: &AggregationResult) -> BucketHistory {
        let mut history = BucketHistory::default();
        // Use the latest bucket as current, or default
        if let Some((latest_bucket, count)) =
            aggregation.time_buckets.iter().max_by_key(|(k, _)| *k)
        {
            history.bucket = latest_bucket.clone();
            history.count = *count;
        }
        history.level_counts = aggregation.by_level.clone();
        history.error_count = aggregation
            .by_level
            .iter()
            .filter(|(level, _)| matches!(level, LogLevel::Error | LogLevel::Fatal))
            .map(|(_, count)| *count)
            .sum();

        // Store field values for pattern detection
        history.field_values = aggregation.by_field.clone();

        history
    }

    fn check_rate_spike(&self, bucket: &str, count: usize) -> Option<Anomaly> {
        if self.history.len() < self.config.min_baseline_buckets {
            return None;
        }

        let counts: Vec<f64> = self.history.iter().map(|h| h.count as f64).collect();
        let mean = counts.iter().sum::<f64>() / counts.len() as f64;
        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / counts.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            // No variance - only flag if count is significantly higher than mean
            if count as f64 > mean * self.config.rate_spike_threshold && mean > 0.0 {
                let severity = self.rate_severity(count as f64, mean, std_dev);
                return Some(Anomaly {
                    anomaly_type: AnomalyType::RateSpike,
                    bucket: bucket.to_string(),
                    description: format!(
                        "Log rate {} is {}x above baseline (avg: {})",
                        count,
                        (count as f64 / mean.max(1.0)) as usize,
                        mean as usize
                    ),
                    severity,
                    value: count,
                    expected: mean as usize,
                });
            }
            return None;
        }

        let z_score = (count as f64 - mean) / std_dev;
        if z_score > self.config.rate_spike_threshold {
            let severity = self.rate_severity(count as f64, mean, std_dev);
            Some(Anomaly {
                anomaly_type: AnomalyType::RateSpike,
                bucket: bucket.to_string(),
                description: format!(
                    "Log rate {} is {:.1} std devs above mean (avg: {})",
                    count, z_score, mean as usize
                ),
                severity,
                value: count,
                expected: mean as usize,
            })
        } else {
            None
        }
    }

    fn check_error_spike(&self, bucket: &str, error_count: usize) -> Option<Anomaly> {
        if self.history.len() < self.config.min_baseline_buckets {
            return None;
        }

        let error_counts: Vec<f64> = self.history.iter().map(|h| h.error_count as f64).collect();
        let mean = error_counts.iter().sum::<f64>() / error_counts.len() as f64;
        let variance = error_counts
            .iter()
            .map(|c| (c - mean).powi(2))
            .sum::<f64>()
            / error_counts.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            if error_count as f64 > mean * self.config.error_spike_threshold && error_count > 0 {
                return Some(Anomaly {
                    anomaly_type: AnomalyType::ErrorSpike,
                    bucket: bucket.to_string(),
                    description: format!(
                        "Error count {} is above baseline (avg: {})",
                        error_count, mean as usize
                    ),
                    severity: AnomalySeverity::High,
                    value: error_count,
                    expected: mean as usize,
                });
            }
            return None;
        }

        let z_score = (error_count as f64 - mean) / std_dev;
        if z_score > self.config.error_spike_threshold && error_count > 0 {
            let severity = if z_score > 4.0 {
                AnomalySeverity::Critical
            } else if z_score > 3.0 {
                AnomalySeverity::High
            } else {
                AnomalySeverity::Medium
            };
            Some(Anomaly {
                anomaly_type: AnomalyType::ErrorSpike,
                bucket: bucket.to_string(),
                description: format!(
                    "Error count {} is {:.1} std devs above mean (avg: {})",
                    error_count, z_score, mean as usize
                ),
                severity,
                value: error_count,
                expected: mean as usize,
            })
        } else {
            None
        }
    }

    fn detect_new_patterns(&self, current: &BucketHistory) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();

        // Collect all known field values from history
        let mut known_field_values: HashMap<String, HashMap<String, bool>> = HashMap::new();
        for hist in &self.history {
            for (field, values) in &hist.field_values {
                let field_map = known_field_values
                    .entry(field.clone())
                    .or_insert_with(HashMap::new);
                for value in values.keys() {
                    field_map.insert(value.clone(), true);
                }
            }
        }

        // Check for new values in current bucket
        for (field, values) in &current.field_values {
            if let Some(known_values) = known_field_values.get(field) {
                for (value, count) in values {
                    if !known_values.contains_key(value) && *count > 0 {
                        anomalies.push(Anomaly {
                            anomaly_type: AnomalyType::NewPattern,
                            bucket: current.bucket.clone(),
                            description: format!(
                                "New '{}' value: '{}' (count: {})",
                                field, value, count
                            ),
                            severity: AnomalySeverity::Low,
                            value: *count,
                            expected: 0,
                        });
                    }
                }
            }
        }

        anomalies
    }

    fn detect_level_shift(&self, current: &BucketHistory) -> Option<Anomaly> {
        if self.history.len() < self.config.min_baseline_buckets {
            return None;
        }

        // Calculate baseline error rate
        let total_baseline_count: usize = self.history.iter().map(|h| h.count).sum();
        let total_baseline_errors: usize = self.history.iter().map(|h| h.error_count).sum();

        if total_baseline_count == 0 {
            return None;
        }

        let baseline_error_rate = total_baseline_errors as f64 / total_baseline_count as f64;
        let current_error_rate = if current.count > 0 {
            current.error_count as f64 / current.count as f64
        } else {
            0.0
        };

        // Detect significant shift in error rate (more than 2x baseline)
        if baseline_error_rate > 0.0
            && current_error_rate > baseline_error_rate * 2.0
            && current.error_count > 1
        {
            let severity = if current_error_rate > baseline_error_rate * 5.0 {
                AnomalySeverity::Critical
            } else if current_error_rate > baseline_error_rate * 3.0 {
                AnomalySeverity::High
            } else {
                AnomalySeverity::Medium
            };

            return Some(Anomaly {
                anomaly_type: AnomalyType::LevelShift,
                bucket: current.bucket.clone(),
                description: format!(
                    "Error rate shifted from {:.1}% to {:.1}%",
                    baseline_error_rate * 100.0,
                    current_error_rate * 100.0
                ),
                severity,
                value: current.error_count,
                expected: (baseline_error_rate * current.count as f64) as usize,
            });
        }

        None
    }

    fn rate_severity(&self, count: f64, mean: f64, std_dev: f64) -> AnomalySeverity {
        if std_dev == 0.0 {
            if count > mean * 5.0 {
                AnomalySeverity::Critical
            } else if count > mean * 3.0 {
                AnomalySeverity::High
            } else {
                AnomalySeverity::Medium
            }
        } else {
            let z_score = (count - mean) / std_dev;
            if z_score > 4.0 {
                AnomalySeverity::Critical
            } else if z_score > 3.0 {
                AnomalySeverity::High
            } else if z_score > 2.0 {
                AnomalySeverity::Medium
            } else {
                AnomalySeverity::Low
            }
        }
    }

    fn update_history(&mut self, current: BucketHistory) {
        // Check if we already have this bucket
        if let Some(existing) = self.history.iter_mut().find(|h| h.bucket == current.bucket) {
            *existing = current;
        } else {
            self.history.push(current);
        }

        // Keep only the most recent buckets
        if self.history.len() > self.config.max_history_size {
            // Sort by bucket and keep the last N
            self.history.sort_by(|a, b| a.bucket.cmp(&b.bucket));
            let start = self.history.len() - self.config.max_history_size;
            self.history.drain(0..start);
        }
    }

    /// Get anomalies sorted by severity (most severe first)
    pub fn sorted_by_severity(anomalies: &[Anomaly]) -> Vec<Anomaly> {
        let mut sorted = anomalies.to_vec();
        sorted.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.bucket.cmp(&a.bucket))
        });
        sorted
    }

    /// Get a summary count of anomalies by severity
    pub fn severity_counts(anomalies: &[Anomaly]) -> HashMap<AnomalySeverity, usize> {
        let mut counts = HashMap::new();
        for anomaly in anomalies {
            *counts.entry(anomaly.severity).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LogLevel;
    use std::collections::HashMap;

    fn make_aggregation(
        total: usize,
        levels: Vec<(LogLevel, usize)>,
        buckets: Vec<(&str, usize)>,
        fields: Vec<(&str, Vec<(&str, usize)>)>,
    ) -> AggregationResult {
        let mut by_level = HashMap::new();
        for (level, count) in levels {
            by_level.insert(level, count);
        }

        let mut time_buckets = HashMap::new();
        for (bucket, count) in buckets {
            time_buckets.insert(bucket.to_string(), count);
        }

        let mut by_field = HashMap::new();
        for (field_name, values) in fields {
            let mut field_map = HashMap::new();
            for (value, count) in values {
                field_map.insert(value.to_string(), count);
            }
            by_field.insert(field_name.to_string(), field_map);
        }

        AggregationResult {
            total_count: total,
            by_level,
            by_field,
            time_buckets,
        }
    }

    #[test]
    fn test_anomaly_config_default() {
        let config = AnomalyConfig::default();
        assert_eq!(config.rate_spike_threshold, 2.5);
        assert_eq!(config.error_spike_threshold, 2.0);
        assert_eq!(config.min_baseline_buckets, 3);
        assert_eq!(config.max_history_size, 30);
    }

    #[test]
    fn test_rate_spike_detection() {
        let config = AnomalyConfig {
            rate_spike_threshold: 2.0,
            min_baseline_buckets: 3,
            ..Default::default()
        };
        let mut engine = AnomalyEngine::new(config);

        // Build baseline with low counts
        for i in 0..5 {
            let agg = make_aggregation(
                5,
                vec![(LogLevel::Info, 5)],
                vec![(
                    &format!("2024-01-15 10:{:02}:00", i),
                    5,
                )],
                vec![],
            );
            engine.detect(&agg);
        }

        // Now spike the rate
        let spike_agg = make_aggregation(
            50,
            vec![(LogLevel::Info, 50)],
            vec![("2024-01-15 10:05:00", 50)],
            vec![],
        );
        let anomalies = engine.detect(&spike_agg);

        assert!(!anomalies.is_empty());
        let rate_spike = anomalies
            .iter()
            .find(|a| a.anomaly_type == AnomalyType::RateSpike);
        assert!(rate_spike.is_some());
        assert_eq!(rate_spike.unwrap().value, 50);
        assert_eq!(rate_spike.unwrap().expected, 5);
    }

    #[test]
    fn test_error_spike_detection() {
        let config = AnomalyConfig {
            error_spike_threshold: 1.5,
            min_baseline_buckets: 3,
            ..Default::default()
        };
        let mut engine = AnomalyEngine::new(config);

        // Build baseline with few errors
        for i in 0..5 {
            let agg = make_aggregation(
                100,
                vec![(LogLevel::Info, 98), (LogLevel::Error, 2)],
                vec![(
                    &format!("2024-01-15 10:{:02}:00", i),
                    100,
                )],
                vec![],
            );
            engine.detect(&agg);
        }

        // Now spike the errors
        let spike_agg = make_aggregation(
            100,
            vec![(LogLevel::Info, 80), (LogLevel::Error, 20)],
            vec![("2024-01-15 10:05:00", 100)],
            vec![],
        );
        let anomalies = engine.detect(&spike_agg);

        assert!(!anomalies.is_empty());
        let error_spike = anomalies
            .iter()
            .find(|a| a.anomaly_type == AnomalyType::ErrorSpike);
        assert!(error_spike.is_some());
        assert_eq!(error_spike.unwrap().value, 20);
    }

    #[test]
    fn test_new_pattern_detection() {
        let config = AnomalyConfig {
            min_baseline_buckets: 2,
            ..Default::default()
        };
        let mut engine = AnomalyEngine::new(config);

        // Build baseline with known status codes
        for i in 0..3 {
            let agg = make_aggregation(
                100,
                vec![],
                vec![(
                    &format!("2024-01-15 10:{:02}:00", i),
                    100,
                )],
                vec![("status", vec![("200", 90), ("404", 10)])],
            );
            engine.detect(&agg);
        }

        // Now introduce a new status code
        let new_pattern_agg = make_aggregation(
            100,
            vec![],
            vec![("2024-01-15 10:03:00", 100)],
            vec![("status", vec![("200", 85), ("404", 10), ("500", 5)])],
        );
        let anomalies = engine.detect(&new_pattern_agg);

        let new_pattern = anomalies
            .iter()
            .find(|a| a.anomaly_type == AnomalyType::NewPattern);
        assert!(new_pattern.is_some());
        assert!(new_pattern
            .unwrap()
            .description
            .contains("500"));
    }

    #[test]
    fn test_level_shift_detection() {
        let config = AnomalyConfig {
            min_baseline_buckets: 3,
            ..Default::default()
        };
        let mut engine = AnomalyEngine::new(config);

        // Build baseline with low error rate (2%)
        for i in 0..5 {
            let agg = make_aggregation(
                100,
                vec![(LogLevel::Info, 98), (LogLevel::Error, 2)],
                vec![(
                    &format!("2024-01-15 10:{:02}:00", i),
                    100,
                )],
                vec![],
            );
            engine.detect(&agg);
        }

        // Now shift error rate to 30%
        let shift_agg = make_aggregation(
            100,
            vec![(LogLevel::Info, 70), (LogLevel::Error, 30)],
            vec![("2024-01-15 10:05:00", 100)],
            vec![],
        );
        let anomalies = engine.detect(&shift_agg);

        let level_shift = anomalies
            .iter()
            .find(|a| a.anomaly_type == AnomalyType::LevelShift);
        assert!(level_shift.is_some());
    }

    #[test]
    fn test_no_anomaly_with_insufficient_baseline() {
        let config = AnomalyConfig {
            min_baseline_buckets: 5,
            ..Default::default()
        };
        let mut engine = AnomalyEngine::new(config);

        // Only 2 buckets - not enough baseline
        for i in 0..2 {
            let agg = make_aggregation(
                5,
                vec![(LogLevel::Info, 5)],
                vec![(
                    &format!("2024-01-15 10:{:02}:00", i),
                    5,
                )],
                vec![],
            );
            engine.detect(&agg);
        }

        let spike_agg = make_aggregation(
            100,
            vec![(LogLevel::Info, 100)],
            vec![("2024-01-15 10:02:00", 100)],
            vec![],
        );
        let anomalies = engine.detect(&spike_agg);

        // Should not detect any anomalies with insufficient baseline
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_sorted_by_severity() {
        let anomalies = vec![
            Anomaly {
                anomaly_type: AnomalyType::RateSpike,
                bucket: "A".to_string(),
                description: "Low".to_string(),
                severity: AnomalySeverity::Low,
                value: 10,
                expected: 5,
            },
            Anomaly {
                anomaly_type: AnomalyType::ErrorSpike,
                bucket: "B".to_string(),
                description: "Critical".to_string(),
                severity: AnomalySeverity::Critical,
                value: 50,
                expected: 5,
            },
            Anomaly {
                anomaly_type: AnomalyType::NewPattern,
                bucket: "C".to_string(),
                description: "High".to_string(),
                severity: AnomalySeverity::High,
                value: 20,
                expected: 0,
            },
        ];

        let sorted = AnomalyEngine::sorted_by_severity(&anomalies);
        assert_eq!(sorted[0].severity, AnomalySeverity::Critical);
        assert_eq!(sorted[1].severity, AnomalySeverity::High);
        assert_eq!(sorted[2].severity, AnomalySeverity::Low);
    }

    #[test]
    fn test_severity_counts() {
        let anomalies = vec![
            Anomaly {
                anomaly_type: AnomalyType::RateSpike,
                bucket: "A".to_string(),
                description: "Low".to_string(),
                severity: AnomalySeverity::Low,
                value: 10,
                expected: 5,
            },
            Anomaly {
                anomaly_type: AnomalyType::ErrorSpike,
                bucket: "B".to_string(),
                description: "Critical".to_string(),
                severity: AnomalySeverity::Critical,
                value: 50,
                expected: 5,
            },
            Anomaly {
                anomaly_type: AnomalyType::NewPattern,
                bucket: "C".to_string(),
                description: "High".to_string(),
                severity: AnomalySeverity::High,
                value: 20,
                expected: 0,
            },
            Anomaly {
                anomaly_type: AnomalyType::RateSpike,
                bucket: "D".to_string(),
                description: "Low2".to_string(),
                severity: AnomalySeverity::Low,
                value: 8,
                expected: 5,
            },
        ];

        let counts = AnomalyEngine::severity_counts(&anomalies);
        assert_eq!(counts.get(&AnomalySeverity::Critical), Some(&1));
        assert_eq!(counts.get(&AnomalySeverity::High), Some(&1));
        assert_eq!(counts.get(&AnomalySeverity::Low), Some(&2));
    }

    #[test]
    fn test_anomaly_severity_ordering() {
        assert!(AnomalySeverity::Critical > AnomalySeverity::High);
        assert!(AnomalySeverity::High > AnomalySeverity::Medium);
        assert!(AnomalySeverity::Medium > AnomalySeverity::Low);
    }
}
