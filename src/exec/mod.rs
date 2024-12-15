use std::io::{self, Write};
use anyhow::Result;
use crate::db::models::Command;
use dialoguer::{Input, theme::ColorfulTheme};
use colored::*;

pub fn execute_command(command: &Command) -> Result<()> {
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

            // Replace parameter in command string
            final_command = final_command.replace(&format!("${{{}}}", param.name), &value);
            final_command = final_command.replace(&format!("${{{}:{}}}", param.name, param.description.as_ref().unwrap_or(&String::new())), &value);
            if let Some(desc) = &param.description {
                final_command = final_command.replace(&format!("${{{}:{}={}}}", param.name, desc, param.default_value.as_ref().unwrap_or(&String::new())), &value);
            }
        }

        // Show preview and confirm
        println!("\n{}: {}", "Command to execute".blue().bold(), final_command.yellow());
        println!("{}: {}", "Directory".green().bold(), command.directory.cyan());
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

    // Execute the command
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&final_command)
        .output()?;

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}
