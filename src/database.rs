use std::{fs, path::PathBuf};

use rusqlite::Connection;

pub struct Session {
    pub id: i32,
    pub started: String,
    pub ended: Option<String>,
}

fn get_db_path() -> PathBuf {
    let home_dir = home::home_dir().unwrap_or_else(|| panic!("failed to find home directory"));
    let data_dir = PathBuf::from(format!("{}/.local/share/pltx", home_dir.to_str().unwrap()));

    // Create the directory if it doesn't exist
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).unwrap_or_else(|_| {
            panic!(
                "Failed to create directory: {}",
                &data_dir.to_str().unwrap()
            )
        });
    }

    data_dir.join("data.db")
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn init() -> Database {
        let db_path = get_db_path();
        let conn = Connection::open(db_path).unwrap();
        Database { conn }
    }

    /// Ensure that the tables needed in the database are created here. If they
    /// don't, then create them.
    /// For screens and popups, they should implement the `InitData` trait and
    /// call the `init_data()` method in `init()`.
    pub fn ensure_tables(&mut self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS session (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started DATETIME DEFAULT CURRENT_TIMESTAMP,
                ended DATETIME
            )",
            (),
        )?;

        Ok(())
    }

    pub fn insert_session(&mut self) -> rusqlite::Result<()> {
        self.conn
            .execute("INSERT INTO session (id) VALUES (NULL)", ())?;
        Ok(())
    }
}
