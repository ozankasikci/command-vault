use command_vault::cli::args::{Cli, Commands, TagCommands};
use clap::Parser;

#[test]
fn test_parse_args_add() {
    let args = vec!["cv", "add", "--", "ls", "-l"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Add { command, tags } => {
            assert_eq!(command, vec!["ls", "-l"]);
            assert!(tags.is_empty());
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_parse_args_add_with_tags() {
    let args = vec!["cv", "add", "-t", "file", "-t", "list", "--", "ls", "-l"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Add { command, tags } => {
            assert_eq!(command, vec!["ls", "-l"]);
            assert_eq!(tags, vec!["file", "list"]);
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_parse_args_ls() {
    let args = vec!["cv", "ls"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Ls { limit, asc } => {
            assert_eq!(limit, 50);
            assert!(!asc);
        }
        _ => panic!("Expected Ls command"),
    }
}

#[test]
fn test_parse_args_ls_with_limit() {
    let args = vec!["cv", "ls", "--limit", "5"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Ls { limit, asc } => {
            assert_eq!(limit, 5);
            assert!(!asc);
        }
        _ => panic!("Expected Ls command"),
    }
}

#[test]
fn test_parse_args_exec() {
    let args = vec!["cv", "exec", "1"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Exec { command_id, debug } => {
            assert_eq!(command_id, 1);
            assert_eq!(debug, false); // Default value should be false
        }
        _ => panic!("Expected Exec command"),
    }

    // Test with debug flag
    let args = vec!["cv", "exec", "1", "--debug"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Exec { command_id, debug } => {
            assert_eq!(command_id, 1);
            assert_eq!(debug, true);
        }
        _ => panic!("Expected Exec command"),
    }
}

#[test]
fn test_parse_args_search() {
    let args = vec!["cv", "search", "git"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Search { query, limit } => {
            assert_eq!(query, "git");
            assert_eq!(limit, 10);
        }
        _ => panic!("Expected Search command"),
    }
}

#[test]
fn test_parse_args_tag_add() {
    let args = vec!["cv", "tag", "add", "1", "--", "git", "vcs"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Tag { action } => {
            match action {
                TagCommands::Add { command_id, tags } => {
                    assert_eq!(command_id, 1);
                    assert_eq!(tags, vec!["git", "vcs"]);
                }
                _ => panic!("Expected Tag Add command"),
            }
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_parse_args_tag_remove() {
    let args = vec!["cv", "tag", "remove", "1", "--", "git"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Tag { action } => {
            match action {
                TagCommands::Remove { command_id, tag } => {
                    assert_eq!(command_id, 1);
                    assert_eq!(tag, "git");
                }
                _ => panic!("Expected Tag Remove command"),
            }
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_parse_args_tag_list() {
    let args = vec!["cv", "tag", "list"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Tag { action } => {
            match action {
                TagCommands::List => (),
                _ => panic!("Expected Tag List command"),
            }
        }
        _ => panic!("Expected Tag command"),
    }
}

#[test]
fn test_parse_args_tag_search() {
    let args = vec!["cv", "tag", "search", "--", "git"];
    let cli = Cli::parse_from(args);
    match cli.command {
        Commands::Tag { action } => {
            match action {
                TagCommands::Search { tag, limit } => {
                    assert_eq!(tag, "git");
                    assert_eq!(limit, 10);
                }
                _ => panic!("Expected Tag Search command"),
            }
        }
        _ => panic!("Expected Tag command"),
    }
}
