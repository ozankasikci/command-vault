# Command Vault

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

## Project Structure

```
src/
├── cli/           # Command-line interface code
│   ├── args.rs    # Command line arguments
│   ├── commands.rs # Command implementations
│   └── mod.rs
├── db/            # Database-related code
│   ├── models.rs  # Database models
│   ├── store.rs   # Database operations
│   └── mod.rs
├── shell/         # Shell integration
│   ├── hooks.rs   # Shell hook implementations
│   └── mod.rs
├── utils/         # Utility functions
│   ├── time.rs    # Time-related utilities
│   └── mod.rs
└── main.rs        # Application entry point
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

### Searching Commands
```bash
# Basic search
command-vault search "git"

# Limit results
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
