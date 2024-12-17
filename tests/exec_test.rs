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
    fn test_git_command_with_spaces() {
        // Create a temporary directory and initialize git repo
        let temp_dir = TempDir::new().unwrap();
        
        // Initialize git repo and configure it
        let git_init = Command {
            id: None,
            command: "git init".to_string(),
            directory: temp_dir.path().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };
        execute_command(&git_init).unwrap();

        // Create a test file and add git config
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        let git_config = Command {
            id: None,
            command: "git config user.name test && git config user.email test@example.com".to_string(),
            directory: temp_dir.path().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };
        execute_command(&git_config).unwrap();

        // Create command with spaces in commit message
        let git_commit = Command {
            id: None,
            command: "git add . && git commit -m \"fix spacing issue\"".to_string(),
            directory: temp_dir.path().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };
        execute_command(&git_commit).unwrap();

        // Verify the commit message
        let git_log = Command {
            id: None,
            command: "git log --format=%s -n 1".to_string(),
            directory: temp_dir.path().to_string_lossy().to_string(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        };
        let output = std::process::Command::new("git")
            .args(&["log", "--format=%s", "-n", "1"])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();

        let commit_msg = String::from_utf8(output.stdout).unwrap().trim().to_string();
        assert_eq!(commit_msg, "fix spacing issue");
    }
}
