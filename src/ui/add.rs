use std::io::{self, Stdout};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Clear},
    Terminal,
};

pub struct AddCommandApp {
    command: String,
    tags: Vec<String>,
    current_tag: String,
    exit_code: Option<i32>,
    cursor_position: usize,
    mode: Mode,
    suggested_tags: Vec<String>,
    message: Option<String>,
}

#[derive(PartialEq)]
enum Mode {
    Command,
    Tags,
    ExitCode,
    Confirm,
}

impl AddCommandApp {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            tags: Vec::new(),
            current_tag: String::new(),
            exit_code: None,
            cursor_position: 0,
            mode: Mode::Command,
            suggested_tags: Vec::new(),
            message: None,
        }
    }

    pub fn run(&mut self) -> Result<Option<(String, Vec<String>, Option<i32>)>> {
        let mut terminal = setup_terminal()?;
        let result = self.run_app(&mut terminal);
        restore_terminal(&mut terminal)?;
        result
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<Option<(String, Vec<String>, Option<i32>)>> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.mode {
                    Mode::Command => {
                        match key.code {
                            KeyCode::Enter => {
                                if !self.command.is_empty() {
                                    self.suggest_tags();
                                    self.mode = Mode::Tags;
                                }
                            }
                            KeyCode::Char(c) => {
                                self.command.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if self.cursor_position > 0 {
                                    self.command.remove(self.cursor_position - 1);
                                    self.cursor_position -= 1;
                                }
                            }
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.command.len() {
                                    self.cursor_position += 1;
                                }
                            }
                            KeyCode::Esc => {
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }
                    Mode::Tags => {
                        match key.code {
                            KeyCode::Enter => {
                                if !self.current_tag.is_empty() {
                                    self.tags.push(self.current_tag.clone());
                                    self.current_tag.clear();
                                } else {
                                    self.mode = Mode::ExitCode;
                                }
                            }
                            KeyCode::Char(c) => {
                                self.current_tag.push(c);
                            }
                            KeyCode::Backspace => {
                                self.current_tag.pop();
                            }
                            KeyCode::Tab => {
                                if !self.suggested_tags.is_empty() {
                                    self.tags.push(self.suggested_tags[0].clone());
                                    self.suggested_tags.remove(0);
                                }
                            }
                            KeyCode::Esc => {
                                self.mode = Mode::Command;
                            }
                            _ => {}
                        }
                    }
                    Mode::ExitCode => {
                        match key.code {
                            KeyCode::Enter => {
                                self.mode = Mode::Confirm;
                            }
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                let digit = c.to_digit(10).unwrap() as i32;
                                self.exit_code = Some(self.exit_code.unwrap_or(0) * 10 + digit);
                            }
                            KeyCode::Backspace => {
                                if let Some(code) = self.exit_code {
                                    self.exit_code = Some(code / 10);
                                    if self.exit_code == Some(0) {
                                        self.exit_code = None;
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                self.mode = Mode::Tags;
                            }
                            _ => {}
                        }
                    }
                    Mode::Confirm => {
                        match key.code {
                            KeyCode::Char('y') => {
                                return Ok(Some((
                                    self.command.clone(),
                                    self.tags.clone(),
                                    self.exit_code,
                                )));
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub fn set_command(&mut self, command: String) {
        self.command = command;
        self.cursor_position = self.command.len();
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    pub fn set_exit_code(&mut self, exit_code: Option<i32>) {
        self.exit_code = exit_code;
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(3),  // Command input
                Constraint::Length(3),  // Tags input
                Constraint::Length(3),  // Exit code input
                Constraint::Min(0),     // Message/Help
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Add Command")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Command input
        let command_input = Paragraph::new(format!(
            "{}\n{}",
            self.command,
            if self.mode == Mode::Command { "│" } else { "" }
        ))
        .style(Style::default().fg(if self.mode == Mode::Command { Color::Yellow } else { Color::Gray }))
        .block(Block::default().borders(Borders::ALL).title("Command"));
        f.render_widget(command_input, chunks[1]);

        // Tags input
        let mut tags_text = self.tags.join(", ");
        if !tags_text.is_empty() {
            tags_text.push_str(", ");
        }
        tags_text.push_str(&self.current_tag);
        if self.mode == Mode::Tags {
            tags_text.push('│');
        }
        let tags_input = Paragraph::new(tags_text)
            .style(Style::default().fg(if self.mode == Mode::Tags { Color::Yellow } else { Color::Gray }))
            .block(Block::default().borders(Borders::ALL).title("Tags"));
        f.render_widget(tags_input, chunks[2]);

        // Exit code input
        let exit_code_text = self.exit_code.map_or_else(String::new, |c| c.to_string());
        let exit_code_input = Paragraph::new(format!(
            "{}\n{}",
            exit_code_text,
            if self.mode == Mode::ExitCode { "│" } else { "" }
        ))
        .style(Style::default().fg(if self.mode == Mode::ExitCode { Color::Yellow } else { Color::Gray }))
        .block(Block::default().borders(Borders::ALL).title("Exit Code"));
        f.render_widget(exit_code_input, chunks[3]);

        // Help text or confirmation prompt
        let help_text = match self.mode {
            Mode::Command => "Enter command (Enter to continue, Esc to cancel)",
            Mode::Tags => "Enter tags (Enter when done, Tab for suggestions, Esc to go back)",
            Mode::ExitCode => "Enter exit code (Enter to continue, Esc to go back)",
            Mode::Confirm => "Save command? (y/n)",
        };
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[4]);

        // Show suggested tags if in tag mode
        if self.mode == Mode::Tags && !self.suggested_tags.is_empty() {
            let area = centered_rect(60, 30, f.size());
            let suggested_tags = Paragraph::new(format!("Suggested tags:\n{}", self.suggested_tags.join(", ")))
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Suggestions"));
            f.render_widget(Clear, area);
            f.render_widget(suggested_tags, area);
        }
    }

    fn suggest_tags(&mut self) {
        self.suggested_tags.clear();
        
        // Simple tag suggestions based on command content
        let command = self.command.to_lowercase();
        
        if command.contains("git") {
            self.suggested_tags.push("git".to_string());
            if command.contains("push") {
                self.suggested_tags.push("push".to_string());
            }
            if command.contains("pull") {
                self.suggested_tags.push("pull".to_string());
            }
        }
        
        if command.contains("docker") {
            self.suggested_tags.push("docker".to_string());
        }
        
        if command.contains("cargo") {
            self.suggested_tags.push("rust".to_string());
            self.suggested_tags.push("cargo".to_string());
        }
        
        if command.contains("npm") || command.contains("yarn") {
            self.suggested_tags.push("javascript".to_string());
            self.suggested_tags.push("node".to_string());
        }
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
