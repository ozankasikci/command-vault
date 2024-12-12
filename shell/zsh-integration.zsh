# Command Vault ZSH Integration

# Function to log commands to command-vault
_command_vault_log_command() {
    local exit_code=$?
    local cmd=$(fc -ln -1)
    
    # Trim whitespace
    cmd=$(echo "$cmd" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
    
    # Skip empty commands
    if [ -z "$cmd" ]; then
        return
    fi
    
    # Skip commands that start with space (if configured to ignore those)
    if [[ "$cmd" =~ ^[[:space:]] ]]; then
        return
    fi
    
    # Log the command using command-vault
    command-vault add --exit-code $exit_code "$cmd" &>/dev/null
}

# Add the function to the precmd hook
autoload -Uz add-zsh-hook
add-zsh-hook precmd _command_vault_log_command
