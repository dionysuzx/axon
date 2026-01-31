use clap::Args;
use dialoguer::{Confirm, Input};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::error::CliError;
use crate::fs_utils::{file_name_string, list_markdown_files};
use crate::pattern::{canonical_pattern_short, exempt_reason, is_valid_filename};
use crate::refactor::{
    build_rename_plans, read_journal, write_journal, RefactorPattern, RenamePlan,
};

const RETRY_FILE: &str = ".axon-retry.json";
const ROLLBACK_FILE: &str = ".axon-rollback.json";

#[derive(Args, Debug)]
pub struct RefactorArgs {
    /// Source pattern
    #[arg(long)]
    pub from: Option<String>,
    /// Target pattern
    #[arg(long)]
    pub to: Option<String>,
    /// Show what would be renamed and exit
    #[arg(long)]
    pub dry_run: bool,
    /// Skip confirmation prompt
    #[arg(long)]
    pub yes: bool,
    /// Use git mv for renames
    #[arg(long, conflicts_with = "no_git")]
    pub git: bool,
    /// Use mv for renames even in git repo
    #[arg(long)]
    pub no_git: bool,
    /// Overwrite existing files
    #[arg(long)]
    pub force: bool,
    /// Retry previously failed renames
    #[arg(long, conflicts_with_all = ["rollback", "from", "to"])]
    pub retry: bool,
    /// Rollback the last refactor
    #[arg(long, conflicts_with_all = ["retry", "from", "to"])]
    pub rollback: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RenameMethod {
    Git,
    Fs,
}

pub fn run(args: RefactorArgs) -> Result<(), CliError> {
    if args.retry {
        return run_retry(&args);
    }
    if args.rollback {
        return run_rollback(&args);
    }

    let default_pattern = canonical_pattern_short();
    let from = match args.from.clone() {
        Some(value) => value,
        None => prompt_pattern(
            "Enter source pattern (or press Enter for current):",
            Some(default_pattern),
        )?,
    };

    let to = match args.to.clone() {
        Some(value) => value,
        None => prompt_pattern("Enter target pattern:", None)?,
    };

    let source_pattern = RefactorPattern::new(&from).map_err(|err| {
        CliError::new(
            2,
            format!(
                "Error: Invalid source pattern \"{from}\"\n  - {err}\n  - Patterns must use {{placeholder}} syntax"
            ),
        )
    })?;
    let target_pattern = RefactorPattern::new(&to).map_err(|err| {
        CliError::new(
            2,
            format!(
                "Error: Invalid target pattern \"{to}\"\n  - {err}\n  - Patterns must use {{placeholder}} syntax"
            ),
        )
    })?;

    if source_pattern.placeholders != target_pattern.placeholders {
        return Err(CliError::new(
            2,
            placeholder_mismatch_message(&source_pattern, &target_pattern),
        ));
    }

    let files = list_markdown_files(Path::new("."))
        .map_err(|err| CliError::new(5, format!("Error: {err}")))?;

    if files.is_empty() {
        return Err(CliError::new(
            1,
            "Error: No markdown files found in current directory\n\nAre you in the prompts directory?"
                .to_string(),
        ));
    }

    let mut markdown = Vec::new();
    let mut exempt = Vec::new();
    for path in files {
        let Some(name) = file_name_string(&path) else {
            continue;
        };
        if exempt_reason(&name).is_some() {
            exempt.push(name);
        } else {
            markdown.push(name);
        }
    }

    let renames = build_rename_plans(&markdown, &source_pattern, &target_pattern)
        .map_err(|err| CliError::new(2, err))?;

    if renames.is_empty() {
        let valid_count = markdown.iter().filter(|name| is_valid_filename(name)).count();
        let mut message = format!("Error: No files match the pattern \"{from}\"\n\n");
        if valid_count > 0 {
            message.push_str(&format!(
                "Found {valid_count} valid files with pattern: {default_pattern}\n"
            ));
            message.push_str("Did you mean to use the current pattern?");
        }
        return Err(CliError::new(1, message));
    }

    println!("Analyzing {} files...\n", markdown.len() + exempt.len());
    println!("Matched: {} files", renames.len());
    if !exempt.is_empty() {
        println!("Skipped: {} files (exempt)", exempt.len());
    }
    let non_matching = markdown.len().saturating_sub(renames.len());
    if non_matching > 0 {
        println!("Skipped: {} files (non-matching)", non_matching);
    }

    let renames: Vec<_> = renames
        .into_iter()
        .filter(|entry| entry.from != entry.to)
        .collect();

    if renames.is_empty() {
        println!("\nNo changes to apply.");
        return Ok(());
    }

    println!("\nPreview:\n");
    print_preview(&renames, Some(3));

    if let Err(err) = check_for_duplicates_in_plans(&renames) {
        return Err(CliError::new(3, err));
    }

    if !args.force {
        if let Err(err) = check_existing_target_paths(&renames) {
            return Err(CliError::new(3, err));
        }
    }

    let method = resolve_method(&args)?;

    if args.dry_run {
        println!("\nDry run (no changes made):\n");
        print_preview(&renames, None);
        return Ok(());
    }

    let chosen_method = method;
    if !args.yes {
        let proceed = Confirm::new()
            .with_prompt(format!(
                "Proceed with {}?",
                match chosen_method {
                    RenameMethod::Git => "git mv",
                    RenameMethod::Fs => "mv",
                }
            ))
            .default(false)
            .interact()
            .map_err(|err| CliError::new(5, format!("Error: {err}")))?;
        if !proceed {
            return Ok(());
        }
    }

    execute_and_report(&renames, chosen_method, args.force, true)
}

fn run_retry(args: &RefactorArgs) -> Result<(), CliError> {
    let journal_path = Path::new(RETRY_FILE);
    if !journal_path.exists() {
        return Err(CliError::new(
            5,
            format!("Error: {RETRY_FILE} not found"),
        ));
    }

    let renames = read_journal(journal_path).map_err(|err| CliError::new(5, err))?;
    if renames.is_empty() {
        return Ok(());
    }

    let method = resolve_method(args)?;
    execute_and_report(&renames, method, args.force, true)?;
    let _ = std::fs::remove_file(RETRY_FILE);
    Ok(())
}

fn run_rollback(args: &RefactorArgs) -> Result<(), CliError> {
    let journal_path = Path::new(ROLLBACK_FILE);
    if !journal_path.exists() {
        return Err(CliError::new(
            5,
            format!("Error: {ROLLBACK_FILE} not found"),
        ));
    }

    let renames = read_journal(journal_path).map_err(|err| CliError::new(5, err))?;
    if renames.is_empty() {
        return Ok(());
    }

    let mut reversed = Vec::new();
    for rename in renames.iter().rev() {
        reversed.push(RenamePlan {
            from: rename.to.clone(),
            to: rename.from.clone(),
        });
    }

    let method = resolve_method(args)?;
    execute_and_report(&reversed, method, args.force, false)?;
    let _ = std::fs::remove_file(ROLLBACK_FILE);
    Ok(())
}

fn resolve_method(args: &RefactorArgs) -> Result<RenameMethod, CliError> {
    let in_git = is_git_repo();
    if args.git {
        if !in_git {
            return Err(CliError::new(
                5,
                "Error: Not in a git repository\n\nUse --no-git to rename with regular mv, or initialize a git repo first.".to_string(),
            ));
        }
        return Ok(RenameMethod::Git);
    }
    if args.no_git {
        return Ok(RenameMethod::Fs);
    }
    Ok(if in_git { RenameMethod::Git } else { RenameMethod::Fs })
}

fn prompt_pattern(prompt: &str, default: Option<&str>) -> Result<String, CliError> {
    let input = Input::new().with_prompt(prompt);
    let input = if let Some(default) = default {
        input.default(default.to_string())
    } else {
        input
    };
    let value = input
        .interact_text()
        .map_err(|err| CliError::new(5, format!("Error: {err}")))?;
    if value.trim().is_empty() {
        if let Some(default) = default {
            return Ok(default.to_string());
        }
    }
    Ok(value)
}

fn print_preview(renames: &[RenamePlan], limit: Option<usize>) {
    let max = limit.unwrap_or(renames.len());
    for entry in renames.iter().take(max) {
        println!("  {}", entry.from);
        println!("    -> {}", entry.to);
        println!();
    }
    if let Some(limit) = limit {
        if renames.len() > limit {
            println!("  ... and {} more", renames.len() - limit);
        }
    }
}

fn check_for_duplicates_in_plans(renames: &[RenamePlan]) -> Result<(), String> {
    let mut targets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for rename in renames {
        targets
            .entry(rename.to.clone())
            .or_default()
            .push(rename.from.clone());
    }

    let mut conflicts = Vec::new();
    for (target, sources) in targets {
        if sources.len() > 1 {
            conflicts.push((target, sources));
        }
    }

    if conflicts.is_empty() {
        return Ok(());
    }

    let mut message = String::from("Error: Target pattern would create duplicate filenames\n\nConflicts:\n");
    for (target, sources) in conflicts {
        message.push_str(&format!("  {target} would be created by:\n"));
        for source in sources {
            message.push_str(&format!("    - {source}\n"));
        }
        message.push('\n');
    }
    message.push_str("Aborting. No files were renamed.");
    Err(message)
}

fn check_existing_target_paths(renames: &[RenamePlan]) -> Result<(), String> {
    let mut conflicts = Vec::new();
    for rename in renames {
        if rename.from == rename.to {
            continue;
        }
        if Path::new(&rename.to).exists() {
            conflicts.push((rename.from.clone(), rename.to.clone()));
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

fn execute_and_report(
    renames: &[RenamePlan],
    method: RenameMethod,
    force: bool,
    write_journals: bool,
) -> Result<(), CliError> {
    println!("Renaming {} files...", renames.len());

    let mut successes = Vec::new();
    let mut failures: Option<(RenamePlan, String)> = None;

    for (idx, entry) in renames.iter().enumerate() {
        let step = idx + 1;
        let total = renames.len();
        match perform_rename(entry, method, force) {
            Ok(()) => {
                println!("  [{step}/{total}] OK: {}", entry.from);
                successes.push(entry.clone());
            }
            Err(err) => {
                println!("  [{step}/{total}] FAILED: {}", entry.from);
                println!("         {err}");
                failures = Some((entry.clone(), err));
                break;
            }
        }
    }

    if let Some((failed_entry, error)) = failures {
        let remaining_index = successes.len();
        let remaining = renames[remaining_index..].to_vec();

        if write_journals {
            let _ = write_journal(Path::new(ROLLBACK_FILE), &successes);
            let _ = write_journal(Path::new(RETRY_FILE), &remaining);
        }

        let mut message = format!(
            "Error: Rename failed after {} successful operations\n\n",
            successes.len()
        );
        if !successes.is_empty() {
            message.push_str("Successfully renamed:\n");
            for entry in &successes {
                message.push_str(&format!("  - {} -> {}\n", entry.from, entry.to));
            }
            message.push('\n');
        }

        message.push_str("Failed:\n");
        message.push_str(&format!("  - {}: {error}\n\n", failed_entry.from));
        let remaining_count = remaining.len().saturating_sub(1);
        if remaining_count > 0 {
            message.push_str(&format!(
                "Remaining {remaining_count} files were not processed.\n\n"
            ));
        }
        message.push_str("To retry failed files: axon refactor --retry\n");
        message.push_str("To rollback successful renames: axon refactor --rollback");
        return Err(CliError::new(4, message));
    }

    if write_journals {
        let _ = write_journal(Path::new(ROLLBACK_FILE), renames);
        let _ = std::fs::remove_file(RETRY_FILE);
    }

    println!("Done. {} files renamed.", renames.len());
    Ok(())
}

fn perform_rename(entry: &RenamePlan, method: RenameMethod, force: bool) -> Result<(), String> {
    match method {
        RenameMethod::Git => {
            let mut cmd = Command::new("git");
            cmd.arg("mv");
            if force {
                cmd.arg("-f");
            }
            cmd.arg("--").arg(&entry.from).arg(&entry.to);
            let output = cmd.output().map_err(|err| err.to_string())?;
            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let message = stderr.trim();
                if message.is_empty() {
                    Err("git mv failed".to_string())
                } else {
                    Err(message.to_string())
                }
            }
        }
        RenameMethod::Fs => {
            if force && Path::new(&entry.to).exists() {
                std::fs::remove_file(&entry.to).map_err(|err| err.to_string())?;
            }
            std::fs::rename(&entry.from, &entry.to).map_err(|err| err.to_string())
        }
    }
}

fn is_git_repo() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output();
    let Ok(output) = output else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim() == "true"
}

fn placeholder_mismatch_message(
    source: &RefactorPattern,
    target: &RefactorPattern,
) -> String {
    let source_list = source
        .placeholders
        .iter()
        .map(|p| format!("{{{}}}", p.name()))
        .collect::<Vec<_>>()
        .join(", ");
    let target_list = target
        .placeholders
        .iter()
        .map(|p| format!("{{{}}}", p.name()))
        .collect::<Vec<_>>()
        .join(", ");

    let mut missing_in_target = Vec::new();
    let mut missing_in_source = Vec::new();
    for placeholder in &source.placeholders {
        if !target.placeholders.contains(placeholder) {
            missing_in_target.push(format!("{{{}}}", placeholder.name()));
        }
    }
    for placeholder in &target.placeholders {
        if !source.placeholders.contains(placeholder) {
            missing_in_source.push(format!("{{{}}}", placeholder.name()));
        }
    }

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
    message
}
