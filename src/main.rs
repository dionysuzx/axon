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
        None => axon::tui::run(),
    };

    if let Err(err) = result {
        err.print();
        std::process::exit(err.code);
    }
}
