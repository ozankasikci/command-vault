use crate::db::models::Parameter;
use regex::Regex;
use anyhow::{Result, anyhow};
use std::io::Write;
use colored::*;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode, KeyModifiers},
    QueueableCommand,
    style::Print,
};
use std::collections::HashMap;

const HEADER_LINE: u16 = 0;
const SEPARATOR_LINE: u16 = 1;
const PARAM_LINE: u16 = 3;
const DEFAULT_LINE: u16 = 3;
const INPUT_LINE: u16 = 4;
const PREVIEW_SEPARATOR_LINE: u16 = 6;
const COMMAND_LINE: u16 = 7;
const WORKDIR_LINE: u16 = 8;

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
    let mut final_command = command.to_string();
    let is_test = std::env::var("COMMAND_VAULT_TEST").is_ok();
    
    // In test mode, use provided test input or default test values
    if is_test {
        let test_values: Vec<&str> = test_input.map(|s| s.split('\n').collect()).unwrap_or_default();

        for (index, param) in parameters.iter().enumerate() {
            let value = if let Some(test_mode) = test_input {
                if test_values.len() > index {
                    let value = test_values[index].to_string();
                    if value.is_empty() && param.description.is_some() {
                        param.description.as_ref().unwrap().clone()
                    } else {
                        value
                    }
                } else if test_mode.is_empty() {
                    param.description.as_ref().map(|d| d.clone()).unwrap_or_default()
                } else {
                    "test_value".to_string()
                }
            } else {
                "test_value".to_string()
            };

            // Quote value if it contains spaces, special characters, or if it's part of a grep command
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
                value
            };

            // Replace parameter placeholders with the value
            if let Some(desc) = &param.description {
                final_command = final_command.replace(&format!("@{}:{}", param.name, desc), &quoted_value);
            }
            final_command = final_command.replace(&format!("@{}", param.name), &quoted_value);
        }
        return Ok(final_command);
    }

    // Interactive mode for non-test environment
    let mut stdout = std::io::stdout();
    let mut param_values: HashMap<String, String> = HashMap::new();
    
    // Only enable raw mode if not in test mode
    if !is_test {
        enable_raw_mode()?;
    }
    
    let result = (|| -> Result<String> {
        let mut stdout = std::io::stdout();
        let mut param_values: HashMap<String, String> = HashMap::new();
        
        // Only show UI in non-test mode
        if !is_test {
            // Clear screen and show header
            stdout.queue(Clear(ClearType::All))?;
            stdout.queue(MoveTo(0, HEADER_LINE))?
                  .queue(Print("Enter values for command parameters:"))?;
            stdout.queue(MoveTo(0, SEPARATOR_LINE))?
                  .queue(Print("─".repeat(45).dimmed()))?;
            stdout.flush()?;
        }

        for param in parameters {
            let desc = param.description.as_deref().unwrap_or("");
            let mut input = String::new();
            let mut cursor_pos = 0;

            // Only show UI and handle input in non-test mode
            if !is_test {
                loop {
                    // Parameter info
                    stdout.queue(MoveTo(0, PARAM_LINE))?
                          .queue(Print(format!("{}: {}", "Parameter".blue().bold(), param.name.yellow())))?;
                    if !desc.is_empty() {
                        stdout.queue(Print(format!(" - {}", desc.dimmed())))?;
                    }

                    // Input field
                    stdout.queue(MoveTo(0, INPUT_LINE))?
                          .queue(Print(format!("{}: {}", "Enter value".dimmed(), input)))?;

                    // Preview section
                    let mut preview_command = command.to_string();
                    for (param_name, value) in &param_values {
                        preview_command = preview_command.replace(&format!("@{}", param_name), value);
                    }
                    if !input.is_empty() {
                        preview_command = preview_command.replace(&format!("@{}", param.name), &input);
                    }

                    // Bottom separator
                    stdout.queue(MoveTo(0, PREVIEW_SEPARATOR_LINE))?
                          .queue(Print("─".repeat(45).dimmed()))?;

                    // Command preview section with softer colors
                    stdout.queue(MoveTo(0, COMMAND_LINE))?
                          .queue(Print(format!("{}: {}", 
                              "Command to execute".blue().bold(), 
                              preview_command.green()
                          )))?;

                    // Working directory with softer colors
                    stdout.queue(MoveTo(0, WORKDIR_LINE))?
                          .queue(Print(format!("{}: {}", 
                              "Working directory".cyan().bold(), 
                              std::env::current_dir()?.to_string_lossy().white()
                          )))?;

                    // Position cursor at input
                    stdout.queue(MoveTo(
                        ("Enter value: ".len() + cursor_pos) as u16,
                        INPUT_LINE
                    ))?;
                    
                    stdout.flush()?;

                    if let Event::Key(key) = event::read()? {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                return Err(anyhow!("Operation cancelled by user"));
                            },
                            (KeyCode::Enter, _) => break,
                            (KeyCode::Char(c), _) => {
                                input.insert(cursor_pos, c);
                                cursor_pos += 1;
                            }
                            (KeyCode::Backspace, _) if cursor_pos > 0 => {
                                input.remove(cursor_pos - 1);
                                cursor_pos -= 1;
                            }
                            (KeyCode::Left, _) if cursor_pos > 0 => {
                                cursor_pos -= 1;
                            }
                            (KeyCode::Right, _) if cursor_pos < input.len() => {
                                cursor_pos += 1;
                            }
                            (KeyCode::Esc, _) => {
                                input.clear();
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            let value = if is_test {
                // In test mode, use test input or default values
                if let Some(test_mode) = test_input {
                    if test_mode.is_empty() {
                        param.description.as_ref().map(|d| d.clone()).unwrap_or_default()
                    } else {
                        test_mode.to_string()
                    }
                } else {
                    "test_value".to_string()
                }
            } else {
                input
            };

            param_values.insert(param.name.clone(), value);
        }

        // Show final command info
        if !is_test {
            stdout.queue(MoveTo(0, PREVIEW_SEPARATOR_LINE))?
                  .queue(Print("─".repeat(45).dimmed()))?;
            stdout.queue(MoveTo(0, COMMAND_LINE))?
                  .queue(Print(format!("{}: {}", 
                      "Command to execute".blue().bold(), 
                      command.green()
                  )))?;
            stdout.queue(MoveTo(0, WORKDIR_LINE))?
                  .queue(Print(format!("{}: {}", 
                      "Working directory".cyan().bold(), 
                      std::env::current_dir()?.to_string_lossy().white()
                  )))?;
            stdout.queue(MoveTo(0, WORKDIR_LINE + 2))?;  // Add extra newline
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
        let _ = stdout.queue(MoveTo(0, 0));
        let _ = stdout.flush();
    }

    result
}
