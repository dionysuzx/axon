use clap::Args;
use serde::Serialize;

use crate::error::CliError;
use crate::pattern::{exempt_reason, parse_filename, ParsedFilename};

#[derive(Args, Debug)]
pub struct ParseArgs {
    pub filename: String,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Serialize)]
#[serde(tag = "category")]
enum ParseJson {
    #[serde(rename = "feat")]
    Feat {
        repo: String,
        feature: String,
        #[serde(rename = "type")]
        doc_type: String,
        variant: String,
        version: u32,
    },
    #[serde(rename = "sop")]
    Sop {
        repo: String,
        name: String,
        version: u32,
    },
}

pub fn run(args: ParseArgs) -> Result<(), CliError> {
    if exempt_reason(&args.filename).is_some() {
        return Err(CliError::new(
            1,
            "Invalid: exempt files do not follow the pattern".to_string(),
        ));
    }

    let parsed = parse_filename(&args.filename).map_err(|err| CliError::new(1, err))?;

    match parsed {
        ParsedFilename::Feat(f) => {
            if args.json {
                let payload = ParseJson::Feat {
                    repo: f.repo,
                    feature: f.feature,
                    doc_type: f.doc_type,
                    variant: f.variant,
                    version: f.version,
                };
                let json = serde_json::to_string_pretty(&payload)
                    .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
                println!("{json}");
            } else {
                println!("repo:     {}", f.repo);
                println!("category: feat");
                println!("feature:  {}", f.feature);
                println!("type:     {}", f.doc_type);
                println!("variant:  {}", f.variant);
                println!("version:  v{}", f.version);
            }
        }
        ParsedFilename::Sop(s) => {
            if args.json {
                let payload = ParseJson::Sop {
                    repo: s.repo,
                    name: s.name,
                    version: s.version,
                };
                let json = serde_json::to_string_pretty(&payload)
                    .map_err(|err| CliError::new(2, format!("Error: {err}")))?;
                println!("{json}");
            } else {
                println!("repo:     {}", s.repo);
                println!("category: sop");
                println!("name:     {}", s.name);
                println!("version:  v{}", s.version);
            }
        }
    }

    Ok(())
}
