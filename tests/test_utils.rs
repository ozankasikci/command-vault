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

#[test]
fn test_parse_datetime_rfc3339() {
    let input = "2023-12-25T15:30:00Z";
    let dt = parse_datetime(input).expect("Failed to parse RFC3339 date");
    
    assert_eq!(dt.year(), 2023);
    assert_eq!(dt.month(), 12);
    assert_eq!(dt.day(), 25);
    assert_eq!(dt.hour(), 15);
    assert_eq!(dt.minute(), 30);
    assert_eq!(dt.second(), 0);
}

#[test]
fn test_parse_datetime_common_formats() {
    let test_cases = vec![
        // Date-only formats
        ("2023-12-25", 2023, 12, 25, 0, 0, 0),
        ("2023/12/25", 2023, 12, 25, 0, 0, 0),
        ("25-12-2023", 2023, 12, 25, 0, 0, 0),
        ("25/12/2023", 2023, 12, 25, 0, 0, 0),
        
        // Date-time formats
        ("2023-12-25 15:30", 2023, 12, 25, 15, 30, 0),
        ("2023-12-25 15:30:45", 2023, 12, 25, 15, 30, 45),
        ("25/12/2023 15:30", 2023, 12, 25, 15, 30, 0),
        ("25/12/2023 15:30:45", 2023, 12, 25, 15, 30, 45),
    ];

    for (input, year, month, day, hour, minute, second) in test_cases {
        let dt = parse_datetime(input).expect(&format!("Failed to parse date: {}", input));
        
        assert_eq!(dt.year(), year, "Year mismatch for {}", input);
        assert_eq!(dt.month(), month, "Month mismatch for {}", input);
        assert_eq!(dt.day(), day, "Day mismatch for {}", input);
        assert_eq!(dt.hour(), hour, "Hour mismatch for {}", input);
        assert_eq!(dt.minute(), minute, "Minute mismatch for {}", input);
        assert_eq!(dt.second(), second, "Second mismatch for {}", input);
    }
}

#[test]
fn test_parse_datetime_invalid() {
    let invalid_dates = vec![
        "invalid",
        "2023-13-01",  // Invalid month
        "2023-12-32",  // Invalid day
        "25-13-2023",  // Invalid month
        "32-12-2023",  // Invalid day
        "2023-12-25 24:00",  // Invalid hour
        "2023-12-25 15:60",  // Invalid minute
        "2023-12-25 15:30:61",  // Invalid second
        "",  // Empty string
        "2023",  // Incomplete date
        "2023-12",  // Incomplete date
        "15:30",  // Time only
    ];

    for input in invalid_dates {
        assert!(parse_datetime(input).is_none(), "Expected None for invalid date: {}", input);
    }
}
