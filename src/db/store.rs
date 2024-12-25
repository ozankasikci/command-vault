//! Database module for command-vault
//! 
//! This module provides SQLite-based storage for commands, tags, and parameters.
//! It handles all database operations including CRUD operations for commands,
//! tag management, and search functionality.

use anyhow::{Result, anyhow};
use rusqlite::Connection;
use chrono::Utc;
use serde_json;

use super::models::Command;

/// The main database interface for command-vault.
/// 
/// Handles all database operations including:
/// - Command storage and retrieval
/// - Tag management
/// - Search functionality
/// 
/// # Example
/// ```no_run
/// use anyhow::Result;
/// use command_vault::db::Database;
/// 
/// fn main() -> Result<()> {
///     let db = Database::new("commands.db")?;
///     db.init()?;
///     Ok(())
/// }
/// ```
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Creates a new database connection.
    /// 
    /// # Arguments
    /// * `path` - Path to the SQLite database file
    /// 
    /// # Returns
    /// * `Result<Database>` - A new database instance
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init()?;
        Ok(db)
    }

    /// Initializes the database schema.
    /// 
    /// Creates the following tables if they don't exist:
    /// - commands: Stores command information
    /// - tags: Stores tag information
    /// - command_tags: Links commands to tags
    pub fn init(&self) -> Result<()> {
        // Create commands table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                directory TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '',
                parameters TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;
        
        // Create tags table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        // Create command_tags table for many-to-many relationship
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS command_tags (
                command_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (command_id, tag_id),
                FOREIGN KEY (command_id) REFERENCES commands(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_commands_command ON commands(command)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_name ON tags(name)",
            [],
        )?;
        
        Ok(())
    }

    /// Adds a new command to the database.
    /// 
    /// # Arguments
    /// * `command` - The command to add
    /// 
    /// # Returns
    /// * `Result<i64>` - The ID of the newly added command
    pub fn add_command(&mut self, command: &Command) -> Result<i64> {
        let tx = self.conn.transaction()?;
        
        // Insert the command
        tx.execute(
            "INSERT INTO commands (command, timestamp, directory, tags, parameters)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &command.command,
                &command.timestamp.to_rfc3339(),
                &command.directory,
                &command.tags.join(","),
                &serde_json::to_string(&command.parameters)?,
            ),
        )?;
        
        let command_id = tx.last_insert_rowid();
        
        // Add tags if present
        for tag in &command.tags {
            // Insert or get tag
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                [tag],
            )?;
            
            let tag_id: i64 = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                [tag],
                |row| row.get(0),
            )?;
            
            // Link command to tag
            tx.execute(
                "INSERT OR IGNORE INTO command_tags (command_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![command_id, tag_id],
            )?;
        }
        
        tx.commit()?;
        Ok(command_id)
    }

    /// Adds tags to an existing command.
    /// 
    /// # Arguments
    /// * `command_id` - The ID of the command to add tags to
    /// * `tags` - The tags to add
    /// 
    /// # Returns
    /// * `Result<()>` - Success or failure
    pub fn add_tags_to_command(&mut self, command_id: i64, tags: &[String]) -> Result<()> {
        let tx = self.conn.transaction()?;
        
        // Verify command exists
        let exists: bool = tx.query_row(
            "SELECT 1 FROM commands WHERE id = ?1",
            [command_id],
            |_| Ok(true),
        ).unwrap_or(false);
        
        if !exists {
            return Err(anyhow!("Command not found"));
        }
        
        // Get current tags
        let mut current_tags = Vec::new();
        {
            let mut stmt = tx.prepare(
                "SELECT t.name 
                 FROM tags t 
                 JOIN command_tags ct ON ct.tag_id = t.id 
                 WHERE ct.command_id = ?1"
            )?;
            let mut rows = stmt.query([command_id])?;
            while let Some(row) = rows.next()? {
                current_tags.push(row.get::<_, String>(0)?);
            }
        }
        
        for tag in tags {
            // Skip if tag already exists
            if current_tags.contains(tag) {
                continue;
            }
            
            // Insert or get tag
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                [tag],
            )?;
            
            let tag_id: i64 = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                [tag],
                |row| row.get(0),
            )?;
            
            // Link command to tag
            tx.execute(
                "INSERT OR IGNORE INTO command_tags (command_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![command_id, tag_id],
            )?;
            
            // Update tags string in commands table
            current_tags.push(tag.clone());
        }
        
        // Update the tags string in the commands table
        tx.execute(
            "UPDATE commands SET tags = ?1 WHERE id = ?2",
            rusqlite::params![current_tags.join(","), command_id],
        )?;
        
        tx.commit()?;
        Ok(())
    }

    /// Removes a tag from a command.
    /// 
    /// # Arguments
    /// * `command_id` - The ID of the command to remove the tag from
    /// * `tag_name` - The name of the tag to remove
    /// 
    /// # Returns
    /// * `Result<()>` - Success or failure
    pub fn remove_tag_from_command(&mut self, command_id: i64, tag_name: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        
        tx.execute(
            "DELETE FROM command_tags 
             WHERE command_id = ?1 
             AND tag_id = (SELECT id FROM tags WHERE name = ?2)",
            rusqlite::params![command_id, tag_name],
        )?;
        
        tx.commit()?;
        Ok(())
    }

    /// Searches for commands containing a given query string.
    /// 
    /// # Arguments
    /// * `query` - The query string to search for
    /// * `limit` - The maximum number of results to return
    /// 
    /// # Returns
    /// * `Result<Vec<Command>>` - A list of matching commands
    pub fn search_commands(&self, query: &str, limit: usize) -> Result<Vec<Command>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
             FROM commands c
             WHERE c.command LIKE '%' || ?1 || '%'
             ORDER BY c.timestamp DESC
             LIMIT ?2"
        )?;

        let mut rows = stmt.query([query, &limit.to_string()])?;
        let mut commands = Vec::new();

        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            commands.push(Command {
                id: Some(id),
                command: row.get(1)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?
                    .with_timezone(&Utc),
                directory: row.get(3)?,
                tags: row.get::<_, String>(4)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(5)?)?,
            });
        }

        Ok(commands)
    }

    /// Searches for commands with a given tag.
    /// 
    /// # Arguments
    /// * `tag` - The tag to search for
    /// * `limit` - The maximum number of results to return
    /// 
    /// # Returns
    /// * `Result<Vec<Command>>` - A list of matching commands
    pub fn search_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<Command>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
             FROM commands c
             JOIN command_tags ct ON ct.command_id = c.id
             JOIN tags t ON t.id = ct.tag_id
             WHERE t.name = ?1
             ORDER BY c.timestamp DESC
             LIMIT ?2"
        )?;

        let mut rows = stmt.query([tag, &limit.to_string()])?;
        let mut commands = Vec::new();

        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            commands.push(Command {
                id: Some(id),
                command: row.get(1)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?
                    .with_timezone(&Utc),
                directory: row.get(3)?,
                tags: row.get::<_, String>(4)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(5)?)?,
            });
        }

        Ok(commands)
    }

    /// Lists all tags in the database.
    /// 
    /// # Returns
    /// * `Result<Vec<(String, i64)>>` - A list of tags with their respective counts
    pub fn list_tags(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name, COUNT(ct.command_id) as count
             FROM tags t
             LEFT JOIN command_tags ct ON ct.tag_id = t.id
             GROUP BY t.id, t.name
             ORDER BY count DESC, t.name"
        )?;
        
        let tags = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(tags)
    }

    /// Lists all commands in the database.
    /// 
    /// # Arguments
    /// * `limit` - The maximum number of results to return
    /// * `ascending` - Whether to return results in ascending order
    /// 
    /// # Returns
    /// * `Result<Vec<Command>>` - A list of commands
    pub fn list_commands(&self, limit: usize, ascending: bool) -> Result<Vec<Command>> {
        let query = if ascending {
            if limit == 0 {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp ASC"
            } else {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp ASC
                 LIMIT ?1"
            }
        } else {
            if limit == 0 {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp DESC"
            } else {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp DESC
                 LIMIT ?1"
            }
        };

        let mut stmt = self.conn.prepare(query)?;
        let mut rows = if limit == 0 {
            stmt.query([])?
        } else {
            stmt.query([limit])?
        };
        
        let mut commands = Vec::new();

        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            commands.push(Command {
                id: Some(id),
                command: row.get(1)?,
                timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?
                    .with_timezone(&Utc),
                directory: row.get(3)?,
                tags: row.get::<_, String>(4)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(5)?)?,
            });
        }

        Ok(commands)
    }

    /// Gets a command by its ID.
    /// 
    /// # Arguments
    /// * `id` - The ID of the command to retrieve
    /// 
    /// # Returns
    /// * `Result<Option<Command>>` - The command if found
    pub fn get_command(&self, id: i64) -> Result<Option<Command>> {
        // First get the command details
        let mut stmt = self.conn.prepare(
            "SELECT command, timestamp, directory, parameters 
             FROM commands 
             WHERE id = ?1"
        )?;

        let command = stmt.query_row([id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        });

        if let Ok((command, timestamp, directory, parameters)) = command {
            // Then get the tags
            let mut stmt = self.conn.prepare(
                "SELECT t.name 
                 FROM tags t 
                 JOIN command_tags ct ON ct.tag_id = t.id 
                 WHERE ct.command_id = ?1"
            )?;

            let mut tags = Vec::new();
            let mut rows = stmt.query([id])?;
            while let Some(row) = rows.next()? {
                tags.push(row.get::<_, String>(0)?);
            }

            Ok(Some(Command {
                id: Some(id),
                command,
                timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)?
                    .with_timezone(&Utc),
                directory,
                tags,
                parameters: serde_json::from_str(&parameters)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Updates an existing command.
    /// 
    /// # Arguments
    /// * `command` - The updated command
    /// 
    /// # Returns
    /// * `Result<()>` - Success or failure
    pub fn update_command(&mut self, command: &Command) -> Result<()> {
        if command.id.is_none() {
            return Err(anyhow!("Cannot update command without id"));
        }

        let tx = self.conn.transaction()?;
        
        // Update command
        tx.execute(
            "UPDATE commands 
             SET command = ?1, 
                 timestamp = ?2,
                 directory = ?3,
                 tags = ?4,
                 parameters = ?5
             WHERE id = ?6",
            rusqlite::params![
                command.command,
                command.timestamp.to_rfc3339(),
                command.directory,
                command.tags.join(","),
                serde_json::to_string(&command.parameters)?,
                command.id.unwrap()
            ],
        )?;

        // Delete existing tags
        tx.execute(
            "DELETE FROM command_tags WHERE command_id = ?1",
            [command.id.unwrap()],
        )?;

        // Add new tags using the same transaction
        for tag in &command.tags {
            // Insert or get tag
            tx.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                [tag],
            )?;
            
            let tag_id: i64 = tx.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                [tag],
                |row| row.get(0),
            )?;
            
            // Link command to tag
            tx.execute(
                "INSERT OR IGNORE INTO command_tags (command_id, tag_id) VALUES (?1, ?2)",
                rusqlite::params![command.id.unwrap(), tag_id],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    /// Deletes a command by its ID.
    /// 
    /// # Arguments
    /// * `command_id` - The ID of the command to delete
    /// 
    /// # Returns
    /// * `Result<()>` - Success or failure
    pub fn delete_command(&mut self, command_id: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        
        // First delete from command_tags
        tx.execute(
            "DELETE FROM command_tags WHERE command_id = ?",
            [command_id],
        )?;

        // Then delete from commands
        let rows_affected = tx.execute(
            "DELETE FROM commands WHERE id = ?",
            [command_id],
        )?;

        if rows_affected == 0 {
            return Err(anyhow!("Command not found"));
        }

        // Clean up unused tags
        tx.execute(
            "DELETE FROM tags WHERE id NOT IN (SELECT DISTINCT tag_id FROM command_tags)",
            [],
        )?;

        tx.commit()?;
        Ok(())
    }
}
