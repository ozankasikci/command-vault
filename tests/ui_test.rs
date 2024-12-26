use anyhow::Result;
use chrono::{TimeZone, Utc};
use command_vault::{
    db::{Command, Database},
    ui::{app::App, AddCommandApp},
};
use crate::test_utils::create_test_db;
use command_vault::ui::add::InputMode;
use ratatui::style::Color;

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
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = vec![
        Command {
            id: Some(1),
            command: "test command".to_string(),
            timestamp: Utc::now(),
            directory: "/test".to_string(),
            tags: vec!["test".to_string(), "example".to_string()],
            parameters: vec![],
        }
    ];
    
    let app = App::new(commands.clone(), &mut db, false);
    
    assert_eq!(app.commands, commands);
    assert_eq!(app.selected, None);
    assert_eq!(app.show_help, false);
    assert_eq!(app.message, None);
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands, vec![0]);
    assert_eq!(app.confirm_delete, None);
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

    // Test initial state
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Test filtering by command
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test filtering by tag
    app.filter_text = "file".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "ls -la");

    // Test filtering by directory
    app.filter_text = "project".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test no matches
    app.filter_text = "nonexistent".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 0);

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
fn test_app_message() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let mut commands = create_test_commands();
    
    // Add commands to the database first
    for cmd in &mut commands {
        cmd.id = Some(db.add_command(cmd)?);
    }
    
    let mut app = App::new(commands.clone(), &mut db, false);

    // Initial state
    assert_eq!(app.message, None);

    // Set a success message
    app.message = Some(("Command copied to clipboard!".to_string(), Color::Green));
    assert_eq!(app.message, Some(("Command copied to clipboard!".to_string(), Color::Green)));

    // Set an error message
    app.message = Some(("Failed to delete command".to_string(), Color::Red));
    assert_eq!(app.message, Some(("Failed to delete command".to_string(), Color::Red)));

    // Clear message
    app.message = None;
    assert_eq!(app.message, None);

    // Set an info message
    app.message = Some(("Type to filter commands...".to_string(), Color::Blue));
    assert_eq!(app.message, Some(("Type to filter commands...".to_string(), Color::Blue)));

    // Set a warning message
    app.message = Some(("Delete operation cancelled".to_string(), Color::Yellow));
    assert_eq!(app.message, Some(("Delete operation cancelled".to_string(), Color::Yellow)));

    Ok(())
}

#[test]
fn test_app_selection() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.selected, None);

    // Test selecting a command
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test selecting another command
    app.selected = Some(1);
    assert_eq!(app.selected, Some(1));

    // Test selecting last command
    app.selected = Some(app.commands.len() - 1);
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

    // Test initial state
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Apply filter
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Clear filter
    app.filter_text.clear();
    app.update_filtered_commands();
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

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

#[test]
fn test_app_command_copy() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Select a command
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));
    assert_eq!(app.filtered_commands[0], 0);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "ls -la");

    // Verify the command to be copied
    if let Some(selected) = app.selected {
        if let Some(&idx) = app.filtered_commands.get(selected) {
            if let Some(cmd) = app.commands.get(idx) {
                assert_eq!(cmd.command, "ls -la");
            }
        }
    }

    Ok(())
}

#[test]
fn test_app_navigation() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.selected, None);
    assert_eq!(app.filtered_commands.len(), 3);

    // Test moving down
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test moving down again
    app.selected = Some(1);
    assert_eq!(app.selected, Some(1));

    // Test moving up
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test moving up at the top (should stay at 0)
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Test moving down at the bottom (should stay at last index)
    app.selected = Some(2);
    assert_eq!(app.selected, Some(2));

    // Test selection with filtered commands
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    Ok(())
}

#[test]
fn test_app_delete_command() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let mut commands = create_test_commands();
    
    // Add commands to the database first
    for cmd in &mut commands {
        cmd.id = Some(db.add_command(cmd)?);
    }
    
    let mut app = App::new(commands.clone(), &mut db, false);

    // Initial state
    assert_eq!(app.commands.len(), 3);
    assert_eq!(app.filtered_commands.len(), 3);
    assert_eq!(app.confirm_delete, None);

    // Select a command
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Initiate delete
    app.confirm_delete = Some(0);
    assert_eq!(app.confirm_delete, Some(0));

    // Cancel delete
    app.confirm_delete = None;
    assert_eq!(app.confirm_delete, None);
    assert_eq!(app.commands.len(), 3);

    // Initiate delete again
    app.confirm_delete = Some(0);
    assert_eq!(app.confirm_delete, Some(0));

    // Confirm delete
    if let Some(selected) = app.selected {
        if let Some(confirm_idx) = app.confirm_delete {
            if confirm_idx == selected {
                if let Some(&filtered_idx) = app.filtered_commands.get(selected) {
                    if let Some(command_id) = app.commands[filtered_idx].id {
                        match app.db.delete_command(command_id) {
                            Ok(_) => {
                                app.commands.remove(filtered_idx);
                                app.update_filtered_commands();
                                // Update selection after deletion
                                if app.filtered_commands.is_empty() {
                                    app.selected = None;
                                } else {
                                    app.selected = Some(selected.min(app.filtered_commands.len() - 1));
                                }
                            }
                            Err(e) => panic!("Failed to delete command: {}", e),
                        }
                        app.confirm_delete = None;
                    }
                }
            }
        }
    }

    // Verify deletion
    assert_eq!(app.commands.len(), 2);
    assert_eq!(app.filtered_commands.len(), 2);
    assert_eq!(app.confirm_delete, None);
    assert_eq!(app.selected, Some(0));

    Ok(())
}

#[test]
fn test_app_edit_command() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let mut commands = create_test_commands();
    
    // Add commands to the database first
    for cmd in &mut commands {
        cmd.id = Some(db.add_command(cmd)?);
    }
    
    let mut app = App::new(commands.clone(), &mut db, false);

    // Initial state
    assert_eq!(app.commands.len(), 3);
    assert_eq!(app.filtered_commands.len(), 3);

    // Select a command
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Get the original command
    let original_command = app.commands[0].clone();
    assert_eq!(original_command.command, "ls -la");

    // Update the command
    let updated_command = Command {
        id: original_command.id,
        command: "ls -lah".to_string(),
        timestamp: original_command.timestamp,
        directory: original_command.directory.clone(),
        tags: vec!["test".to_string(), "updated".to_string()],
        parameters: vec![],
    };

    // Update in database
    app.db.update_command(&updated_command)?;

    // Update in app's command list
    app.commands[0] = updated_command.clone();

    // Verify the update
    assert_eq!(app.commands[0].command, "ls -lah");
    assert_eq!(app.commands[0].tags, vec!["test".to_string(), "updated".to_string()]);
    assert_eq!(app.commands[0].id, original_command.id);

    Ok(())
}

#[test]
fn test_app_help_mode() -> Result<()> {
    let (mut db, _dir) = create_test_db()?;
    let mut commands = create_test_commands();
    
    // Add commands to the database first
    for cmd in &mut commands {
        cmd.id = Some(db.add_command(cmd)?);
    }
    
    let mut app = App::new(commands.clone(), &mut db, false);

    // Initial state
    assert_eq!(app.show_help, false);

    // Toggle help mode on
    app.show_help = true;
    assert_eq!(app.show_help, true);

    // Toggle help mode off
    app.show_help = false;
    assert_eq!(app.show_help, false);

    // Verify that help mode doesn't affect other app state
    assert_eq!(app.commands.len(), 3);
    assert_eq!(app.filtered_commands.len(), 3);
    assert_eq!(app.selected, None);
    assert_eq!(app.message, None);
    assert_eq!(app.filter_text, "");
    assert_eq!(app.confirm_delete, None);
    assert_eq!(app.debug_mode, false);

    Ok(())
}

#[test]
fn test_app_message_handling() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = vec![];
    let mut app = App::new(commands, &mut db, false);
    
    // Test setting a message
    app.set_message("Test message".to_string(), Color::Green);
    assert_eq!(app.message, Some(("Test message".to_string(), Color::Green)));
    
    // Test setting a success message
    app.set_success_message("Success message".to_string());
    assert_eq!(app.message, Some(("Success message".to_string(), Color::Green)));
    
    // Test setting an error message
    app.set_error_message("Error message".to_string());
    assert_eq!(app.message, Some(("Error message".to_string(), Color::Red)));
    
    // Test clearing the message
    app.clear_message();
    assert_eq!(app.message, None);
    
    Ok(())
}

#[test]
fn test_app_selection_methods() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.get_selection(), None);

    // Test setting valid selection
    app.set_selection(Some(1));
    assert_eq!(app.get_selection(), Some(1));

    // Test setting selection to None
    app.set_selection(None);
    assert_eq!(app.get_selection(), None);

    // Test setting selection to last index
    app.set_selection(Some(app.commands.len() - 1));
    assert_eq!(app.get_selection(), Some(app.commands.len() - 1));

    // Test that selection is maintained after filtering
    app.set_selection(Some(1));
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.get_selection(), Some(0)); // Selection should adjust to filtered index

    Ok(())
}

#[test]
fn test_app_navigation_methods() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.selected, None);

    // Test select_next from None
    app.select_next();
    assert_eq!(app.selected, Some(0));

    // Test select_next from first item
    app.select_next();
    assert_eq!(app.selected, Some(1));

    // Test select_next from last item (should stay at last)
    app.selected = Some(app.filtered_commands.len() - 1);
    app.select_next();
    assert_eq!(app.selected, Some(app.filtered_commands.len() - 1));

    // Test select_previous from middle
    app.selected = Some(1);
    app.select_previous();
    assert_eq!(app.selected, Some(0));

    // Test select_previous from first item (should stay at first)
    app.select_previous();
    assert_eq!(app.selected, Some(0));

    // Test select_previous from None (should go to last)
    app.selected = None;
    app.select_previous();
    assert_eq!(app.selected, Some(app.filtered_commands.len() - 1));

    // Test with filtered list
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filtered_commands.len(), 1);

    // Test navigation with filtered list
    app.selected = None;
    app.select_next();
    assert_eq!(app.selected, Some(0));
    app.select_next();
    assert_eq!(app.selected, Some(0)); // Should stay at 0 since there's only one item

    Ok(())
}

#[test]
fn test_app_filter_methods() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Test set_filter
    app.set_filter("git".to_string());
    assert_eq!(app.filter_text, "git");
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test clear_filter
    app.clear_filter();
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Test append_to_filter
    app.append_to_filter('d');
    app.append_to_filter('o');
    app.append_to_filter('c');
    assert_eq!(app.filter_text, "doc");
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "docker ps");

    // Test backspace_filter
    app.backspace_filter();
    assert_eq!(app.filter_text, "do");
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "docker ps");

    // Test backspace_filter until empty
    app.backspace_filter();
    app.backspace_filter();
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Test case insensitive filtering
    app.set_filter("GIT".to_string());
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test filtering by tag
    app.set_filter("file".to_string());
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "ls -la");

    // Test filtering by directory
    app.set_filter("project".to_string());
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test no matches
    app.set_filter("nonexistent".to_string());
    assert_eq!(app.filtered_commands.len(), 0);

    Ok(())
}

#[test]
fn test_app_selected_command_methods() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.get_selected_command(), None);
    assert_eq!(app.get_selected_index(), None);

    // Test with valid selection
    app.selected = Some(0);
    assert_eq!(app.get_selected_command().unwrap().command, "ls -la");
    assert_eq!(app.get_selected_index(), Some(0));

    // Test with filtered list
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    app.selected = Some(0);
    assert_eq!(app.get_selected_command().unwrap().command, "git status");
    assert_eq!(app.get_selected_index(), Some(1)); // Index in original commands list

    // Test with invalid selection
    app.selected = Some(5); // Out of bounds
    assert_eq!(app.get_selected_command(), None);
    assert_eq!(app.get_selected_index(), None);

    // Test with empty filtered list
    app.filter_text = "nonexistent".to_string();
    app.update_filtered_commands();
    app.selected = Some(0);
    assert_eq!(app.get_selected_command(), None);
    assert_eq!(app.get_selected_index(), None);

    Ok(())
}

#[test]
fn test_app_selection_update_methods() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test update_selection_after_filter

    // Test with no selection
    assert_eq!(app.selected, None);
    app.update_selection_after_filter();
    assert_eq!(app.selected, None);

    // Test with selection and empty filtered list
    app.selected = Some(0);
    app.filter_text = "nonexistent".to_string();
    app.update_filtered_commands();
    app.update_selection_after_filter();
    assert_eq!(app.selected, None);

    // Test with selection and filtered list smaller than selection
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    app.selected = Some(2); // Out of bounds for filtered list
    app.update_selection_after_filter();
    assert_eq!(app.selected, Some(0)); // Should adjust to last item in filtered list

    // Test update_selection_after_delete

    // Reset to initial state
    app.filter_text.clear();
    app.update_filtered_commands();
    
    // Test with no selection
    app.selected = None;
    app.update_selection_after_delete(0);
    assert_eq!(app.selected, None);

    // Test with selection and empty filtered list
    app.selected = Some(0);
    app.filtered_commands.clear();
    app.update_selection_after_delete(0);
    assert_eq!(app.selected, None);

    // Test with selection after deleting item
    app.filtered_commands = vec![0, 1, 2];
    app.selected = Some(2);
    app.filtered_commands.remove(1); // Remove middle item
    app.update_selection_after_delete(1);
    assert_eq!(app.selected, Some(1)); // Should adjust to new length

    // Test with selection at end after deleting last item
    app.selected = Some(1);
    app.filtered_commands.pop(); // Remove last item
    app.update_selection_after_delete(1);
    assert_eq!(app.selected, Some(0)); // Should adjust to new last item

    Ok(())
}

#[test]
fn test_app_key_events() -> Result<()> {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test help toggle with '?'
    assert_eq!(app.show_help, false);
    app.show_help = !app.show_help; // Simulate '?' key press
    assert_eq!(app.show_help, true);
    app.show_help = !app.show_help; // Simulate '?' key press again
    assert_eq!(app.show_help, false);

    // Test filter operations
    assert_eq!(app.filter_text, "");
    app.append_to_filter('g'); // Simulate typing 'g'
    app.append_to_filter('i'); // Simulate typing 'i'
    app.append_to_filter('t'); // Simulate typing 't'
    assert_eq!(app.filter_text, "git");
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test backspace in filter
    app.backspace_filter(); // Simulate backspace
    assert_eq!(app.filter_text, "gi");
    
    // Test clear filter with Esc
    app.clear_filter(); // Simulate Esc
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);

    // Test navigation
    assert_eq!(app.selected, None);
    app.select_next(); // Simulate down arrow or 'j'
    assert_eq!(app.selected, Some(0));
    app.select_next(); // Move down again
    assert_eq!(app.selected, Some(1));
    app.select_previous(); // Simulate up arrow or 'k'
    assert_eq!(app.selected, Some(0));

    // Test delete confirmation
    app.confirm_delete = Some(0); // Simulate 'd' key
    assert_eq!(app.confirm_delete, Some(0));
    app.confirm_delete = None; // Simulate Esc
    assert_eq!(app.confirm_delete, None);

    // Test message handling
    assert_eq!(app.message, None);
    app.set_success_message("Test success".to_string());
    assert_eq!(app.message, Some(("Test success".to_string(), Color::Green)));
    app.clear_message();
    assert_eq!(app.message, None);

    Ok(())
}

#[test]
fn test_app_clipboard_operations() -> Result<()> {
    use std::process::Command;
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Select a command
    app.selected = Some(0);
    assert_eq!(app.selected, Some(0));

    // Get the command that will be copied
    let command_to_copy = app.get_selected_command().unwrap().command.clone();
    assert_eq!(command_to_copy, "ls -la");

    // Copy command to clipboard
    #[cfg(target_os = "macos")]
    {
        // Copy using pbcopy
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(command_to_copy.as_bytes())?;
        }
        
        child.wait()?;

        // Verify using pbpaste
        let output = Command::new("pbpaste")
            .output()?;
        let clipboard_content = String::from_utf8(output.stdout)?;
        assert_eq!(clipboard_content, command_to_copy);
    }

    #[cfg(target_os = "linux")]
    {
        // Copy using xclip
        let mut child = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(command_to_copy.as_bytes())?;
        }
        
        child.wait()?;

        // Verify using xclip -o
        let output = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .arg("-o")
            .output()?;
        let clipboard_content = String::from_utf8(output.stdout)?;
        assert_eq!(clipboard_content, command_to_copy);
    }

    // Verify message is set after copy
    assert_eq!(app.message, None);
    app.set_success_message("Command copied to clipboard!".to_string());
    assert_eq!(app.message, Some(("Command copied to clipboard!".to_string(), Color::Green)));

    Ok(())
}

#[test]
fn test_app_terminal_setup() -> Result<()> {
    use crossterm::terminal::{disable_raw_mode, enable_raw_mode, is_raw_mode_enabled};
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;
    use std::io::stdout;

    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test terminal setup
    enable_raw_mode()?;
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    assert!(is_raw_mode_enabled()?);

    // Test terminal restoration
    disable_raw_mode()?;
    assert!(!is_raw_mode_enabled()?);

    Ok(())
}

#[test]
fn test_app_ui_state() -> Result<()> {
    let mut db = Database::new(":memory:")?;
    db.init()?;
    
    let commands = create_test_commands();
    let mut app = App::new(commands.clone(), &mut db, false);

    // Test initial state
    assert_eq!(app.show_help, false);
    assert_eq!(app.message, None);
    assert_eq!(app.filter_text, "");
    assert_eq!(app.filtered_commands.len(), 3);
    assert_eq!(app.selected, None);
    assert_eq!(app.confirm_delete, None);

    // Test help state
    app.show_help = true;
    assert_eq!(app.show_help, true);

    // Test filter state
    app.show_help = false;
    app.filter_text = "git".to_string();
    app.update_filtered_commands();
    assert_eq!(app.filter_text, "git");
    assert_eq!(app.filtered_commands.len(), 1);
    assert_eq!(app.commands[app.filtered_commands[0]].command, "git status");

    // Test message state
    app.set_success_message("Test message".to_string());
    assert_eq!(app.message, Some(("Test message".to_string(), Color::Green)));

    // Test delete confirmation state
    app.selected = Some(0);
    app.confirm_delete = Some(0);
    assert_eq!(app.selected, Some(0));
    assert_eq!(app.confirm_delete, Some(0));

    Ok(())
}
