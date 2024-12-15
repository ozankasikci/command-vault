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

## Example Use Cases

Here are some common scenarios where command-vault can be particularly useful:

### Docker Commands
```bash
# Clean up all unused containers, networks, and dangling images
cv add --tags docker,cleanup 'docker system prune -af --volumes'

# Run PostgreSQL container with specific config
cv add --tags docker,db 'docker run --name postgres -e POSTGRES_PASSWORD=mysecretpassword -p 5432:5432 -v pgdata:/var/lib/postgresql/data -d postgres:15'

# Build and run docker-compose with specific env file
cv add --tags docker,compose 'docker-compose -f docker-compose.prod.yml --env-file .env.production up -d --build'
```

### Git Operations
```bash
# Undo last commit but keep changes
cv add --tags git 'git reset --soft HEAD~1'

# Clean up local branches that were merged to main
cv add --tags git,cleanup 'git branch --merged main | grep -v "^[ *]*main" | xargs git branch -d'

# Complex git log format
cv add --tags git,log 'git log --graph --pretty=format:"%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset" --abbrev-commit'
```

### System Administration
```bash
# Find large files and directories
cv add --tags system 'find . -type f -size +100M -exec ls -lh {} \; | sort -k5,5 -h'

# Monitor system resources
cv add --tags system,monitoring 'top -b -n 1 | head -n 20 > system_status.log && free -h >> system_status.log && df -h >> system_status.log'

# Complex file search and replace
cv add --tags files 'find . -type f -name "*.js" -exec sed -i "s/oldText/newText/g" {} +'
```

### Development Tools
```bash
# Start development environment with specific config
cv add --tags dev 'export NODE_ENV=development && npm run build && npm run start:dev'

# Run tests with specific configuration
cv add --tags test 'pytest --cov=app --cov-report=html --verbose --log-level=DEBUG tests/'

# Complex database query
cv add --tags db 'psql -h localhost -U myuser -d mydb -c "SELECT table_name, pg_size_pretty(pg_total_relation_size(table_name::text)) AS size FROM information_schema.tables WHERE table_schema = '\''public'\''"'
```

### AWS CLI Commands
```bash
# List EC2 instances with specific tags
cv add --tags aws 'aws ec2 describe-instances --filters "Name=tag:Environment,Values=Production" --query "Reservations[].Instances[].{ID:InstanceId,Type:InstanceType,Name:Tags[?Key=='\''Name'\''].Value|[0]}" --output table'

# S3 sync with specific excludes
cv add --tags aws,s3 'aws s3 sync . s3://my-bucket/path --exclude "*.tmp" --exclude "node_modules/*" --delete'
```

These commands can be easily retrieved later using tags:
```bash
# Find all docker-related commands
cv search --tag docker

# Find all cleanup commands
cv search --tag cleanup
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
