use anyhow::Result;
use chrono::{Datelike, Timelike};
use command_vault::db::Database;
use command_vault::utils::time::parse_datetime;
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
