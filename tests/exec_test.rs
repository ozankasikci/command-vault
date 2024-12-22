use command_vault::exec::execute_command;
use command_vault::db::models::{Command, Parameter};
use std::env;
use std::fs;
use std::path::Path;
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
        let mut command = create_test_command("echo '@message'");
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
        let mut command = create_test_command("echo '@message'");
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
    fn test_command_with_multiple_env_vars() {
        let command = create_test_command("echo \"$TEST_VAR1 $TEST_VAR2 $TEST_VAR3\"");
        env::set_var("TEST_VAR1", "value1");
        env::set_var("TEST_VAR2", "value2");
        env::set_var("TEST_VAR3", "value3");
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
        env::remove_var("TEST_VAR3");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_chaining() {
        let command = create_test_command("printf 'a' && printf 'b' || printf 'c'");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_pipeline() {
        let command = create_test_command("echo 'test' | grep test");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_substitution() {
        let command = create_test_command("echo `echo test`");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_shell_options() {
        let command = create_test_command("set -e; false; printf 'should not print'");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_with_invalid_shell() {
        let mut command = create_test_command("printf 'test'");
        env::set_var("SHELL", "/nonexistent/shell");
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("SHELL");
        // Should fall back to /bin/sh
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_directory_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();
        let attempted_path = temp_path.join("../../../etc");
        
        let mut command = create_test_command("pwd");
        command.directory = attempted_path.to_string_lossy().to_string();
        
        env::set_var("COMMAND_VAULT_TEST", "1");
        let result = execute_command(&command);
        env::remove_var("COMMAND_VAULT_TEST");
        
        // The command should fail because we don't allow directory traversal
        assert!(result.is_err());
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
        let command = create_test_command("yes 'test' | head -n 1000");
        
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

    #[test]
    fn test_command_with_special_shell_chars() {
        let special_chars = vec![
            "echo 'test > file'",
            "echo 'test < file'",
            "echo 'test | grep test'",
            "echo 'test ; echo more'",
            "echo 'test && echo more'",
            "echo 'test || echo more'",
            "echo 'test $(echo more)'",
            "echo 'test `echo more`'",
            "echo 'test # comment'",
        ];

        for cmd in special_chars {
            let command = create_test_command(cmd);
            env::set_var("COMMAND_VAULT_TEST", "1");
            let result = execute_command(&command);
            env::remove_var("COMMAND_VAULT_TEST");
            assert!(result.is_ok(), "Command failed: {:?} for input: {}", result.err(), cmd);
        }
    }
}
