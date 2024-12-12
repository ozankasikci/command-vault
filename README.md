# Lazy History

An advanced command history manager that helps you track and search your shell commands across sessions.

## Features

- Automatically logs all commands with timestamps and exit codes
- Search through command history with powerful filters
- Tag commands for better organization
- Cross-shell support (Bash, Zsh)
- Local SQLite database for fast searching
- Displays commands with context (directory, exit code, timestamp)

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
source /path/to/lazy-history/shell/zsh-integration.zsh
```

### Bash

Add this to your `~/.bashrc`:
```bash
source /path/to/lazy-history/shell/bash-integration.sh
```

## Usage

### Adding Commands Manually
```bash
# Add a simple command
lazy-history add "your command here"

# Add a command with tags
lazy-history add "git push origin main" -t important -t git
```

### Searching Commands
```bash
# Basic search
lazy-history search "git"

# Limit results
lazy-history search "docker" --limit 5
```

### Managing Tags
```bash
# Add tags to an existing command
lazy-history tag add 123 important git

# Remove a tag from a command
lazy-history tag remove 123 important

# List all tags and their usage
lazy-history tag list

# Search commands by tag
lazy-history tag search git
```

## Database Location

The command history is stored in `~/.lazy-history/history.db`

## License

MIT
