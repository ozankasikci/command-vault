use anyhow::Result;
use clap::Parser;
use command_vault::cli::args::Cli;
use command_vault::cli::handle_command;
use command_vault::db::Database;
use std::path::PathBuf;

mod cli;
mod db;
mod shell;
mod utils;
pub mod ui;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Get the database path from the user's home directory
    let db_path = dirs::home_dir()
        .map(|mut path| {
            path.push(".command-vault");
            path.push("history.db");
            path
        })
        .unwrap_or_else(|| PathBuf::from("history.db"));

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut db = Database::new(db_path.to_str().unwrap())?;
    handle_command(cli.command, &mut db)?;

    Ok(())
}
