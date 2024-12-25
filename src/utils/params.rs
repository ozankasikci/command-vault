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
use std::{
    collections::HashMap,
    io::{stdout, Stdout, Write},
};

use crate::db::models::Parameter;

pub fn parse_parameters(command: &str) -> Vec<Parameter> {
    let re = Regex::new(r"@([a-zA-Z_][a-zA-Z0-9_]*)(?::([^@\s][^@]*))?").unwrap();
    let mut parameters = Vec::new();
    
    for cap in re.captures_iter(command) {
        let name = cap[1].to_string();
        let description = cap.get(2).map(|m| {
            let desc = m.as_str().trim_end();
            if let Some(space_pos) = desc.find(char::is_whitespace) {
                &desc[..space_pos]
            } else {
                desc
            }.to_string()
        });
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
            // When no test input is provided, use descriptions
            parameters.iter()
                .map(|p| p.description.as_deref().unwrap_or(""))
                .collect()
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
                             command.contains('>') ||
                             command.contains('<') ||
                             command.contains('|') ||
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
        let mut param_values: HashMap<String, String> = HashMap::new();
        let mut final_command = String::new();

        for param in parameters {
            let value = if is_test {
                if let Some(input) = test_input {
                    input.to_string()
                } else {
                    param.description.clone().unwrap_or_default()
                }
            } else {
                enable_raw_mode()?;
                let mut stdout = stdout();
                stdout.queue(Clear(ClearType::All))?;
                
                // Function to update the preview
                let update_preview = |stdout: &mut Stdout, current_value: &str| -> Result<()> {
                    let mut preview_command = command.to_string();
                    
                    // Add all previous parameter values
                    for (name, value) in &param_values {
                        let needs_quotes = value.is_empty() || 
                            value.contains(' ') || 
                            value.contains('*') || 
                            value.contains(';') ||
                            value.contains('|') ||
                            value.contains('>') ||
                            value.contains('<') ||
                            preview_command.starts_with("grep");

                        let quoted_value = if needs_quotes && !value.starts_with('\'') && !value.starts_with('"') {
                            format!("'{}'", value.replace('\'', "'\\''"))
                        } else {
                            value.clone()
                        };

                        preview_command = preview_command.replace(&format!("@{}", name), &quoted_value);
                    }

                    // Add current parameter value
                    let needs_quotes = current_value.is_empty() || 
                        current_value.contains(' ') || 
                        current_value.contains('*') || 
                        current_value.contains(';') ||
                        current_value.contains('|') ||
                        current_value.contains('>') ||
                        current_value.contains('<') ||
                        preview_command.starts_with("grep");

                    let quoted_value = if needs_quotes && !current_value.starts_with('\'') && !current_value.starts_with('"') {
                        format!("'{}'", current_value.replace('\'', "'\\''"))
                    } else {
                        current_value.to_string()
                    };

                    preview_command = preview_command.replace(&format!("@{}", param.name), &quoted_value);

                    stdout.queue(MoveTo(0, 0))?
                          .queue(Print("─".repeat(45).dimmed()))?;
                    stdout.queue(MoveTo(0, 1))?
                          .queue(Clear(ClearType::CurrentLine))?
                          .queue(Print(format!("{}: {}", 
                              "Command to execute".blue().bold(), 
                              preview_command.green()
                          )))?;
                    stdout.queue(MoveTo(0, 2))?
                          .queue(Print(format!("{}: {}", 
                              "Working directory".cyan().bold(), 
                              std::env::current_dir()?.to_string_lossy().white()
                          )))?;
                    Ok(())
                };

                // Initial display
                update_preview(&mut stdout, "")?;

                stdout.queue(MoveTo(0, 4))?
                      .queue(Print("─".repeat(45).dimmed()))?;
                stdout.queue(MoveTo(0, 5))?
                      .queue(Print(format!("{}: {}", 
                          "Parameter".blue().bold(), 
                          param.name.green()
                      )))?;
                if let Some(desc) = &param.description {
                    stdout.queue(MoveTo(0, 6))?
                          .queue(Print(format!("{}: {}", 
                              "Description".cyan().bold(), 
                              desc.white()
                          )))?;
                }
                stdout.queue(MoveTo(0, 7))?
                      .queue(Print(format!("{}: ", "Enter value".yellow().bold())))?;
                stdout.flush()?;

                let mut value = String::new();
                let mut cursor_pos = 0;

                loop {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Enter => break,
                            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                                // Handle Ctrl+C
                                disable_raw_mode()?;
                                stdout.queue(Clear(ClearType::All))?;
                                stdout.queue(MoveTo(0, 0))?;
                                stdout.flush()?;
                                return Err(anyhow::anyhow!("Operation cancelled by user"));
                            }
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

                        // Update command preview
                        update_preview(&mut stdout, &value)?;

                        // Redraw the value line
                        stdout.queue(MoveTo(0, 7))?
                              .queue(Clear(ClearType::CurrentLine))?
                              .queue(Print(format!("{}: {}", 
                                  "Enter value".yellow().bold(), 
                                  value
                              )))?;
                        stdout.queue(MoveTo((cursor_pos + 13) as u16, 7))?;
                        stdout.flush()?;
                    }
                }

                disable_raw_mode()?;
                value
            };

            param_values.insert(param.name.clone(), value);
        }

        // Build final command
        final_command = command.to_string();
        for (name, value) in &param_values {
            let needs_quotes = value.is_empty() || 
                             value.contains(' ') || 
                             value.contains('*') || 
                             value.contains(';') ||
                             value.contains('|') ||
                             value.contains('>') ||
                             value.contains('<') ||
                             command.contains('>') ||
                             command.contains('<') ||
                             command.contains('|') ||
                             final_command.starts_with("grep");

            let quoted_value = if needs_quotes && !value.starts_with('\'') && !value.starts_with('"') {
                format!("'{}'", value.replace('\'', "'\\''"))
            } else {
                value.clone()
            };

            final_command = final_command.replace(&format!("@{}", name), &quoted_value);
            
            // Remove the description part from the command
            if let Some(desc) = &parameters.iter().find(|p| p.name == *name).unwrap().description {
                final_command = final_command.replace(&format!(":{}", desc), "");
            }
        }

        if !is_test {
            let mut stdout = stdout();
            stdout.queue(Clear(ClearType::All))?;
            stdout.queue(MoveTo(0, 0))?;
            stdout.flush()?;
        }

        Ok(final_command)
    })();

    if !is_test {
        disable_raw_mode()?;
    }

    result
}
