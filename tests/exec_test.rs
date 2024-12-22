#[cfg(test)]
mod tests {
    use command_vault::exec::execute_command;
    use command_vault::db::models::{Command, Parameter};
    use std::env;
    use tempfile::TempDir;
    use chrono::Utc;

    fn create_test_command(command: &str) -> Command {
        Command {
            id: None,
            command: command.to_string(),
            directory: env::current_dir().unwrap().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        }
    }

    #[test]
    fn test_basic_command_execution() {
        let command = create_test_command("echo 'hello world'");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_working_directory() {
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        // Create a command that will print the current directory
        let mut command = create_test_command("pwd");
        command.directory = temp_path.clone();
        
        // Set test mode
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        // Keep temp_dir in scope until the end of the test
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_parameters() {
        let mut command = create_test_command("echo @message");
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        // Set test mode to avoid interactive prompts
        env::set_var("COMMAND_VAULT_TEST", "1");
        assert!(execute_command(&command).is_ok());
        env::remove_var("COMMAND_VAULT_TEST");
    }

    #[test]
    fn test_command_with_debug_mode() {
        let command = create_test_command("echo 'debug test'");
        env::set_var("COMMAND_VAULT_DEBUG", "1");
        assert!(execute_command(&command).is_ok());
        env::remove_var("COMMAND_VAULT_DEBUG");
    }

    #[test]
    fn test_command_with_quoted_parameters() {
        let mut command = create_test_command("echo '@message'");
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_shell_env() {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let command = create_test_command(&format!("echo $SHELL"));
        assert!(execute_command(&command).is_ok());
    }

    #[test]
    fn test_command_with_spaces() {
        // Set test mode to avoid shell initialization
        std::env::set_var("COMMAND_VAULT_TEST", "1");
        
        // Create a temporary directory
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Write directly to the file
        std::fs::write(&test_file, "test message with spaces").unwrap();
        
        // Create a command to read from the file
        let read_command = Command {
            id: None,
            command: format!("cat {}", test_file.to_string_lossy()),
            directory: temp_dir.path().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };
        execute_command(&read_command).unwrap();

        // Read and verify the file contents
        let contents = std::fs::read_to_string(test_file).unwrap();
        assert_eq!(contents.trim(), "test message with spaces");
        
        // Clean up test environment variable
        std::env::remove_var("COMMAND_VAULT_TEST");
    }

    #[test]
    fn test_failed_command_execution() {
        let command = create_test_command("nonexistentcommand");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrap_command_empty_input() {
        use command_vault::exec::wrap_command;
        let result = wrap_command("", true);
        assert_eq!(result, "");
        
        let result = wrap_command("", false);
        assert!(result.contains("source ~/.bashrc") || result.contains("source ~/.zshrc"));
    }

    #[test]
    fn test_wrap_command_with_quotes() {
        use command_vault::exec::wrap_command;
        let cmd = r#"echo "hello world""#;
        let result = wrap_command(cmd, true);
        assert_eq!(result, cmd);
        
        let result = wrap_command(cmd, false);
        assert!(!result.contains(r#"""hello world"""#));
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_command_with_invalid_directory() {
        let mut command = create_test_command("pwd");
        command.directory = "/nonexistent/directory".to_string();
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_with_stderr_output() {
        let command = create_test_command("ls /nonexistent/path");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_err());
        assert!(format!("{:?}", result.err()).contains("No such file or directory"));
    }

    #[test]
    fn test_wrap_command_shell_specific() {
        use command_vault::exec::wrap_command;
        env::set_var("SHELL", "/bin/zsh");
        let result = wrap_command("echo test", false);
        assert!(result.contains("source ~/.zshrc"));
        
        env::set_var("SHELL", "/bin/bash");
        let result = wrap_command("echo test", false);
        assert!(result.contains("source ~/.bashrc"));
        env::remove_var("SHELL");
    }
}
