use anyhow::{Result, anyhow};
use chrono::{Local, Utc};
use std::io::{self, Stdout};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use colored::*;

use crate::db::{Command, Database};
use crate::ui::App;
use crate::utils::params::parse_parameters;
use crate::utils::params::substitute_parameters;
use crate::exec::{ExecutionContext, execute_shell_command};

use super::args::{Commands, TagCommands};

fn print_commands(commands: &[Command]) -> Result<()> {
    let terminal_result = setup_terminal();
    
    match terminal_result {
        Ok(mut terminal) => {
            let res = print_commands_ui(&mut terminal, commands);
            restore_terminal(&mut terminal)?;
            res
        }
        Err(_) => {
            // Fallback to simple text output
            println!("Command History:");
            println!("─────────────────────────────────────────────");
            for cmd in commands {
                let local_time = cmd.timestamp.with_timezone(&Local);
                println!("{} │ {}", local_time.format("%Y-%m-%d %H:%M:%S"), cmd.command);
                if !cmd.tags.is_empty() {
                    println!("    Tags: {}", cmd.tags.join(", "));
                }
                if !cmd.parameters.is_empty() {
                    println!("    Parameters:");
                    for param in &cmd.parameters {
                        let desc = param.description.as_deref().unwrap_or("None");
                        println!("      - {}: {} (default: {})", param.name, desc, "None");
                    }
                }
                println!("    Directory: {}", cmd.directory);
                println!();
            }
            Ok(())
        }
    }
}

fn print_commands_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, commands: &[Command]) -> Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0)])
            .split(f.size());

        let mut lines = vec![];
        lines.push(Line::from(Span::styled(
            "Command History:",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::raw("─────────────────────────────────────────────")));

        for cmd in commands {
            let local_time = cmd.timestamp.with_timezone(&Local);
            lines.push(Line::from(vec![
                Span::styled(local_time.format("%Y-%m-%d %H:%M:%S").to_string(), Style::default().fg(Color::Yellow)),
                Span::raw(" │ "),
                Span::raw(&cmd.command),
            ]));
            lines.push(Line::from(vec![
                Span::raw("    Directory: "),
                Span::raw(&cmd.directory),
            ]));
            if !cmd.tags.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("    Tags: "),
                    Span::raw(cmd.tags.join(", ")),
                ]));
            }
            lines.push(Line::from(Span::raw("─────────────────────────────────────────────")));
        }

        let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, chunks[0]);
    })?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(|e| e.into())
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn handle_command(command: Commands, db: &mut Database, debug: bool) -> Result<()> {
    match command {
        Commands::Add { command, tags } => {
            // Preserve quotes in arguments that need them
            let command_str = command.iter().enumerate().fold(String::new(), |mut acc, (i, arg)| {
                if i > 0 {
                    acc.push(' ');
                }
                // Special case for git format strings - ensure they're properly quoted
                if arg.starts_with("--pretty=format:") {
                    if arg.contains('"') {
                        acc.push_str(&format!("'{}'", arg)); // Use single quotes if format contains double quotes
                    } else {
                        acc.push_str(&format!("\"{}\"", arg)); // Use double quotes by default
                    }
                }
                // If the argument contains special characters or spaces, preserve its quotes
                else if arg.contains(':') || arg.contains('%') || arg.contains(' ') {
                    if (arg.starts_with('"') && arg.ends_with('"')) || (arg.starts_with('\'') && arg.ends_with('\'')) {
                        acc.push_str(arg); // Already quoted
                    } else if arg.contains('"') {
                        acc.push_str(&format!("'{}'", arg)); // Use single quotes if arg contains double quotes
                    } else {
                        acc.push_str(&format!("\"{}\"", arg)); // Use double quotes by default
                    }
                } else {
                    acc.push_str(arg);
                }
                acc
            });
            
            // Don't allow empty commands
            if command_str.trim().is_empty() {
                return Err(anyhow!("Cannot add empty command"));
            }
            
            // Get the current directory
            let directory = std::env::current_dir()?
                .to_string_lossy()
                .to_string();
            
            let timestamp = Local::now().with_timezone(&Utc);
            
            // Parse parameters from command string
            let parameters = parse_parameters(&command_str);
            
            let cmd = Command {
                id: None,
                command: command_str.clone(),
                timestamp,
                directory,
                tags,
                parameters,
            };
            let id = db.add_command(&cmd)?;
            println!("Command added to history with ID: {}", id);
            
            // If command has parameters, show them
            if !cmd.parameters.is_empty() {
                println!("\nDetected parameters:");
                for param in &cmd.parameters {
                    let desc = param.description.as_deref().unwrap_or("None");
                    println!("  {} - Description: {}", param.name.yellow(), desc);
                }
            }
        }
        Commands::Search { query, limit } => {
            let commands = db.search_commands(&query, limit)?;
            let mut app = App::new(commands.clone(), db, debug);
            match app.run() {
                Ok(_) => (),
                Err(e) => {
                    if e.to_string() == "Operation cancelled by user" {
                        print!("\n{}", "Operation cancelled.".yellow());
                        return Ok(());
                    }
                    eprintln!("Failed to start TUI mode: {}", e);
                    print_commands(&commands)?;
                }
            }
        }
        Commands::Ls { limit, asc } => {
            let commands = db.list_commands(limit, asc)?;
            if commands.is_empty() {
                print!("No commands found.");
                return Ok(());
            }

            // Check if TUI should be disabled (useful for testing or non-interactive environments)
            if std::env::var("COMMAND_VAULT_NO_TUI").is_ok() {
                for cmd in commands {
                    print!("{}: {} ({})", cmd.id.unwrap_or(0), cmd.command, cmd.directory);
                }
                return Ok(());
            }

            let mut app = App::new(commands.clone(), db, debug);
            match app.run() {
                Ok(_) => (),
                Err(e) => {
                    if e.to_string() == "Operation cancelled by user" {
                        print!("\n{}", "Operation cancelled.".yellow());
                        return Ok(());
                    }
                    eprintln!("Failed to start TUI mode: {}", e);
                    print_commands(&commands)?;
                }
            }
        }
        Commands::Tag { action } => match action {
            TagCommands::Add { command_id, tags } => {
                match db.add_tags_to_command(command_id, &tags) {
                    Ok(_) => print!("Tags added successfully"),
                    Err(e) => eprintln!("Failed to add tags: {}", e),
                }
            }
            TagCommands::Remove { command_id, tag } => {
                match db.remove_tag_from_command(command_id, &tag) {
                    Ok(_) => print!("Tag removed successfully"),
                    Err(e) => eprintln!("Failed to remove tag: {}", e),
                }
            }
            TagCommands::List => {
                match db.list_tags() {
                    Ok(tags) => {
                        if tags.is_empty() {
                            print!("No tags found");
                            return Ok(());
                        }
                        
                        print!("\nTags and their usage:");
                        print!("─────────────────────────────────────────────");
                        for (tag, count) in tags {
                            print!("{}: {} command{}", tag, count, if count == 1 { "" } else { "s" });
                        }
                    }
                    Err(e) => eprintln!("Failed to list tags: {}", e),
                }
            }
            TagCommands::Search { tag, limit } => {
                match db.search_by_tag(&tag, limit) {
                    Ok(commands) => print_commands(&commands)?,
                    Err(e) => eprintln!("Failed to search by tag: {}", e),
                }
            }
        },
        Commands::Exec { command_id, debug } => {
            let command = db.get_command(command_id)?
                .ok_or_else(|| anyhow!("Command not found with ID: {}", command_id))?;
            
            // Create the directory if it doesn't exist
            if !std::path::Path::new(&command.directory).exists() {
                std::fs::create_dir_all(&command.directory)?;
            }
            
            let current_params = parse_parameters(&command.command);
            let ctx = ExecutionContext {
                command: substitute_parameters(&command.command, &current_params, None)?,
                directory: command.directory.clone(),
                test_mode: std::env::var("COMMAND_VAULT_TEST").is_ok(),
                debug_mode: debug,
            };
            execute_shell_command(&ctx)?;
        }
        Commands::ShellInit { shell } => {
            let script_path = crate::shell::hooks::init_shell(shell)?;
            if !script_path.exists() {
                return Err(anyhow!("Shell integration script not found at: {}", script_path.display()));
            }
            print!("{}", script_path.display());
            return Ok(());
        },
        Commands::Delete { command_id } => {
            // First check if the command exists
            if let Some(command) = db.get_command(command_id)? {
                // Show the command that will be deleted
                println!("Deleting command:");
                print_commands(&[command])?;
                
                // Delete the command
                db.delete_command(command_id)?;
                println!("Command deleted successfully");
            } else {
                return Err(anyhow!("Command with ID {} not found", command_id));
            }
        }
    }
    Ok(())
}
