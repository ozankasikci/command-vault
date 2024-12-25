use chrono::{DateTime, TimeZone, Utc, NaiveDate};

pub fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    // Try RFC3339 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try common date formats
    let date_formats = [
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%m-%Y",
        "%d/%m/%Y",
    ];

    let datetime_formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S UTC",
        "%d/%m/%Y %H:%M",
        "%d/%m/%Y %H:%M:%S",
    ];

    // Try date-only formats first
    for format in date_formats {
        if let Ok(naive_date) = NaiveDate::parse_from_str(s, format) {
            let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
            return Some(Utc.from_utc_datetime(&naive_datetime));
        }
    }

    // Then try datetime formats
    for format in datetime_formats {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, format) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }

    None
}
