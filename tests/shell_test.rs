use std::env;
use std::path::PathBuf;
use anyhow::Result;
use serial_test::serial;
use command_vault::shell::hooks::{
    detect_current_shell, get_shell_integration_dir, get_shell_integration_script,
    get_zsh_integration_path, get_bash_integration_path, get_fish_integration_path, init_shell
};

#[test]
#[serial]
fn test_detect_current_shell() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Test default case (no environment variables)
    assert_eq!(
        detect_current_shell(),
        "bash",
        "Should default to bash when no shell is set"
    );
    
    // Test with FISH_VERSION (highest priority)
    env::remove_var("SHELL"); // Ensure SHELL is not set
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(
        detect_current_shell(),
        "fish",
        "Should detect Fish via FISH_VERSION"
    );
    
    // Test with SHELL environment variable
    env::remove_var("FISH_VERSION");
    env::set_var("SHELL", "/bin/zsh");
    assert_eq!(
        detect_current_shell(),
        "zsh",
        "Should detect Zsh via SHELL"
    );
    
    // Test with SHELL environment variable (bash)
    env::set_var("SHELL", "/bin/bash");
    assert_eq!(
        detect_current_shell(),
        "bash",
        "Should detect Bash via SHELL"
    );
    
    // Test that FISH_VERSION takes precedence over SHELL
    env::set_var("SHELL", "/bin/zsh");
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(
        detect_current_shell(),
        "fish",
        "FISH_VERSION should take precedence over SHELL"
    );
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    } else {
        env::remove_var("SHELL");
    }
    if let Some(fish_version) = original_fish_version {
        env::set_var("FISH_VERSION", fish_version);
    } else {
        env::remove_var("FISH_VERSION");
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
#[serial]
fn test_init_shell() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Test default shell (bash)
    env::set_var("SHELL", "/bin/bash");
    let path = init_shell(None).unwrap();
    assert!(path.ends_with("bash-integration.sh"), "Path should end with bash-integration.sh");
    
    // Test with shell override
    let path = init_shell(Some("fish".to_string())).unwrap();
    assert!(path.ends_with("fish-integration.fish"), "Path should end with fish-integration.fish");
    
    // Clean up test environment
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    }
}

#[test]
#[serial]
fn test_detect_current_shell_fish_env() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Test FISH_VERSION detection
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(
        detect_current_shell(),
        "fish",
        "Should detect Fish via FISH_VERSION"
    );
    
    // Test FISH_VERSION takes precedence over SHELL
    env::set_var("SHELL", "/bin/zsh");
    env::set_var("FISH_VERSION", "3.1.2");
    assert_eq!(
        detect_current_shell(),
        "fish",
        "FISH_VERSION should take precedence over SHELL"
    );
    
    // Clean up test environment
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    }
}

#[test]
#[serial]
fn test_init_shell_explicit_fish() -> Result<()> {
    // Test with explicit fish shell override
    let path = init_shell(Some("fish".to_string()))?;
    assert!(path.ends_with("fish-integration.fish"));
    Ok(())
}

#[test]
#[serial]
fn test_detect_current_shell_fish_variants() {
    // Save original environment
    let original_shell = env::var("SHELL").ok();
    let original_fish_version = env::var("FISH_VERSION").ok();
    
    // Clean environment for testing
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Test various fish shell paths
    let fish_paths = vec![
        "/usr/local/bin/fish",
        "/opt/homebrew/bin/fish",
        "fish",
        "/usr/bin/fish",
        "/bin/fish",
    ];
    
    for path in fish_paths {
        env::remove_var("FISH_VERSION"); // Ensure FISH_VERSION doesn't interfere
        env::remove_var("SHELL"); // Clean SHELL before setting
        env::set_var("SHELL", path);
        assert_eq!(
            detect_current_shell(),
            "fish",
            "Failed to detect fish shell at path: {}",
            path
        );
    }
    
    // Clean up test environment
    env::remove_var("FISH_VERSION");
    env::remove_var("SHELL");
    
    // Restore original environment
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    }
    if let Some(version) = original_fish_version {
        env::set_var("FISH_VERSION", version);
    }
}

#[test]
fn test_shell_integration_paths() -> Result<()> {
    // Test shell integration directory
    let integration_dir = get_shell_integration_dir();
    assert!(integration_dir.ends_with("shell"));

    // Test zsh integration path
    let zsh_path = get_zsh_integration_path();
    assert!(zsh_path.ends_with("zsh-integration.zsh"));

    // Test bash integration path
    let bash_path = get_bash_integration_path();
    assert!(bash_path.ends_with("bash-integration.sh"));

    // Test fish integration path
    let fish_path = get_fish_integration_path();
    assert!(fish_path.ends_with("fish-integration.fish"));

    Ok(())
}

#[test]
fn test_init_shell_invalid_shell() -> Result<()> {
    // Test with an invalid shell
    let result = init_shell(Some("invalid_shell".to_string()));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported shell"));

    Ok(())
}
