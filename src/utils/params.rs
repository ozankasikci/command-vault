use crate::db::models::Parameter;
use regex::Regex;
use anyhow::{Result, anyhow};
use std::io::Write;
use colored::*;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
    event::{self, Event, KeyCode, KeyModifiers},
    ExecutableCommand, QueueableCommand,
    style::Print,
};
use std::collections::HashMap;
use std::io;

const HEADER_LINE: u16 = 0;
const SEPARATOR_LINE: u16 = 1;
const PARAM_LINE: u16 = 3;
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

/// Quote a parameter value according to shell rules
fn quote_parameter_value(value: &str, needs_quotes: bool) -> String {
    if !needs_quotes {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Check if a value needs to be quoted
fn needs_quotes(value: &str, command: &str) -> bool {
    value.is_empty() || 
    value.contains(' ') || 
    value.contains('*') || 
    command.starts_with("grep")
}

/// Handle parameter substitution in test mode
fn substitute_parameters_test_mode(
    command: &str,
    parameters: &[Parameter],
    test_input: Option<&str>
) -> Result<String> {
    if parameters.is_empty() {
        return Ok(command.to_string());
    }

    let mut final_command = command.to_string();
    let test_values: Vec<&str> = test_input.map(|s| s.split('\n').collect()).unwrap_or_default();

    for (index, param) in parameters.iter().enumerate() {
        let value = if let Some(test_mode) = test_input {
            if test_values.len() > index {
                test_values[index].to_string()
            } else if test_mode.is_empty() {
                "".to_string()
            } else {
                "test_value".to_string()
            }
        } else {
            "test_value".to_string()
        };

        let quoted_value = quote_parameter_value(&value, needs_quotes(&value, command));

        // Replace parameter placeholders with the value
        if let Some(desc) = &param.description {
            final_command = final_command.replace(&format!("@{}:{}", param.name, desc), &quoted_value);
        }
        final_command = final_command.replace(&format!("@{}", param.name), &quoted_value);
    }

    Ok(final_command)
}

/// Handle parameter substitution in interactive mode
fn substitute_parameters_interactive_mode(
    command: &str,
    parameters: &[Parameter]
) -> Result<String> {
    let mut stdout = std::io::stdout();
    let mut param_values: HashMap<String, String> = HashMap::new();
    let mut final_command = command.to_string();

    // Enable raw mode for the duration of interactive input
    let _raw_mode_guard = enable_raw_mode().map_err(|e| anyhow!("Failed to enable raw mode: {}", e))?;

    let result = (|| -> Result<String> {
        for param in parameters {
            let desc = param.description.as_deref().unwrap_or("");
            let mut input = String::new();
            let mut cursor_pos = 0;

            loop {
                // Clear screen
                stdout.queue(Clear(ClearType::All))?;

                // Header
                stdout.queue(MoveTo(0, HEADER_LINE))?
                      .queue(Print("Enter values for command parameters:"))?;

                // Top separator
                stdout.queue(MoveTo(0, SEPARATOR_LINE))?
                      .queue(Print("─".repeat(45).dimmed()))?;

                // Parameter name and description
                stdout.queue(MoveTo(0, PARAM_LINE))?
                      .queue(Print(format!("Parameter: {}", param.name)))?;
                if !desc.is_empty() {
                    stdout.queue(Print(format!(" ({})", desc)))?;
                }

                // Input line
                stdout.queue(MoveTo(0, INPUT_LINE))?
                      .queue(Print("Enter value: "))?
                      .queue(Print(&input))?;

                // Bottom separator
                stdout.queue(MoveTo(0, PREVIEW_SEPARATOR_LINE))?
                      .queue(Print("─".repeat(45).dimmed()))?;

                // Command preview
                let mut preview = command.to_string();
                for (name, value) in &param_values {
                    preview = preview.replace(&format!("@{}", name), value);
                }
                let preview_value = if input.contains(' ') {
                    format!("'{}'", input)
                } else {
                    input.clone()
                };
                preview = preview.replace(&format!("@{}", param.name), &preview_value);

                stdout.queue(MoveTo(0, COMMAND_LINE))?
                      .queue(Print("Command to execute: "))?
                      .queue(Print(preview))?;

                // Working directory
                stdout.queue(MoveTo(0, WORKDIR_LINE))?
                      .queue(Print("Working directory: "))?
                      .queue(Print(std::env::current_dir()?.to_string_lossy().to_string()))?;

                stdout.queue(MoveTo(12 + cursor_pos as u16, INPUT_LINE))?;
                stdout.flush()?;

                // Handle input
                match event::read()? {
                    Event::Key(key) => {
                        match key.code {
                            KeyCode::Enter => {
                                let value = input.clone();
                                let quoted_value = quote_parameter_value(&value, needs_quotes(&value, command));
                                param_values.insert(param.name.clone(), quoted_value);
                                break;
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Err(anyhow!("Operation cancelled"));
                            }
                            KeyCode::Char(c) => {
                                input.insert(cursor_pos, c);
                                cursor_pos += 1;
                            }
                            KeyCode::Backspace if cursor_pos > 0 => {
                                input.remove(cursor_pos - 1);
                                cursor_pos -= 1;
                            }
                            KeyCode::Delete if cursor_pos < input.len() => {
                                input.remove(cursor_pos);
                            }
                            KeyCode::Left if cursor_pos > 0 => {
                                cursor_pos -= 1;
                            }
                            KeyCode::Right if cursor_pos < input.len() => {
                                cursor_pos += 1;
                            }
                            KeyCode::Home => {
                                cursor_pos = 0;
                            }
                            KeyCode::End => {
                                cursor_pos = input.len();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        // Build final command with all parameter values
        for (name, value) in &param_values {
            final_command = final_command.replace(&format!("@{}", name), value);
        }

        Ok(final_command)
    })();

    // Always restore terminal state
    let _ = disable_raw_mode();
    let _ = stdout.execute(Clear(ClearType::All));
    
    result
}

pub fn substitute_parameters(command: &str, parameters: &[Parameter], test_input: Option<&str>) -> Result<String> {
    let is_test = std::env::var("COMMAND_VAULT_TEST").is_ok();
    
    if is_test {
        substitute_parameters_test_mode(command, parameters, test_input)
    } else {
        substitute_parameters_interactive_mode(command, parameters)
    }
}
