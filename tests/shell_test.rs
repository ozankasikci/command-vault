use std::env;
use std::path::PathBuf;
use anyhow::Result;
use command_vault::shell::hooks::{
    detect_current_shell, get_shell_integration_dir, get_shell_integration_script,
    get_zsh_integration_path, get_bash_integration_path, init_shell
};

#[test]
fn test_detect_current_shell() {
    // Save original SHELL env var
    let original_shell = env::var("SHELL").ok();
    
    // Test zsh detection
    env::set_var("SHELL", "/bin/zsh");
    assert_eq!(detect_current_shell(), Some("zsh".to_string()));
    
    // Test bash detection
    env::set_var("SHELL", "/bin/bash");
    assert_eq!(detect_current_shell(), Some("bash".to_string()));
    
    // Test unknown shell
    env::set_var("SHELL", "/bin/unknown");
    assert_eq!(detect_current_shell(), None);
    
    // Test no shell env var
    env::remove_var("SHELL");
    assert_eq!(detect_current_shell(), None);

    // Test shell paths with spaces
    env::set_var("SHELL", "/path with spaces/zsh");
    assert_eq!(detect_current_shell(), Some("zsh".to_string()));
    
    // Test shell paths with mixed case
    env::set_var("SHELL", "/bin/ZsH");
    let result = detect_current_shell();
    assert!(result.is_some());
    assert_eq!(result.unwrap().to_lowercase(), "zsh");
    
    // Restore original SHELL env var
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
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
    
    assert!(zsh_script.exists(), "ZSH integration script should exist");
    assert!(bash_script.exists(), "Bash integration script should exist");
}

#[test]
fn test_get_shell_specific_paths() {
    let zsh_path = get_zsh_integration_path();
    let bash_path = get_bash_integration_path();
    
    assert!(zsh_path.ends_with("zsh-integration.zsh"), "ZSH path should end with correct filename");
    assert!(bash_path.ends_with("bash-integration.sh"), "Bash path should end with correct filename");
    assert!(zsh_path.exists(), "ZSH integration script should exist");
    assert!(bash_path.exists(), "Bash integration script should exist");
}

#[test]
fn test_get_shell_integration_script() -> Result<()> {
    // Test valid shells
    let zsh_script = get_shell_integration_script("zsh")?;
    let bash_script = get_shell_integration_script("bash")?;
    
    assert!(zsh_script.ends_with("zsh-integration.zsh"));
    assert!(bash_script.ends_with("bash-integration.sh"));
    
    // Test case insensitivity
    let upper_zsh = get_shell_integration_script("ZSH")?;
    assert_eq!(upper_zsh, zsh_script);
    
    // Test error cases
    let fish_result = get_shell_integration_script("fish");
    assert!(fish_result.is_err());
    assert!(fish_result.unwrap_err().to_string().contains("Unsupported shell"));
    
    Ok(())
}

#[test]
fn test_init_shell() -> Result<()> {
    // Save original SHELL env var
    let original_shell = env::var("SHELL").ok();

    // Test with shell override
    let path = init_shell(Some("zsh".to_string()))?;
    assert!(path.ends_with("zsh-integration.zsh"));

    // Test with environment detection
    env::set_var("SHELL", "/bin/bash");
    let path = init_shell(None)?;
    assert!(path.ends_with("bash-integration.sh"));

    // Test error case - unknown shell
    env::set_var("SHELL", "/bin/fish");
    let result = init_shell(None);
    assert!(result.is_err());

    // Test error case - no shell env var
    env::remove_var("SHELL");
    let result = init_shell(None);
    assert!(result.is_err());

    // Restore original shell env var
    if let Some(shell) = original_shell {
        env::set_var("SHELL", shell);
    } else {
        env::remove_var("SHELL");
    }

    Ok(())
}
