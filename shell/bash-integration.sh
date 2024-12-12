# Lazy History Bash Integration

# Function to log commands to lazy-history
_lazy_history_log_command() {
    local exit_code=$?
    local cmd=$(history 1)
    
    # Extract just the command part (remove the history number)
    cmd=$(echo "$cmd" | sed 's/^[[:space:]]*[0-9]*[[:space:]]*//')
    
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
    
    # Log the command using lazy-history
    lazy-history add --exit-code $exit_code "$cmd" &>/dev/null
}

# Add the function to PROMPT_COMMAND
if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="_lazy_history_log_command"
else
    PROMPT_COMMAND="_lazy_history_log_command;$PROMPT_COMMAND"
fi
