# Command Vault
[![Crates.io](https://img.shields.io/crates/v/command-vault.svg?style=flat-square)](https://crates.io/crates/command-vault)
[![Documentation](https://docs.rs/command-vault/badge.svg)](https://docs.rs/command-vault)
![example workflow](https://github.com/ozankasikci/command-vault/actions/workflows/test.yml/badge.svg)
[![codecov](https://codecov.io/gh/ozankasikci/command-vault/branch/main/graph/badge.svg)](https://codecov.io/gh/ozankasikci/command-vault)

Command Vault is a command manager for storing, and executing your complex commands. It provides a user-friendly interface to search, list, and delete commands, as well as tag commands for better organization.

## Table of Contents
- [Features](#features)
- [Usage](#usage)
  - [Add Commands](#add-commands)
  - [Search Commands](#search-commands)
  - [List Commands](#list-commands)
  - [Delete Commands](#delete-commands)
  - [Tag Commands](#tag-commands)
- [Installation](#installation)
  - [From Releases](#from-releases)
  - [Shell Integration](#shell-integration)
  - [Building from Source](#building-from-source)
- [Development](#development)
- [Shell Aliases](#shell-aliases)
- [License](#license)

## Features

- 🔍 Smart search through command history
- 🏷️ Tag commands for better organization
- 🐚 Cross-shell support (Bash, Zsh)
- 💾 Local SQLite database for fast searching
- 🔐 Safe command execution with validation

## Usage

### Add Commands
```bash
# Add a command with tags
command-vault add --tags git,deploy git push origin main
command-vault add echo "Hello, world!"

# Add a command with parameters
command-vault add "git commit -m @message:Commit message"
command-vault add "curl -X POST @url:API endpoint -d @data:JSON payload"
```
![Add Command](demo/add-command3.gif)

### Parameters
You can add dynamic parameters to your commands using the `@parameter` syntax:
- Simple parameter: `@name`

Examples:
```bash
# Git commit with message parameter
git commit -m "@message"
```

When executing a command with parameters, Command Vault will prompt you to enter values for each parameter.

### Search Commands
```bash
# Search commands
command-vault search "git push"
```
![Search Commands](demo/search-command.gif)

### List Commands
```bash
# List recent commands
command-vault ls
```
![List Commands](demo/ls-command2.gif)

### Delete Commands
```bash
# Delete a command
command-vault delete <command-id>
```
![Delete Commands](demo/delete-command.gif)

### Tag Commands
```bash
# Show tag command
command-vault tag # Show tag related commands
command-vault tag list # List tag related commands
```
![Tag Commands](demo/tag-command.gif)

## Installation

### From Releases

You can download the latest release for your platform from the [releases page](https://github.com/yourusername/command-vault/releases).

#### Linux and macOS
```bash
# Download the latest release (replace X.Y.Z with the version number)
curl -LO https://github.com/yourusername/command-vault/releases/download/v0.1.5/command-vault-linux-amd64
# Make it executable
chmod +x command-vault-linux-amd64
# Move it to your PATH
sudo mv command-vault-linux-amd64 /usr/local/bin/command-vault

# Initialize shell integration (add to your .bashrc or .zshrc)
source "$(command-vault shell-init)"
```

#### Windows
Download the Windows executable from the releases page and add it to your PATH.

### Shell Integration

Command Vault needs to be integrated with your shell to automatically track commands. Add this to your shell's RC file:

```bash
# For Bash (~/.bashrc)
source "$(command-vault shell-init)"

# For Zsh (~/.zshrc)
source "$(command-vault shell-init)"
```

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

Add the following to your shell's configuration file (`~/.bashrc` or `~/.zshrc`):
   ```bash
   source "$(command-vault shell-init)"
   ```

## Development

### Running Tests
```bash
cargo test
```

### Code Coverage
```bash
# Generate coverage report (requires cargo-tarpaulin)
./scripts/coverage.sh

# View the report in your browser
open coverage/tarpaulin-report.html
```

## Shell Aliases

For easier access, you can add aliases to your shell configuration:

### For Bash/Zsh (add to ~/.zshrc or ~/.bashrc)
```bash
# to use as cmdv:
alias cmdv='command-vault'
# or to use as cv:
alias cv='command-vault'
```

After adding the aliases, restart your shell or run:
```bash
source ~/.zshrc  # for Zsh
source ~/.bashrc # for Bash
```

Now you can use shorter commands:
```bash
cv add 'echo Hello'
cmdv ls
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
