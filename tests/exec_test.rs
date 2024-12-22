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
        env::set_var("COMMAND_VAULT_TEST_INPUT", "test_value");
        env::set_var("SHELL", "/bin/sh");
    }

    fn cleanup_test_env() {
        env::remove_var("COMMAND_VAULT_TEST");
        env::remove_var("COMMAND_VAULT_TEST_INPUT");
        env::remove_var("SHELL");
    }

    fn get_safe_temp_dir() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        fs::create_dir_all(&temp_path).unwrap();
        // Ensure the directory exists and is accessible
        assert!(temp_path.exists());
        assert!(temp_path.is_dir());
        (temp_dir, temp_path)
    }

    fn setup_test_dir(temp_path: &PathBuf) -> Result<(), std::io::Error> {
        // Create a test file
        let test_file = temp_path.join("test.txt");
        fs::write(&test_file, "test content")?;
        // Verify the file was created
        assert!(test_file.exists());
        Ok(())
    }

    #[test]
    fn test_basic_command_execution() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        let mut command = create_test_command("echo 'hello world'");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_working_directory() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        setup_test_dir(&temp_path).unwrap();
        
        let mut command = create_test_command("ls test.txt");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_parameters() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        let mut command = create_test_command("echo '@message'");
        command.directory = dir_path;
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test message".to_string())
            ),
        ];

        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_quoted_parameters() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        let mut command = create_test_command("printf '%s\\n' '@message'");
        command.directory = dir_path;
        command.parameters = vec![
            Parameter::with_description(
                "message".to_string(),
                Some("Test 'quoted' message".to_string())
            ),
        ];

        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_multiple_env_vars() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        let mut command = create_test_command("printf '%s %s\\n' \"$TEST_VAR1\" \"$TEST_VAR2\"");
        command.directory = dir_path;
        
        setup_test_env();
        env::set_var("TEST_VAR1", "value1");
        env::set_var("TEST_VAR2", "value2");
        
        let result = execute_command(&command);
        
        env::remove_var("TEST_VAR1");
        env::remove_var("TEST_VAR2");
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_directory_traversal() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        // Create a test directory structure
        let test_dir = temp_path.join("test_dir");
        fs::create_dir_all(&test_dir).unwrap();
        
        // Create a test file in the test directory
        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, "test content").unwrap();
        
        // Attempt to traverse outside the test directory
        let mut command = create_test_command("cat ../test.txt");
        command.directory = test_dir.canonicalize().unwrap().to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_err(), "Directory traversal should be prevented");
        drop(temp_dir);
    }

    #[test]
    fn test_command_with_special_shell_chars() {
        let (temp_dir, temp_path) = get_safe_temp_dir();
        let dir_path = temp_path.canonicalize().unwrap().to_string_lossy().to_string();
        
        // Create a test file first
        setup_test_dir(&temp_path).unwrap();
        
        let mut command = create_test_command("printf 'test' > output.txt && cat output.txt");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
    }
}
