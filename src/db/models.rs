//! Database models for command-vault
//! 
//! This module defines the core data structures used throughout the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a command with its metadata.
/// 
/// A command includes the actual command string, execution directory,
/// timestamp, tags, and parameters.
/// 
/// # Example
/// ```rust
/// use command_vault::db::models::Command;
/// use chrono::Utc;
/// 
/// let cmd = Command {
///     id: None,
///     command: "git push origin main".to_string(),
///     timestamp: Utc::now(),
///     directory: "/project".to_string(),
///     tags: vec!["git".to_string()],
///     parameters: vec![],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Unique identifier for the command
    pub id: Option<i64>,
    
    /// The actual command string
    pub command: String,
    
    /// When the command was created or last modified
    pub timestamp: DateTime<Utc>,
    
    /// Directory where the command should be executed
    pub directory: String,
    
    /// Tags associated with the command
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    
    /// Parameters that can be substituted in the command
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub parameters: Vec<Parameter>,
}

/// Represents a parameter that can be substituted in a command.
/// 
/// Parameters allow commands to be more flexible by providing
/// placeholders that can be filled in at runtime.
/// 
/// # Example
/// ```rust
/// use command_vault::db::models::Parameter;
/// 
/// let param = Parameter {
///     name: "branch".to_string(),
///     description: Some("Git branch name".to_string()),
///     default_value: Some("main".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// Name of the parameter (used in substitution)
    pub name: String,
    
    /// Optional description of what the parameter does
    pub description: Option<String>,
    
    /// Optional default value for the parameter
    pub default_value: Option<String>,
}
