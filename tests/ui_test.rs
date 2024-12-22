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
        Command {
            id: Some(3),
            command: "docker ps".to_string(),
            timestamp: Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 2).unwrap(),
            directory: "/home/user".to_string(),
            tags: vec!["docker".to_string()],
            parameters: vec![],
        },
    ]
}

#[test]
fn test_app_new() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = vec![];
    let app = App::new(commands.clone(), &mut db, false);
    assert_eq!(app.commands.len(), 0);
    assert_eq!(app.selected, None);
    assert_eq!(app.show_help, false);
    assert_eq!(app.message, None);
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 0);
    assert_eq!(app.debug_mode, false);
    Ok(())
}

#[test]
fn test_app_filter() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = vec![
        Command {
            id: Some(1),
            command: "ls -l".to_string(),
            timestamp: Utc::now(),
            directory: "/".to_string(),
            tags: vec![],
            parameters: vec![],
        },
        Command {
            id: Some(2),
            command: "pwd".to_string(),
            timestamp: Utc::now(),
            directory: "/".to_string(),
            tags: vec![],
            parameters: vec![],
        },
    ];
    let mut app = App::new(commands.clone(), &mut db, false);
    app.filter_text = "ls".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.filtered_commands[0], 0);
    Ok(())
}

#[test]
fn test_app_filtering() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test filtering by command
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test filtering by tag
    app.filter_text = "docker".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "docker ps");

    // Test filtering by directory
    app.filter_text = "project".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test no matches
    app.filter_text = "nonexistent".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 0);

    // Test empty filter
    app.filter_text = "".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 3);

    Ok(())
}

#[test]
fn test_add_command_app_new() {
    let app = AddCommandApp::new();
    assert!(app.command.is_empty());
    assert!(app.tags.is_empty());
    assert!(app.current_tag.is_empty());
    assert_eq!(app.command_cursor, 0);
}

#[test]
fn test_add_command_app_command_input() {
    let mut app = AddCommandApp::new();
    app.set_command("ls -la".to_string());
    assert_eq!(app.command, "ls -la");
}

#[test]
fn test_add_command_app_tag_input() {
    let mut app = AddCommandApp::new();
    
    // Test single tag
    app.set_tags(vec!["git".to_string()]);
    assert_eq!(app.tags, vec!["git"]);
    
    // Test multiple tags
    app.set_tags(vec!["git".to_string(), "docker".to_string()]);
    assert_eq!(app.tags, vec!["git", "docker"]);
}

#[test]
fn test_app_message_handling() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

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
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.selected, None);

    // Test selecting first item
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test selecting next item
    app.selected = Some(1);
    assert_eq!(app.selected, Some(1));

    // Test selecting previous item
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test selecting last item
    app.selected = Some(2);
    assert_eq!(app.selected, Some(2));

    Ok(())
}
