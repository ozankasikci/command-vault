# Project Scope

## Project Overview
Command Vault is a command manager for storing and executing complex commands. It offers a terminal user interface (TUI) that allows users to:
- Store commands with parameters
- Tag commands for better organization
- Search through stored commands
- Execute commands with parameter substitution
- Cross-shell compatibility

## Key Features
- Smart search for finding commands
- Parameter substitution for dynamic command execution
- Tag-based organization
- Cross-shell support (bash, zsh, fish)
- Local SQLite database for storing commands

## Maintenance Guidelines

### CHANGELOG.md
The CHANGELOG.md file should be updated with each significant change to the codebase:
- New features should be added under "### Added"
- Bug fixes should be added under "### Fixed"
- Breaking changes should be added under "### Changed"
- Deprecated features should be added under "### Deprecated"

Each entry should include:
- A clear description of the change
- Reference to relevant issue numbers (if applicable)
- Any special notes for users

### Parameter Handling
When making changes to parameter handling, be careful to:
1. Maintain the format `@param` for simple parameters and `@param:Description` for parameters with descriptions
2. Ensure descriptions are properly removed from the final command
3. Test with different types of parameters to ensure proper substitution

### Code Structure
The codebase is organized into modules:
- `src/db`: Database interactions and models
- `src/exec`: Command execution
- `src/cli`: Command-line interface
- `src/ui`: Terminal UI components
- `src/utils`: Utility functions 
- `src/shell`: Shell integration

When adding new functionality, place it in the appropriate module. 