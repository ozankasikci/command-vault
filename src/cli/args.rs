use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "command-vault")]
#[command(about = "An advanced command history manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
    /// List all commands in chronological order
    Ls {
        /// Maximum number of results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
        
        /// Sort in ascending order (oldest first)
        #[arg(short = 'a', long)]
        asc: bool,
    },
    /// Tag related operations
    Tag {
        #[command(subcommand)]
        action: TagCommands,
    },
}

#[derive(Subcommand)]
pub enum TagCommands {
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
