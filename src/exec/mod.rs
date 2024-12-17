use std::io::{self, Write};
use anyhow::Result;
use crate::db::models::Command;
use dialoguer::{Input, theme::ColorfulTheme};
use colored::*;

pub struct ExecutionContext {
    pub command: String,
    pub directory: String,
    pub test_mode: bool,
}

pub fn wrap_command(command: &str, test_mode: bool) -> String {
    if test_mode {
        command.to_string()
    } else {
        let shell_type = crate::shell::hooks::detect_current_shell()
            .unwrap_or_else(|| "sh".to_string());
        
        match shell_type.as_str() {
            "zsh" => format!(". ~/.zshrc 2>/dev/null && {}", command),
            "bash" => format!(". ~/.bashrc 2>/dev/null && {}", command),
            _ => command.to_string(),
        }
    }
}

pub fn execute_shell_command(ctx: &ExecutionContext) -> Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let wrapped_command = wrap_command(&ctx.command, ctx.test_mode);

    if ctx.test_mode {
        // In test mode, execute commands directly without shell
        for cmd in wrapped_command.split("&&").map(str::trim) {
            // Parse command preserving quoted arguments
            let mut parts = Vec::new();
            let mut current = String::new();
            let mut in_quotes = false;
            let mut chars = cmd.chars().peekable();

            while let Some(c) = chars.next() {
                match c {
                    '"' => {
                        if in_quotes {
                            if current.len() > 0 {
                                parts.push(current);
                                current = String::new();
                            }
                            in_quotes = false;
                        } else {
                            in_quotes = true;
                        }
                    }
                    ' ' if !in_quotes => {
                        if current.len() > 0 {
                            parts.push(current);
                            current = String::new();
                        }
                    }
                    _ => current.push(c),
                }
            }
            if current.len() > 0 {
                parts.push(current);
            }

            // Execute command with parsed arguments
            if parts.is_empty() {
                continue;
            }

            let program = parts[0].clone();
            let args = &parts[1..];

            let mut command = std::process::Command::new(&program);
            command
                .args(args)
                .current_dir(&ctx.directory)
                .env("COMMAND_VAULT_TEST", "1")
                .env("GIT_TERMINAL_PROMPT", "0")  // Disable git terminal prompts
                .env("GIT_ASKPASS", "echo")       // Make git use echo for password prompts
                .envs(std::env::vars());

            // Configure stdio based on whether we're in a test environment
            if std::env::var("COMMAND_VAULT_TEST").is_ok() {
                command
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
            }

            let output = command.output()?;

            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;

            if !output.status.success() {
                return Err(anyhow::anyhow!("Command failed with status: {}", output.status));
            }
        }
        Ok(())
    } else {
        // In normal mode, use interactive shell
        let output = std::process::Command::new(&shell)
            .args(&["-l", "-i", "-c", &wrapped_command])
            .current_dir(&ctx.directory)
            .env("SHELL", &shell)
            .envs(std::env::vars())
            .output()?;

        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Command failed with status: {}", output.status));
        }

        Ok(())
    }
}

pub fn execute_command(command: &Command) -> Result<()> {
    let test_mode = std::env::var("COMMAND_VAULT_TEST").is_ok();
    let current_params = crate::utils::params::parse_parameters(&command.command);
    let mut final_command = crate::utils::params::substitute_parameters(&command.command, &current_params)?;

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
                println!("\n{}: {}", "Working directory".green().bold(), command.directory.cyan());
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

    let ctx = ExecutionContext {
        command: final_command,
        directory: command.directory.clone(),
        test_mode,
    };

    execute_shell_command(&ctx)
}
