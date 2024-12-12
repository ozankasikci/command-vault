use std::path::PathBuf;

pub fn get_shell_integration_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("shell");
    path
}

pub fn get_zsh_integration_path() -> PathBuf {
    let mut path = get_shell_integration_dir();
    path.push("zsh-integration.zsh");
    path
}

pub fn get_bash_integration_path() -> PathBuf {
    let mut path = get_shell_integration_dir();
    path.push("bash-integration.sh");
    path
}
