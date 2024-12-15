# Command Vault

An advanced command history manager that helps you track and search your shell commands across sessions. Command Vault stores your shell commands with rich context including working directory, exit codes, and tags, making it easy to find and reuse commands later.

## Features

- 🔍 Smart search through command history
- 🏷️ Tag commands for better organization
- 📂 Track working directory for each command
- ❌ Record exit codes to identify failed commands
- 🕒 Chronological command history
- 🐚 Cross-shell support (Bash, Zsh)
- 💾 Local SQLite database for fast searching
- 🔄 Automatic command logging
- 📱 Terminal User Interface (TUI) for interactive usage
- 🔐 Safe command execution with validation

## Installation

### From Releases

You can download the latest release for your platform from the [releases page](https://github.com/yourusername/command-vault/releases).

#### Linux and macOS
```bash
# Download the latest release (replace X.Y.Z with the version number)
curl -LO https://github.com/yourusername/command-vault/releases/download/vX.Y.Z/command-vault-linux-amd64
# Make it executable
chmod +x command-vault-linux-amd64
# Move it to your PATH
sudo mv command-vault-linux-amd64 /usr/local/bin/command-vault
```

#### Windows
Download the Windows executable from the releases page and add it to your PATH.

### Building from Source

If you prefer to build from source, you'll need Rust installed on your system:

```bash
# Clone the repository
git clone https://github.com/yourusername/command-vault.git
cd command-vault

# Build the project
cargo build --release

# The binary will be available in target/release/command-vault
```

1. Add the following to your shell's configuration file (`~/.bashrc` or `~/.zshrc`):
   ```bash
   source "$(command-vault shell-init)"
   ```

## Usage

Command Vault can be used both from the command line and through its Terminal User Interface (TUI).

### Command Line Interface

```bash
# Add a command with tags
command-vault add --tags git,deploy git push origin main

# Search commands
command-vault search "git push"

# List recent commands
command-vault ls

# Delete a command
command-vault delete <command-id>

# Show command details
command-vault show <command-id>
```

## Development

Requirements:
- Rust 1.70 or higher
- SQLite 3.x

Run tests:
```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
