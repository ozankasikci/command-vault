use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

/// Get the directory containing shell integration scripts
pub fn get_shell_integration_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("shell");
    path
}

/// Get the path to the ZSH integration script
pub fn get_zsh_integration_path() -> PathBuf {
    let mut path = get_shell_integration_dir();
    path.push("zsh-integration.zsh");
    path
}

/// Get the path to the Bash integration script
pub fn get_bash_integration_path() -> PathBuf {
    let mut path = get_shell_integration_dir();
    path.push("bash-integration.sh");
    path
}

/// Get the path to the Fish integration script
pub fn get_fish_integration_path() -> PathBuf {
    let mut path = get_shell_integration_dir();
    path.push("fish-integration.fish");
    path
}

/// Detect the current shell from environment variables
pub fn detect_current_shell() -> Option<String> {
    // First check for Fish-specific environment variable
    if env::var("FISH_VERSION").is_ok() {
        return Some("fish".to_string());
    }

    // Then check SHELL environment variable
    if let Ok(shell) = env::var("SHELL") {
        let shell_lower = shell.to_lowercase();
        
        // Check for each shell type in order
        if shell_lower.contains("zsh") || shell_lower.ends_with("/zsh") {
            Some("zsh".to_string())
        } else if shell_lower.contains("bash") || shell_lower.ends_with("/bash") {
            Some("bash".to_string())
        } else if shell_lower.contains("fish") || shell_lower.ends_with("/fish") {
            Some("fish".to_string())
        } else {
            None
        }
    } else {
        None
    }
}

/// Get the shell integration script path for a specific shell
pub fn get_shell_integration_script(shell: &str) -> Result<PathBuf> {
    let shell_lower = shell.to_lowercase();
    match shell_lower.as_str() {
        "zsh" => Ok(get_zsh_integration_path()),
        "bash" => Ok(get_bash_integration_path()),
        "fish" => Ok(get_fish_integration_path()),
        _ => Err(anyhow!("Unsupported shell: {}", shell)),
    }
}

/// Initialize shell integration
pub fn init_shell(shell_override: Option<String>) -> Result<PathBuf> {
    let shell = if let Some(shell) = shell_override {
        shell
    } else {
        detect_current_shell().ok_or_else(|| anyhow!("Could not detect shell"))?
    };

    get_shell_integration_script(&shell)
}
