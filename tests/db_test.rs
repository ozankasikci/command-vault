use anyhow::Result;
use chrono::Utc;
use command_vault::db::{
    models::{Command, Parameter},
    Database,
};

mod test_utils;
use test_utils::create_test_db;

#[test]
fn test_command_crud() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;

    // Test Create
    let cmd = Command {
        id: None,
        command: "ls -la".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["file".to_string()],
        parameters: vec![],
    };
    let id = db.add_command(&cmd)?;
    assert!(id > 0);

    // Test Read
    let retrieved = db.get_command(id)?.expect("Command should exist");
    assert_eq!(retrieved.command, "ls -la");
    assert_eq!(retrieved.directory, "/test");
    let mut tags = retrieved.tags.clone();
    tags.sort();
    assert_eq!(tags, vec!["file".to_string()]);

    // Test Update
    let mut updated = retrieved.clone();
    updated.command = "ls -lah".to_string();
    updated.tags = vec!["file".to_string(), "list".to_string()];  // Update tags in the command
    db.update_command(&updated)?;

    let retrieved = db.get_command(id)?.expect("Command should exist");
    assert_eq!(retrieved.command, "ls -lah");
    let mut tags = retrieved.tags;
    tags.sort();
    assert_eq!(tags, vec!["file".to_string(), "list".to_string()]);

    // Test Delete
    db.delete_command(id)?;
    assert!(db.get_command(id)?.is_none());

    Ok(())
}

#[test]
fn test_command_with_parameters() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;

    let params = vec![
        Parameter::with_description(
            "file".to_string(),
            Some("Target file".to_string())
        ),
        Parameter::with_description(
            "mode".to_string(),
            None
        ),
    ];

    let cmd = Command {
        id: None,
        command: "touch @file --mode=@mode".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: params.clone(),
    };

    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.expect("Command should exist");
    
    assert_eq!(retrieved.parameters.len(), 2);
    assert_eq!(retrieved.parameters[0].name, "file");
    assert_eq!(retrieved.parameters[0].description, Some("Target file".to_string()));
    assert_eq!(retrieved.parameters[1].name, "mode");
    assert_eq!(retrieved.parameters[1].description, None);

    Ok(())
}

#[test]
fn test_tag_operations() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;

    // Add commands with tags
    let cmd1 = Command {
        id: None,
        command: "git commit".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["git".to_string(), "vcs".to_string()],
        parameters: vec![],
    };
    let cmd2 = Command {
        id: None,
        command: "git push".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec!["git".to_string(), "sync".to_string()],
        parameters: vec![],
    };

    let id1 = db.add_command(&cmd1)?;
    let _id2 = db.add_command(&cmd2)?;

    // Test tag listing
    let tags = db.list_tags()?;
    assert_eq!(tags.len(), 3); // git, vcs, sync
    let git_tag = tags.iter().find(|(name, _)| name == "git").expect("git tag should exist");
    assert_eq!(git_tag.1, 2); // git tag should be used twice

    // Test tag search
    let git_commands = db.search_by_tag("git", 10)?;
    assert_eq!(git_commands.len(), 2);
    assert!(git_commands.iter().any(|c| c.command == "git commit"));
    assert!(git_commands.iter().any(|c| c.command == "git push"));

    // Test tag removal
    let mut cmd = db.get_command(id1)?.expect("Command should exist");
    cmd.tags = vec!["git".to_string()];  // Update tags in the command
    db.update_command(&cmd)?;
    db.remove_tag_from_command(id1, "vcs")?;

    let cmd = db.get_command(id1)?.expect("Command should exist");
    assert_eq!(cmd.tags, vec!["git".to_string()]);

    Ok(())
}

#[test]
fn test_command_search() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;

    // Add test commands
    let commands = vec![
        ("git commit -m 'test'", vec!["git"]),
        ("git push origin main", vec!["git"]),
        ("ls -la", vec!["file"]),
        ("docker ps", vec!["container"]),
    ];

    for (cmd, tags) in commands {
        db.add_command(&Command {
            id: None,
            command: cmd.to_string(),
            timestamp: Utc::now(),
            directory: "/test".to_string(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            parameters: vec![],
        })?;
    }

    // Test full text search
    let results = db.search_commands("commit", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "git commit -m 'test'");

    // Test listing with limit
    let results = db.list_commands(2, true)?;
    assert_eq!(results.len(), 2);

    // Test listing with ascending order
    let results = db.list_commands(10, true)?;
    assert_eq!(results.len(), 4);
    assert!(results[0].command.contains("git commit"));
    
    // Test listing with descending order
    let results = db.list_commands(10, false)?;
    assert_eq!(results.len(), 4);
    assert!(results[0].command.contains("docker ps"));

    Ok(())
}

#[test]
fn test_edge_cases() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;

    // Test empty command
    let cmd = Command {
        id: None,
        command: "".to_string(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![],
    };
    let _id = db.add_command(&cmd)?;

    // Test very long command
    let long_cmd = "x".repeat(10000);
    let cmd = Command {
        id: None,
        command: long_cmd.clone(),
        timestamp: Utc::now(),
        directory: "/test".to_string(),
        tags: vec![],
        parameters: vec![],
    };
    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.expect("Command should exist");
    assert_eq!(retrieved.command, long_cmd);

    // Test non-existent command
    assert!(db.get_command(99999)?.is_none());

    // Test removing non-existent tag
    let result = db.remove_tag_from_command(id, "nonexistent");
    assert!(result.is_ok());

    // Test searching with empty query
    let results = db.search_commands("", 10)?;
    assert_eq!(results.len(), 2); // Should find both the empty and long commands

    // Test searching with very long query
    let results = db.search_commands(&"x".repeat(1000), 10)?;
    assert_eq!(results.len(), 1); // Should find our very long command

    Ok(())
}
