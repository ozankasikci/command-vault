use std::io::{self, Write};
use anyhow::Result;
use crate::db::models::Command;
use dialoguer::{Input, theme::ColorfulTheme};
use colored::*;
use crate::shell::hooks::detect_current_shell;

pub struct ExecutionContext {
    pub command: String,
    pub directory: String,
    pub test_mode: bool,
    pub debug_mode: bool,
}

pub fn wrap_command(command: &str, test_mode: bool) -> String {
    if test_mode {
        command.to_string()
    } else {
        // Detect the current shell and source appropriate config
        let shell_type = detect_current_shell().unwrap_or_else(|| "bash".to_string());
        // Remove any surrounding quotes from the command
        let clean_command = command.trim_matches('"').to_string();
        match shell_type.as_str() {
            "zsh" => format!(
                r#"if [ -f ~/.zshrc ]; then source ~/.zshrc 2>/dev/null || true; fi; {}"#,
                clean_command
            ),
            "bash" | _ => format!(
                r#"if [ -f ~/.bashrc ]; then source ~/.bashrc 2>/dev/null || true; fi; if [ -f ~/.bash_profile ]; then source ~/.bash_profile 2>/dev/null || true; fi; {}"#,
                clean_command
            ),
        }
    }
}

pub fn execute_shell_command(ctx: &ExecutionContext) -> Result<()> {
    let shell = if ctx.test_mode {
        "/bin/sh".to_string()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    };
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
        // In normal mode, use interactive login shell
        let mut command = std::process::Command::new(&shell);
        
        // Special handling for git log format strings
        let escaped_cmd = if wrapped_command.contains("--pretty=format:") {
            // For git log format strings, wrap in a function to preserve formatting
            format!(r#"f() {{ {}; }}; f"#, wrapped_command.trim())
        } else {
            // Just use the wrapped command as is, no additional wrapping needed
            wrapped_command
        };

        command
            .arg("-i")  // Interactive shell
            .arg("-l")  // Login shell
            .args(&["-c", &escaped_cmd])
            .current_dir(&ctx.directory)
            .env("SHELL", &shell)
            .envs(std::env::vars())
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        // Only print debug information if debug mode is enabled
        if ctx.debug_mode {
            println!("{} {}", "Running command:".yellow(), &command.get_program().to_string_lossy());
            println!("{} {:?}", "Arguments:".yellow(), command.get_args());
        }

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
        debug_mode: false, // default debug mode to false
    };

    execute_shell_command(&ctx)
}
