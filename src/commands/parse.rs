use clap::Args;
use serde::Serialize;

use crate::error::CliError;
use crate::pattern::{exempt_reason, parse_filename};

#[derive(Args, Debug)]
pub struct ParseArgs {
    pub filename: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
struct ParseJson {
    repo: String,
    feature: String,
    #[serde(rename = "type")]
    doc_type: String,
    variant: String,
    version: u32,
}

pub fn run(args: ParseArgs) -> Result<(), CliError> {
    if exempt_reason(&args.filename).is_some() {
        return Err(CliError::new(
            1,
            "Invalid: exempt files do not follow the pattern".to_string(),
        ));
    }

    let parsed = parse_filename(&args.filename).map_err(|err| CliError::new(1, err))?;

    if args.json {
        let payload = ParseJson {
            repo: parsed.repo,
            feature: parsed.feature,
            doc_type: parsed.doc_type,
            variant: parsed.variant,
            version: parsed.version,
        };
        let json = serde_json::to_string_pretty(&payload)
            .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
        println!("{json}");
        return Ok(());
    }

    println!("repo:    {}", parsed.repo);
    println!("feature: {}", parsed.feature);
    println!("type:    {}", parsed.doc_type);
    println!("variant: {}", parsed.variant);
    println!("version: v{}", parsed.version);
    Ok(())
}
