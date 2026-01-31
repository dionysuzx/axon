use clap::{Parser, Subcommand};

use axon::commands;

#[derive(Parser)]
#[command(name = "axon", version, about = "Validate and refactor prompt filenames")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Health(commands::health::HealthArgs),
    Validate(commands::validate::ValidateArgs),
    Parse(commands::parse::ParseArgs),
    Refactor(commands::refactor::RefactorArgs),
    List(commands::list::ListArgs),
    Stats(commands::stats::StatsArgs),
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Health(args) => commands::health::run(args),
        Commands::Validate(args) => commands::validate::run(args),
        Commands::Parse(args) => commands::parse::run(args),
        Commands::Refactor(args) => commands::refactor::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Stats(args) => commands::stats::run(args),
    };

    if let Err(err) = result {
        err.print();
        std::process::exit(err.code);
    }
}
