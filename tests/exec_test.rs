#[cfg(test)]
mod tests {
    use command_vault::exec::execute_command;
    use command_vault::db::models::{Command, Parameter};
    use std::env;
    use tempfile::TempDir;
    use chrono::Utc;
    use std::fs;
    use std::path::Path;

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
        assert!(execute_command(&command).is_ok());
    }

    #[test]
    fn test_command_with_working_directory() {
        // Create a temporary directory and file
        let temp_dir = TempDir::new().unwrap();
        let test_file = "test.txt";
        let test_content = "test content";
        
        // Get the canonical path to avoid UNC issues on Windows
        let canonical_temp_dir = temp_dir.path().canonicalize().unwrap();
        let test_file_path = canonical_temp_dir.join(test_file);
        
        // Write test content to file
        fs::write(&test_file_path, test_content).unwrap();
        
        // Create appropriate command based on platform
        let command_str = if cfg!(windows) {
            format!("cmd.exe /c type {}", test_file)
        } else {
            format!("cat {}", test_file)
        };
        
        // Create and configure the command
        let mut command = create_test_command(&command_str);
        command.directory = canonical_temp_dir.to_string_lossy().to_string();
        
        // Set test mode and execute
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        assert!(result.is_ok(), "Command execution failed: {:?}", result.err());
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
        let mut command = create_test_command("echo @message");
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        env::set_var("COMMAND_VAULT_TEST", "1");
        assert!(execute_command(&command).is_ok());
        env::remove_var("COMMAND_VAULT_TEST");
    }

    #[test]
    fn test_command_with_shell_env() {
        let command = Command {
            id: None,
            command: "echo".to_string(),
            directory: env::current_dir().unwrap().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };

        let result = execute_command(&command);
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("raw mode"));
    }

    #[test]
    fn test_command_with_shell_env_updated() {
        let command = Command {
            id: None,
            command: "echo".to_string(),
            directory: env::current_dir().unwrap().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };

        let result = execute_command(&command);
        assert!(result.is_ok() || result.unwrap_err().to_string().contains("raw mode"));
    }
}
