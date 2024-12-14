use anyhow::Result;
use command_vault::cli::{self, args::Commands};
use command_vault::db::Command;
use tempfile::tempdir;
use chrono::{Utc, TimeZone};
use std::env;

mod test_utils;
use test_utils::create_test_db;

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
        exit_code: None,
        tags: vec![],
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
            exit_code: None,
            tags: vec![],
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
            exit_code: None,
            tags: vec![],
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
        exit_code: None,
        tags: vec![],
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
        exit_code: None,
        tags: vec![],
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

    // Change to the test directory
    let original_dir = env::current_dir()?;
    let test_dir = temp_dir.path().canonicalize()?;
    env::set_current_dir(&test_dir)?;
    
    let command = "test command".to_string();
    let add_command = Commands::Add { 
        command: vec![command.clone()], 
        exit_code: None, 
        tags: vec!["tag1".to_string(), "tag2".to_string()] 
    };
    
    cli::handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "test command");
    assert_eq!(commands[0].tags, vec!["tag1", "tag2"]);
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_execute_command() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    let temp_dir = tempdir()?;

    // Change to the test directory
    let original_dir = env::current_dir()?;
    let test_dir = temp_dir.path().canonicalize()?;
    env::set_current_dir(&test_dir)?;
    
    let command = "echo test".to_string();
    let add_command = Commands::Add { 
        command: vec![command.clone()], 
        exit_code: None, 
        tags: vec![] 
    };
    
    cli::handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "echo test");
    assert_eq!(commands[0].exit_code, Some(0));
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_empty_command_validation() -> Result<()> {
    let (mut db, _db_dir) = create_test_db()?;
    
    // Try adding an empty command
    let add_command = Commands::Add { 
        command: vec!["".to_string()], 
        exit_code: None, 
        tags: vec![] 
    };
    
    let result = cli::handle_command(add_command, &mut db);
    assert!(result.is_err());
    
    Ok(())
}
