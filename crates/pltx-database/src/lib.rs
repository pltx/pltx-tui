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
    pub fn ensure_tables(&self) -> rusqlite::Result<()> {
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

    pub fn insert_session(&self) -> rusqlite::Result<()> {
        self.conn
            .execute("INSERT INTO session (id) VALUES (NULL)", ())?;
        Ok(())
    }

    pub fn get_position(&self, table: &str, id: i32) -> rusqlite::Result<i32> {
        let query = format!("SELECT position FROM {} WHERE id = ?1", table);
        let mut stmt = self.conn.prepare(&query)?;
        let position: i32 = stmt.query_row([id], |r| r.get(0))?;
        Ok(position)
    }

    pub fn get_highest_position(&self, table: &str) -> rusqlite::Result<i32> {
        let query = format!(
            "SELECT position from {table} WHERE position = (SELECT MAX(position) FROM {table})"
        );
        let mut stmt = self.conn.prepare(&query)?;
        let highest_position: i32 = stmt.query_row([], |r| r.get(0)).unwrap_or(-1);
        Ok(highest_position)
    }

    pub fn update_positions(&self, table: &str, old_position: i32) -> rusqlite::Result<()> {
        let update_position_query = format!(
            "UPDATE {} SET position = position - 1 WHERE position > ?1",
            table
        );
        let mut update_position_stmt = self.conn.prepare(&update_position_query)?;
        update_position_stmt.execute([old_position])?;
        Ok(())
    }

    pub fn last_row_id(&self, table: &str) -> rusqlite::Result<i32> {
        let query = format!(
            "SELECT id from {} WHERE id = (SELECT MAX(id) FROM {})",
            table, table
        );
        let mut stmt = self.conn.prepare(&query).unwrap();
        let recent_id: i32 = stmt.query_row((), |r| r.get(0)).unwrap();
        Ok(recent_id)
    }
}
