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
        assert!(execute_command(&command).is_ok());
    }

    #[test]
    fn test_command_with_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut command = create_test_command("pwd");
        command.directory = temp_dir.path().to_string_lossy().to_string();
        
        assert!(execute_command(&command).is_ok());
    }

    #[test]
    fn test_command_with_parameters() {
        let mut command = create_test_command("echo @message");
        command.parameters = vec![
            Parameter {
                name: "message".to_string(),
                description: Some("Test message".to_string()),
                default_value: Some("default".to_string()),
            }
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
            Parameter {
                name: "message".to_string(),
                description: Some("Test message".to_string()),
                default_value: Some("hello world".to_string()),
            }
        ];

        env::set_var("COMMAND_VAULT_TEST", "1");
        assert!(execute_command(&command).is_ok());
        env::remove_var("COMMAND_VAULT_TEST");
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
}
