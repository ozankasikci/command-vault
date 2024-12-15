use std::io::Stdout;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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

/// Type alias for the command result tuple
pub type CommandResult = Option<(String, Vec<String>, Option<i32>)>;

#[derive(Default)]
pub struct AddCommandApp {
    /// The command being entered
    command: String,
    /// Current cursor position in the command
    command_cursor: usize,
    /// Current line in multi-line command
    command_line: usize,
    /// Tags for the command
    tags: Vec<String>,
    /// Current tag being entered
    current_tag: String,
    /// Current input mode
    input_mode: InputMode,
    /// Suggested tags
    suggested_tags: Vec<String>,
    /// Previous input mode (for returning from help)
    previous_mode: InputMode,
}

#[derive(Default, PartialEq, Clone)]
enum InputMode {
    #[default]
    Command,
    Tag,
    Confirm,
    Help,
}

impl AddCommandApp {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self) -> Result<CommandResult> {
        let mut terminal = setup_terminal()?;
        let result = self.run_app(&mut terminal);
        restore_terminal(&mut terminal)?;
        result
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<CommandResult> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Help => match key.code {
                        KeyCode::Char('?') | KeyCode::Esc => {
                            self.input_mode = self.previous_mode.clone();
                        }
                        _ => {}
                    },
                    _ => match key.code {
                        KeyCode::Char('?') => {
                            eprintln!("Debug: ? key pressed, switching to help mode");
                            self.previous_mode = self.input_mode.clone();
                            self.input_mode = InputMode::Help;
                            eprintln!("Debug: Input mode is now Help");
                        }
                        _ => match self.input_mode {
                            InputMode::Command => match key.code {
                                KeyCode::Enter => {
                                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                                        // Add newline to command
                                        self.command.insert(self.command_cursor, '\n');
                                        self.command_cursor += 1;
                                        self.command_line += 1;
                                    } else {
                                        if !self.command.is_empty() {
                                            self.suggest_tags();
                                            self.input_mode = InputMode::Tag;
                                        }
                                    }
                                }
                                KeyCode::Char(c) => {
                                    self.command.insert(self.command_cursor, c);
                                    self.command_cursor += 1;
                                }
                                KeyCode::Backspace => {
                                    if self.command_cursor > 0 {
                                        self.command.remove(self.command_cursor - 1);
                                        self.command_cursor -= 1;
                                        if self.command_cursor > 0 && self.command.chars().nth(self.command_cursor - 1) == Some('\n') {
                                            self.command_line -= 1;
                                        }
                                    }
                                }
                                KeyCode::Left => {
                                    if self.command_cursor > 0 {
                                        self.command_cursor -= 1;
                                        if self.command_cursor > 0 && self.command.chars().nth(self.command_cursor - 1) == Some('\n') {
                                            self.command_line -= 1;
                                        }
                                    }
                                }
                                KeyCode::Right => {
                                    if self.command_cursor < self.command.len() {
                                        if self.command.chars().nth(self.command_cursor) == Some('\n') {
                                            self.command_line += 1;
                                        }
                                        self.command_cursor += 1;
                                    }
                                }
                                KeyCode::Up => {
                                    // Move cursor to previous line
                                    let current_line_start = self.command[..self.command_cursor]
                                        .rfind('\n')
                                        .map(|pos| pos + 1)
                                        .unwrap_or(0);
                                    if let Some(prev_line_start) = self.command[..current_line_start.saturating_sub(1)]
                                        .rfind('\n')
                                        .map(|pos| pos + 1) {
                                        let column = self.command_cursor - current_line_start;
                                        self.command_cursor = prev_line_start + column.min(
                                            self.command[prev_line_start..current_line_start.saturating_sub(1)]
                                                .chars()
                                                .count(),
                                        );
                                        self.command_line -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    // Move cursor to next line
                                    let current_line_start = self.command[..self.command_cursor]
                                        .rfind('\n')
                                        .map(|pos| pos + 1)
                                        .unwrap_or(0);
                                    if let Some(next_line_start) = self.command[self.command_cursor..]
                                        .find('\n')
                                        .map(|pos| self.command_cursor + pos + 1) {
                                        let column = self.command_cursor - current_line_start;
                                        let next_line_end = self.command[next_line_start..]
                                            .find('\n')
                                            .map(|pos| next_line_start + pos)
                                            .unwrap_or_else(|| self.command.len());
                                        self.command_cursor = next_line_start + column.min(next_line_end - next_line_start);
                                        self.command_line += 1;
                                    }
                                }
                                KeyCode::Esc => {
                                    return Ok(None);
                                }
                                _ => {}
                            },
                            InputMode::Tag => {
                                match key.code {
                                    KeyCode::Enter => {
                                        if !self.current_tag.is_empty() {
                                            self.tags.push(self.current_tag.clone());
                                            self.current_tag.clear();
                                        } else {
                                            self.input_mode = InputMode::Confirm;
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
                                        self.input_mode = InputMode::Command;
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::Confirm => {
                                match key.code {
                                    KeyCode::Char('y') => {
                                        return Ok(Some((
                                            self.command.clone(),
                                            self.tags.clone(),
                                            None,
                                        )));
                                    }
                                    KeyCode::Char('n') | KeyCode::Esc => {
                                        return Ok(None);
                                    }
                                    _ => {}
                                }
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
        self.command_cursor = self.command.len();
    }

    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
    }

    fn ui(&self, f: &mut ratatui::Frame) {
        match self.input_mode {
            InputMode::Help => {
                let help_text = vec![
                    "Command Vault Help",
                    "",
                    "Global Commands:",
                    "  ?      - Toggle this help screen",
                    "  Esc    - Go back / Cancel",
                    "",
                    "Command Input Mode:",
                    "  Enter        - Continue to tag input",
                    "  Shift+Enter  - Add new line",
                    "  ←/→         - Move cursor",
                    "  ↑/↓         - Navigate between lines",
                    "",
                    "Tag Input Mode:",
                    "  Enter  - Add tag",
                    "  Tab    - Show tag suggestions",
                    "",
                    "Confirmation Mode:",
                    "  y/Y    - Save command",
                    "  n/N    - Cancel",
                ];

                let help_paragraph = Paragraph::new(help_text.join("\n"))
                    .style(Style::default().fg(Color::White))
                    .block(Block::default().borders(Borders::ALL).title("Help (press ? or Esc to close)"));

                // Center the help window
                let area = centered_rect(60, 80, f.size());
                f.render_widget(Clear, area); // Clear the background
                f.render_widget(help_paragraph, area);
            }
            _ => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),  // Title
                        Constraint::Min(5),     // Command input
                        Constraint::Length(3),  // Tags input
                        Constraint::Min(0),     // Message/Help
                    ])
                    .split(f.size());

                // Title
                let title = Paragraph::new("Add Command")
                    .style(Style::default().fg(Color::Cyan))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                // Command input
                let mut command_text = self.command.clone();
                if self.input_mode == InputMode::Command {
                    command_text.insert(self.command_cursor, '│'); // Add cursor
                }
                let command_input = Paragraph::new(command_text)
                    .style(Style::default().fg(if self.input_mode == InputMode::Command {
                        Color::Yellow
                    } else {
                        Color::Gray
                    }))
                    .block(Block::default().borders(Borders::ALL).title("Command (Shift+Enter for new line)"))
                    .wrap(ratatui::widgets::Wrap { trim: false });
                f.render_widget(command_input, chunks[1]);

                // Tags input
                let mut tags_text = self.tags.join(", ");
                if !tags_text.is_empty() {
                    tags_text.push_str(", ");
                }
                tags_text.push_str(&self.current_tag);
                if self.input_mode == InputMode::Tag {
                    tags_text.push('│');
                }
                let tags_input = Paragraph::new(tags_text)
                    .style(Style::default().fg(if self.input_mode == InputMode::Tag {
                        Color::Yellow
                    } else {
                        Color::Gray
                    }))
                    .block(Block::default().borders(Borders::ALL).title("Tags"));
                f.render_widget(tags_input, chunks[2]);

                // Help text or confirmation prompt
                let help_text = match self.input_mode {
                    InputMode::Command => "Press ? for help",
                    InputMode::Tag => "Press ? for help",
                    InputMode::Confirm => "Save command? (y/n)",
                    InputMode::Help => unreachable!(),
                };
                let help = Paragraph::new(help_text)
                    .style(Style::default().fg(Color::White))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(help, chunks[3]);
            }
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
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.show_cursor()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
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
