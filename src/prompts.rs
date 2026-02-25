use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn prompts_dir() -> PathBuf {
    let cfg = crate::global_config::load();
    cfg.prompts_dir()
}

pub fn list_prompts() -> Vec<String> {
    let dir = prompts_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().ok().map_or(false, |ft| ft.is_file()))
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if !name.ends_with(".md") {
                return None;
            }
            if name == "README.md" || name == "prompts.md" {
                return None;
            }
            Some(name)
        })
        .collect();

    files.sort();
    files
}

pub fn create_and_open_prompt(name: &str) -> std::io::Result<()> {
    let dir = prompts_dir();
    let filename = if name.ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    };
    let path = dir.join(&filename);

    if !path.exists() {
        fs::create_dir_all(&dir)?;
        fs::write(&path, "")?;
    }

    Command::new("yazi")
        .arg(&path)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    Ok(())
}

pub fn open_claude_session() -> std::io::Result<()> {
    let dir = prompts_dir();
    let prompts_md = dir.join("prompts.md");

    let instruction = format!(
        "read {} and ask for next instruction",
        prompts_md.display()
    );

    Command::new("claude")
        .arg(&instruction)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    Ok(())
}

pub fn run_prompt_with_claude(prompt_filename: &str) -> std::io::Result<()> {
    let dir = prompts_dir();
    let prompt_path = dir.join(prompt_filename);
    let prompts_md = dir.join("prompts.md");

    let instruction = format!(
        "read {} and {} and ask for next instruction",
        prompts_md.display(),
        prompt_path.display()
    );

    Command::new("claude")
        .arg(&instruction)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    Ok(())
}

pub const OPEN_SESSION: &str = "[open session]";
