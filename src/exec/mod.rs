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
        command.to_string()
    } else {
        // Detect the current shell and source appropriate config
        let shell_type = detect_current_shell().unwrap_or_else(|| "bash".to_string());
        // Remove any surrounding quotes from the command and replace && with && echo -e '\n'
        let clean_command = command
            .trim_matches('"')
            .replace(" && ", " && echo -e '\\n' && ")
            .to_string();
            
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

    if ctx.debug_mode {
        println!("Running command: {}", shell);
    }

    // In test mode, execute commands directly
    let mut command = ProcessCommand::new(&shell);
    
    if ctx.test_mode {
        command.arg("-c");
    } else {
        command.arg("-i").arg("-l").arg("-c");
    }
    
    command.arg(&wrapped_command)
           .current_dir(&ctx.directory);

    if ctx.debug_mode {
        println!("Command: {:?}", command);
    }

    // Ensure we're in normal terminal mode before executing command
    if !ctx.test_mode {
        terminal::disable_raw_mode()?;
        // Reset cursor position
        let mut stdout = io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::cursor::MoveTo(0, crossterm::cursor::position()?.1)
        )?;
        println!(); // Add a newline before command output
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Command failed with status: {}. stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Process output line by line
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout_str.split('\n').collect();
    
    // Print each line, forcing cursor to start of line each time
    let mut stdout = io::stdout();
    for line in lines.iter().filter(|l| !l.is_empty()) {
        crossterm::execute!(
            stdout,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
        )?;
        write!(stdout, "{}\n", line.trim())?;
        stdout.flush()?;
    }

    // Handle stderr similarly
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let err_lines: Vec<&str> = stderr_str.split('\n').collect();
    
    let mut stderr = io::stderr();
    for line in err_lines.iter().filter(|l| !l.is_empty()) {
        crossterm::execute!(
            stderr,
            crossterm::cursor::MoveToColumn(0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine)
        )?;
        write!(stderr, "{}\n", line.trim())?;
        stderr.flush()?;
    }

    if ctx.debug_mode {
        if !ctx.test_mode {
            println!("Arguments: CommandArgs {{ inner: {:?} }}", 
                    if ctx.test_mode { vec!["-c", &wrapped_command] } 
                    else { vec!["-i", "-l", "-c", &wrapped_command] });
        }
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
