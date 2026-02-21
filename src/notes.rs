use std::fs;
use std::path::PathBuf;
use std::process::Command;

use chrono::{Datelike, Local};

use crate::config;

pub fn notes_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("AXON_NOTES_DIR") {
        return PathBuf::from(dir);
    }
    dirs_home().join("notes")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn open_daily() -> std::io::Result<()> {
    let dir = notes_dir();
    let path = create_daily(&dir)?;
    open_yazi(&path)
}

pub fn create_daily(notes_dir: &std::path::Path) -> std::io::Result<PathBuf> {
    let date = Local::now().format("%Y.%m.%d");
    let filename = format!("daily.{date}.md");
    let path = notes_dir.join(&filename);

    if !path.exists() {
        fs::create_dir_all(notes_dir)?;

        let cfg = config::load_config(notes_dir);
        let content = cfg.resolve_schema(notes_dir, &filename).unwrap_or_default();
        fs::write(&path, content)?;
    }

    Ok(path)
}

pub fn open_weekly() -> std::io::Result<()> {
    let dir = notes_dir();
    let path = create_weekly(&dir)?;
    open_yazi(&path)
}

pub fn create_weekly(notes_dir: &std::path::Path) -> std::io::Result<PathBuf> {
    let now = Local::now().date_naive();
    let monday = now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
    let date = monday.format("%Y.%m.%d");
    let filename = format!("weekly.{date}.md");
    let path = notes_dir.join(&filename);

    if !path.exists() {
        fs::create_dir_all(notes_dir)?;

        let cfg = config::load_config(notes_dir);
        let content = cfg.resolve_schema(notes_dir, &filename).unwrap_or_default();
        fs::write(&path, content)?;
    }

    Ok(path)
}

pub fn open_monthly() -> std::io::Result<()> {
    let dir = notes_dir();
    let path = create_monthly(&dir)?;
    open_yazi(&path)
}

pub fn create_monthly(notes_dir: &std::path::Path) -> std::io::Result<PathBuf> {
    let now = Local::now();
    let filename = format!("monthly.{}.{:02}.md", now.year(), now.month());
    let path = notes_dir.join(&filename);

    if !path.exists() {
        fs::create_dir_all(notes_dir)?;

        let cfg = config::load_config(notes_dir);
        let content = cfg.resolve_schema(notes_dir, &filename).unwrap_or_default();
        fs::write(&path, content)?;
    }

    Ok(path)
}

pub fn open_scratch() -> std::io::Result<()> {
    let dir = notes_dir();
    let path = create_scratch(&dir)?;
    open_yazi(&path)
}

pub fn create_scratch(notes_dir: &std::path::Path) -> std::io::Result<PathBuf> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let filename = format!("scratch.{timestamp}.md");
    let path = notes_dir.join(&filename);

    fs::create_dir_all(notes_dir)?;

    let cfg = config::load_config(notes_dir);
    let content = cfg.resolve_schema(notes_dir, &filename).unwrap_or_default();
    fs::write(&path, content)?;

    Ok(path)
}

pub fn list_notes() -> Vec<String> {
    let dir = notes_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let ft = e.file_type().ok();
            ft.map_or(false, |ft| ft.is_file())
        })
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if !name.ends_with(".md") || name.starts_with("schema.") {
                return None;
            }
            Some(name)
        })
        .collect();

    files.sort();
    files
}

pub fn open_note(filename: &str) -> std::io::Result<()> {
    let path = notes_dir().join(filename);
    open_yazi(&path)
}

pub fn create_and_open_note(name: &str) -> std::io::Result<()> {
    let dir = notes_dir();
    let filename = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let path = dir.join(&filename);

    if !path.exists() {
        fs::create_dir_all(&dir)?;
        let cfg = config::load_config(&dir);
        let content = cfg.resolve_schema(&dir, &filename).unwrap_or_default();
        fs::write(&path, content)?;
    }

    open_yazi(&path)
}

fn open_yazi(path: &std::path::Path) -> std::io::Result<()> {
    Command::new("yazi")
        .arg(path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    Ok(())
}
