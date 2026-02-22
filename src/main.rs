use clap::{Parser, Subcommand};

use axon::commands;

#[derive(Parser)]
#[command(name = "axon", version, about = "Validate and refactor prompt filenames")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Health(commands::health::HealthArgs),
    Validate(commands::validate::ValidateArgs),
    Parse(commands::parse::ParseArgs),
    Refactor(commands::refactor::RefactorArgs),
Stats(commands::stats::StatsArgs),
    /// Open today's daily notes directory in yazi
    D,
    /// Create a new note with schema applied
    N {
        /// Filename for the new note (e.g. weekly.2026.03.02.md)
        filename: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Commands::Health(args)) => commands::health::run(args),
        Some(Commands::Validate(args)) => commands::validate::run(args),
        Some(Commands::Parse(args)) => commands::parse::run(args),
        Some(Commands::Refactor(args)) => commands::refactor::run(args),
        Some(Commands::Stats(args)) => commands::stats::run(args),
        Some(Commands::D) => axon::notes::open_daily().map_err(|e| axon::error::CliError {
            code: 1,
            message: format!("daily note error: {e}"),
        }),
        Some(Commands::N { filename }) => axon::notes::create_and_open_note(&filename).map_err(|e| axon::error::CliError {
            code: 1,
            message: format!("note error: {e}"),
        }),
        None => axon::tui::run(),
    };

    if let Err(err) = result {
        err.print();
        std::process::exit(err.code);
    }
}
