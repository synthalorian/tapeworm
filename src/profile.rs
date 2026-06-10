use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Profile not found: {0}")]
    #[allow(dead_code)]
    NotFound(String),
}

/// A single field definition in a log profile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDef {
    pub name: String,
    #[serde(default)]
    pub field_type: String,
}

/// A log parsing profile that defines regex patterns and field mappings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogProfile {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub regex: String,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
    #[serde(default)]
    pub level_field: Option<String>,
    #[serde(default)]
    pub timestamp_field: Option<String>,
}

impl LogProfile {
    /// Create a built-in syslog profile
    pub fn syslog() -> Self {
        Self {
            name: "syslog".to_string(),
            description: "Standard syslog format".to_string(),
            regex: r"^(?P<timestamp>\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+(?P<host>\S+)\s+(?P<process>[^:]+)(?:\[(?P<pid>\d+)\])?:\s+(?P<message>.+)$".to_string(),
            fields: vec![
                FieldDef { name: "timestamp".to_string(), field_type: "string".to_string() },
                FieldDef { name: "host".to_string(), field_type: "string".to_string() },
                FieldDef { name: "process".to_string(), field_type: "string".to_string() },
                FieldDef { name: "pid".to_string(), field_type: "string".to_string() },
                FieldDef { name: "message".to_string(), field_type: "string".to_string() },
            ],
            level_field: Some("message".to_string()),
            timestamp_field: Some("timestamp".to_string()),
        }
    }

    /// Create a built-in nginx access log profile
    pub fn nginx() -> Self {
        Self {
            name: "nginx".to_string(),
            description: "Nginx combined log format".to_string(),
            regex: r#"^(?P<remote_addr>\S+)\s+-\s+(?P<remote_user>\S+)\s+\[(?P<timestamp>[^\]]+)\]\s+"(?P<request>[^"]+)"\s+(?P<status>\d{3})\s+(?P<body_bytes_sent>\d+)\s+"(?P<http_referer>[^"]*)"\s+"(?P<http_user_agent>[^"]*)"$"#.to_string(),
            fields: vec![
                FieldDef { name: "remote_addr".to_string(), field_type: "string".to_string() },
                FieldDef { name: "remote_user".to_string(), field_type: "string".to_string() },
                FieldDef { name: "timestamp".to_string(), field_type: "string".to_string() },
                FieldDef { name: "request".to_string(), field_type: "string".to_string() },
                FieldDef { name: "status".to_string(), field_type: "string".to_string() },
                FieldDef { name: "body_bytes_sent".to_string(), field_type: "string".to_string() },
                FieldDef { name: "http_referer".to_string(), field_type: "string".to_string() },
                FieldDef { name: "http_user_agent".to_string(), field_type: "string".to_string() },
            ],
            level_field: Some("status".to_string()),
            timestamp_field: Some("timestamp".to_string()),
        }
    }

    /// Create a built-in JSON log profile
    pub fn json() -> Self {
        Self {
            name: "json".to_string(),
            description: "Generic JSON log format".to_string(),
            regex: r"^\s*\{(?P<json_body>.*)\}\s*$".to_string(),
            fields: vec![
                FieldDef { name: "json_body".to_string(), field_type: "string".to_string() },
            ],
            level_field: None,
            timestamp_field: None,
        }
    }

    /// Load a profile from a JSON file
    pub fn from_file(path: &Path) -> Result<Self, ProfileError> {
        let content = std::fs::read_to_string(path)?;
        let profile: LogProfile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Save a profile to a JSON file
    #[allow(dead_code)]
    pub fn to_file(&self, path: &Path) -> Result<(), ProfileError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Manages a collection of log profiles
#[derive(Debug, Clone)]
pub struct ProfileManager {
    profiles: HashMap<String, LogProfile>,
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    pub fn new() -> Self {
        let mut manager = Self {
            profiles: HashMap::new(),
        };
        // Register built-in profiles
        manager.register(LogProfile::syslog());
        manager.register(LogProfile::nginx());
        manager.register(LogProfile::json());
        manager
    }

    pub fn register(&mut self, profile: LogProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn get(&self, name: &str) -> Option<&LogProfile> {
        self.profiles.get(name)
    }

    #[allow(dead_code)]
    pub fn load_from_file(&mut self, path: &Path) -> Result<(), ProfileError> {
        let profile = LogProfile::from_file(path)?;
        self.register(profile);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_builtin_syslog_profile() {
        let profile = LogProfile::syslog();
        assert_eq!(profile.name, "syslog");
        assert!(!profile.regex.is_empty());
        assert_eq!(profile.fields.len(), 5);
    }

    #[test]
    fn test_builtin_nginx_profile() {
        let profile = LogProfile::nginx();
        assert_eq!(profile.name, "nginx");
        assert!(!profile.regex.is_empty());
        assert_eq!(profile.fields.len(), 8);
    }

    #[test]
    fn test_profile_manager_builtins() {
        let manager = ProfileManager::new();
        let profiles = manager.list_profiles();
        assert!(profiles.contains(&"syslog"));
        assert!(profiles.contains(&"nginx"));
        assert!(profiles.contains(&"json"));
    }

    #[test]
    fn test_profile_roundtrip() {
        let profile = LogProfile::syslog();
        let temp_file = NamedTempFile::new().unwrap();
        profile.to_file(temp_file.path()).unwrap();
        
        let loaded = LogProfile::from_file(temp_file.path()).unwrap();
        assert_eq!(profile.name, loaded.name);
        assert_eq!(profile.regex, loaded.regex);
        assert_eq!(profile.fields, loaded.fields);
    }

    #[test]
    fn test_profile_manager_load() {
        let profile = LogProfile::syslog();
        let temp_file = NamedTempFile::new().unwrap();
        profile.to_file(temp_file.path()).unwrap();
        
        let mut manager = ProfileManager::new();
        manager.load_from_file(temp_file.path()).unwrap();
        
        // Should overwrite the built-in
        assert!(manager.get("syslog").is_some());
    }

    #[test]
    fn test_custom_profile_serialization() {
        let profile = LogProfile {
            name: "custom".to_string(),
            description: "Test profile".to_string(),
            regex: r"^(?P<level>\w+):\s+(?P<message>.+)$".to_string(),
            fields: vec![
                FieldDef { name: "level".to_string(), field_type: "string".to_string() },
                FieldDef { name: "message".to_string(), field_type: "string".to_string() },
            ],
            level_field: Some("level".to_string()),
            timestamp_field: None,
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: LogProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_profile_deserialize_from_json() {
        let json = r#"{
            "name": "test",
            "description": "A test profile",
            "regex": "^(?P<time>\\d+): (?P<msg>.+)$",
            "fields": [
                {"name": "time", "field_type": "string"},
                {"name": "msg", "field_type": "string"}
            ],
            "level_field": "msg",
            "timestamp_field": "time"
        }"#;

        let profile: LogProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.name, "test");
        assert_eq!(profile.fields.len(), 2);
        assert_eq!(profile.level_field, Some("msg".to_string()));
    }
}
