use command_vault::exec::execute_command;
use command_vault::db::models::{Command, Parameter};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use chrono::Utc;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_command(command: &str) -> Command {
        Command {
            id: None,
            command: command.to_string(),
            directory: String::new(),
            timestamp: Utc::now(),
            tags: vec![],
            parameters: vec![],
        }
    }

    fn setup_test_env() {
        env::set_var("COMMAND_VAULT_TEST", "1");
    }

    fn cleanup_test_env() {
        env::remove_var("COMMAND_VAULT_TEST");
    }

    fn get_safe_temp_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&temp_path).unwrap();
        (temp_dir, temp_path)
    }

    #[test]
    fn test_basic_command_execution() {
        let mut command = create_test_command("echo 'hello world'");
        let (temp_dir, temp_path) = get_safe_temp_dir();
        command.directory = temp_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_working_directory() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.to_string_lossy().to_string();
        
        let mut command = create_test_command("pwd");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_parameters() {
        let mut command = create_test_command("echo '@message'");
        let (temp_dir, temp_path) = get_safe_temp_dir();
        command.directory = temp_path.to_string_lossy().to_string();
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        setup_test_env();
        env::set_var("COMMAND_VAULT_TEST_INPUT", "Test message");
        let result = execute_command(&command);
        cleanup_test_env();
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_quoted_parameters() {
        let mut command = create_test_command("echo '@message'");
        let (temp_dir, temp_path) = get_safe_temp_dir();
        command.directory = temp_path.to_string_lossy().to_string();
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test 'quoted' message".to_string())
            ),
        ];

        setup_test_env();
        env::set_var("COMMAND_VAULT_TEST_INPUT", "Test 'quoted' message");
        let result = execute_command(&command);
        cleanup_test_env();
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_multiple_env_vars() {
        let mut command = create_test_command("echo \"$TEST_VAR1 $TEST_VAR2 $TEST_VAR3\"");
        let (temp_dir, temp_path) = get_safe_temp_dir();
        command.directory = temp_path.to_string_lossy().to_string();
        env::set_var("TEST_VAR1", "value1");
        env::set_var("TEST_VAR2", "value2");
        env::set_var("TEST_VAR3", "value3");
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
        env::remove_var("TEST_VAR3");
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_directory_traversal() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let base_path = temp_path.canonicalize().unwrap();
        let attempted_path = base_path.join("../../../etc");
        
        let mut command = create_test_command("pwd");
        command.directory = attempted_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        // The command should fail because we don't allow directory traversal
        assert!(result.is_err(), "Directory traversal should be prevented");
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_readonly_file() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        
        // Create a readonly file
        let readonly_file = temp_path.join("readonly.txt");
        fs::write(&readonly_file, "test content").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&readonly_file, fs::Permissions::from_mode(0o444)).unwrap();
        }
        
        let mut command = create_test_command("cat readonly.txt");
        command.directory = temp_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_long_output() {
        let mut command = create_test_command("yes 'test' | head -n 100");
        let (temp_dir, temp_path) = get_safe_temp_dir();
        command.directory = temp_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_special_shell_chars() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.to_string_lossy().to_string();
        
        // Test each special character in isolation
        let test_cases = vec![
            "echo test",
            "echo test > output.txt && cat output.txt",
            "echo test | cat",
            "echo test1; echo test2",
            "echo test1 && echo test2",
            "false || echo test",
            "echo $(echo nested)",
            "echo `echo nested`",
        ];

        for cmd in test_cases {
            let mut command = create_test_command(cmd);
            command.directory = dir_path.clone();
            setup_test_env();
            let result = execute_command(&command);
            cleanup_test_env();
            assert!(result.is_ok(), "Command failed: {:?} for input: {}", result.err(), cmd);
        }
        drop(temp_dir);
    }
}
