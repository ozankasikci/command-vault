use std::sync::Once;
use anyhow::Result;
use command_vault::db::Database;
use tempfile::{tempdir, TempDir};
use std::fs;

static INIT: Once = Once::new();

pub fn init_test_env() {
    INIT.call_once(|| {
        // Disable TUI for tests
        std::env::set_var("COMMAND_VAULT_NO_TUI", "1");
    });
}

pub fn create_test_db() -> Result<(Database, TempDir)> {
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    fs::create_dir_all(temp_dir.path())?;
    let db = Database::new(db_path.to_str().unwrap())?;
    Ok((db, temp_dir))
}
