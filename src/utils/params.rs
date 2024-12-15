use crate::db::models::Parameter;
use regex::Regex;
use anyhow::Result;
use std::io::Write;
use colored::*;
use crossterm::{
    cursor::MoveTo,
    terminal::{Clear, ClearType},
    event::{self, Event, KeyCode},
    ExecutableCommand, QueueableCommand,
    style::Print,
};
use std::collections::HashMap;

const HEADER_LINE: u16 = 0;
const SEPARATOR_LINE: u16 = 1;
const PARAM_LINE: u16 = 3;
const DEFAULT_LINE: u16 = 4;
const INPUT_LINE: u16 = 5;
const PREVIEW_SEPARATOR_LINE: u16 = 7;
const COMMAND_LINE: u16 = 8;
const WORKDIR_LINE: u16 = 9;

pub fn parse_parameters(command: &str) -> Vec<Parameter> {
    let re = Regex::new(r"@([a-zA-Z][a-zA-Z0-9_]*)(?:=([^\s]+))?").unwrap();
    let mut parameters = Vec::new();
    
    for cap in re.captures_iter(command) {
        let name = cap[1].to_string();
        let default_value = cap.get(2).map(|m| m.as_str().to_string());
        
        parameters.push(Parameter {
            name,
            description: None,
            default_value,
        });
    }
    
    parameters
}

pub fn substitute_parameters(command: &str, parameters: &[Parameter]) -> Result<String> {
    let mut final_command = command.to_string();
    let is_test = std::env::var("COMMAND_VAULT_TEST").is_ok();
    
    // In test mode, just use default values without interactive UI
    if is_test {
        for param in parameters {
            let value = param.default_value.clone().unwrap_or_default();
            final_command = final_command.replace(&format!("@{}", param.name), &value);
        }
        return Ok(final_command);
    }
    
    let mut stdout = std::io::stdout();
    let mut param_values: HashMap<String, String> = HashMap::new();
    
    crossterm::terminal::enable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?;

    for param in parameters {
        let default_str = param.default_value.as_deref().unwrap_or("");
        let desc = param.description.as_deref().unwrap_or("");
        let mut input = default_str.to_string();
        let mut cursor_pos = input.len();

        loop {
            // Clear screen
            stdout.queue(Clear(ClearType::All))?;

            // Header
            stdout.queue(MoveTo(0, HEADER_LINE))?
                  .queue(Print("Enter values for command parameters:"))?;

            // Top separator
            stdout.queue(MoveTo(0, SEPARATOR_LINE))?
                  .queue(Print("─".repeat(45).dimmed()))?;

            // Parameter info
            stdout.queue(MoveTo(0, PARAM_LINE))?
                  .queue(Print(format!("{}: {}", "Parameter".blue().bold(), param.name.yellow())))?;
            if !desc.is_empty() {
                stdout.queue(Print(format!(" - {}", desc.dimmed())))?;
            }

            // Default value
            stdout.queue(MoveTo(0, DEFAULT_LINE))?
                  .queue(Print(format!("{}: [{}]", 
                      "Default value".green().bold(), 
                      default_str.cyan()
                  )))?;

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
            } else if let Some(default) = &param.default_value {
                preview_command = preview_command.replace(&format!("@{}", param.name), default);
            }

            // Bottom separator
            stdout.queue(MoveTo(0, PREVIEW_SEPARATOR_LINE))?
                  .queue(Print("─".repeat(45).dimmed()))?;

            // Command preview
            stdout.queue(MoveTo(0, COMMAND_LINE))?
                  .queue(Print(format!("{}: {}", 
                      "Command to execute".blue().bold(), 
                      preview_command.yellow()
                  )))?;

            // Working directory
            stdout.queue(MoveTo(0, WORKDIR_LINE))?
                  .queue(Print(format!("{}: {}", 
                      "Working directory".green().bold(), 
                      std::env::current_dir()?.to_string_lossy().cyan()
                  )))?;

            // Position cursor at input
            let input_prompt = "Enter value: ";
            stdout.queue(MoveTo(
                (input_prompt.len() + cursor_pos) as u16,
                INPUT_LINE
            ))?;
            
            stdout.flush()?;

            // Handle input
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char(c) => {
                        input.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                    KeyCode::Backspace if cursor_pos > 0 => {
                        input.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                    }
                    KeyCode::Left if cursor_pos > 0 => {
                        cursor_pos -= 1;
                    }
                    KeyCode::Right if cursor_pos < input.len() => {
                        cursor_pos += 1;
                    }
                    KeyCode::Esc => {
                        input.clear();
                        break;
                    }
                    _ => {}
                }
            }
        }

        let value = if input.is_empty() {
            param.default_value.clone().unwrap_or_default()
        } else {
            input
        };
        param_values.insert(param.name.clone(), value.clone());
        final_command = final_command.replace(&format!("@{}", param.name), &value);
    }

    // Cleanup and restore terminal
    crossterm::terminal::disable_raw_mode()?;
    stdout.execute(Clear(ClearType::All))?
          .execute(MoveTo(0, 0))?;

    // Final display
    println!("\n{}: {}", "Command to execute".blue().bold(), final_command.yellow());
    println!("{}: {}", "Working directory".green().bold(), std::env::current_dir()?.to_string_lossy().cyan());
    
    Ok(final_command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parameters() {
        let command = "docker run -p @port=8080 -v @volume @image";
        let params = parse_parameters(command);
        
        assert_eq!(params.len(), 3);
        
        assert_eq!(params[0].name, "port");
        assert_eq!(params[0].description, None);
        assert_eq!(params[0].default_value, Some("8080".to_string()));
        
        assert_eq!(params[1].name, "volume");
        assert_eq!(params[1].description, None);
        assert_eq!(params[1].default_value, None);
        
        assert_eq!(params[2].name, "image");
        assert_eq!(params[2].description, None);
        assert_eq!(params[2].default_value, None);
    }
}
