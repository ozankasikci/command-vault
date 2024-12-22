use anyhow::Result;
use clap::Parser;
use command_vault::{
    cli::{args::Cli, commands::handle_command},
    db::store::Database,
};
use std::path::PathBuf;

mod cli;
mod db;
mod shell;
mod ui;
mod utils;
mod exec;

fn main() -> Result<()> {
    // Enable colors globally
    colored::control::set_override(true);
    
    let args = Cli::parse();
    
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
        .join("command-vault");
    std::fs::create_dir_all(&data_dir)?;
    
    let db_path = data_dir.join("commands.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    let result = handle_command(args.command, &mut db, args.debug);
    
    // Re-enable colors before exiting
    colored::control::set_override(true);
    
    result
}
