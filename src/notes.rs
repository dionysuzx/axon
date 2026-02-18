use std::fs;
use std::path::PathBuf;
use std::process::Command;

use chrono::Local;

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
    open_editor(&path)
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

fn open_editor(path: &std::path::Path) -> std::io::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".into());
    Command::new(&editor)
        .arg(path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;
    Ok(())
}
