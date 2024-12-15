use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: Option<String>,
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    /// Unique identifier for the command
    pub id: Option<i64>,
    /// The actual command string
    pub command: String,
    /// When the command was executed
    pub timestamp: DateTime<Utc>,
    /// Directory where the command was executed
    pub directory: String,
    /// Tags associated with the command
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    /// Parameters for the command
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<Parameter>,
}
