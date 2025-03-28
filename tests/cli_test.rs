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
        "--",
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

    // Test add command without any command (missing --)
    let result = Cli::try_parse_from([
        "command-vault",
        "add",
    ]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("the following required arguments were not provided"));

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
fn test_ls_command_default_behavior() -> Result<()> {
    // Test ls with default values
    let args = Cli::try_parse_from([
        "command-vault",
        "ls",
    ])?;

    match args.command {
        Commands::Ls { limit, asc } => {
            assert_eq!(limit, 50); // Default limit is 50
            assert!(!asc); // Default is descending order
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
        Commands::Exec { command_id, debug } => {
            assert_eq!(command_id, 42);
            assert_eq!(debug, false);
        }
        _ => panic!("Expected Exec command"),
    }
    Ok(())
}

#[test]
fn test_parse_exec_command() {
    let args = vec!["command-vault", "exec", "123"];
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.command {
        Commands::Exec { command_id, debug } => {
            assert_eq!(command_id, 123);
            assert_eq!(debug, false);
        }
        _ => panic!("Expected Exec command"),
    }

    // Test with debug flag
    let args = vec!["command-vault", "exec", "123", "--debug"];
    let cli = Cli::try_parse_from(args).unwrap();
    match cli.command {
        Commands::Exec { command_id, debug } => {
            assert_eq!(command_id, 123);
            assert_eq!(debug, true);
        }
        _ => panic!("Expected Exec command"),
    }
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

#[test]
fn test_search_command_default_limit() -> Result<()> {
    // Test search with default limit
    let args = Cli::try_parse_from([
        "command-vault",
        "search",
        "git commit",
    ])?;

    match args.command {
        Commands::Search { query, limit } => {
            assert_eq!(query, "git commit");
            assert_eq!(limit, 10); // Default limit is 10
        }
        _ => panic!("Expected Search command"),
    }
    Ok(())
}

#[test]
fn test_delete_command_parsing() -> Result<()> {
    // Test basic delete
    let args = Cli::try_parse_from([
        "command-vault",
        "delete",
        "42",
    ])?;

    match args.command {
        Commands::Delete { command_id } => {
            assert_eq!(command_id, 42);
        }
        _ => panic!("Expected Delete command"),
    }

    // Test missing command ID
    let result = Cli::try_parse_from([
        "command-vault",
        "delete",
    ]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("required"));

    Ok(())
}

#[test]
fn test_shell_init_command_parsing() -> Result<()> {
    // Test default shell initialization
    let args = Cli::try_parse_from([
        "command-vault",
        "shell-init",
    ])?;

    match args.command {
        Commands::ShellInit { shell } => {
            assert!(shell.is_none());
        }
        _ => panic!("Expected ShellInit command"),
    }

    // Test explicit shell override
    let args = Cli::try_parse_from([
        "command-vault",
        "shell-init",
        "--shell",
        "fish",
    ])?;

    match args.command {
        Commands::ShellInit { shell } => {
            assert_eq!(shell, Some("fish".to_string()));
        }
        _ => panic!("Expected ShellInit command"),
    }

    Ok(())
}

#[test]
fn test_tag_commands_all() -> Result<()> {
    // Test tag remove
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "remove",
        "1",
        "git",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::Remove { command_id, tag } } => {
            assert_eq!(command_id, 1);
            assert_eq!(tag, "git");
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

    // Test tag search with default limit
    let args = Cli::try_parse_from([
        "command-vault",
        "tag",
        "search",
        "git",
    ])?;

    match args.command {
        Commands::Tag { action: TagCommands::Search { tag, limit } } => {
            assert_eq!(tag, "git");
            assert_eq!(limit, 10); // Default limit is 10
        }
        _ => panic!("Expected Tag Search command"),
    }

    Ok(())
}

#[test]
fn test_add_command_with_parameters() -> Result<()> {
    // Test add command with basic parameter
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "--",
        "touch",
        "@filename",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "touch @filename");
            assert_eq!(tags, Vec::<String>::new());
        }
        _ => panic!("Expected Add command"),
    }

    // Test add command with parameter description
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "--",
        "touch",
        "@filename:Name of file to create",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "touch @filename:Name of file to create");
            assert_eq!(tags, Vec::<String>::new());
        }
        _ => panic!("Expected Add command"),
    }

    // Test add command with parameter default value
    let args = Cli::try_parse_from([
        "command-vault",
        "add",
        "--",
        "touch",
        "@filename:Name of file to create=test.txt",
    ])?;

    match args.command {
        Commands::Add { command, tags } => {
            assert_eq!(command.join(" "), "touch @filename:Name of file to create=test.txt");
            assert_eq!(tags, Vec::<String>::new());
        }
        _ => panic!("Expected Add command"),
    }

    Ok(())
}
