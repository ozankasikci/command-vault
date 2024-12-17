use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a command to history
    ///
    /// Parameters can be specified using @name:description=default syntax
    /// Examples:
    ///   - Basic parameter: @filename
    ///   - With description: @filename:Name of file to create
    ///   - With default: @filename:Name of file to create=test.txt
    Add {
        /// Tags to add to the command
        #[arg(short, long)]
        tags: Vec<String>,
        
        /// Command to add
        #[arg(last = true)]
        command: Vec<String>,
    },
    
    /// Execute a command by id (in the current shell)
    Exec {
        /// Command ID to execute
        command_id: i64,
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
        /// Maximum number of results to show. Use 0 to show all commands.
        #[arg(short, long, default_value = "50")]
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
    /// Initialize shell integration
    ShellInit {
        /// Shell to initialize (defaults to current shell)
        #[arg(short, long)]
        shell: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
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
