use std::env;
use std::path::PathBuf;
use anyhow::Result;
use command_vault::shell::hooks::{
    detect_current_shell, get_shell_integration_dir, get_shell_integration_script,
    get_zsh_integration_path, get_bash_integration_path, get_fish_integration_path, init_shell
};

#[test]
fn test_detect_current_shell() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Test with zsh
    env::set_var("SHELL", "/bin/zsh");
    assert_eq!(detect_current_shell(), Some("zsh".to_string()));
    env::remove_var("SHELL");  // Clean up after test
    
    // Test with bash
    env::set_var("SHELL", "/bin/bash");
    assert_eq!(detect_current_shell(), Some("bash".to_string()));
    env::remove_var("SHELL");  // Clean up after test
    
    // Test with fish
    env::set_var("SHELL", "/bin/fish");
    assert_eq!(detect_current_shell(), Some("fish".to_string()));
    env::remove_var("SHELL");  // Clean up after test
    
    // Test with unknown shell
    env::set_var("SHELL", "/bin/unknown");
    assert_eq!(detect_current_shell(), None);
    env::remove_var("SHELL");  // Clean up after test
    
    // Test with no SHELL variable
    assert_eq!(detect_current_shell(), None);
    
    // Test with FISH_VERSION set
    env::remove_var("SHELL");  // Make sure SHELL is not set
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(detect_current_shell(), Some("fish".to_string()));
    env::remove_var("FISH_VERSION");  // Clean up after test
    
    // Test FISH_VERSION takes precedence over SHELL
    env::set_var("FISH_VERSION", "3.1.2");
    env::set_var("SHELL", "/bin/zsh");
    assert_eq!(detect_current_shell(), Some("fish".to_string()));
    env::remove_var("FISH_VERSION");  // Clean up after test
    env::remove_var("SHELL");  // Clean up after test
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    }
}

#[test]
fn test_get_shell_integration_dir() {
    let dir = get_shell_integration_dir();
    assert!(dir.is_dir(), "Shell integration directory should exist");
    assert!(dir.ends_with("shell"), "Directory should end with 'shell'");
    
    // Check if shell scripts exist
    let zsh_script = dir.join("zsh-integration.zsh");
    let bash_script = dir.join("bash-integration.sh");
    let fish_script = dir.join("fish-integration.fish");
    
    assert!(zsh_script.exists(), "ZSH integration script should exist");
    assert!(bash_script.exists(), "Bash integration script should exist");
    assert!(fish_script.exists(), "Fish integration script should exist");
}

#[test]
fn test_get_shell_specific_paths() {
    let zsh_path = get_zsh_integration_path();
    let bash_path = get_bash_integration_path();
    let fish_path = get_fish_integration_path();
    
    assert!(zsh_path.ends_with("zsh-integration.zsh"), "ZSH path should end with correct filename");
    assert!(bash_path.ends_with("bash-integration.sh"), "Bash path should end with correct filename");
    assert!(fish_path.ends_with("fish-integration.fish"), "Fish path should end with correct filename");
    assert!(zsh_path.exists(), "ZSH integration script should exist");
    assert!(bash_path.exists(), "Bash integration script should exist");
    assert!(fish_path.exists(), "Fish integration script should exist");
}

#[test]
fn test_get_shell_integration_script() -> Result<()> {
    // Test valid shells
    let zsh_script = get_shell_integration_script("zsh")?;
    let bash_script = get_shell_integration_script("bash")?;
    let fish_script = get_shell_integration_script("fish")?;
    
    assert!(zsh_script.ends_with("zsh-integration.zsh"));
    assert!(bash_script.ends_with("bash-integration.sh"));
    assert!(fish_script.ends_with("fish-integration.fish"));
    
    // Test case insensitivity
    let upper_zsh = get_shell_integration_script("ZSH")?;
    assert_eq!(upper_zsh, zsh_script);
    
    // Test error cases
    let unknown_result = get_shell_integration_script("unknown");
    assert!(unknown_result.is_err());
    assert!(unknown_result.unwrap_err().to_string().contains("Unsupported shell"));
    
    Ok(())
}

#[test]
fn test_init_shell() -> Result<()> {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");

    // Test with shell override (should work regardless of env)
    let path = init_shell(Some("zsh".to_string()))?;
    assert!(path.ends_with("zsh-integration.zsh"));

    // Test with environment detection - bash with full path
    env::remove_var("FISH_VERSION");  // Make sure FISH_VERSION is not set
    env::set_var("SHELL", "/usr/local/bin/bash");
    let path = init_shell(None)?;
    assert!(path.ends_with("bash-integration.sh"));
    env::remove_var("SHELL");

    // Test with environment detection - bash with relative path
    env::remove_var("FISH_VERSION");  // Make sure FISH_VERSION is not set
    env::set_var("SHELL", "bash");
    let path = init_shell(None)?;
    assert!(path.ends_with("bash-integration.sh"));
    env::remove_var("SHELL");

    // Test with environment detection - zsh
    env::remove_var("FISH_VERSION");  // Make sure FISH_VERSION is not set
    env::set_var("SHELL", "/bin/zsh");
    let path = init_shell(None)?;
    assert!(path.ends_with("zsh-integration.zsh"));
    env::remove_var("SHELL");

    // Test with environment detection - fish
    env::remove_var("FISH_VERSION");  // Make sure FISH_VERSION is not set
    env::set_var("SHELL", "/bin/fish");
    let path = init_shell(None)?;
    assert!(path.ends_with("fish-integration.fish"));
    env::remove_var("SHELL");

    // Test error case - unknown shell
    env::remove_var("FISH_VERSION");  // Make sure FISH_VERSION is not set
    env::set_var("SHELL", "/bin/unknown");
    let result = init_shell(None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Could not detect shell"));
    env::remove_var("SHELL");

    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    }

    Ok(())
}

#[test]
fn test_init_shell_explicit_fish() -> Result<()> {
    // Test with explicit fish shell override
    let path = init_shell(Some("fish".to_string()))?;
    assert!(path.ends_with("fish-integration.fish"));
    Ok(())
}

#[test]
fn test_detect_current_shell_fish_variants() {
    // Save original SHELL env var
    let original_shell = env::var("SHELL").ok();
    
    // Test various fish shell paths
    let fish_paths = vec![
        "/usr/local/bin/fish",
        "/opt/homebrew/bin/fish",
        "fish",
        "/usr/bin/fish",
        "/bin/fish",
    ];
    
    for path in fish_paths {
        env::set_var("SHELL", path);
        assert_eq!(detect_current_shell(), Some("fish".to_string()), "Failed to detect fish shell at path: {}", path);
    }
    
    // Restore original SHELL env var
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
}

#[test]
fn test_detect_current_shell_fish_env() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Test Fish detection via FISH_VERSION
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(detect_current_shell(), Some("fish".to_string()), "Should detect Fish via FISH_VERSION");
    
    // Test Fish detection via FISH_VERSION even when SHELL is set to something else
    env::set_var("SHELL", "/bin/zsh");
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(detect_current_shell(), Some("fish".to_string()), "Should detect Fish via FISH_VERSION even when SHELL is set to something else");
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    } else {
        env::remove_var("SHELL");
    }
    
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    } else {
        env::remove_var("FISH_VERSION");
    }
}
