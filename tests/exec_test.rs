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

    #[test]
    fn test_basic_command_execution() {
        let mut command = create_test_command("echo 'hello world'");
        command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();
        
        let mut command = create_test_command("pwd");
        command.directory = temp_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
    }

    #[test]
    fn test_command_with_parameters() {
        let mut command = create_test_command("echo '@message'");
        command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
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
    }

    #[test]
    fn test_command_with_quoted_parameters() {
        let mut command = create_test_command("echo '@message'");
        command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
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
    }

    #[test]
    fn test_command_with_multiple_env_vars() {
        let mut command = create_test_command("echo \"$TEST_VAR1 $TEST_VAR2 $TEST_VAR3\"");
        command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
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
    }

    #[test]
    fn test_command_with_directory_traversal() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();
        let attempted_path = temp_path.join("../../../etc");
        
        let mut command = create_test_command("pwd");
        command.directory = attempted_path.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_err(), "Directory traversal should be prevented");
    }

    #[test]
    fn test_command_with_readonly_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().canonicalize().unwrap();
        
        // Create a readonly file
        let readonly_file = temp_path.join("readonly.txt");
        fs::write(&readonly_file, "test").unwrap();
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
    }

    #[test]
    fn test_command_with_long_output() {
        let mut command = create_test_command("yes 'test' | head -n 100");
        command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
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
            let mut command = create_test_command(cmd);
            command.directory = env::current_dir().unwrap().to_string_lossy().to_string();
            setup_test_env();
            let result = execute_command(&command);
            cleanup_test_env();
            assert!(result.is_ok(), "Command failed: {:?} for input: {}", result.err(), cmd);
        }
    }

    // ... keep other tests unchanged ...
}
