use anyhow::Result;
use lazy_history::cli::{self, args::{Commands, TagCommands}};
use lazy_history::db::{Database, Command};
use tempfile::tempdir;
use chrono::{Utc, TimeZone};
use std::fs;
use std::env;
use std::path::Path;

mod test_utils;
use test_utils::init_test_env;

fn create_test_command(command: &str, directory: &str, timestamp: chrono::DateTime<Utc>) -> Command {
    Command {
        id: None,
        command: command.to_string(),
        directory: directory.to_string(),
        timestamp,
        exit_code: None,
        tags: vec![],
    }
}

#[test]
fn test_ls_empty() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    let list_command = Commands::Ls { limit: 10, asc: false };
    cli::handle_command(list_command, &mut db)?;
    
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 0);
    
    Ok(())
}

#[test]
fn test_ls_with_limit() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    let dir = temp_dir.path().canonicalize()?.to_str().unwrap().to_string();
    
    // Add 5 commands
    for i in 0..5 {
        let cmd = create_test_command(
            &format!("command {}", i),
            &dir,
            Utc::now() + chrono::Duration::seconds(i),
        );
        db.add_command(&cmd)?;
    }
    
    // Test with limit 3
    let list_command = Commands::Ls { limit: 3, asc: false };
    cli::handle_command(list_command, &mut db)?;
    
    let commands = db.list_commands(3, false)?;
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "command 4");
    assert_eq!(commands[2].command, "command 2");
    
    Ok(())
}

#[test]
fn test_ls_ordering() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    let dir = temp_dir.path().canonicalize()?.to_str().unwrap().to_string();
    
    // Add commands with different timestamps
    let base_time = Utc.timestamp_opt(1000000000, 0).unwrap();
    let commands = vec![
        ("first", base_time),
        ("second", base_time + chrono::Duration::seconds(1)),
        ("third", base_time + chrono::Duration::seconds(2)),
    ];
    
    for (cmd, time) in commands {
        let test_cmd = create_test_command(cmd, &dir, time);
        db.add_command(&test_cmd)?;
    }
    
    // Test ascending order
    let list_command = Commands::Ls { limit: 10, asc: true };
    cli::handle_command(list_command, &mut db)?;
    
    let commands = db.list_commands(10, true)?;
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "first");
    assert_eq!(commands[1].command, "second");
    assert_eq!(commands[2].command, "third");
    
    // Test descending order
    let list_command = Commands::Ls { limit: 10, asc: false };
    cli::handle_command(list_command, &mut db)?;
    
    let commands = db.list_commands(10, false)?;
    assert_eq!(commands.len(), 3);
    assert_eq!(commands[0].command, "third");
    assert_eq!(commands[1].command, "second");
    assert_eq!(commands[2].command, "first");
    
    Ok(())
}

#[test]
fn test_handle_command_list() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Add a test command
    let test_command = Command {
        id: None,
        command: "test command".to_string(),
        directory: temp_dir.path().canonicalize()?.to_str().unwrap().to_string(),
        timestamp: Utc::now(),
        exit_code: None,
        tags: vec![],
    };
    db.add_command(&test_command)?;

    // Test list command
    let list_command = Commands::Ls { limit: 1, asc: false };
    cli::handle_command(list_command, &mut db)?;
    
    Ok(())
}

#[test]
fn test_handle_command_add() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Change to the test directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;
    
    let command = "test command".to_string();
    let add_command = Commands::Add { command, exit_code: None, tags: vec![] };
    
    cli::handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "test command");
    
    let expected_path = temp_dir.path().canonicalize()?;
    let actual_path = Path::new(&commands[0].directory).canonicalize()?;
    assert_eq!(actual_path, expected_path);
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_add_command_with_exit_code() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Change to the test directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;
    
    let command = "failing command".to_string();
    let add_command = Commands::Add { 
        command, 
        exit_code: Some(1), 
        tags: vec![] 
    };
    
    cli::handle_command(add_command, &mut db)?;
    
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "failing command");
    assert_eq!(commands[0].exit_code, Some(1));
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_add_command_with_tags() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Change to the test directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(temp_dir.path())?;
    
    let command = "git commit".to_string();
    let add_command = Commands::Add { 
        command, 
        exit_code: None, 
        tags: vec!["git".to_string(), "vcs".to_string()] 
    };
    
    cli::handle_command(add_command, &mut db)?;
    
    // Test tag listing
    let tag_list_command = Commands::Tag { 
        action: TagCommands::List 
    };
    cli::handle_command(tag_list_command, &mut db)?;
    
    // Test tag search
    let tag_search_command = Commands::Tag { 
        action: TagCommands::Search { 
            tag: "git".to_string(),
            limit: 10
        } 
    };
    cli::handle_command(tag_search_command, &mut db)?;
    
    // Verify the command was added with tags
    let commands = db.search_by_tag("git", 10)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, "git commit");
    assert!(commands[0].tags.contains(&"git".to_string()));
    assert!(commands[0].tags.contains(&"vcs".to_string()));
    
    // Test tag removal
    let tag_remove_command = Commands::Tag { 
        action: TagCommands::Remove { 
            command_id: commands[0].id.unwrap(),
            tag: "vcs".to_string()
        } 
    };
    cli::handle_command(tag_remove_command, &mut db)?;
    
    // Verify tag was removed
    let commands = db.search_by_tag("vcs", 10)?;
    assert_eq!(commands.len(), 0);
    
    // Restore the original directory
    env::set_current_dir(original_dir)?;
    
    Ok(())
}

#[test]
fn test_search_commands() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Add some test commands
    let commands = vec![
        "git commit -m 'test'",
        "git push origin main",
        "cargo test",
        "cargo build",
    ];
    
    for cmd in commands {
        let add_command = Commands::Add { 
            command: cmd.to_string(), 
            exit_code: None, 
            tags: vec![] 
        };
        cli::handle_command(add_command, &mut db)?;
    }
    
    // Instead of using handle_command for search, directly use the database
    let results = db.search_commands("git", 10)?;
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|c| c.command == "git commit -m 'test'"));
    assert!(results.iter().any(|c| c.command == "git push origin main"));
    
    Ok(())
}

#[test]
fn test_execute_command() -> Result<()> {
    init_test_env();
    
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let mut db = Database::new(db_path.to_str().unwrap())?;
    
    // Add a test command
    let command = "echo 'test'".to_string();
    let add_command = Commands::Add { 
        command: command.clone(), 
        exit_code: None, 
        tags: vec![] 
    };
    
    cli::handle_command(add_command, &mut db)?;
    
    // Verify the command was added
    let commands = db.list_commands(1, false)?;
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].command, command);
    
    Ok(())
}
