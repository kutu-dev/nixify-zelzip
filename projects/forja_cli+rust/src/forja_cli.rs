//! Management tool for multiple tasks on the monorepo.

use color_eyre::Result;
use tracing::info;
use util::setup_logging_for_cli;

mod cli;
mod todo;

fn main() -> Result<()> {
    println!("HELLO WORLD");
    setup_logging_for_cli();

    let matches = cli::get_matches();

    if let Some(matches) = matches.subcommand_matches("todo") {
        todo::print_tasks();
    }

    if let Some(matches) = matches.subcommand_matches("generate-ignores") {
        info!("IGNORES");
    }

    if let Some(matches) = matches.subcommand_matches("check") {
        info!("CHECK");
    }

    if let Some(matches) = matches.subcommand_matches("fix") {
        info!("FIX");
    }

    if let Some(matches) = matches.subcommand_matches("dev") {
        info!("DEV");
    }

    if let Some(matches) = matches.subcommand_matches("build") {
        info!("BUILD");
    }

    if let Some(matches) = matches.subcommand_matches("docs") {
        info!("DOCS");
    }

    if let Some(matches) = matches.subcommand_matches("test") {
        info!("TEST");
    }

    Ok(())
}
