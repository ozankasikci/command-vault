use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: Option<i64>,
    pub command: String,
    pub timestamp: DateTime<Utc>,
    pub directory: String,
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}
