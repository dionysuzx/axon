use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedFilename {
    Feat(FeatFilename),
    Sop(SopFilename),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatFilename {
    pub repo: String,
    pub feature: String,
    pub doc_type: String,
    pub variant: String,
    pub version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SopFilename {
    pub repo: String,
    pub name: String,
    pub version: u32,
}

impl ParsedFilename {
    pub fn repo(&self) -> &str {
        match self {
            ParsedFilename::Feat(f) => &f.repo,
            ParsedFilename::Sop(s) => &s.repo,
        }
    }

    pub fn category(&self) -> &str {
        match self {
            ParsedFilename::Feat(_) => "feat",
            ParsedFilename::Sop(_) => "sop",
        }
    }

    pub fn version(&self) -> u32 {
        match self {
            ParsedFilename::Feat(f) => f.version,
            ParsedFilename::Sop(s) => s.version,
        }
    }
}

static FEAT_REGEX: OnceLock<Regex> = OnceLock::new();
static SOP_REGEX: OnceLock<Regex> = OnceLock::new();

fn feat_regex() -> &'static Regex {
    FEAT_REGEX.get_or_init(|| {
        Regex::new(
            r"^([a-z][a-z0-9-]*)\.feat\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\.v([1-9]\d*)\.md$",
        )
        .expect("invalid feat regex")
    })
}

fn sop_regex() -> &'static Regex {
    SOP_REGEX.get_or_init(|| {
        Regex::new(
            r"^([a-z][a-z0-9-]*)\.sop\.([a-z][a-z0-9-]*)\.v([1-9]\d*)\.md$",
        )
        .expect("invalid sop regex")
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
    feat_regex().is_match(name) || sop_regex().is_match(name)
}

pub fn parse_filename(name: &str) -> Result<ParsedFilename, String> {
    if let Some(caps) = feat_regex().captures(name) {
        let version: u32 = caps
            .get(5)
            .ok_or_else(|| "Missing version".to_string())?
            .as_str()
            .parse()
            .map_err(|_| "Invalid version".to_string())?;

        return Ok(ParsedFilename::Feat(FeatFilename {
            repo: caps.get(1).unwrap().as_str().to_string(),
            feature: caps.get(2).unwrap().as_str().to_string(),
            doc_type: caps.get(3).unwrap().as_str().to_string(),
            variant: caps.get(4).unwrap().as_str().to_string(),
            version,
        }));
    }

    if let Some(caps) = sop_regex().captures(name) {
        let version: u32 = caps
            .get(3)
            .ok_or_else(|| "Missing version".to_string())?
            .as_str()
            .parse()
            .map_err(|_| "Invalid version".to_string())?;

        return Ok(ParsedFilename::Sop(SopFilename {
            repo: caps.get(1).unwrap().as_str().to_string(),
            name: caps.get(2).unwrap().as_str().to_string(),
            version,
        }));
    }

    Err(format!("Invalid: does not match pattern\n  feat: {}\n  sop:  {}",
        canonical_pattern_feat(), canonical_pattern_sop()))
}

pub fn canonical_pattern_feat() -> &'static str {
    "{repo}.feat.{feature}.{type}.{variant}.v{N}.md"
}

pub fn canonical_pattern_sop() -> &'static str {
    "{repo}.sop.{name}.v{N}.md"
}

pub fn canonical_pattern() -> &'static str {
    "{repo}.feat.{feature}.{type}.{variant}.v{N}.md"
}

pub fn canonical_pattern_short() -> &'static str {
    "{repo}.feat.{feature}.{type}.{variant}.v{N}"
}
