use anyhow::Result;
use clap::Parser;
use cli::args::Cli;
use cli::commands::handle_command;
use crate::db::store::Database;
use colored::*;

mod cli;
mod db;
mod shell;
mod ui;
mod utils;
mod exec;

fn main() -> Result<()> {
    // Enable colors globally
    colored::control::set_override(true);
    
    let cli = Cli::parse();
    
    // Get the data directory, preferring %APPDATA% on Windows
    let data_dir = if cfg!(windows) {
        dirs::data_local_dir()
    } else {
        dirs::data_dir()
    }
    .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
    .join("command-vault");

    // Create data directory and its parent directories if they don't exist
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create data directory: {}", e))?;
    }
    
    let db_path = data_dir.join("commands.db");
    let mut db = Database::new(db_path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid database path"))?)?;
    
    let result = handle_command(cli.command, &mut db);
    
    // Re-enable colors before exiting
    colored::control::set_override(true);
    
    result
}
