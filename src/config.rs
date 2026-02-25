use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub schemas: BTreeMap<String, String>,
}

pub fn load_config(notes_dir: &Path) -> Config {
    let path = notes_dir.join("axon.toml");
    let Ok(content) = fs::read_to_string(&path) else {
        return Config::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

impl Config {
    pub fn resolve_schema(&self, notes_dir: &Path, filename: &str) -> Option<String> {
        let schema_file = self
            .schemas
            .iter()
            .find(|(pattern, _)| glob_match(pattern, filename))
            .map(|(_, schema)| schema)?;

        let content = fs::read_to_string(notes_dir.join(schema_file)).ok()?;

        // Extract date from filename: "weekly.2026.02.23.md" -> "2026.02.23"
        let date = filename
            .strip_suffix(".md")
            .and_then(|s| s.split_once('.'))
            .map(|(_, d)| d)
            .unwrap_or("");

        Some(content.replace("{{date}}", date))
    }
}

pub fn glob_match(pattern: &str, value: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        return pattern == value;
    }

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        match value[pos..].find(part) {
            Some(offset) => {
                // First segment must match at start
                if i == 0 && offset != 0 {
                    return false;
                }
                pos += offset + part.len();
            }
            None => return false,
        }
    }

    // Last segment must match at end
    if let Some(last) = parts.last() {
        if !last.is_empty() && !value.ends_with(last) {
            return false;
        }
    }

    true
}
