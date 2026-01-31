use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Placeholder(String);

impl Placeholder {
    pub fn new(value: &str) -> Result<Self, String> {
        if value.is_empty() {
            return Err("Empty placeholder".to_string());
        }
        Ok(Self(value.to_string()))
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub fn is_number(&self) -> bool {
        self.0 == "N"
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Literal(String),
    Placeholder(Placeholder),
}

pub fn normalize_pattern(pattern: &str) -> String {
    if pattern.contains(".md") {
        pattern.to_string()
    } else {
        format!("{pattern}.md")
    }
}

pub fn parse_refactor_pattern(pattern: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut literal = String::new();
    let mut chars = pattern.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        match ch {
            '{' => {
                if !literal.is_empty() {
                    tokens.push(Token::Literal(literal.clone()));
                    literal.clear();
                }
                let mut name = String::new();
                let mut closed = false;
                while let Some((_, inner)) = chars.next() {
                    if inner == '}' {
                        closed = true;
                        break;
                    }
                    if inner == '{' {
                        return Err("Nested placeholder".to_string());
                    }
                    name.push(inner);
                }
                if !closed {
                    return Err("Unclosed placeholder".to_string());
                }
                let placeholder = Placeholder::new(&name)?;
                tokens.push(Token::Placeholder(placeholder));
            }
            '}' => return Err(format!("Unopened placeholder at position {}", idx)),
            '[' => return Err(format!("Unclosed bracket at position {}", idx)),
            ']' => return Err(format!("Unopened bracket at position {}", idx)),
            _ => literal.push(ch),
        }
    }

    if !literal.is_empty() {
        tokens.push(Token::Literal(literal));
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
pub struct RefactorPattern {
    pub raw: String,
    pub normalized: String,
    pub tokens: Vec<Token>,
    pub placeholders: BTreeSet<Placeholder>,
}

impl RefactorPattern {
    pub fn new(raw: &str) -> Result<Self, String> {
        let normalized = normalize_pattern(raw);
        let tokens = parse_refactor_pattern(&normalized)?;
        let mut placeholders = BTreeSet::new();
        for token in &tokens {
            if let Token::Placeholder(ph) = token {
                if !placeholders.insert(ph.clone()) {
                    return Err(format!("Duplicate placeholder {{{}}}", ph.name()));
                }
            }
        }
        Ok(Self {
            raw: raw.to_string(),
            normalized,
            tokens,
            placeholders,
        })
    }
}

pub fn validate_placeholder_match(source: &str, target: &str) -> Result<(), String> {
    let source_pattern = RefactorPattern::new(source)?;
    let target_pattern = RefactorPattern::new(target)?;

    if source_pattern.placeholders == target_pattern.placeholders {
        return Ok(());
    }

    let source_set = &source_pattern.placeholders;
    let target_set = &target_pattern.placeholders;

    let mut missing_in_target = Vec::new();
    let mut missing_in_source = Vec::new();

    for placeholder in source_set {
        if !target_set.contains(placeholder) {
            missing_in_target.push(format!("{{{}}}", placeholder.name()));
        }
    }
    for placeholder in target_set {
        if !source_set.contains(placeholder) {
            missing_in_source.push(format!("{{{}}}", placeholder.name()));
        }
    }

    let source_list = source_set
        .iter()
        .map(|p| format!("{{{}}}", p.name()))
        .collect::<Vec<_>>()
        .join(", ");
    let target_list = target_set
        .iter()
        .map(|p| format!("{{{}}}", p.name()))
        .collect::<Vec<_>>()
        .join(", ");

    let mut message = String::from("Error: Placeholder mismatch between patterns\n");
    message.push_str(&format!("  - Source has: {source_list}\n"));
    message.push_str(&format!("  - Target has: {target_list}\n"));
    if !missing_in_target.is_empty() {
        message.push_str(&format!(
            "  - Missing in target: {}\n",
            missing_in_target.join(", ")
        ));
    }
    if !missing_in_source.is_empty() {
        message.push_str(&format!(
            "  - Missing in source: {}\n",
            missing_in_source.join(", ")
        ));
    }
    message.push_str("\nBoth patterns must use the same placeholders.");
    Err(message)
}

#[derive(Debug)]
pub struct PatternMatcher {
    regex: Regex,
    order: Vec<Placeholder>,
}

impl PatternMatcher {
    pub fn new(pattern: &RefactorPattern) -> Result<Self, String> {
        let mut regex = String::from("^");
        let mut order = Vec::new();

        for token in &pattern.tokens {
            match token {
                Token::Literal(text) => regex.push_str(&regex::escape(text)),
                Token::Placeholder(ph) => {
                    order.push(ph.clone());
                    let part = if ph.is_number() {
                        r"([1-9]\d*)"
                    } else {
                        r"([a-z][a-z0-9-]*)"
                    };
                    regex.push_str(part);
                }
            }
        }
        regex.push('$');

        let regex = Regex::new(&regex).map_err(|err| err.to_string())?;
        Ok(Self { regex, order })
    }

    pub fn captures(&self, value: &str) -> Option<HashMap<Placeholder, String>> {
        let caps = self.regex.captures(value)?;
        let mut values = HashMap::new();
        for (idx, placeholder) in self.order.iter().enumerate() {
            let value = caps.get(idx + 1)?.as_str().to_string();
            values.insert(placeholder.clone(), value);
        }
        Some(values)
    }
}

pub fn apply_pattern(pattern: &RefactorPattern, values: &HashMap<Placeholder, String>) -> String {
    let mut output = String::new();
    for token in &pattern.tokens {
        match token {
            Token::Literal(text) => output.push_str(text),
            Token::Placeholder(ph) => {
                if let Some(value) = values.get(ph) {
                    output.push_str(value);
                }
            }
        }
    }
    output
}

pub fn refactor_filename(
    filename: &str,
    source_pattern: &str,
    target_pattern: &str,
) -> Result<String, String> {
    let source = RefactorPattern::new(source_pattern)?;
    let target = RefactorPattern::new(target_pattern)?;
    let matcher = PatternMatcher::new(&source)?;
    let values = matcher
        .captures(filename)
        .ok_or_else(|| "Filename does not match source pattern".to_string())?;
    Ok(apply_pattern(&target, &values))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePlan {
    pub from: String,
    pub to: String,
}

pub fn match_files(files: &[&str], pattern: &str) -> Vec<String> {
    let normalized = normalize_pattern(pattern);
    let tokens = match parse_refactor_pattern(&normalized) {
        Ok(tokens) => tokens,
        Err(_) => return Vec::new(),
    };
    let mut placeholders = BTreeSet::new();
    for token in &tokens {
        if let Token::Placeholder(ph) = token {
            if !placeholders.insert(ph.clone()) {
                return Vec::new();
            }
        }
    }
    let refactor_pattern = RefactorPattern {
        raw: pattern.to_string(),
        normalized,
        tokens,
        placeholders,
    };
    let matcher = match PatternMatcher::new(&refactor_pattern) {
        Ok(matcher) => matcher,
        Err(_) => return Vec::new(),
    };

    files
        .iter()
        .filter_map(|file| {
            if matcher.captures(file).is_some() {
                Some((*file).to_string())
            } else {
                None
            }
        })
        .collect()
}

pub fn check_for_duplicates(
    files: &[&str],
    source_pattern: &str,
    target_pattern: &str,
) -> Result<(), String> {
    let source = RefactorPattern::new(source_pattern)?;
    let target = RefactorPattern::new(target_pattern)?;
    let matcher = PatternMatcher::new(&source)?;

    let mut targets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        if let Some(values) = matcher.captures(file) {
            let target_name = apply_pattern(&target, &values);
            targets
                .entry(target_name)
                .or_default()
                .push((*file).to_string());
        }
    }

    let mut conflicts = Vec::new();
    for (target_name, sources) in targets {
        if sources.len() > 1 {
            conflicts.push((target_name, sources));
        }
    }

    if conflicts.is_empty() {
        return Ok(());
    }

    let mut message = String::from("Error: Target pattern would create duplicate filenames\n\nConflicts:\n");
    for (target_name, sources) in conflicts {
        message.push_str(&format!("  {target_name} would be created by:\n"));
        for source in sources {
            message.push_str(&format!("    - {source}\n"));
        }
        message.push('\n');
    }
    message.push_str("Aborting. No files were renamed.");
    Err(message)
}

pub fn check_existing_targets(
    renames: &[(&str, &str)],
    existing: &[&str],
) -> Result<(), String> {
    let existing_set: BTreeSet<&str> = existing.iter().copied().collect();
    let mut conflicts = Vec::new();

    for (from, to) in renames {
        if from == to {
            continue;
        }
        if existing_set.contains(*to) {
            conflicts.push((from.to_string(), to.to_string()));
        }
    }

    if conflicts.is_empty() {
        return Ok(());
    }

    let mut message = String::from("Error: Target filename already exists\n\n");
    for (from, to) in conflicts {
        message.push_str(&format!("  {to} already exists\n  (source: {from})\n\n"));
    }
    message.push_str("Use --force to overwrite existing files (dangerous).\nAborting. No files were renamed.");
    Err(message)
}

pub fn build_rename_plans(
    files: &[String],
    source: &RefactorPattern,
    target: &RefactorPattern,
) -> Result<Vec<RenamePlan>, String> {
    let matcher = PatternMatcher::new(source)?;
    let mut renames = Vec::new();
    for file in files {
        if let Some(values) = matcher.captures(file) {
            let target_name = apply_pattern(target, &values);
            renames.push(RenamePlan {
                from: file.to_string(),
                to: target_name,
            });
        }
    }
    Ok(renames)
}

pub fn write_journal(path: &Path, renames: &[RenamePlan]) -> Result<(), String> {
    let payload = Journal { renames: renames.to_vec() };
    let contents = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    std::fs::write(path, contents).map_err(|err| err.to_string())
}

pub fn read_journal(path: &Path) -> Result<Vec<RenamePlan>, String> {
    let contents = std::fs::read_to_string(path).map_err(|err| err.to_string())?;
    let journal: Journal = serde_json::from_str(&contents).map_err(|err| err.to_string())?;
    Ok(journal.renames)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Journal {
    renames: Vec<RenamePlan>,
}
