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

## Installation

1. Build from source:
   ```bash
   cargo build --release
   ```

2. Install the binary:
   ```bash
   cargo install --path .
   ```

## Shell Integration

### Zsh

Add this to your `~/.zshrc`:
```bash
source /path/to/command-vault/shell/zsh-integration.zsh
```

### Bash

Add this to your `~/.bashrc`:
```bash
source /path/to/command-vault/shell/bash-integration.sh
```

## Usage

### Adding Commands
```bash
# Add a simple command
command-vault add "your command here"

# Add a command with tags
command-vault add "git push origin main" -t important -t git

# Add a command with exit code
command-vault add "make build" --exit-code 1
```

### Listing Commands
```bash
# List recent commands (newest first)
command-vault ls

# List oldest commands first
command-vault ls --asc

# List only the last 5 commands
command-vault ls --limit 5

# List the first 5 commands
command-vault ls --asc -l 5
```

### Searching Commands
```bash
# Basic search
command-vault search "git"

# Limit search results
command-vault search "docker" --limit 5
```

### Managing Tags
```bash
# Add tags to an existing command
command-vault tag add 123 important git

# Remove a tag from a command
command-vault tag remove 123 important

# List all tags and their usage
command-vault tag list

# Search commands by tag
command-vault tag search git
```

## Command Output Format

Commands are displayed with rich context:
```
(123) [2024-12-12 04:41:03] git push origin main
    Directory: /path/to/project
    Tags: git, important
```

Each command entry shows:
- Command ID in parentheses
- Timestamp in local timezone
- The actual command
- Working directory
- Exit code (if non-zero)
- Associated tags (if any)

## Development

### Prerequisites

- Rust 1.70 or later
- SQLite 3.x

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

## License

MIT
