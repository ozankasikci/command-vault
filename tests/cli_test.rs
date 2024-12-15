use anyhow::Result;
use command_vault::cli::args::{Cli, Commands, TagCommands};
use clap::Parser;

#[test]
fn test_add_command_parsing() -> Result<()> {
    // Test basic command without tags
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "--",
        "git",
        "commit",
        "-m",
        "test",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "git commit -m test");
            assert_eq!(tags, Vec::<String>::new());
        }
        _ => panic!("Expected Add command"),
    }

    // Test with tags
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "--tags",
        "git",
        "--tags",
        "vcs",
        "--",
        "git",
        "commit",
        "-m",
        "test",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "git commit -m test");
            assert_eq!(tags, vec!["git", "vcs"]);
        }
        _ => panic!("Expected Add command"),
    }

    // Test with multiple words in command (no dashes)
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "echo",
        "hello world",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "echo hello world");
            assert_eq!(tags, Vec::<String>::new());
        }
        _ => panic!("Expected Add command"),
    }

    Ok(())
}

#[test]
fn test_search_command_parsing() -> Result<()> {
    let args = Cli::try_parse_from([
        "command-vault",
        "search",
        "git commit",
        "--limit",
        "5",
    ])?;

    match args.command {
        Commands::Search { query, limit } => {
            assert_eq!(query, "git commit");
            assert_eq!(limit, 5);
        }
        _ => panic!("Expected Search command"),
    }
    Ok(())
}

#[test]
fn test_ls_command_parsing() -> Result<()> {
    let args = Cli::try_parse_from([
        "command-vault",
        "ls",
        "--limit",
        "20",
        "--asc",
    ])?;

    match args.command {
        Commands::Ls { limit, asc } => {
            assert_eq!(limit, 20);
            assert!(asc);
        }
        _ => panic!("Expected Ls command"),
    }
    Ok(())
}

#[test]
fn test_tag_commands_parsing() -> Result<()> {
    // Test tag add
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "add",
        "1",
        "important",
        "urgent",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::Add { command_id, tags } } => {
            assert_eq!(command_id, 1);
            assert_eq!(tags, vec!["important", "urgent"]);
        }
        _ => panic!("Expected Tag Add command"),
    }

    // Test tag remove
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "remove",
        "1",
        "urgent",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::Remove { command_id, tag } } => {
            assert_eq!(command_id, 1);
            assert_eq!(tag, "urgent");
        }
        _ => panic!("Expected Tag Remove command"),
    }

    // Test tag list
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "list",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::List } => (),
        _ => panic!("Expected Tag List command"),
    }

    // Test tag search
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "search",
        "git",
        "--limit",
        "5",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::Search { tag, limit } } => {
            assert_eq!(tag, "git");
            assert_eq!(limit, 5);
        }
        _ => panic!("Expected Tag Search command"),
    }
    Ok(())
}

#[test]
fn test_exec_command_parsing() -> Result<()> {
    let args = Cli::try_parse_from([
        "command-vault",
        "exec",
        "42",
    ])?;

    match args.command {
        Commands::Exec { command_id } => {
            assert_eq!(command_id, 42);
        }
        _ => panic!("Expected Exec command"),
    }
    Ok(())
}

#[test]
fn test_invalid_command_id() {
    let result = Cli::try_parse_from([
        "command-vault",
        "exec",
        "not_a_number",
    ]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("invalid value 'not_a_number'"));
}

#[test]
fn test_missing_required_args() {
    let result = Cli::try_parse_from([
        "command-vault",
        "search",
    ]);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("<QUERY>"));
}
