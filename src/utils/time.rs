use chrono::{DateTime, TimeZone, Utc};

pub fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try common date formats
    let formats = [
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%d/%m/%Y",
        "%d/%m/%Y %H:%M",
        "%d/%m/%Y %H:%M:%S",
    ];

    for format in formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, format) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }

    None
}
