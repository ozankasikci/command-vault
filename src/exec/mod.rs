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
        // Don't try to source rc files, just return the command
        command.to_string()
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
        let mut command = std::process::Command::new(&shell);
        
        // Special handling for git log format strings
        let escaped_cmd = if wrapped_command.contains("--pretty=format:") {
            // Keep the exact command as is, just wrap it in a function
            format!(r#"
                f() {{
                    {}
                }}
                f
            "#, 
                wrapped_command
            )
        } else {
            wrapped_command.to_string()
        };

        command
            .args(&["-c", &escaped_cmd])
            .current_dir(&ctx.directory)
            .env("SHELL", &shell)
            .envs(std::env::vars())
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let status = command.status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Command failed with status: {}", status));
        }

        Ok(())
    }
}

pub fn execute_command(command: &Command) -> Result<()> {
    let test_mode = std::env::var("COMMAND_VAULT_TEST").is_ok();
    let current_params = crate::utils::params::parse_parameters(&command.command);
    let mut final_command = crate::utils::params::substitute_parameters(&command.command, &current_params, None)?;

    // If command has parameters, prompt for values
    if !command.parameters.is_empty() {
        if !test_mode {
            println!("Enter parameters for command:");
        }
        for param in &command.parameters {
            let prompt = match &param.description {
                Some(desc) => format!("{} ({})", param.name, desc),
                None => param.name.clone(),
            };

            let value = if test_mode {
                "test_value".to_string()
            } else {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(&prompt)
                    .allow_empty(true)
                    .interact_text()?;
                
                if input.contains(' ') {
                    format!("'{}'", input.replace("'", "'\\''"))
                } else {
                    input
                }
            };

            final_command = final_command.replace(&format!("@{}", param.name), &value);
        }
    }

    let ctx = ExecutionContext {
        command: final_command,
        directory: command.directory.clone(),
        test_mode,
    };

    execute_shell_command(&ctx)
}
