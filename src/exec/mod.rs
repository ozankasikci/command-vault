use std::io::{self, Write};
use std::process::Command as ProcessCommand;
use std::env;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crossterm::terminal;
use dialoguer::{theme::ColorfulTheme, Input};
use crate::db::models::Command;

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
        // Wrap the command to set environment variables and handle shell integration
        format!("COMMAND_VAULT_ACTIVE=1 {}", command)
    }
}

fn is_path_traversal_attempt(command: &str, working_dir: &Path) -> bool {
    // Check if the command contains path traversal attempts
    if command.contains("..") {
        // Get the absolute path of the working directory
        if let Ok(working_dir) = working_dir.canonicalize() {
            // Try to resolve any path in the command relative to working_dir
            let potential_path = working_dir.join(command);
            if let Ok(resolved_path) = potential_path.canonicalize() {
                // Check if the resolved path is outside the working directory
                return !resolved_path.starts_with(working_dir);
            }
        }
        // If we can't resolve the paths, assume it's a traversal attempt
        return true;
    }
    false
}

pub fn execute_shell_command(ctx: &ExecutionContext) -> Result<()> {
    // Get the current shell
    let shell = if cfg!(windows) {
        String::from("cmd.exe")
    } else {
        env::var("SHELL").unwrap_or_else(|_| String::from("/bin/sh"))
    };

    // Wrap the command for shell execution
    let wrapped_command = wrap_command(&ctx.command, ctx.test_mode);

    // Check for directory traversal attempts
    if is_path_traversal_attempt(&wrapped_command, Path::new(&ctx.directory)) {
        return Err(anyhow::anyhow!("Directory traversal attempt detected"));
    }

    // Create command with the appropriate shell
    let mut command = ProcessCommand::new(&shell);
    
    // In test mode, use simple shell execution
    if ctx.test_mode {
        command.args(&["-c", &wrapped_command]);
    } else {
        // Use -c for both interactive and non-interactive mode to ensure consistent behavior
        command.args(&["-c", &wrapped_command]);
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
    let mut final_command = command.command.clone();

    // If command has parameters, prompt for values first
    if !command.parameters.is_empty() {
        for param in &command.parameters {
            println!("Parameter: {}", param.name);
            println!();

            let value = if test_mode {
                let value = std::env::var("COMMAND_VAULT_TEST_INPUT")
                    .unwrap_or_else(|_| "test_value".to_string());
                println!("Enter value: {}", value);
                println!();
                value
            } else {
                let input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter value")
                    .allow_empty(true)
                    .interact_text()?;
                println!();
                
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

    // Print command details only once
    println!("─────────────────────────────────────────────");
    println!();
    println!("Command to execute: {}", ctx.command);
    println!("Working directory: {}", ctx.directory);
    println!();

    execute_shell_command(&ctx)
}
