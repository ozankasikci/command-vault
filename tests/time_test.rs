use command_vault::utils::time::parse_datetime;
use chrono::{DateTime, Utc};

#[test]
fn test_parse_datetime_valid() {
    let input = "2024-01-01T12:00:00Z";
    let result = parse_datetime(input);
    assert!(result.is_some());
    let dt: DateTime<Utc> = result.unwrap();
    assert_eq!(dt.to_rfc3339(), "2024-01-01T12:00:00+00:00");
}

#[test]
fn test_parse_datetime_invalid() {
    let input = "invalid date";
    let result = parse_datetime(input);
    assert!(result.is_none());
}

#[test]
fn test_parse_datetime_empty() {
    let input = "";
    let result = parse_datetime(input);
    assert!(result.is_none());
}

#[test]
fn test_parse_datetime_different_formats() {
    let inputs = vec![
        "2024-01-01T12:00:00+00:00",
        "2024-01-01T12:00:00-05:00",
        "2024-01-01 12:00:00 UTC",
    ];

    for input in inputs {
        let result = parse_datetime(input);
        assert!(result.is_some(), "Failed to parse: {}", input);
    }
}
