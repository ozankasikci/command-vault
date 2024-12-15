use anyhow::Result;
use command_vault::db::Database;
use tempfile::TempDir;

pub fn create_test_db() -> Result<(Database, TempDir)> {
    let dir = tempfile::tempdir()?;
    let db = Database::new(dir.path().join("test.db").to_str().unwrap())?;
    Ok((db, dir))
}
