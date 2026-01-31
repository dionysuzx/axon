use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFilename {
    pub repo: String,
    pub feature: String,
    pub doc_type: String,
    pub variant: String,
    pub version: u32,
}

static FILENAME_REGEX: OnceLock<Regex> = OnceLock::new();

fn filename_regex() -> &'static Regex {
    FILENAME_REGEX.get_or_init(|| {
        Regex::new(
            r"^([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\.v([1-9]\d*)\.md$",
        )
        .expect("invalid filename regex")
    })
}

pub fn exempt_reason(name: &str) -> Option<&'static str> {
    match name {
        "README.md" | "prompts.md" => Some("documentation"),
        ".gitignore" | ".DS_Store" => Some("system"),
        _ => None,
    }
}

pub fn is_valid_filename(name: &str) -> bool {
    filename_regex().is_match(name)
}

pub fn parse_filename(name: &str) -> Result<ParsedFilename, String> {
    let caps = filename_regex()
        .captures(name)
        .ok_or_else(|| format!("Invalid: does not match pattern {}", canonical_pattern()))?;

    let version: u32 = caps
        .get(5)
        .ok_or_else(|| "Missing version".to_string())?
        .as_str()
        .parse()
        .map_err(|_| "Invalid version".to_string())?;

    Ok(ParsedFilename {
        repo: caps
            .get(1)
            .ok_or_else(|| "Missing repo".to_string())?
            .as_str()
            .to_string(),
        feature: caps
            .get(2)
            .ok_or_else(|| "Missing feature".to_string())?
            .as_str()
            .to_string(),
        doc_type: caps
            .get(3)
            .ok_or_else(|| "Missing type".to_string())?
            .as_str()
            .to_string(),
        variant: caps
            .get(4)
            .ok_or_else(|| "Missing variant".to_string())?
            .as_str()
            .to_string(),
        version,
    })
}

pub fn canonical_pattern() -> &'static str {
    "{repo}.{feature}.{type}.{variant}.v{N}.md"
}

pub fn canonical_pattern_short() -> &'static str {
    "{repo}.{feature}.{type}.{variant}.v{N}"
}
