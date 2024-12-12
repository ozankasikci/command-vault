use anyhow::Result;
use chrono::Local;

use crate::db::{Command, Database};
use super::args::{Commands, TagCommands};

pub fn handle_command(command: Commands, db: &mut Database) -> Result<()> {
    match command {
        Commands::Add { command, exit_code, tags } => {
            let cmd = Command {
                id: None,
                command,
                timestamp: chrono::Utc::now(),
                directory: std::env::current_dir()?.to_string_lossy().into_owned(),
                exit_code,
                tags,
            };
            let id = db.add_command(&cmd)?;
            println!("Command added to history with ID: {}", id);
        }
        Commands::Search { query, limit } => {
            let commands = db.search_commands(&query, limit)?;
            print_commands(&commands);
        }
        Commands::Ls { limit, asc } => {
            let commands = db.list_commands(limit, asc)?;
            print_commands(&commands);
        }
        Commands::Tag { action } => match action {
            TagCommands::Add { command_id, tags } => {
                db.add_tags_to_command(command_id, &tags)?;
                println!("Tags added successfully");
            }
            TagCommands::Remove { command_id, tag } => {
                db.remove_tag_from_command(command_id, &tag)?;
                println!("Tag removed successfully");
            }
            TagCommands::List => {
                let tags = db.list_tags()?;
                if tags.is_empty() {
                    println!("No tags found");
                    return Ok(());
                }
                
                println!("\nTags and their usage:");
                println!("─────────────────────────────────────────────");
                for (tag, count) in tags {
                    println!("{}: {} command{}", tag, count, if count == 1 { "" } else { "s" });
                }
            }
            TagCommands::Search { tag, limit } => {
                let commands = db.search_by_tag(&tag, limit)?;
                print_commands(&commands);
            }
        }
    }
    Ok(())
}

fn print_commands(commands: &[Command]) {
    if commands.is_empty() {
        println!("No matching commands found.");
        return;
    }

    println!("\nFound {} matching commands:", commands.len());
    println!("─────────────────────────────────────────────");
    
    for cmd in commands {
        let local_time = cmd.timestamp.with_timezone(&Local);
        println!("({}) [{}] {}", 
            cmd.id.unwrap_or(0),
            local_time.format("%Y-%m-%d %H:%M:%S"),
            cmd.command
        );
        
        // If there's an exit code, show it
        if let Some(code) = cmd.exit_code {
            if code != 0 {
                println!("    Exit Code: {}", code);
            }
        }
        
        // Show the directory
        println!("    Directory: {}", cmd.directory);
        
        // Show tags if present
        if !cmd.tags.is_empty() {
            println!("    Tags: {}", cmd.tags.join(", "));
        }
        
        println!("─────────────────────────────────────────────");
    }
}
