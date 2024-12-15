use anyhow::{Result, anyhow};
use rusqlite::Connection;
use chrono::{DateTime, Utc};
use serde_json;

use super::models::Command;

pub struct Database {
    conn: Connection,
    path: String,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn, path: path.to_string() };
        db.init()?;
        Ok(db)
    }

    pub fn clone_with_new_connection(&self) -> Result<Self> {
        let conn = Connection::open(&self.path)?;
        Ok(Database {
            conn,
            path: self.path.clone(),
        })
    }

    fn init(&self) -> Result<()> {
        // Create commands table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS commands (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                directory TEXT NOT NULL,
                exit_code INTEGER,
                tags TEXT DEFAULT '',
                parameters TEXT DEFAULT '[]'
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
                command_id INTEGER,
                tag_id INTEGER,
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

    pub fn add_command(&mut self, command: &Command) -> Result<i64> {
        let tx = self.conn.transaction()?;
        
        // Insert the command
        tx.execute(
            "INSERT INTO commands (command, timestamp, directory, exit_code, tags, parameters)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &command.command,
                &command.timestamp.to_rfc3339(),
                &command.directory,
                &command.exit_code,
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
        
        for tag in tags {
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
        Ok(())
    }

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

    pub fn get_command_tags(&self, command_id: i64) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name 
             FROM tags t
             JOIN command_tags ct ON ct.tag_id = t.id
             WHERE ct.command_id = ?1
             ORDER BY t.name"
        )?;
        
        let tags = stmt.query_map([command_id], |row| {
            row.get::<_, String>(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(tags)
    }

    pub fn search_commands(&self, query: &str, limit: usize) -> Result<Vec<Command>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
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
                exit_code: row.get(4)?,
                tags: row.get::<_, String>(5)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(6)?)
                    .map_err(|e| anyhow!("Failed to parse parameters: {}", e))?,
            });
        }

        Ok(commands)
    }

    pub fn search_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<Command>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
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
                exit_code: row.get(4)?,
                tags: row.get::<_, String>(5)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(6)?)
                    .map_err(|e| anyhow!("Failed to parse parameters: {}", e))?,
            });
        }

        Ok(commands)
    }

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

    pub fn list_commands(&self, limit: usize, ascending: bool) -> Result<Vec<Command>> {
        let query = if ascending {
            if limit == 0 {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp ASC"
            } else {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp ASC
                 LIMIT ?1"
            }
        } else {
            if limit == 0 {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
                 FROM commands c
                 ORDER BY c.timestamp DESC"
            } else {
                "SELECT c.id, c.command, c.timestamp, c.directory, c.exit_code, c.tags, c.parameters 
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
                exit_code: row.get(4)?,
                tags: row.get::<_, String>(5)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(6)?)
                    .map_err(|e| anyhow!("Failed to parse parameters: {}", e))?,
            });
        }

        Ok(commands)
    }

    pub fn get_command(&self, id: i64) -> Result<Option<Command>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, command, timestamp, directory, exit_code, tags, parameters 
             FROM commands
             WHERE id = ?"
        )?;

        let mut rows = stmt.query([id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Command {
                id: Some(row.get(0)?),
                command: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)?.with_timezone(&Utc),
                directory: row.get(3)?,
                exit_code: row.get(4)?,
                tags: row.get::<_, String>(5)?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect(),
                parameters: serde_json::from_str(&row.get::<_, String>(6)?)?,
            }))
        } else {
            Ok(None)
        }
    }

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
                 exit_code = ?4,
                 tags = ?5,
                 parameters = ?6
             WHERE id = ?7",
            rusqlite::params![
                command.command,
                command.timestamp.to_rfc3339(),
                command.directory,
                command.exit_code,
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

    pub fn delete_command(&mut self, command_id: i64) -> Result<()> {
        // First delete from command_tags
        self.conn.execute(
            "DELETE FROM command_tags WHERE command_id = ?",
            [command_id],
        )?;

        // Then delete from commands
        let rows_affected = self.conn.execute(
            "DELETE FROM commands WHERE id = ?",
            [command_id],
        )?;

        if rows_affected == 0 {
            return Err(anyhow!("Command not found"));
        }

        Ok(())
    }
}
