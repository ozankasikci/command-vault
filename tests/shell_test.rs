use std::env;
use std::path::PathBuf;
use command_vault::shell::hooks::{detect_current_shell, get_shell_integration_dir};

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
