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

1. Build from source:
   ```bash
   cargo build --release
   ```

2. Add the following to your shell's configuration file (`~/.bashrc` or `~/.zshrc`):
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

### Terminal User Interface (TUI)

Launch the TUI mode:
```bash
command-vault tui
```

In TUI mode, you can:
- Browse through your command history
- Search commands with real-time filtering
- Add new commands with tags
- View command details including exit codes and timestamps
- Delete commands

## Project Structure

```
src/
├── cli/        # Command-line interface implementation
├── db/         # Database operations and models
├── shell/      # Shell integration and hooks
├── ui/         # Terminal User Interface components
├── utils/      # Utility functions
├── lib.rs      # Library interface
└── main.rs     # Application entry point
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
