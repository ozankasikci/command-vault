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

    if ctx.debug_mode {
        println!("Running command: {}", shell);
    }

    // In test mode, execute commands directly
    use std::process::Command;
    let mut command = Command::new(&shell);
    
    if ctx.test_mode {
        command.arg("-c");
    } else {
        command.arg("-i").arg("-l").arg("-c");
    }
    
    command.arg(&wrapped_command)
           .current_dir(&ctx.directory);

    let output = command.output()?;

    if ctx.debug_mode {
        if !ctx.test_mode {
            println!("Arguments: CommandArgs {{ inner: {:?} }}", 
                    if ctx.test_mode { vec!["-c", &wrapped_command] } 
                    else { vec!["-i", "-l", "-c", &wrapped_command] });
        }
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
    }

    Ok(())
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
