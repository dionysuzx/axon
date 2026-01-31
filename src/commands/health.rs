use clap::Args;
use serde::Serialize;
use std::path::Path;

use crate::error::CliError;
use crate::fs_utils::{file_name_string, list_markdown_files};
use crate::pattern::{canonical_pattern, exempt_reason, is_valid_filename};

#[derive(Args, Debug)]
pub struct HealthArgs {
    /// Treat exempt files as errors
    #[arg(long)]
    pub strict: bool,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
    /// Only output errors
    #[arg(long)]
    pub quiet: bool,
}

#[derive(Serialize)]
struct HealthJson {
    checked: usize,
    valid: usize,
    invalid: usize,
    exempt: usize,
    strict: bool,
    invalid_files: Vec<FileEntry>,
    exempt_files: Vec<FileEntry>,
}

#[derive(Serialize)]
struct FileEntry {
    file: String,
    detail: String,
}

pub fn run(args: HealthArgs) -> Result<(), CliError> {
    let files = list_markdown_files(Path::new("."))
        .map_err(|err| CliError::new(2, format!("Error: {err}")))?;

    let mut valid = 0;
    let mut invalid_files = Vec::new();
    let mut exempt_files = Vec::new();

    for path in files.iter() {
        let Some(name) = file_name_string(path) else {
            continue;
        };
        if let Some(reason) = exempt_reason(&name) {
            exempt_files.push(FileEntry {
                file: name,
                detail: format!("exempt: {reason}"),
            });
            continue;
        }
        if is_valid_filename(&name) {
            valid += 1;
        } else {
            invalid_files.push(FileEntry {
                file: name,
                detail: format!("error: does not match pattern {}", canonical_pattern()),
            });
        }
    }

    let exempt_count = exempt_files.len();
    let invalid_count = invalid_files.len();
    let invalid_total = if args.strict {
        invalid_count + exempt_count
    } else {
        invalid_count
    };

    if args.json {
        let payload = HealthJson {
            checked: valid + invalid_count + exempt_count,
            valid,
            invalid: invalid_total,
            exempt: exempt_count,
            strict: args.strict,
            invalid_files,
            exempt_files,
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
        println!("{json}");
        if invalid_total > 0 {
            return Err(CliError::new(1, String::new()));
        }
        return Ok(());
    }

    if args.quiet {
        if invalid_total > 0 {
            for entry in &invalid_files {
                println!("{} ({})", entry.file, entry.detail);
            }
            if args.strict {
                for entry in &exempt_files {
                    println!("{} ({})", entry.file, entry.detail);
                }
            }
        }
        if invalid_total > 0 {
            return Err(CliError::new(1, String::new()));
        }
        return Ok(());
    }

    println!("Checking {} markdown files...\n", valid + invalid_count + exempt_count);
    println!("Valid: {} files", valid);
    println!("Invalid: {} files", invalid_total);
    if !args.strict {
        println!("Exempt: {} files", exempt_count);
    }

    if invalid_total > 0 {
        println!("\nInvalid files:");
        for entry in &invalid_files {
            println!("  - {} ({})", entry.file, entry.detail);
        }
        if args.strict {
            for entry in &exempt_files {
                println!("  - {} ({})", entry.file, entry.detail);
            }
        }
    }

    if !args.strict && !exempt_files.is_empty() {
        println!("\nExempt files:");
        for entry in &exempt_files {
            println!("  - {} ({})", entry.file, entry.detail);
        }
    }

    if invalid_total == 0 {
        println!("\nHealth: OK");
        Ok(())
    } else {
        println!("\nHealth: FAIL");
        Err(CliError::new(1, String::new()))
    }
}
