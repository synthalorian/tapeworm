use crate::aggregate::AggregationResult;
use crate::app::FileState;
use crate::parser::ParsedLine;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(ExportFormat::Json),
            "csv" => Some(ExportFormat::Csv),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Csv => "csv",
        }
    }

    pub fn file_extension(&self) -> &'static str {
        match self {
            ExportFormat::Json => ".json",
            ExportFormat::Csv => ".csv",
        }
    }
}

/// Export data for a single file
#[derive(Debug, Clone, serde::Serialize)]
struct FileExport {
    pub path: String,
    pub line_count: usize,
    pub lines: Vec<LineExport>,
    pub aggregation: Option<AggregationExport>,
}

/// Export data for a single line
#[derive(Debug, Clone, serde::Serialize)]
struct LineExport {
    pub raw: String,
    pub level: Option<String>,
    pub timestamp: Option<String>,
    pub fields: HashMap<String, String>,
}

/// Export data for aggregation results
#[derive(Debug, Clone, serde::Serialize)]
struct AggregationExport {
    pub total_count: usize,
    pub by_level: HashMap<String, usize>,
    pub by_field: HashMap<String, HashMap<String, usize>>,
    pub time_buckets: HashMap<String, usize>,
}

/// Export all file data to the specified format and path
pub fn export_files(
    files: &[FileState],
    format: ExportFormat,
    output_path: &Path,
) -> io::Result<()> {
    match format {
        ExportFormat::Json => export_json(files, output_path),
        ExportFormat::Csv => export_csv(files, output_path),
    }
}

/// Export to JSON format
fn export_json(files: &[FileState], output_path: &Path) -> io::Result<()> {
    let export_data: Vec<FileExport> = files.iter().map(build_file_export).collect();
    let json = serde_json::to_string_pretty(&export_data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JSON serialization error: {}", e)))?;
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Export to CSV format
/// Each row represents one log line with file, raw text, level, timestamp, and fields
fn export_csv(files: &[FileState], output_path: &Path) -> io::Result<()> {
    let mut writer = csv::Writer::from_path(output_path)?;

    // Collect all unique field names across all files
    let mut all_field_names = std::collections::BTreeSet::new();
    for file_state in files {
        for parsed in &file_state.parsed_lines {
            for key in parsed.fields.keys() {
                all_field_names.insert(key.clone());
            }
        }
    }
    let field_names: Vec<String> = all_field_names.into_iter().collect();

    // Write header
    let mut header = vec!["file", "raw", "level", "timestamp"];
    for name in &field_names {
        header.push(name.as_str());
    }
    writer.write_record(&header)?;

    // Write data rows
    for file_state in files {
        let file_name = file_state.name();

        // Export parsed lines if available, otherwise raw lines
        if !file_state.parsed_lines.is_empty() {
            for parsed in &file_state.parsed_lines {
                let mut row = vec![
                    file_name.clone(),
                    parsed.raw.clone(),
                    parsed.level.map(|l| l.as_str().to_string()).unwrap_or_default(),
                    parsed.timestamp.clone().unwrap_or_default(),
                ];
                for field_name in &field_names {
                    row.push(parsed.fields.get(field_name).cloned().unwrap_or_default());
                }
                writer.write_record(&row)?;
            }
        } else {
            for raw in &file_state.lines {
                let mut row = vec![
                    file_name.clone(),
                    raw.clone(),
                    String::new(),
                    String::new(),
                ];
                for _ in &field_names {
                    row.push(String::new());
                }
                writer.write_record(&row)?;
            }
        }
    }

    writer.flush()?;
    Ok(())
}

/// Build export data structure from a FileState
fn build_file_export(file_state: &FileState) -> FileExport {
    let lines: Vec<LineExport> = if !file_state.parsed_lines.is_empty() {
        file_state.parsed_lines.iter().map(build_line_export).collect()
    } else {
        file_state.lines.iter().map(|raw| LineExport {
            raw: raw.clone(),
            level: None,
            timestamp: None,
            fields: HashMap::new(),
        }).collect()
    };

    let aggregation = file_state.aggregation.as_ref().map(build_aggregation_export);

    FileExport {
        path: file_state.path.to_string_lossy().to_string(),
        line_count: file_state.line_count,
        lines,
        aggregation,
    }
}

/// Build export data for a single parsed line
fn build_line_export(parsed: &ParsedLine) -> LineExport {
    LineExport {
        raw: parsed.raw.clone(),
        level: parsed.level.map(|l| l.as_str().to_string()),
        timestamp: parsed.timestamp.clone(),
        fields: parsed.fields.clone(),
    }
}

/// Build export data for aggregation results
fn build_aggregation_export(agg: &AggregationResult) -> AggregationExport {
    let by_level: HashMap<String, usize> = agg
        .by_level
        .iter()
        .map(|(level, count)| (level.as_str().to_string(), *count))
        .collect();

    AggregationExport {
        total_count: agg.total_count,
        by_level,
        by_field: agg.by_field.clone(),
        time_buckets: agg.time_buckets.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{LogLevel, ParsedLine};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn make_test_file_state() -> FileState {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        file.push_lines(vec![
            "2024-01-15 ERROR Something went wrong".to_string(),
            "2024-01-15 INFO Server started".to_string(),
        ]);
        file.push_parsed_lines(vec![
            ParsedLine {
                raw: "2024-01-15 ERROR Something went wrong".to_string(),
                fields: {
                    let mut m = HashMap::new();
                    m.insert("timestamp".to_string(), "2024-01-15".to_string());
                    m.insert("level".to_string(), "ERROR".to_string());
                    m.insert("message".to_string(), "Something went wrong".to_string());
                    m
                },
                level: Some(LogLevel::Error),
                timestamp: Some("2024-01-15".to_string()),
            },
            ParsedLine {
                raw: "2024-01-15 INFO Server started".to_string(),
                fields: {
                    let mut m = HashMap::new();
                    m.insert("timestamp".to_string(), "2024-01-15".to_string());
                    m.insert("level".to_string(), "INFO".to_string());
                    m.insert("message".to_string(), "Server started".to_string());
                    m
                },
                level: Some(LogLevel::Info),
                timestamp: Some("2024-01-15".to_string()),
            },
        ]);
        file
    }

    #[test]
    fn test_export_format_from_str() {
        assert_eq!(ExportFormat::from_str("json"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("JSON"), Some(ExportFormat::Json));
        assert_eq!(ExportFormat::from_str("csv"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_str("CSV"), Some(ExportFormat::Csv));
        assert_eq!(ExportFormat::from_str("xml"), None);
    }

    #[test]
    fn test_export_json() {
        let file = make_test_file_state();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        export_json(&[file], output_path).unwrap();

        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("/tmp/test.log"));
        assert!(content.contains("ERROR"));
        assert!(content.contains("INFO"));
        assert!(content.contains("Something went wrong"));
    }

    #[test]
    fn test_export_csv() {
        let file = make_test_file_state();
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        export_csv(&[file], output_path).unwrap();

        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("file,raw,level,timestamp"));
        assert!(content.contains("ERROR"));
        assert!(content.contains("INFO"));
        assert!(content.contains("Something went wrong"));
    }

    #[test]
    fn test_export_csv_raw_only() {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        file.push_lines(vec![
            "plain line 1".to_string(),
            "plain line 2".to_string(),
        ]);

        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path();

        export_csv(&[file], output_path).unwrap();

        let content = std::fs::read_to_string(output_path).unwrap();
        assert!(content.contains("plain line 1"));
        assert!(content.contains("plain line 2"));
    }
}
