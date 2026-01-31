use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::error::CliError;
use crate::fs_utils::{file_name_string, list_markdown_files};
use crate::pattern::{parse_filename, ParsedFilename};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by repository
    #[arg(long)]
    pub repo: Option<String>,
    /// Filter by category (feat or sop)
    #[arg(long)]
    pub category: Option<String>,
    /// Filter by feature (feat only)
    #[arg(long)]
    pub feature: Option<String>,
    /// Filter by type (feat only)
    #[arg(long, value_name = "type")]
    pub doc_type: Option<String>,
    /// Filter by variant (feat only)
    #[arg(long)]
    pub variant: Option<String>,
    /// Filter by name (sop only)
    #[arg(long)]
    pub name: Option<String>,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct ListJson(Vec<String>);

pub fn run(args: ListArgs) -> Result<(), CliError> {
    let files = list_markdown_files(Path::new("."))
        .map_err(|err| CliError::new(2, format!("Error: {err}")))?;

    let mut results = Vec::new();
    for path in files {
        let Some(name) = file_name_string(&path) else {
            continue;
        };
        let parsed = match parse_filename(&name) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if let Some(repo) = &args.repo {
            if parsed.repo() != repo {
                continue;
            }
        }

        if let Some(category) = &args.category {
            if parsed.category() != category {
                continue;
            }
        }

        match &parsed {
            ParsedFilename::Feat(f) => {
                if let Some(feature) = &args.feature {
                    if f.feature != *feature {
                        continue;
                    }
                }
                if let Some(doc_type) = &args.doc_type {
                    if f.doc_type != *doc_type {
                        continue;
                    }
                }
                if let Some(variant) = &args.variant {
                    if f.variant != *variant {
                        continue;
                    }
                }
                // Skip feat files if filtering by sop name
                if args.name.is_some() {
                    continue;
                }
            }
            ParsedFilename::Sop(s) => {
                if let Some(sop_name) = &args.name {
                    if s.name != *sop_name {
                        continue;
                    }
                }
                // Skip sop files if filtering by feat-specific fields
                if args.feature.is_some() || args.doc_type.is_some() || args.variant.is_some() {
                    continue;
                }
            }
        }

        results.push(name);
    }

    results.sort();

    if args.json {
        let json = serde_json::to_string_pretty(&ListJson(results))
            .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
        println!("{json}");
        return Ok(());
    }

    for item in results {
        println!("{item}");
    }
    Ok(())
}
