use anyhow::Result;
use chrono::{TimeZone, Utc};
use command_vault::{
    db::{Command, Database},
    ui::{app::App, AddCommandApp},
};
use crate::test_utils::create_test_db;
use command_vault::ui::add::InputMode;

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

#[test]
fn test_app_confirm_delete() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test setting confirm delete
    app.selected = Some(0);
    app.confirm_delete = Some(0);
    assert_eq!(app.confirm_delete, Some(0));

    // Test canceling delete
    app.confirm_delete = None;
    assert_eq!(app.confirm_delete, None);

    Ok(())
}

#[test]
fn test_app_debug_mode() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    
    // Test debug mode enabled
    let mut app = App::new(commands.clone(), &mut db, true);
    assert_eq!(app.debug_mode, true);
    
    // Test debug mode disabled
    let mut app = App::new(commands.clone(), &mut db, false);
    assert_eq!(app.debug_mode, false);

    Ok(())
}

#[test]
fn test_app_help_toggle() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.show_help, false);

    // Test toggling help on
    app.show_help = true;
    assert_eq!(app.show_help, true);

    // Test toggling help off
    app.show_help = false;
    assert_eq!(app.show_help, false);

    Ok(())
}

#[test]
fn test_add_command_app_cursor_movement() {
    let mut app = AddCommandApp::new();
    
    // Test initial cursor position
    assert_eq!(app.command_cursor, 0);
    
    // Test setting cursor position
    app.set_command("ls -la".to_string());
    app.command_cursor = 3;
    assert_eq!(app.command_cursor, 3);
}

#[test]
fn test_app_filter_clear() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Set filter text
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);

    // Clear filter text
    app.filter_text.clear();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), commands.len());

    Ok(())
}

#[test]
fn test_add_command_app_tag_operations() {
    let mut app = AddCommandApp::new();
    
    // Test adding a tag
    app.current_tag = "git".to_string();
    app.tags.push(app.current_tag.clone());
    assert_eq!(app.tags, vec!["git"]);
    
    // Test clearing current tag
    app.current_tag.clear();
    assert!(app.current_tag.is_empty());
    
    // Test adding multiple tags
    app.tags.push("docker".to_string());
    app.tags.push("test".to_string());
    assert_eq!(app.tags, vec!["git", "docker", "test"]);
}

#[test]
fn test_add_command_app_input_modes() {
    use command_vault::ui::add::InputMode;
    let mut app = AddCommandApp::new();
    
    // Test initial mode
    assert!(matches!(app.input_mode, InputMode::Command));
    
    // Test switching to Tag mode
    app.input_mode = InputMode::Tag;
    assert!(matches!(app.input_mode, InputMode::Tag));
    
    // Test switching to Confirm mode
    app.input_mode = InputMode::Confirm;
    assert!(matches!(app.input_mode, InputMode::Confirm));
    
    // Test switching to Help mode
    app.input_mode = InputMode::Help;
    assert!(matches!(app.input_mode, InputMode::Help));
    
    // Test storing previous mode
    app.previous_mode = InputMode::Command;
    assert!(matches!(app.previous_mode, InputMode::Command));
}

#[test]
fn test_add_command_app_tag_suggestions() {
    let mut app = AddCommandApp::new();
    
    // Test initial state
    assert!(app.suggested_tags.is_empty());
    
    // Test adding suggested tags
    app.suggested_tags = vec!["git".to_string(), "docker".to_string()];
    assert_eq!(app.suggested_tags, vec!["git", "docker"]);
    
    // Test clearing suggestions
    app.suggested_tags.clear();
    assert!(app.suggested_tags.is_empty());
}

#[test]
fn test_add_command_app_multiline() {
    let mut app = AddCommandApp::new();
    
    // Test initial state
    assert_eq!(app.command_line, 0);
    
    // Test setting command with multiple lines
    app.set_command("line1\nline2\nline3".to_string());
    assert_eq!(app.command, "line1\nline2\nline3");
    
    // Test cursor movement across lines
    app.command_cursor = 6;  // After "line1\n"
    assert_eq!(app.command_cursor, 6);
}

#[test]
fn test_add_command_app_key_events() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    let mut app = AddCommandApp::new();

    // Test command input
    app.handle_key_event(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
    assert_eq!(app.command, "ls");
    assert_eq!(app.command_cursor, 2);

    // Test backspace
    app.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
    assert_eq!(app.command, "l");
    assert_eq!(app.command_cursor, 1);

    // Test cursor movement
    app.handle_key_event(KeyEvent::new(KeyCode::Left, KeyModifiers::empty()));
    assert_eq!(app.command_cursor, 0);
    app.handle_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::empty()));
    assert_eq!(app.command_cursor, 1);

    // Test multiline command
    app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
    assert_eq!(app.command, "l\na");
    assert_eq!(app.command_line, 1);

    // Test switching to tag mode
    app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(app.input_mode, InputMode::Tag);

    // Test tag input
    app.handle_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::empty()));
    assert_eq!(app.current_tag, "tag");

    // Test adding tag
    app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
    assert_eq!(app.tags, vec!["tag"]);
    assert_eq!(app.current_tag, "");

    // Test help mode
    app.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()));
    assert_eq!(app.input_mode, InputMode::Help);
    assert_eq!(app.previous_mode, InputMode::Tag);

    // Test exiting help mode
    app.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()));
    assert_eq!(app.input_mode, InputMode::Tag);
}

#[test]
fn test_add_command_app_help_mode() {
    use command_vault::ui::add::InputMode;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = AddCommandApp::new();
    
    // Test entering help mode
    app.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()));
    assert!(matches!(app.input_mode, InputMode::Help));
    assert!(matches!(app.previous_mode, InputMode::Command));
    
    // Test exiting help mode with ?
    app.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()));
    assert!(matches!(app.input_mode, InputMode::Command));
    
    // Test exiting help mode with Esc
    app.handle_key_event(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty()));
    assert!(matches!(app.input_mode, InputMode::Help));
    app.handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()));
    assert!(matches!(app.input_mode, InputMode::Command));
}

#[test]
fn test_add_command_app_multiline_command() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut app = AddCommandApp::new();
    
    // Test entering multiline command
    app.handle_key_event(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty()));
    app.handle_key_event(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty()));
    
    assert_eq!(app.command, "ls\npwd");
    assert_eq!(app.command_line, 1);
    assert_eq!(app.command_cursor, 6);
}
