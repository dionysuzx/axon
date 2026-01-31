use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

pub fn list_markdown_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension() == Some(OsStr::new("md")) {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

pub fn file_name_string(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}
