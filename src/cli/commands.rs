use anyhow::{Result, anyhow};
use chrono::Local;
use std::io::{self, Write, Stdout};
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

use crate::db::{Command, Database};
use crate::ui::App;
use super::args::{Commands, TagCommands};

fn print_commands(commands: &[Command]) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let res = print_commands_ui(&mut terminal, commands);
    restore_terminal(&mut terminal)?;
    res
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
            if let Some(code) = cmd.exit_code {
                if code != 0 {
                    lines.push(Line::from(vec![
                        Span::raw("    Exit Code: "),
                        Span::styled(code.to_string(), Style::default().fg(Color::Red)),
                    ]));
                }
            }
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

pub fn handle_command(command: Commands, db: &mut Database) -> Result<()> {
    match command {
        Commands::Add { command, exit_code, tags } => {
            let command = command.join(" ");
            // Don't allow empty commands
            if command.trim().is_empty() {
                return Err(anyhow!("Cannot add empty command"));
            }

            let (final_exit_code, _output) = if cfg!(test) {
                // In test mode, don't actually execute the command
                // Use the provided exit code or default to 0
                (exit_code.unwrap_or(0), String::new())
            } else {
                // Execute the command
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&command)
                    .output()?;
                
                // Print command output
                if !output.stdout.is_empty() {
                    io::stdout().write_all(&output.stdout)?;
                }
                if !output.stderr.is_empty() {
                    io::stderr().write_all(&output.stderr)?;
                }
                
                (exit_code.unwrap_or_else(|| output.status.code().unwrap_or(0)), 
                 String::from_utf8_lossy(&output.stdout).into_owned())
            };

            let directory = std::env::current_dir()?.canonicalize()?.to_string_lossy().into_owned();
            let timestamp = chrono::Utc::now();
            let cmd = Command {
                id: None,
                command,
                timestamp,
                directory,
                exit_code: Some(final_exit_code),
                tags,
            };
            let id = db.add_command(&cmd)?;
            println!("Command added to history with ID: {}", id);
        }
        Commands::Search { query, limit } => {
            let commands = db.search_commands(&query, limit)?;
            let mut app = App::new(commands.clone(), db);
            match app.run() {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to start TUI mode: {}", e);
                    print_commands(&commands)?;
                }
            }
        }
        Commands::Ls { limit, asc } => {
            let commands = db.list_commands(limit, asc)?;
            if commands.is_empty() {
                println!("No commands found.");
                return Ok(());
            }

            // Check if TUI should be disabled (useful for testing or non-interactive environments)
            if std::env::var("COMMAND_VAULT_NO_TUI").is_ok() {
                for cmd in commands {
                    println!("{}: {} ({})", cmd.id.unwrap_or(0), cmd.command, cmd.directory);
                }
                return Ok(());
            }

            let mut app = App::new(commands.clone(), db);
            match app.run() {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Failed to start TUI mode: {}", e);
                    print_commands(&commands)?;
                }
            }
        }
        Commands::Tag { action } => match action {
            TagCommands::Add { command_id, tags } => {
                match db.add_tags_to_command(command_id, &tags) {
                    Ok(_) => println!("Tags added successfully"),
                    Err(e) => eprintln!("Failed to add tags: {}", e),
                }
            }
            TagCommands::Remove { command_id, tag } => {
                match db.remove_tag_from_command(command_id, &tag) {
                    Ok(_) => println!("Tag removed successfully"),
                    Err(e) => eprintln!("Failed to remove tag: {}", e),
                }
            }
            TagCommands::List => {
                match db.list_tags() {
                    Ok(tags) => {
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
        Commands::ShellInit { shell } => {
            let script_path = crate::shell::hooks::init_shell(shell)?;
            if !script_path.exists() {
                return Err(anyhow!("Shell integration script not found at: {}", script_path.display()));
            }
            println!("{}", script_path.display());
            return Ok(());
        },
    }
    Ok(())
}
