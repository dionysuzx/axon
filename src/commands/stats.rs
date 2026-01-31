use clap::Args;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

use crate::error::CliError;
use crate::fs_utils::{file_name_string, list_markdown_files};
use crate::pattern::{exempt_reason, parse_filename};

#[derive(Args, Debug)]
pub struct StatsArgs {
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct StatsJson {
    total: usize,
    valid: usize,
    exempt: usize,
    invalid: usize,
    by_repo: BTreeMap<String, usize>,
    by_type: BTreeMap<String, usize>,
    by_variant: BTreeMap<String, usize>,
}

pub fn run(args: StatsArgs) -> Result<(), CliError> {
    let files = list_markdown_files(Path::new("."))
        .map_err(|err| CliError::new(2, format!("Error: {err}")))?;

    let mut valid = 0;
    let mut exempt = 0;
    let mut invalid = 0;
    let mut by_repo = BTreeMap::new();
    let mut by_type = BTreeMap::new();
    let mut by_variant = BTreeMap::new();

    for path in files {
        let Some(name) = file_name_string(&path) else {
            continue;
        };
        if exempt_reason(&name).is_some() {
            exempt += 1;
            continue;
        }
        let parsed = match parse_filename(&name) {
            Ok(parsed) => parsed,
            Err(_) => {
                invalid += 1;
                continue;
            }
        };
        valid += 1;
        *by_repo.entry(parsed.repo).or_insert(0) += 1;
        *by_type.entry(parsed.doc_type).or_insert(0) += 1;
        *by_variant.entry(parsed.variant).or_insert(0) += 1;
    }

    let total = valid + invalid + exempt;

    if args.json {
        let payload = StatsJson {
            total,
            valid,
            exempt,
            invalid,
            by_repo,
            by_type,
            by_variant,
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
        println!("{json}");
        return Ok(());
    }

    let mut summary = format!("Files: {total} total ({valid} valid, {exempt} exempt");
    if invalid > 0 {
        summary.push_str(&format!(", {invalid} invalid"));
    }
    summary.push(')');
    println!("{summary}\n");

    print_map("By repo", &by_repo);
    print_map("By type", &by_type);
    print_map("By variant", &by_variant);

    Ok(())
}

fn print_map(title: &str, map: &BTreeMap<String, usize>) {
    if map.is_empty() {
        return;
    }
    println!("{title}:");
    let key_width = map.keys().map(|k| k.len()).max().unwrap_or(0);
    let val_width = map
        .values()
        .map(|v| v.to_string().len())
        .max()
        .unwrap_or(0);

    for (key, value) in map {
        println!(
            "  {key:width$}: {value:val_width$} files",
            width = key_width,
            val_width = val_width
        );
    }
    println!();
}
