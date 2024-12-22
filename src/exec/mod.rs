use std::io::{self, Write};
use std::process::Command as ProcessCommand;
use anyhow::Result;
use dialoguer::{Input, theme::ColorfulTheme};
use crate::shell::hooks::detect_current_shell;
use crossterm::terminal;
use crate::db::models::Command;

pub struct ExecutionContext {
    pub command: String,
    pub directory: String,
    pub test_mode: bool,
    pub debug_mode: bool,
}

pub fn wrap_command(command: &str, test_mode: bool) -> String {
    if test_mode {
        // In test mode, just return the command as is
        command.to_string()
    } else {
        // For interactive mode, handle shell initialization
        let shell_type = detect_current_shell().unwrap_or_else(|| "bash".to_string());
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
    // Always use /bin/sh in test mode, otherwise use the user's shell
    let shell = if ctx.test_mode {
        "/bin/sh".to_string()
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    };

    let wrapped_command = wrap_command(&ctx.command, ctx.test_mode);

    if ctx.debug_mode {
        println!("Running command: {}", shell);
        println!("Working directory: {}", ctx.directory);
        println!("Wrapped command: {}", wrapped_command);
    }

    // Create command with the appropriate shell
    let mut command = ProcessCommand::new(&shell);
    
    // In test mode, use simple shell execution
    if ctx.test_mode {
        command.args(&["-c", &wrapped_command]);
    } else {
        command.args(&["-i", "-l", "-c", &wrapped_command]);
    }
    
    // Set working directory
    command.current_dir(&ctx.directory);

    if ctx.debug_mode {
        println!("Full command: {:?}", command);
    }

    // Disable raw mode only in interactive mode
    if !ctx.test_mode {
        let _ = terminal::disable_raw_mode();
        // Reset cursor position
        let mut stdout = io::stdout();
        let _ = crossterm::execute!(
            stdout,
            crossterm::cursor::MoveTo(0, crossterm::cursor::position()?.1)
        );
        println!(); // Add a newline before command output
    }

    // Execute the command and capture output
    let output = command.output()?;

    // Handle command output
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "Command failed with status: {}. stderr: {}",
            output.status,
            stderr
        ));
    }

    // Print stdout
    if !output.stdout.is_empty() {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        print!("{}", stdout_str);
    }

    // Print stderr
    if !output.stderr.is_empty() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        eprint!("{}", stderr_str);
    }

    Ok(())
}

pub fn execute_command(command: &Command) -> Result<()> {
    let test_mode = std::env::var("COMMAND_VAULT_TEST").is_ok();
    let debug_mode = std::env::var("COMMAND_VAULT_DEBUG").is_ok();
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
        debug_mode,
    };

    execute_shell_command(&ctx)
}
