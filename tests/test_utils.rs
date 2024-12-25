use anyhow::Result;
use chrono::{Datelike, Timelike};
use command_vault::db::Database;
use command_vault::utils::time::parse_datetime;
use command_vault::utils::params::{parse_parameters, substitute_parameters};
use tempfile::TempDir;

pub fn create_test_db() -> Result<(Database, TempDir)> {
    let dir = tempfile::tempdir()?;
    let db = Database::new(dir.path().join("test.db").to_str().unwrap())?;
    Ok((db, dir))
}

// Time parsing tests
#[test]
fn test_parse_datetime_rfc3339() {
    let input = "2024-03-14T15:30:00Z";
    let result = parse_datetime(input).unwrap();
    assert_eq!(result.year(), 2024);
    assert_eq!(result.month(), 3);
    assert_eq!(result.day(), 14);
    assert_eq!(result.hour(), 15);
    assert_eq!(result.minute(), 30);
    assert_eq!(result.second(), 0);
}

#[test]
fn test_parse_datetime_common_formats() {
    let test_cases = vec![
        // Date-only formats
        ("2024-03-14", 2024, 3, 14, 0, 0, 0),
        ("2024/03/14", 2024, 3, 14, 0, 0, 0),
        ("14-03-2024", 2024, 3, 14, 0, 0, 0),
        ("14/03/2024", 2024, 3, 14, 0, 0, 0),
        
        // Date-time formats
        ("2024-03-14 15:30", 2024, 3, 14, 15, 30, 0),
        ("2024-03-14 15:30:45", 2024, 3, 14, 15, 30, 45),
        ("14/03/2024 15:30", 2024, 3, 14, 15, 30, 0),
        ("14/03/2024 15:30:45", 2024, 3, 14, 15, 30, 45),
    ];

    for (input, year, month, day, hour, minute, second) in test_cases {
        let result = parse_datetime(input).unwrap_or_else(|| panic!("Failed to parse date: {}", input));
        assert_eq!(result.year(), year, "Year mismatch for {}", input);
        assert_eq!(result.month(), month, "Month mismatch for {}", input);
        assert_eq!(result.day(), day, "Day mismatch for {}", input);
        assert_eq!(result.hour(), hour, "Hour mismatch for {}", input);
        assert_eq!(result.minute(), minute, "Minute mismatch for {}", input);
        assert_eq!(result.second(), second, "Second mismatch for {}", input);
    }
}

#[test]
fn test_parse_datetime_invalid() {
    let test_cases = vec![
        "invalid",
        "2024",
        "2024-13-01",  // Invalid month
        "2024-01-32",  // Invalid day
        "03/14/24",    // Two-digit year not supported
        "",
    ];

    for input in test_cases {
        assert!(parse_datetime(input).is_none(), "Expected None for input: {}", input);
    }
}

// Parameter parsing tests
#[test]
fn test_parse_parameters_basic() {
    let command = "git checkout @branch";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "branch");
    assert_eq!(params[0].description, None);
}

#[test]
fn test_parse_parameters_with_description() {
    let command = "git checkout @branch:main-branch @tag:latest-tag";
    let params = parse_parameters(command);
    
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "branch");
    assert_eq!(params[0].description, Some("main-branch".to_string()));
    assert_eq!(params[1].name, "tag");
    assert_eq!(params[1].description, Some("latest-tag".to_string()));
}

#[test]
fn test_parse_parameters_description_edge_cases() {
    let test_cases = vec![
        // Description with spaces (should only take up to first space)
        ("git checkout @branch:main branch", "branch", Some("main")),
        // Description with special characters
        ("git checkout @branch:main-branch-2.0", "branch", Some("main-branch-2.0")),
        // Multiple parameters with descriptions
        ("git checkout @branch:main @tag:v1.0", "tag", Some("v1.0")),
        // Empty description
        ("git checkout @branch:", "branch", None),
        // Description with underscore
        ("git checkout @branch:main_branch", "branch", Some("main_branch")),
    ];

    for (command, param_name, expected_desc) in test_cases {
        let params = parse_parameters(command);
        let param = params.iter().find(|p| p.name == param_name).unwrap_or_else(|| panic!("Parameter {} not found", param_name));
        assert_eq!(param.description, expected_desc.map(String::from), "Failed for command: {}", command);
    }
}

#[test]
fn test_parse_parameters_multiple() {
    let command = "git checkout @branch:main && git pull @remote:origin @branch:main";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 3);
    
    assert_eq!(params[0].name, "branch");
    assert_eq!(params[0].description, Some("main".to_string()));
    
    assert_eq!(params[1].name, "remote");
    assert_eq!(params[1].description, Some("origin".to_string()));
    
    assert_eq!(params[2].name, "branch");
    assert_eq!(params[2].description, Some("main".to_string()));
}

#[test]
fn test_parse_parameters_no_parameters() {
    let command = "git status";
    let params = parse_parameters(command);
    assert!(params.is_empty());
}

#[test]
fn test_parse_parameters_with_spaces() {
    let command = "grep @pattern:search term file.txt";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "pattern");
    assert_eq!(params[0].description, Some("search".to_string()));
}

#[test]
fn test_parse_parameters_underscore() {
    let command = "git checkout @feature_branch:main_branch";
    let params = parse_parameters(command);
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "feature_branch");
    assert_eq!(params[0].description, Some("main_branch".to_string()));
}

// Parameter substitution tests
#[test]
fn test_substitute_parameters_basic() -> Result<()> {
    let command = "git checkout @branch";
    let params = parse_parameters(command);
    let test_input = Some("main");
    
    let result = substitute_parameters(command, &params, test_input)?;
    assert_eq!(result, "git checkout main");
    
    Ok(())
}

#[test]
fn test_substitute_parameters_with_quotes() -> Result<()> {
    let test_cases = vec![
        // Value with spaces should be quoted
        ("grep @pattern", Some("search term"), "grep 'search term'"),
        // Value with special chars should be quoted
        ("grep @pattern", Some("term*"), "grep 'term*'"),
        // Empty value should be quoted
        ("grep @pattern", Some(""), "grep ''"),
        // Value with semicolon should be quoted
        ("echo @value", Some("a;b"), "echo 'a;b'"),
        // Value with pipe should be quoted
        ("echo @value", Some("a|b"), "echo 'a|b'"),
    ];

    for (command, input, expected) in test_cases {
        let params = parse_parameters(command);
        let result = substitute_parameters(command, &params, input)?;
        assert_eq!(result, expected, "Failed for command: {}", command);
    }
    
    Ok(())
}

#[test]
fn test_substitute_parameters_multiple() -> Result<()> {
    let command = "git checkout @branch && git pull @remote @branch";
    let params = parse_parameters(command);
    let test_input = Some("main\norigin");
    
    let result = substitute_parameters(command, &params, test_input)?;
    assert_eq!(result, "git checkout main && git pull origin main");
    
    Ok(())
}

#[test]
fn test_substitute_parameters_special_cases() -> Result<()> {
    let test_cases = vec![
        // Command with no parameters
        ("git status", None, "git status"),
        // Command with empty parameter list
        ("git checkout @branch", Some(""), "git checkout ''"),
        // Command with multiple identical parameters
        ("echo @value @value", Some("test"), "echo test test"),
        // Command with redirection
        ("echo @value > file.txt", Some("test"), "echo 'test' > file.txt"),
    ];

    for (command, input, expected) in test_cases {
        let params = parse_parameters(command);
        let result = substitute_parameters(command, &params, input)?;
        assert_eq!(result, expected, "Failed for command: {}", command);
    }
    
    Ok(())
}
