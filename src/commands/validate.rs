use clap::Args;

use crate::error::CliError;
use crate::pattern::{canonical_pattern, exempt_reason, is_valid_filename};

#[derive(Args, Debug)]
pub struct ValidateArgs {
    pub filename: String,
}

pub fn run(args: ValidateArgs) -> Result<(), CliError> {
    if let Some(reason) = exempt_reason(&args.filename) {
        println!("Valid (exempt: {reason})");
        return Ok(());
    }

    if is_valid_filename(&args.filename) {
        println!("Valid");
        Ok(())
    } else {
        Err(CliError::new(
            1,
            format!(
                "Invalid: does not match pattern {}",
                canonical_pattern()
            ),
        ))
    }
}
