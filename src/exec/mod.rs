use std::io::{self, Write};
use anyhow::Result;
use crate::db::models::Command;
use dialoguer::{Input, theme::ColorfulTheme};
use colored::*;
use shell_escape::escape;
use std::borrow::Cow;

pub fn execute_command(command: &Command) -> Result<()> {
    let test_mode = std::env::var("COMMAND_VAULT_TEST").is_ok();
    let mut final_command = command.command.clone();

    // If command has parameters, prompt for values
    if !command.parameters.is_empty() {
        if !test_mode {
            println!("Enter parameters for command:");
        }
        for param in &command.parameters {
            let prompt = match (&param.description, &param.default_value) {
                (Some(desc), Some(default)) => format!("{} ({}) [{}]", param.name, desc, default),
                (Some(desc), None) => format!("{} ({})", param.name, desc),
                (None, Some(default)) => format!("{} [{}]", param.name, default),
                (None, None) => param.name.clone(),
            };

            let value: String = if test_mode {
                param.default_value.clone().unwrap_or_else(|| "test_value".to_string())
            } else if let Some(default) = &param.default_value {
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

            // Replace parameter in command string with properly quoted value if it contains spaces
            let quoted_value = if value.contains(' ') {
                format!("'{}'", value.replace("'", "'\\''"))
            } else {
                value.clone()
            };

            // Replace parameter in command string
            final_command = final_command.replace(&format!("@{}", param.name), &quoted_value);
            if let Some(desc) = &param.description {
                final_command = final_command.replace(&format!("@{}:{}", param.name, desc), &quoted_value);
                if let Some(default) = &param.default_value {
                    final_command = final_command.replace(&format!("@{}:{}={}", param.name, desc, default), &quoted_value);
                }
            }

            // Show preview and confirm
            if !test_mode {
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
        }
    }

    // Execute the command through the shell's functions and aliases
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    
    let wrapped_command = format!(". ~/.zshrc 2>/dev/null && {}", final_command);

    let output = std::process::Command::new(&shell)
        .args(&["-l", "-i", "-c", &wrapped_command])
        .current_dir(&command.directory)
        .env("SHELL", &shell)
        .envs(std::env::vars())
        .output()?;

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}
