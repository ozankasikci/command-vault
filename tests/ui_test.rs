use anyhow::Result;
use chrono::{TimeZone, Utc};
use command_vault::{
    db::{Command, Database},
    ui::{App, AddCommandApp},
};
use std::collections::HashSet;
use crate::test_utils::create_test_db;

mod test_utils;

fn create_test_commands() -> Vec<Command> {
    vec![
        Command {
            id: Some(1),
            command: "ls -la".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            directory: "/home/user".to_string(),
            tags: vec!["file".to_string(), "list".to_string()],
            parameters: vec![],
        },
        Command {
            id: Some(2),
            command: "git status".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 1).unwrap(),
            directory: "/home/user/project".to_string(),
            tags: vec!["git".to_string()],
            parameters: vec![],
        },
    ]
}

#[test]
fn test_app_new() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let app = App::new(commands.clone(), &mut db);
    
    assert_eq!(app.commands.len(), 2);
    assert_eq!(app.filtered_commands.len(), 2);
    assert_eq!(app.selected, None);
    assert!(!app.show_help);
    assert!(app.message.is_none());
    assert!(app.filter_text.is_empty());
    
    Ok(())
}

#[test]
fn test_app_filtering() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands, &mut db);
    
    // Filter by command text
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");
    
    // Filter by tag
    app.filter_text = "file".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "ls -la");
    
    // Filter with no matches
    app.filter_text = "nonexistent".to_string();
    app.update_filtered_commands();
    assert!(app.filtered_commands.is_empty());
    
    // Clear filter
    app.filter_text.clear();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 2);
    
    Ok(())
}

#[test]
fn test_add_command_app_new() {
    let app = AddCommandApp::new();
    
    // Test initial state
    assert!(app.command.is_empty());
    assert!(app.tags.is_empty());
    assert!(app.current_tag.is_empty());
    assert_eq!(app.command_cursor, 0);
}

#[test]
fn test_add_command_app_command_input() {
    let mut app = AddCommandApp::new();
    
    // Test command input
    app.command = "test command".to_string();
    app.command_cursor = 4;
    assert_eq!(app.command, "test command");
    assert_eq!(app.command_cursor, 4);
}

#[test]
fn test_add_command_app_tag_input() {
    let mut app = AddCommandApp::new();
    
    // Test tag input
    app.tags = vec!["tag1".to_string(), "tag2".to_string()];
    app.current_tag = "tag3".to_string();
    assert_eq!(app.tags.len(), 2);
    assert_eq!(app.current_tag, "tag3");
    
    // Test tag uniqueness
    let tags_set: HashSet<_> = app.tags.iter().collect();
    assert_eq!(app.tags.len(), tags_set.len(), "Tags should be unique");
}

#[test]
fn test_app_message_handling() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands, &mut db);
    
    // Test setting message
    app.message = Some(("Test message".to_string(), ratatui::style::Color::Green));
    assert!(app.message.is_some());
    let (msg, color) = app.message.as_ref().unwrap();
    assert_eq!(msg, "Test message");
    assert_eq!(color, &ratatui::style::Color::Green);
    
    // Test clearing message
    app.message = None;
    assert!(app.message.is_none());
    
    Ok(())
}

#[test]
fn test_app_selection() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands, &mut db);
    
    // Test initial selection
    assert!(app.selected.is_none());
    
    // Test selecting an item
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));
    
    // Test selecting last item
    app.selected = Some(app.commands.len() - 1);
    assert_eq!(app.selected, Some(1));
    
    // Test selection with filtering
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");
    
    Ok(())
}
