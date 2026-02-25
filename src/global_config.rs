use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub notes_dir: Option<String>,
    #[serde(default)]
    pub prompts_dir: Option<String>,
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config/axon/config.toml")
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(rest)
    } else {
        PathBuf::from(path)
    }
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn load() -> GlobalConfig {
    let path = config_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return GlobalConfig::default();
    };
    toml::from_str(&content).unwrap_or_default()
}

impl GlobalConfig {
    pub fn notes_dir(&self) -> PathBuf {
        if let Ok(dir) = std::env::var("AXON_NOTES_DIR") {
            return PathBuf::from(dir);
        }
        if let Some(ref dir) = self.notes_dir {
            return expand_tilde(dir);
        }
        home_dir().join("notes")
    }

    pub fn prompts_dir(&self) -> PathBuf {
        if let Ok(dir) = std::env::var("AXON_PROMPTS_DIR") {
            return PathBuf::from(dir);
        }
        if let Some(ref dir) = self.prompts_dir {
            return expand_tilde(dir);
        }
        home_dir().join("prompts")
    }
}
