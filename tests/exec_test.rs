use command_vault::exec::execute_command;
use command_vault::db::models::{Command, Parameter};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use chrono::Utc;
use std::thread;
use std::time::Duration;

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

    fn ensure_directory_exists(path: &PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        // Small delay to ensure filesystem operations complete
        thread::sleep(Duration::from_millis(100));
        Ok(())
    }

    fn get_safe_temp_dir() -> std::io::Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path().to_path_buf();
        ensure_directory_exists(&temp_path)?;
        
        // Verify the directory exists and is accessible
        if !temp_path.exists() || !temp_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create temporary directory"
            ));
        }
        
        Ok((temp_dir, temp_path))
    }

    fn setup_test_dir(temp_path: &PathBuf) -> std::io::Result<()> {
        ensure_directory_exists(temp_path)?;
        
        // Create a test file
        let test_file = temp_path.join("test.txt");
        fs::write(&test_file, "test content")?;
        
        // Small delay to ensure file is written
        thread::sleep(Duration::from_millis(100));
        
        // Verify the file was created
        if !test_file.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to create test file"
            ));
        }
        
        Ok(())
    }

    #[test]
    fn test_basic_command_execution() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        let mut command = create_test_command("echo 'hello world'");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
        Ok(())
    }

    #[test]
    fn test_command_with_working_directory() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        setup_test_dir(&temp_path)?;
        
        let mut command = create_test_command("cat test.txt");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
        Ok(())
    }

    #[test]
    fn test_command_with_parameters() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        // Create a simple command that just echoes the parameter
        let mut command = create_test_command("echo @message");
        command.directory = dir_path.clone();
        command.parameters = vec![
            Parameter {
                name: "message".to_string(),
                description: Some("Test message".to_string()),
            },
        ];

        // Set up test environment with a known test value
        setup_test_env();
        env::set_var("COMMAND_VAULT_TEST_INPUT", "test_message");
        
        // Execute the command
        let result = execute_command(&command);
        
        // Clean up
        cleanup_test_env();
        drop(temp_dir);
        
        // Verify the result
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        Ok(())
    }

    #[test]
    fn test_command_with_quoted_parameters() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        let mut command = create_test_command("echo '@message'");
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
        Ok(())
    }

    #[test]
    fn test_command_with_multiple_env_vars() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        let mut command = create_test_command("echo \"$TEST_VAR1 $TEST_VAR2\"");
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
        Ok(())
    }

    #[test]
    fn test_command_with_directory_traversal() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        // Create a test directory structure
        let test_dir = temp_path.join("test_dir");
        ensure_directory_exists(&test_dir)?;
        
        // Create a test file in the test directory
        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, "test content")?;
        
        // Small delay to ensure file is written
        thread::sleep(Duration::from_millis(100));
        
        // Attempt to traverse outside the test directory
        let mut command = create_test_command("cat ../test.txt");
        command.directory = test_dir.canonicalize()?.to_string_lossy().to_string();
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_err(), "Directory traversal should be prevented");
        drop(temp_dir);
        Ok(())
    }

    #[test]
    fn test_command_with_special_shell_chars() -> std::io::Result<()> {
        let (temp_dir, temp_path) = get_safe_temp_dir()?;
        let dir_path = temp_path.canonicalize()?.to_string_lossy().to_string();
        
        // Create a test file first
        setup_test_dir(&temp_path)?;
        
        let mut command = create_test_command("echo test > output.txt && cat output.txt");
        command.directory = dir_path;
        
        setup_test_env();
        let result = execute_command(&command);
        cleanup_test_env();
        
        assert!(result.is_ok(), "Command failed: {:?}", result.err());
        drop(temp_dir);
        Ok(())
    }
}
