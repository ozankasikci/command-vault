use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod cli;
mod db;
mod shell;
mod utils;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();
    
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

    let mut db = db::Database::new(db_path.to_str().unwrap())?;
    cli::handle_command(cli.command, &mut db)?;

    Ok(())
}
