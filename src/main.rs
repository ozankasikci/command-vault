use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use chrono::Local;

mod db;

#[derive(Parser)]
#[command(name = "lazy-history")]
#[command(about = "An advanced command history manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new command to history
    Add {
        /// The command to add
        #[arg(required = true)]
        command: String,
        
        /// Optional exit code of the command
        #[arg(long)]
        exit_code: Option<i32>,

        /// Tags to add to the command
        #[arg(short, long)]
        tags: Vec<String>,
    },
    /// Search through command history
    Search {
        /// Search query
        #[arg(required = true)]
        query: String,
        
        /// Maximum number of results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Tag related operations
    Tag {
        #[command(subcommand)]
        action: TagCommands,
    },
}

#[derive(Subcommand)]
enum TagCommands {
    /// Add tags to a command
    Add {
        /// Command ID to tag
        #[arg(required = true)]
        command_id: i64,
        
        /// Tags to add
        #[arg(required = true)]
        tags: Vec<String>,
    },
    /// Remove a tag from a command
    Remove {
        /// Command ID to remove tag from
        #[arg(required = true)]
        command_id: i64,
        
        /// Tag to remove
        #[arg(required = true)]
        tag: String,
    },
    /// List all tags and their usage count
    List,
    /// Search commands by tag
    Search {
        /// Tag to search for
        #[arg(required = true)]
        tag: String,
        
        /// Maximum number of results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

fn print_commands(commands: &[db::Command]) {
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Get the database path from the user's home directory
    let db_path = dirs::home_dir()
        .map(|mut path| {
            path.push(".lazy-history");
            path.push("history.db");
            path
        })
        .unwrap_or_else(|| PathBuf::from("history.db"));

    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut db = db::Database::new(db_path.to_str().unwrap())?;

    match cli.command {
        Commands::Add { command, exit_code, tags } => {
            let cmd = db::Command {
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
