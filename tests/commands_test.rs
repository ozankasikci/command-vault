use anyhow::Result;
use chrono::{TimeZone, Utc};
use command_vault::{
    cli::{args::Commands, commands::handle_command},
    db::{Command, models::Parameter},
};
use tempfile::tempdir;
use std::env;

mod test_utils;
use test_utils::create_test_db;

// Set up test environment
#[ctor::ctor]
fn setup() {
    std::env::set_var("COMMAND_VAULT_TEST", "1");
}

#[test]
fn test_ls_empty() -> Result<()> {
    let (db, _db_dir) = create_test_db()?;
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 0);
    Ok(())
}

#[test]
fn test_handle_command_list() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: Vec::new(),
    };
    db.add_command(&command)?;
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "test command");
    Ok(())
}

#[test]
fn test_ls_with_limit() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    for i in 0..5 {
        let command = Command {
            id: None,
            command: format!("command {}", i),
            timestamp: Utc::now(),
            directory: "/test".to_string(),
            tags: vec![],
            parameters: Vec::new(),
        };
        db.add_command(&command)?;
    }
    let commands = db.list_commands(3, false)?;
    assert_eq!(commands.len(), 3);
    Ok(())
}

#[test]
fn test_ls_ordering() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let timestamps = vec![
        Utc.with_ymd_and_hms(2022, 1, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2022, 1, 2, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2022, 1, 3, 0, 0, 0).unwrap(),
    ];
    
    for (i, timestamp) in timestamps.iter().enumerate() {
        let command = Command {
            id: None,
            command: format!("command {}", i),
            timestamp: *timestamp,
            directory: "/test".to_string(),
            tags: vec![],
            parameters: Vec::new(),
        };
        db.add_command(&command)?;
    }
    
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "command 2");
    assert_eq!(commands[1].command, "command 1");
    assert_eq!(commands[2].command, "command 0");
    Ok(())
}

#[test]
fn test_delete_command() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: Vec::new(),
    };
    let id = db.add_command(&command)?;
    db.delete_command(id)?;
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 0);
    Ok(())
}

#[test]
fn test_search_commands() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: Vec::new(),
    };
    db.add_command(&command)?;
    let commands = db.search_commands("test", 10)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "test command");
    Ok(())
}

#[test]
fn test_add_command_with_tags() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let temp_dir = tempdir()?;
    std::fs::create_dir_all(temp_dir.path())?;

    // Change to the test directory
    let original_dir = env::current_dir()?;
    let test_dir = temp_dir.path().canonicalize()?;
    env::set_current_dir(&test_dir)?;
    
    let command = vec!["test".to_string(), "command".to_string()];
    let add_command = Commands::Add { 
        command: command.clone(), 
        tags: vec!["tag1".to_string(), "tag2".to_string()] 
    };
    
    handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "test command");
    assert_eq!(commands[0].tags, vec!["tag1", "tag2"]);
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_command_with_output() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Test command that would produce output
    let command = vec!["echo".to_string(), "\"Hello, World!\"".to_string()];
    let add_command = Commands::Add { 
        command: command.clone(), 
        tags: vec![] 
    };
    
    handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "echo \"Hello, World!\"");
    
    Ok(())
}

#[test]
fn test_command_with_stderr() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Test command that would produce stderr
    let command = vec!["ls".to_string(), "nonexistent_directory".to_string()];
    let add_command = Commands::Add { 
        command: command.clone(), 
        tags: vec![] 
    };
    
    handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "ls nonexistent_directory");
    
    Ok(())
}

#[test]
fn test_git_log_format_command() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let temp_dir = tempdir()?;
    std::fs::create_dir_all(temp_dir.path())?;

    // Change to the test directory
    let original_dir = env::current_dir()?;
    let test_dir = temp_dir.path().canonicalize()?;
    env::set_current_dir(&test_dir)?;
    
    // Add the git log command with format string
    let format_str = "%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset";
    let command = vec![
        "git".to_string(),
        "log".to_string(),
        "--graph".to_string(),
        format!("--pretty=format:{}", format_str),
        "--abbrev-commit".to_string(),
    ];
    
    let add_command = Commands::Add { 
        command: command.clone(), 
        tags: vec![] 
    };
    
    handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].command, 
        format!("git log --graph \"--pretty=format:{}\" --abbrev-commit", format_str)
    );
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_parameter_parsing() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Test basic parameter
    let command = Command {
        id: None,
        command: "echo @message".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![Parameter::with_description(
            "message".to_string(),
            Some("User_name".to_string())
        )],
    };
    let id = db.add_command(&command)?;
    let saved = db.get_command(id)?.unwrap();
    assert_eq!(saved.parameters.len(), 1);
    assert_eq!(saved.parameters[0].name, "message");
    assert_eq!(saved.parameters[0].description, Some("User_name".to_string()));
    
    // Test parameter with description
    let command = Command {
        id: None,
        command: "echo @message:User_name".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![Parameter::with_description(
            "message".to_string(),
            Some("User_name".to_string())
        )],
    };
    let id = db.add_command(&command)?;
    let saved = db.get_command(id)?.unwrap();
    assert_eq!(saved.parameters.len(), 1);
    assert_eq!(saved.parameters[0].name, "message");
    assert_eq!(saved.parameters[0].description, Some("User_name".to_string()));
    
    Ok(())
}

#[test]
fn test_exec_command_with_parameters() -> Result<()> {
    // Ensure we're in test mode
    std::env::set_var("COMMAND_VAULT_TEST", "1");
    
    let (mut db, _db_dir) = create_test_db()?;
    let temp_dir = tempdir()?;
    let test_dir = temp_dir.path().canonicalize()?;
    
    // Add a command with parameters
    let command = Command {
        id: None,
        command: "echo @message".to_string(),
        timestamp: Utc::now(),
        directory: test_dir.to_string_lossy().to_string(),
        tags: vec![],
        parameters: vec![Parameter::with_description(
            "message".to_string(),
            Some("test message".to_string())
        )],
    };
    let id = db.add_command(&command)?;
    
    // Execute command with default parameter
    let exec_command = Commands::Exec { command_id: id };
    handle_command(exec_command, &mut db)?;
    
    // Verify command was saved correctly
    let saved = db.get_command(id)?.unwrap();
    assert_eq!(saved.parameters.len(), 1);
    assert_eq!(saved.parameters[0].name, "message");
    assert_eq!(saved.parameters[0].description, Some("test message".to_string()));
    
    Ok(())
}

#[test]
fn test_exec_command_not_found() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Try to execute a non-existent command
    let exec_command = Commands::Exec { command_id: 999 };
    let result = handle_command(exec_command, &mut db);
    
    // Verify that we get an error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Command not found"));
    
    Ok(())
}

#[test]
fn test_parameter_validation() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Test invalid parameter name (starts with number)
    let command = Command {
        id: None,
        command: "echo @1name".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![],
    };
    let id = db.add_command(&command)?;
    let saved = db.get_command(id)?.unwrap();
    assert_eq!(saved.parameters.len(), 0); // Invalid parameter should be ignored
    
    // Test invalid parameter name (special characters)
    let command = Command {
        id: None,
        command: "echo @name!".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![],
    };
    let id = db.add_command(&command)?;
    let saved = db.get_command(id)?.unwrap();
    assert_eq!(saved.parameters.len(), 0); // Invalid parameter should be ignored
    
    Ok(())
}

#[test]
fn test_command_with_spaces_in_parameters() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "echo @message".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["test".to_string()],
        parameters: vec![Parameter::with_description(
            "message".to_string(),
            Some("A test message".to_string())
        )],
    };
    
    db.add_command(&command)?;
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "echo @message");
    assert_eq!(commands[0].parameters[0].name, "message");
    assert_eq!(commands[0].parameters[0].description, Some("A test message".to_string()));
    Ok(())
}

#[test]
fn test_command_with_multiple_tags() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
        parameters: Vec::new(),
    };
    
    db.add_command(&command)?;
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].tags.len(), 3);
    assert!(commands[0].tags.contains(&"tag1".to_string()));
    assert!(commands[0].tags.contains(&"tag2".to_string()));
    assert!(commands[0].tags.contains(&"tag3".to_string()));
    Ok(())
}

#[test]
fn test_command_with_special_chars() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let command = Command {
        id: None,
        command: "grep -r \"@pattern\" @directory".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["search".to_string()],
        parameters: vec![
            Parameter::with_description(
                "pattern".to_string(),
                Some("Search pattern".to_string())
            ),
            Parameter::with_description(
                "directory".to_string(),
                Some("Directory to search in".to_string())
            ),
        ],
    };
    
    db.add_command(&command)?;
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].parameters.len(), 2);
    assert_eq!(commands[0].parameters[0].name, "pattern");
    assert_eq!(commands[0].parameters[0].description, Some("Search pattern".to_string()));
    assert_eq!(commands[0].parameters[1].name, "directory");
    assert_eq!(commands[0].parameters[1].description, Some("Directory to search in".to_string()));
    Ok(())
}

#[test]
fn test_handle_command_delete() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Add a test command
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: Vec::new(),
    };
    let id = db.add_command(&command)?;
    
    // Verify command exists
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 1);
    
    // Delete the command
    handle_command(Commands::Delete { command_id: id }, &mut db)?;
    
    // Verify command was deleted
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 0);
    Ok(())
}

#[test]
fn test_handle_command_delete_nonexistent() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Try to delete a command that doesn't exist
    let result = handle_command(Commands::Delete { command_id: 999 }, &mut db);
    
    // Verify we get an error
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Command with ID 999 not found"));
    Ok(())
}

#[test]
fn test_handle_command_delete_with_tags() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Add a test command with tags
    let command = Command {
        id: None,
        command: "test command".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["test".to_string(), "example".to_string()],
        parameters: Vec::new(),
    };
    let id = db.add_command(&command)?;
    
    // Verify command exists with tags
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].tags.len(), 2);
    
    // Delete the command
    handle_command(Commands::Delete { command_id: id }, &mut db)?;
    
    // Verify command and its tags were deleted
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 0);
    
    // Verify tags were removed
    let tags = db.list_tags()?;
    assert_eq!(tags.len(), 0);
    Ok(())
}
