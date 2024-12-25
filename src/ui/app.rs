use std::io::{self, Stdout};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use crate::db::{Command, Database};
use crate::utils::params::{substitute_parameters, parse_parameters};
use crate::exec::{ExecutionContext, execute_shell_command};
use crate::ui::AddCommandApp;

pub struct App<'a> {
    pub commands: Vec<Command>,
    pub selected: Option<usize>,
    pub show_help: bool,
    pub message: Option<(String, Color)>,
    pub filter_text: String,
    pub filtered_commands: Vec<usize>,
    pub db: &'a mut Database,
    pub confirm_delete: Option<usize>, // Index of command pending deletion
    pub debug_mode: bool,
}

impl<'a> App<'a> {
    pub fn new(commands: Vec<Command>, db: &'a mut Database, debug_mode: bool) -> App<'a> {
        let filtered_commands: Vec<usize> = (0..commands.len()).collect();
        App {
            commands,
            selected: None,
            show_help: false,
            message: None,
            filter_text: String::new(),
            filtered_commands,
            db,
            confirm_delete: None,
            debug_mode,
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
                        if !self.filter_text.is_empty() {
                            self.filter_text.clear();
                            self.update_filtered_commands();
                        } else if self.confirm_delete.is_some() {
                            self.confirm_delete = None;
                        } else if self.show_help {
                            self.show_help = false;
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('?') => {
                        self.show_help = !self.show_help;
                        continue; // Skip further processing when toggling help
                    }
                    _ if self.show_help => {
                        // If help is shown, ignore all other keys except those above
                        continue;
                    }
                    KeyCode::Char('c') => {
                        if let Some(cmd) = self.get_selected_command() {
                            copy_to_clipboard(&cmd.command)?;
                            self.set_success_message("Command copied to clipboard!".to_string());
                        }
                    }
                    KeyCode::Char('y') => {
                        if let Some(cmd) = self.get_selected_command() {
                            copy_to_clipboard(&cmd.command)?;
                            self.set_success_message("Command copied to clipboard!".to_string());
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(selected) = self.get_selection() {
                            if let Some(confirm_idx) = self.confirm_delete {
                                if confirm_idx == selected {
                                    if let Some(idx) = self.get_selected_index() {
                                        if let Some(command_id) = self.commands[idx].id {
                                            match self.db.delete_command(command_id) {
                                                Ok(_) => {
                                                    self.commands.remove(idx);
                                                    self.set_success_message("Command deleted successfully".to_string());
                                                    self.update_filtered_commands();
                                                    self.update_selection_after_delete(idx);
                                                }
                                                Err(e) => {
                                                    self.set_error_message(format!("Failed to delete command: {}", e));
                                                }
                                            }
                                            self.confirm_delete = None;
                                        }
                                    }
                                }
                            } else if let Some(cmd) = self.get_selected_command() {
                                // Exit TUI temporarily
                                restore_terminal(terminal)?;
                                
                                // Re-enable colors after restoring terminal
                                colored::control::set_override(true);

                                // If command has parameters, substitute them with user input
                                let current_params = parse_parameters(&cmd.command);
                                let final_command = substitute_parameters(&cmd.command, &current_params, None)?;
                                let ctx = ExecutionContext {
                                    command: final_command,
                                    directory: cmd.directory.clone(),
                                    test_mode: false,
                                    debug_mode: self.debug_mode,
                                };
                                execute_shell_command(&ctx)?;
                                
                                return Ok(());
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.select_next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.select_previous();
                    }
                    KeyCode::Char('/') => {
                        self.clear_filter();
                        self.set_message("Type to filter commands...".to_string(), Color::Blue);
                    }
                    KeyCode::Char('e') => {
                        if let Some(selected) = self.get_selection() {
                            if let Some(&idx) = self.filtered_commands.get(selected) {
                                if let Some(cmd) = self.commands.get(idx).cloned() {
                                    // Exit TUI temporarily
                                    restore_terminal(terminal)?;
                                    
                                    // Create AddCommandApp with existing command data
                                    let mut add_app = AddCommandApp::new();
                                    add_app.set_command(cmd.command.clone());
                                    add_app.set_tags(cmd.tags.clone());
                                    
                                    let result = add_app.run();
                                    
                                    // Re-initialize terminal and force redraw
                                    let mut new_terminal = setup_terminal()?;
                                    new_terminal.clear()?;
                                    *terminal = new_terminal;
                                    terminal.draw(|f| self.ui(f))?;
                                    
                                    match result {
                                        Ok(Some((new_command, new_tags, _))) => {
                                            // Update command
                                            let updated_cmd = Command {
                                                id: cmd.id,
                                                command: new_command.clone(),
                                                timestamp: cmd.timestamp,
                                                directory: cmd.directory.clone(),
                                                tags: new_tags,
                                                parameters: crate::utils::params::parse_parameters(&new_command),
                                            };
                                            
                                            if let Err(e) = self.db.update_command(&updated_cmd) {
                                                self.set_error_message(format!("Failed to update command: {}", e));
                                            } else {
                                                // Update local command list
                                                if let Some(cmd) = self.commands.get_mut(idx) {
                                                    *cmd = updated_cmd;
                                                }
                                                self.set_success_message("Command updated successfully!".to_string());
                                            }
                                        }
                                        Ok(None) => {
                                            self.set_message("Edit cancelled".to_string(), Color::Yellow);
                                        }
                                        Err(e) => {
                                            self.set_error_message(format!("Error during edit: {}", e));
                                        }
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    KeyCode::Char('d') => {
                        if let Some(selected) = self.get_selection() {
                            if let Some(&filtered_idx) = self.filtered_commands.get(selected) {
                                if let Some(command_id) = self.commands[filtered_idx].id {
                                    self.confirm_delete = Some(selected);
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        if c == '/' {  // Skip if it's the '/' character that started filter mode
                            self.clear_filter();
                            self.set_message("Type to filter commands...".to_string(), Color::Blue);
                        } else if c != '/' {  // Skip if it's the '/' character that started filter mode
                            self.append_to_filter(c);
                        }
                    }
                    KeyCode::Backspace => {
                        self.backspace_filter();
                    }
                    KeyCode::Esc => {
                        if !self.filter_text.is_empty() {
                            self.clear_filter();
                        } else if self.confirm_delete.is_some() {
                            self.confirm_delete = None;
                            self.set_message("Delete operation cancelled".to_string(), Color::Yellow);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.update_filtered_commands();
    }

    pub fn set_filter(&mut self, text: String) {
        self.filter_text = text;
        self.update_filtered_commands();
    }

    pub fn append_to_filter(&mut self, c: char) {
        self.filter_text.push(c);
        self.update_filtered_commands();
    }

    pub fn backspace_filter(&mut self) {
        if !self.filter_text.is_empty() {
            self.filter_text.pop();
            self.update_filtered_commands();
        }
    }

    fn matches_filter(&self, command: &Command, search_term: &str) -> bool {
        let search_term = search_term.to_lowercase();
        command.command.to_lowercase().contains(&search_term) ||
        command.tags.iter().any(|tag| tag.to_lowercase().contains(&search_term)) ||
        command.directory.to_lowercase().contains(&search_term)
    }

    pub fn update_filtered_commands(&mut self) {
        self.filtered_commands = (0..self.commands.len())
            .filter(|&i| self.matches_filter(&self.commands[i], &self.filter_text))
            .collect::<Vec<usize>>();
        
        self.update_selection_after_filter();
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        if self.show_help {
            let help_text = vec![
                "Command Vault Help",
                "",
                "Navigation:",
                "  ↑/k      - Move cursor up",
                "  ↓/j      - Move cursor down",
                "  q        - Quit (or clear filter/cancel delete/close help)",
                "  Ctrl+c   - Force quit",
                "",
                "Command Actions:",
                "  Enter    - Execute selected command",
                "  c/y      - Copy command to clipboard",
                "  e        - Edit selected command (text, tags, directory)",
                "  d        - Delete selected command (requires confirmation)",
                "",
                "Search and Filter:",
                "  /        - Start filtering commands",
                "  [type]   - Filter by command text, tags, or directory",
                "  Esc      - Clear filter or cancel current operation",
                "  Backspace- Remove last character from filter",
                "",
                "Display:",
                "  ?        - Toggle this help screen",
                "",
                "Command Format:",
                "  - (@param) Parameters are shown with @ prefix",
                "  - (#tag)  Tags are shown in green with # prefix",
                "  - (dir)   Working directory is shown if set",
                "  - (id)    Command IDs are shown in parentheses",
                "",
                "Tips:",
                "  - Use descriptive tags to organize commands",
                "  - Parameters (@param) allow dynamic input",
                "  - Filter works on commands, tags, and directories",
                "  - Working directory affects command execution",
                "",
                "Note:",
                "  - Debug mode can be enabled for troubleshooting",
                "  - All commands are executed in the current shell",
                "  - Command history is preserved in the database"
            ];

            let help_paragraph = Paragraph::new(help_text.join("\n"))
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Help (press ? to close)"));

            // Center the help window
            let area = centered_rect(80, 80, f.size());
            f.render_widget(Clear, area); // Clear the background
            f.render_widget(help_paragraph, area);
            return;
        }

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
                Span::styled("↑↓/jk", Style::default().fg(Color::Yellow)),
                Span::raw(" to navigate, "),
                Span::styled("c", Style::default().fg(Color::Yellow)),
                Span::raw(" or "),
                Span::styled("y", Style::default().fg(Color::Yellow)),
                Span::raw(" to copy, "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" for help"),
            ]
        } else {
            vec![
                Span::raw("Press "),
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw(" for help"),
            ]
        };

        let status = Paragraph::new(Line::from(status))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[3]);

        // Render delete confirmation dialog if needed
        if let Some(idx) = self.confirm_delete {
            if let Some(&cmd_idx) = self.filtered_commands.get(idx) {
                if let Some(cmd) = self.commands.get(cmd_idx) {
                    let command_str = format!("Command: {}", cmd.command);
                    let id_str = format!("ID: {}", cmd.id.unwrap_or(0));
                    
                    let dialog_text = vec![
                        "Are you sure you want to delete this command?",
                        "",
                        &command_str,
                        &id_str,
                        "",
                        "Press Enter to confirm or Esc to cancel",
                    ];

                    let dialog = Paragraph::new(dialog_text.join("\n"))
                        .style(Style::default().fg(Color::White))
                        .block(Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Red))
                            .title("Confirm Delete"));

                    // Center the dialog
                    let area = centered_rect(60, 40, f.size());
                    f.render_widget(Clear, area);
                    f.render_widget(dialog, area);
                }
            }
        }
    }

    pub fn set_message(&mut self, text: String, color: Color) {
        self.message = Some((text, color));
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn set_success_message(&mut self, text: String) {
        self.set_message(text, Color::Green);
    }

    pub fn set_error_message(&mut self, text: String) {
        self.set_message(text, Color::Red);
    }

    pub fn get_selection(&self) -> Option<usize> {
        self.selected
    }

    pub fn set_selection(&mut self, index: Option<usize>) {
        if let Some(idx) = index {
            if idx < self.filtered_commands.len() {
                self.selected = Some(idx);
            }
        } else {
            self.selected = None;
        }
    }

    pub fn update_selection_after_filter(&mut self) {
        if self.filtered_commands.is_empty() {
            self.selected = None;
        } else if let Some(selected) = self.selected {
            if selected >= self.filtered_commands.len() {
                self.selected = Some(self.filtered_commands.len() - 1);
            }
        }
    }

    pub fn update_selection_after_delete(&mut self, deleted_index: usize) {
        if self.filtered_commands.is_empty() {
            self.selected = None;
        } else if let Some(selected) = self.selected {
            self.selected = Some(selected.min(self.filtered_commands.len() - 1));
        }
    }

    pub fn select_next(&mut self) {
        if let Some(selected) = self.selected {
            if selected < self.filtered_commands.len() - 1 {
                self.selected = Some(selected + 1);
            }
        } else if !self.filtered_commands.is_empty() {
            self.selected = Some(0);
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(selected) = self.selected {
            if selected > 0 {
                self.selected = Some(selected - 1);
            }
        } else if !self.filtered_commands.is_empty() {
            self.selected = Some(self.filtered_commands.len() - 1);
        }
    }

    pub fn get_selected_command(&self) -> Option<&Command> {
        self.selected
            .and_then(|selected| self.filtered_commands.get(selected))
            .and_then(|&idx| self.commands.get(idx))
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected
            .and_then(|selected| self.filtered_commands.get(selected))
            .copied()
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Calculate popup size based on percentage of screen size
    let popup_width = (r.width as f32 * (percent_x as f32 / 100.0)) as u16;
    let popup_height = (r.height as f32 * (percent_y as f32 / 100.0)) as u16;

    // Calculate popup position to center it
    let popup_x = ((r.width - popup_width) / 2) + r.x;
    let popup_y = ((r.height - popup_height) / 2) + r.y;

    Rect::new(popup_x, popup_y, popup_width, popup_height)
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
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
    colored::control::set_override(true);
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
