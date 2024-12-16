use std::io::{self, Write};
use anyhow::Result;
use crate::db::models::Command;
use dialoguer::{Input, theme::ColorfulTheme};
use colored::*;
use shell_escape::escape;

pub fn execute_command(command: &Command) -> Result<()> {
    let debug = std::env::var("COMMAND_VAULT_DEBUG").is_ok();
    let mut final_command = command.command.clone();

    // If command has parameters, prompt for values
    if !command.parameters.is_empty() {
        println!("Enter parameters for command:");
        for param in &command.parameters {
            let prompt = match (&param.description, &param.default_value) {
                (Some(desc), Some(default)) => format!("{} ({}) [{}]", param.name, desc, default),
                (Some(desc), None) => format!("{} ({})", param.name, desc),
                (None, Some(default)) => format!("{} [{}]", param.name, default),
                (None, None) => param.name.clone(),
            };

            let value: String = if let Some(default) = &param.default_value {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(&prompt)
                    .default(default.clone())
                    .interact()
                    .map_err(|e| anyhow::anyhow!("Failed to get input: {}", e))?
            } else {
                Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(&prompt)
                    .interact()
                    .map_err(|e| anyhow::anyhow!("Failed to get input: {}", e))?
            };

            // Properly escape the value for shell
            let escaped_value = escape(std::borrow::Cow::from(&value)).to_string();
            
            // Replace parameter in command string
            final_command = final_command.replace(&format!("${{{}}}", param.name), &escaped_value);
            final_command = final_command.replace(&format!("${{{}:{}}}", param.name, param.description.as_ref().unwrap_or(&String::new())), &escaped_value);
            if let Some(desc) = &param.description {
                final_command = final_command.replace(&format!("${{{}:{}={}}}", param.name, desc, param.default_value.as_ref().unwrap_or(&String::new())), &escaped_value);
            }

            if debug {
                eprintln!("DEBUG: Parameter value: {}", &value);
                eprintln!("DEBUG: Escaped value: {}", &escaped_value);
            }

            if debug {
                eprintln!("DEBUG: Command after replacement: {}", final_command);
            }
        }

        // Show preview and confirm
        println!("\n{}: {}", "Command to execute".blue().bold(), final_command.yellow());
        println!("{}: {}", "Working directory".green().bold(), command.directory.cyan());
        println!("{}", "Press Enter to continue or Ctrl+C to cancel...".dimmed());
        print!("\n");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if response.trim().to_lowercase() == "n" {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Execute the command through the shell's functions and aliases
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    
    if debug {
        eprintln!("DEBUG: Final command to execute: {}", final_command);
        eprintln!("DEBUG: Shell being used: {}", shell);
    }
    
    let output = std::process::Command::new(&shell)
        .args(&["-l", "-i", "-c", &final_command])
        .current_dir(&command.directory)
        .env("SHELL", &shell)
        .envs(std::env::vars())
        .output()?;

    if debug {
        eprintln!("DEBUG: Command args: {:?}", &["-l", "-i", "-c", &final_command]);
    }

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}
