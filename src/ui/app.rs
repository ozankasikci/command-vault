use std::io::{self, Stdout};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use crate::db::{Command, Database};
use crate::ui::AddCommandApp;

pub struct App<'a> {
    pub commands: Vec<Command>,
    pub selected: Option<usize>,
    pub show_help: bool,
    pub message: Option<(String, Color)>,
    pub filter_text: String,
    pub filtered_commands: Vec<usize>,
    pub db: &'a mut Database,
}

impl<'a> App<'a> {
    pub fn new(commands: Vec<Command>, db: &'a mut Database) -> App<'a> {
        let filtered_commands: Vec<usize> = (0..commands.len()).collect();
        App {
            commands,
            selected: None,
            show_help: false,
            message: None,
            filter_text: String::new(),
            filtered_commands,
            db,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = setup_terminal()?;
        let res = self.run_app(&mut terminal);
        restore_terminal(&mut terminal)?;
        res
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('?') => {
                        self.show_help = !self.show_help;
                    }
                    KeyCode::Char('c') => {
                        if let Some(selected) = self.selected {
                            if let Some(&idx) = self.filtered_commands.get(selected) {
                                if let Some(cmd) = self.commands.get(idx) {
                                    copy_to_clipboard(&cmd.command)?;
                                    self.message = Some(("Command copied to clipboard!".to_string(), Color::Green));
                                }
                            }
                        }
                    }
                    KeyCode::Char('y') => {
                        if let Some(selected) = self.selected {
                            if let Some(&idx) = self.filtered_commands.get(selected) {
                                if let Some(cmd) = self.commands.get(idx) {
                                    copy_to_clipboard(&cmd.command)?;
                                    self.message = Some(("Command copied to clipboard!".to_string(), Color::Green));
                                }
                            }
                        }
                    }
                    KeyCode::Enter | KeyCode::Char('x') => {
                        if let Some(selected) = self.selected {
                            if let Some(&idx) = self.filtered_commands.get(selected) {
                                if let Some(cmd) = self.commands.get(idx) {
                                    // Exit TUI temporarily
                                    restore_terminal(terminal)?;
                                    
                                    // Execute the command
                                    let output = std::process::Command::new("sh")
                                        .arg("-c")
                                        .arg(&cmd.command)
                                        .status();

                                    match output {
                                        Ok(_) => return Ok(()),
                                        Err(e) => {
                                            // Re-enable TUI and show error
                                            setup_terminal()?;
                                            self.message = Some((format!("Failed to execute command: {}", e), Color::Red));
                                            continue;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Down => {
                        if let Some(selected) = self.selected {
                            if selected < self.filtered_commands.len() - 1 {
                                self.selected = Some(selected + 1);
                            }
                        } else if !self.filtered_commands.is_empty() {
                            self.selected = Some(0);
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = self.selected {
                            if selected > 0 {
                                self.selected = Some(selected - 1);
                            }
                        } else if !self.filtered_commands.is_empty() {
                            self.selected = Some(self.filtered_commands.len() - 1);
                        }
                    }
                    KeyCode::Char('/') => {
                        self.filter_text.clear();
                        self.message = Some(("Type to filter commands...".to_string(), Color::Blue));
                    }
                    KeyCode::Char('e') => {
                        if let Some(selected) = self.selected {
                            if let Some(&idx) = self.filtered_commands.get(selected) {
                                if let Some(cmd) = self.commands.get(idx).cloned() {
                                    // Exit TUI temporarily
                                    restore_terminal(terminal)?;
                                    
                                    // Create AddCommandApp with existing command data
                                    let mut add_app = AddCommandApp::new();
                                    add_app.set_command(cmd.command.clone());
                                    add_app.set_tags(cmd.tags.clone());
                                    add_app.set_exit_code(cmd.exit_code);
                                    
                                    // Run the add UI
                                    if let Ok(Some((new_command, new_tags, new_exit_code))) = add_app.run() {
                                        // Update command
                                        let updated_cmd = Command {
                                            id: cmd.id,
                                            command: new_command,
                                            timestamp: cmd.timestamp,
                                            directory: cmd.directory,
                                            exit_code: new_exit_code,
                                            tags: new_tags,
                                        };
                                        
                                        if let Err(e) = self.db.update_command(&updated_cmd) {
                                            setup_terminal()?;
                                            self.message = Some((format!("Failed to update command: {}", e), Color::Red));
                                        } else {
                                            // Update local command list
                                            if let Some(cmd) = self.commands.get_mut(idx) {
                                                *cmd = updated_cmd;
                                            }
                                            setup_terminal()?;
                                            self.message = Some(("Command updated successfully!".to_string(), Color::Green));
                                        }
                                    } else {
                                        setup_terminal()?;
                                    }
                                    continue;
                                }
                            }
                        }
                    }
                    KeyCode::Char('d') => {
                        if let Some(selected) = self.selected {
                            if let Some(&filtered_idx) = self.filtered_commands.get(selected) {
                                if let Some(command_id) = self.commands[filtered_idx].id {
                                    match self.db.delete_command(command_id) {
                                        Ok(_) => {
                                            self.commands.remove(filtered_idx);
                                            self.message = Some(("Command deleted successfully".to_string(), Color::Green));
                                            self.update_filtered_commands();
                                            if self.selected.unwrap() >= self.filtered_commands.len() {
                                                self.selected = if self.filtered_commands.is_empty() {
                                                    None
                                                } else {
                                                    Some(self.filtered_commands.len() - 1)
                                                };
                                            }
                                        }
                                        Err(e) => {
                                            self.message = Some((format!("Failed to delete command: {}", e), Color::Red));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if c == '/' {  // Skip if it's the '/' character that started filter mode
                            self.filter_text.clear();
                            self.message = Some(("Type to filter commands...".to_string(), Color::Blue));
                        } else if c != '/' {  // Skip if it's the '/' character that started filter mode
                            self.filter_text.push(c);
                            self.update_filtered_commands();
                        }
                    }
                    KeyCode::Backspace if !self.filter_text.is_empty() => {
                        self.filter_text.pop();
                        self.update_filtered_commands();
                    }
                    KeyCode::Esc => {
                        if !self.filter_text.is_empty() {
                            self.filter_text.clear();
                            self.update_filtered_commands();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn update_filtered_commands(&mut self) {
        let search_term = self.filter_text.to_lowercase();
        self.filtered_commands = (0..self.commands.len())
            .filter(|&i| {
                let cmd = &self.commands[i];
                cmd.command.to_lowercase().contains(&search_term) ||
                cmd.tags.iter().any(|tag| tag.to_lowercase().contains(&search_term))
            })
            .collect();
        
        // Update selection
        if self.filtered_commands.is_empty() {
            self.selected = None;
        } else if let Some(selected) = self.selected {
            if selected >= self.filtered_commands.len() {
                self.selected = Some(self.filtered_commands.len() - 1);
            }
        }
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(0),     // Commands list
                Constraint::Length(1),  // Filter
                Constraint::Length(3),  // Status bar
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Command Vault")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Commands list
        let commands: Vec<ListItem> = self.filtered_commands.iter()
            .map(|&i| {
                let cmd = &self.commands[i];
                let local_time = cmd.timestamp.with_timezone(&chrono::Local);
                let time_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();
                
                let mut spans = vec![
                    Span::styled(
                        format!("({}) ", cmd.id.unwrap_or(0)),
                        Style::default().fg(Color::DarkGray)
                    ),
                    Span::styled(
                        format!("[{}] ", time_str),
                        Style::default().fg(Color::Yellow)
                    ),
                    Span::raw(&cmd.command),
                ];

                if let Some(code) = cmd.exit_code {
                    if code != 0 {
                        spans.push(Span::styled(
                            format!(" [{}]", code),
                            Style::default().fg(Color::Red)
                        ));
                    }
                }

                if !cmd.tags.is_empty() {
                    spans.push(Span::raw(" "));
                    for tag in &cmd.tags {
                        spans.push(Span::styled(
                            format!("#{} ", tag),
                            Style::default().fg(Color::Green)
                        ));
                    }
                }

                ListItem::new(Line::from(spans))
            })
            .collect();

        let commands = List::new(commands)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        
        let commands_state = self.selected.map(|i| {
            let mut state = ratatui::widgets::ListState::default();
            state.select(Some(i));
            state
        });

        if let Some(state) = commands_state {
            f.render_stateful_widget(commands, chunks[1], &mut state.clone());
        } else {
            f.render_widget(commands, chunks[1]);
        }

        // Filter
        if !self.filter_text.is_empty() {
            let filter = Paragraph::new(format!("Filter: {}", self.filter_text))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(filter, chunks[2]);
        }

        // Status bar with help text or message
        let status = if let Some((msg, color)) = &self.message {
            vec![Span::styled(msg, Style::default().fg(*color))]
        } else if self.show_help {
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" to quit, "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" to navigate, "),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw(" or "),
                Span::styled("y", Style::default().fg(Color::Yellow)),
                Span::raw(" to copy, "),
                Span::styled("e", Style::default().fg(Color::Yellow)),
                Span::raw(" to edit, "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(" to delete, "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(" or "),
                Span::styled("x", Style::default().fg(Color::Yellow)),
                Span::raw(" to execute, "),
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw(" to filter, "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" to toggle help"),
            ]
        } else {
            vec![
                Span::raw("Press "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" for help"),
            ]
        };

        let status = Paragraph::new(Line::from(status))
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
        f.render_widget(status, chunks[3]);
    }
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

fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }
        
        child.wait()?;
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let mut child = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }
        
        child.wait()?;
    }
    
    Ok(())
}
