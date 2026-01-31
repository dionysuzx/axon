use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::error::CliError;
use crate::fs_utils::{file_name_string, list_markdown_files};
use crate::pattern::parse_filename;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by repository
    #[arg(long)]
    pub repo: Option<String>,
    /// Filter by feature
    #[arg(long)]
    pub feature: Option<String>,
    /// Filter by type
    #[arg(long, value_name = "type")]
    pub doc_type: Option<String>,
    /// Filter by variant
    #[arg(long)]
    pub variant: Option<String>,
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
            if parsed.repo != *repo {
                continue;
            }
        }
        if let Some(feature) = &args.feature {
            if parsed.feature != *feature {
                continue;
            }
        }
        if let Some(doc_type) = &args.doc_type {
            if parsed.doc_type != *doc_type {
                continue;
            }
        }
        if let Some(variant) = &args.variant {
            if parsed.variant != *variant {
                continue;
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
