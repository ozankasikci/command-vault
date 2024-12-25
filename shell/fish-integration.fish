# Command Vault Fish Integration

# Function to log commands to command-vault
function _command_vault_log_command --on-event fish_postexec
    set -l exit_code $status
    set -l cmd $argv[1]
    
    # Skip empty commands
    if test -z "$cmd"
        return
    end
    
    # Skip commands that start with space (if configured to ignore those)
    if string match -q " *" -- "$cmd"
        return
    end
    
    # Skip command-vault commands to prevent recursion
    if string match -q "command-vault *" -- "$cmd"
        return
    end
    
    # Log the command using command-vault
    command command-vault add --exit-code $exit_code "$cmd" &>/dev/null
end

# Initialize command-vault integration
if status is-interactive
    # Register the event handler
    functions -q _command_vault_log_command
    or source (status filename)
end 