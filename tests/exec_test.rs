use command_vault::exec::execute_command;
use command_vault::db::models::{Command, Parameter};
use std::env;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

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
        let command = create_test_command("printf 'hello world'");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let mut command = create_test_command("pwd");
        command.directory = temp_path.clone();
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_parameters() {
        let mut command = create_test_command("printf '%s'");
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        env::set_var("COMMAND_VAULT_TEST", "1");
        env::set_var("COMMAND_VAULT_TEST_INPUT", "Test message");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_debug_mode() {
        let command = create_test_command("printf 'debug test'");
        env::set_var("COMMAND_VAULT_DEBUG", "1");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_DEBUG");
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_with_quoted_parameters() {
        let mut command = create_test_command("printf '%s'");
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test 'quoted' message".to_string())
            ),
        ];

        env::set_var("COMMAND_VAULT_TEST", "1");
        env::set_var("COMMAND_VAULT_TEST_INPUT", "Test 'quoted' message");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
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
    }

    #[test]
    fn test_command_with_env_vars() {
        let mut command = create_test_command("printf '%s'");
        env::set_var("TEST_VAR", "test_value");
        command.command = "printf \"$TEST_VAR\"".to_string();
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("TEST_VAR");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_stdin() {
        let command = create_test_command("cat");
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        env::set_var("COMMAND_VAULT_TEST_INPUT", "test input");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_readonly_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        // Create a readonly file
        let readonly_file = format!("{}/readonly.txt", temp_path);
        fs::write(&readonly_file, "test").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&readonly_file, fs::Permissions::from_mode(0o444)).unwrap();
        }
        
        let mut command = create_test_command("cat readonly.txt");
        command.directory = temp_path;
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_long_output() {
        let command = create_test_command("printf '%s' \"$(printf 'a%.0s' {1..1000})\"");
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_wrap_command_shell_specific() {
        use command_vault::exec::wrap_command;
        
        // Test with zsh
        env::set_var("SHELL", "/bin/zsh");
        let result = wrap_command("printf test", false);
        assert!(result.contains("source ~/.zshrc"));
        
        // Test with bash
        env::set_var("SHELL", "/bin/bash");
        let result = wrap_command("printf test", false);
        assert!(result.contains("source ~/.bashrc"));
        
        // Test with sh (fallback)
        env::set_var("SHELL", "/bin/sh");
        let result = wrap_command("printf test", false);
        assert!(result.contains("source ~/.bashrc")); // sh falls back to bash
        
        env::remove_var("SHELL");
    }

    #[test]
    fn test_wrap_command_test_mode() {
        use command_vault::exec::wrap_command;
        let result = wrap_command("printf test", true);
        assert!(result.contains("export COMMAND_VAULT_TEST=1"));
    }
}
