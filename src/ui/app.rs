use std::io::{self, Stdout};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
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

use crate::db::Command;

pub struct App {
    pub commands: Vec<Command>,
    pub selected: Option<usize>,
    pub show_help: bool,
}

impl App {
    pub fn new(commands: Vec<Command>) -> App {
        App {
            commands,
            selected: None,
            show_help: false,
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
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('?') => self.show_help = !self.show_help,
                    KeyCode::Down => {
                        if let Some(selected) = self.selected {
                            if selected < self.commands.len() - 1 {
                                self.selected = Some(selected + 1);
                            }
                        } else {
                            self.selected = Some(0);
                        }
                    }
                    KeyCode::Up => {
                        if let Some(selected) = self.selected {
                            if selected > 0 {
                                self.selected = Some(selected - 1);
                            }
                        } else {
                            self.selected = Some(0);
                        }
                    }
                    _ => {}
                }
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
                Constraint::Length(3),  // Status bar
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("Command Vault")
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Commands list
        let commands: Vec<ListItem> = self
            .commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
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

                let style = if Some(i) == self.selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let commands = List::new(commands)
            .block(Block::default().borders(Borders::ALL).title("Commands"))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_widget(commands, chunks[1]);

        // Status bar
        let status = if self.show_help {
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw(" to quit, "),
                Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                Span::raw(" to navigate, "),
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
        f.render_widget(status, chunks[2]);
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
