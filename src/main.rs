use anyhow::Result;
use clap::Parser;
use cli::args::Cli;
use cli::commands::handle_command;
use crate::db::store::Database;

mod cli;
mod db;
mod shell;
mod ui;
mod utils;
mod exec;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
        .join("command-vault");
    std::fs::create_dir_all(&data_dir)?;
    
    let db_path = data_dir.join("commands.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    handle_command(cli.command, &mut db)?;

    Ok(())
}
