use anyhow::Result;
use chrono::Utc;
use command_vault::db::{
    models::{Command, Parameter},
    Database,
};
use std::fs;
use tempfile::tempdir;

fn create_test_command(command: &str, tags: Vec<String>, parameters: Vec<Parameter>) -> Command {
    Command {
        id: None,
        command: command.to_string(),
        timestamp: Utc::now(),
        directory: "/test/dir".to_string(),
        tags,
        parameters,
    }
}

#[test]
fn test_command_crud() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;

    // Test adding a command
    let cmd = create_test_command(
        "echo test",
        vec!["test".to_string()],
        vec![],
    );
    let id = db.add_command(&cmd)?;
    assert!(id > 0);

    // Test retrieving the command
    let retrieved = db.get_command(id)?.unwrap();
    assert_eq!(retrieved.command, "echo test");
    assert_eq!(retrieved.tags, vec!["test"]);
    assert!(retrieved.parameters.is_empty());

    // Test updating the command
    let mut updated_cmd = retrieved.clone();
    updated_cmd.command = "echo updated".to_string();
    db.update_command(&updated_cmd)?;

    let retrieved_updated = db.get_command(id)?.unwrap();
    assert_eq!(retrieved_updated.command, "echo updated");

    // Test deleting the command
    db.delete_command(id)?;
    assert!(db.get_command(id)?.is_none());

    Ok(())
}

#[test]
fn test_tag_operations() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;

    // Add command with initial tags
    let cmd = create_test_command(
        "git status",
        vec!["git".to_string()],
        vec![],
    );
    let id = db.add_command(&cmd)?;

    // Add more tags
    db.add_tags_to_command(id, &vec!["vcs".to_string(), "status".to_string()])?;
    let cmd = db.get_command(id)?.unwrap();
    assert!(cmd.tags.contains(&"git".to_string()));
    assert!(cmd.tags.contains(&"vcs".to_string()));
    assert!(cmd.tags.contains(&"status".to_string()));

    // Remove a tag
    db.remove_tag_from_command(id, "status")?;
    let cmd = db.get_command(id)?.unwrap();
    assert!(!cmd.tags.contains(&"status".to_string()));

    // Test tag search
    let results = db.search_by_tag("git", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "git status");

    // Test listing tags
    let tags = db.list_tags()?;
    assert!(tags.iter().any(|(name, _)| name == "git"));
    assert!(tags.iter().any(|(name, _)| name == "vcs"));

    Ok(())
}

#[test]
fn test_command_with_parameters() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;

    // Create command with parameters
    let params = vec![
        Parameter::new("branch".to_string()),
        Parameter::with_description(
            "message".to_string(),
            Some("Commit message".to_string()),
        ),
    ];
    let cmd = create_test_command(
        "git commit -m @message && git push origin @branch",
        vec!["git".to_string()],
        params.clone(),
    );

    // Add and retrieve
    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.unwrap();

    assert_eq!(retrieved.parameters.len(), 2);
    assert_eq!(retrieved.parameters[0].name, "branch");
    assert_eq!(retrieved.parameters[1].name, "message");
    assert_eq!(retrieved.parameters[1].description, Some("Commit message".to_string()));

    Ok(())
}

#[test]
fn test_command_search() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;

    // Add multiple commands
    let commands = vec![
        ("git status", vec!["git".to_string()]),
        ("git push", vec!["git".to_string()]),
        ("ls -la", vec!["system".to_string()]),
        ("echo test", vec!["test".to_string()]),
    ];

    for (cmd, tags) in commands {
        let command = create_test_command(cmd, tags, vec![]);
        db.add_command(&command)?;
    }

    // Test exact match
    let results = db.search_commands("git status", 10)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "git status");

    // Test partial match
    let results = db.search_commands("git", 10)?;
    assert_eq!(results.len(), 2);

    // Test with limit
    let results = db.search_commands("git", 1)?;
    assert_eq!(results.len(), 1);

    // Test case sensitivity
    let results = db.search_commands("GIT", 10)?;
    assert!(!results.is_empty());

    // Test tag search
    let results = db.search_by_tag("git", 10)?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn test_edge_cases() -> Result<()> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let mut db = Database::new(db_path.to_str().unwrap())?;

    // Test empty command
    let cmd = create_test_command("", vec![], vec![]);
    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.unwrap();
    assert_eq!(retrieved.command, "");

    // Test very long command
    let long_cmd = "x".repeat(1000);
    let cmd = create_test_command(&long_cmd, vec![], vec![]);
    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.unwrap();
    assert_eq!(retrieved.command, long_cmd);

    // Test special characters in command
    let special_cmd = "echo 'test' && ls -la | grep \"something\" > output.txt";
    let cmd = create_test_command(special_cmd, vec![], vec![]);
    let id = db.add_command(&cmd)?;
    let retrieved = db.get_command(id)?.unwrap();
    assert_eq!(retrieved.command, special_cmd);

    // Test non-existent command
    assert!(db.get_command(9999)?.is_none());

    // Test deleting non-existent command
    assert!(db.delete_command(9999).is_err());

    // Test adding tags to non-existent command
    assert!(db.add_tags_to_command(9999, &vec!["test".to_string()]).is_err());

    // Test removing non-existent tag
    let cmd = create_test_command("test", vec!["tag1".to_string()], vec![]);
    let id = db.add_command(&cmd)?;
    assert!(db.remove_tag_from_command(id, "nonexistent").is_ok());

    Ok(())
}
