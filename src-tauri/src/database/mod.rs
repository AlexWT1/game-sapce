use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

pub mod migrations;

pub struct Database {
    conn: Mutex<Connection>,
    path: PathBuf,
}

impl Database {
    pub fn open(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrations::run(&conn)?;
        log::info!("Database ready at {}", path.display());
        Ok(Self {
            conn: Mutex::new(conn),
            path,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Runs a closure with exclusive access to the connection.
    pub fn with<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let guard = self.conn.lock().map_err(|_| {
            AppError::Other("database lock poisoned".into())
        })?;
        f(&guard)
    }
}
