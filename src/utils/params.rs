use anyhow::Result;
use colored::*;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    QueueableCommand,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use regex::Regex;
use std::{collections::HashMap, io::{stdout, Write}};

use crate::db::models::Parameter;

pub fn parse_parameters(command: &str) -> Vec<Parameter> {
    let re = Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)(?::([^@\s]+))?").unwrap();
    let mut parameters = Vec::new();
    
    for cap in re.captures_iter(command) {
        let name = cap[1].to_string();
        let description = cap.get(2).map(|m| m.as_str().to_string());
        parameters.push(Parameter::with_description(name, description));
    }
    
    parameters
}

pub fn substitute_parameters(command: &str, parameters: &[Parameter], test_input: Option<&str>) -> Result<String> {
    let is_test = test_input.is_some() || std::env::var("COMMAND_VAULT_TEST").is_ok();
    if is_test {
        let mut final_command = command.to_string();
        let test_values: Vec<&str> = if let Some(input) = test_input {
            if input.is_empty() {
                parameters.iter()
                    .map(|p| p.description.as_deref().unwrap_or(""))
                    .collect()
            } else {
                input.split('\n').collect()
            }
        } else {
            vec!["test_value"; parameters.len()]
        };

        for (i, param) in parameters.iter().enumerate() {
            let value = if i < test_values.len() {
                test_values[i]
            } else {
                param.description.as_deref().unwrap_or("")
            };

            let needs_quotes = value.is_empty() || 
                             value.contains(' ') || 
                             value.contains('*') || 
                             value.contains(';') ||
                             value.contains('|') ||
                             value.contains('>') ||
                             value.contains('<') ||
                             final_command.starts_with("grep");

            let quoted_value = if needs_quotes && !value.starts_with('\'') && !value.starts_with('"') {
                format!("'{}'", value.replace('\'', "'\\''"))
            } else {
                value.to_string()
            };

            final_command = final_command.replace(&format!("@{}", param.name), &quoted_value);
            
            // Remove the description part from the command
            if let Some(desc) = &param.description {
                final_command = final_command.replace(&format!(":{}", desc), "");
            }
        }
        Ok(final_command)
    } else {
        prompt_parameters(command, parameters, test_input)
    }
}

pub fn prompt_parameters(command: &str, parameters: &[Parameter], test_input: Option<&str>) -> Result<String> {
    let is_test = test_input.is_some() || std::env::var("COMMAND_VAULT_TEST").is_ok();
    let result = (|| -> Result<String> {
        if !is_test {
            enable_raw_mode()?;
        }

        let mut stdout = stdout();
        let mut param_values = HashMap::new();

        for param in parameters {
            let value = if let Some(input) = test_input {
                input.to_string()
            } else if is_test {
                "test_value".to_string()
            } else {
                stdout.queue(Clear(ClearType::All))?;
                stdout.queue(MoveTo(0, 0))?
                      .queue(Print("─".repeat(45).dimmed()))?;
                stdout.queue(MoveTo(0, 1))?
                      .queue(Print(format!("{}: {}", 
                          "Parameter".blue().bold(), 
                          param.name.green()
                      )))?;
                if let Some(desc) = &param.description {
                    stdout.queue(MoveTo(0, 2))?
                          .queue(Print(format!("{}: {}", 
                              "Description".cyan().bold(), 
                              desc.white()
                          )))?;
                }
                stdout.queue(MoveTo(0, 3))?
                      .queue(Print(format!("{}: ", "Enter value".yellow().bold())))?;
                stdout.flush()?;

                let mut value = String::new();
                let mut cursor_pos = 0;

                loop {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => break,
                            KeyCode::Char(c) => {
                                value.insert(cursor_pos, c);
                                cursor_pos += 1;
                            }
                            KeyCode::Backspace if cursor_pos > 0 => {
                                value.remove(cursor_pos - 1);
                                cursor_pos -= 1;
                            }
                            KeyCode::Left if cursor_pos > 0 => {
                                cursor_pos -= 1;
                            }
                            KeyCode::Right if cursor_pos < value.len() => {
                                cursor_pos += 1;
                            }
                            _ => {}
                        }

                        // Redraw the value line
                        stdout.queue(MoveTo(0, 3))?
                              .queue(Clear(ClearType::CurrentLine))?
                              .queue(Print(format!("{}: {}", 
                                  "Enter value".yellow().bold(), 
                                  value
                              )))?;
                        stdout.queue(MoveTo((cursor_pos + 13) as u16, 3))?;
                        stdout.flush()?;
                    }
                }

                value
            };

            param_values.insert(param.name.clone(), value);
        }

        // Show final command info
        if !is_test {
            stdout.queue(Clear(ClearType::All))?;
            stdout.queue(MoveTo(0, 0))?
                  .queue(Print("─".repeat(45).dimmed()))?;
            stdout.queue(MoveTo(0, 1))?
                  .queue(Print(format!("{}: {}", 
                      "Command to execute".blue().bold(), 
                      command.green()
                  )))?;
            stdout.queue(MoveTo(0, 2))?
                  .queue(Print(format!("{}: {}", 
                      "Working directory".cyan().bold(), 
                      std::env::current_dir()?.to_string_lossy().white()
                  )))?;
            stdout.queue(MoveTo(0, 4))?;  // Add extra newline
            stdout.flush()?;
        }

        let mut final_command = command.to_string();
        for (name, value) in &param_values {
            // Quote value if it contains spaces or special characters
            let needs_quotes = value.is_empty() || 
                             value.contains(' ') || 
                             value.contains('*') || 
                             value.contains(';') ||
                             value.contains('|') ||
                             value.contains('>') ||
                             value.contains('<') ||
                             final_command.starts_with("grep");

            let quoted_value = if needs_quotes && !value.starts_with('\'') && !value.starts_with('"') {
                format!("'{}'", value.replace('\'', "'\\''"))
            } else {
                value.clone()
            };

            final_command = final_command.replace(&format!("@{}", name), &quoted_value);
        }

        Ok(final_command)
    })();

    // Always ensure raw mode is disabled and cursor is reset
    if !is_test {
        let _ = disable_raw_mode();
        let _ = stdout().queue(MoveTo(0, 0));
        let _ = stdout().flush();
    }

    result
}
