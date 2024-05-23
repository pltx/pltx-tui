use std::{fs, path::PathBuf, thread, time::Duration};

use pltx_tracing::trace_panic;
use pltx_utils::DateTime;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;

fn get_db_path(filename: &str) -> PathBuf {
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

    data_dir.join(filename)
}

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
    filename: String,
    session_started: bool,
    pub session_id: Option<i32>,
}

impl Database {
    pub fn init(filename: &str) -> Database {
        let db_file_path = get_db_path(filename);
        let manager = SqliteConnectionManager::file(db_file_path);
        let pool = Pool::new(manager).unwrap();

        Database {
            pool,
            filename: filename.to_string(),
            session_id: None,
            session_started: false,
        }
    }

    /// Access the pooled connection.
    pub fn conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool.get().unwrap()
    }

    pub fn start_session(&mut self) -> rusqlite::Result<()> {
        self.ensure_tables()?;
        self.conn().execute(
            "INSERT INTO session (started, ended) VALUES (?1, ?2)",
            [DateTime::now(), DateTime::now()],
        )?;
        self.session_id = Some(self.last_row_id("session").unwrap());
        self.session_started = true;
        self.create_sync_session_thread()?;
        Ok(())
    }

    /// Ensure that the tables needed in the database are created here. If they
    /// don't, then create them.
    /// Non-global modules, popups, etc, manage their own data initialization.
    fn ensure_tables(&self) -> rusqlite::Result<()> {
        self.conn().execute(
            "CREATE TABLE IF NOT EXISTS session (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started DATETIME NOT NULL,
                ended DATETIME NOT NULL
            )",
            (),
        )?;

        Ok(())
    }

    fn create_sync_session_thread(&self) -> rusqlite::Result<()> {
        let pool = self.pool.clone();
        let session_id = self.session_id;

        thread::spawn(move || loop {
            let conn = pool.get().unwrap();
            conn.execute(
                "UPDATE session SET ended = ?1 WHERE id = ?2",
                (DateTime::now(), session_id),
            )
            .unwrap();
            thread::sleep(Duration::from_secs(1));
        });

        Ok(())
    }

    pub fn reset(&self) {
        fs::remove_file(get_db_path(&self.filename)).unwrap();
    }

    pub fn get_position(&self, table: &str, id: i32) -> rusqlite::Result<i32> {
        let query = format!("SELECT position FROM {} WHERE id = ?1", table);
        let conn = self.conn();
        let position: i32 = conn.prepare(&query)?.query_row([id], |r| r.get(0))?;
        Ok(position)
    }

    pub fn get_highest_position(&self, table: &str) -> rusqlite::Result<i32> {
        let query = format!(
            "SELECT position from {table} WHERE position = (SELECT MAX(position) FROM {table})"
        );
        let highest_position: i32 = self
            .conn()
            .prepare(&query)?
            .query_row([], |r| r.get(0))
            .unwrap_or(-1);
        Ok(highest_position)
    }

    pub fn get_highest_position_where<T>(
        &self,
        table: &str,
        field: &str,
        equals: T,
    ) -> rusqlite::Result<i32>
    where
        T: rusqlite::ToSql,
    {
        let query = format!(
            "SELECT position from {} WHERE position = (SELECT MAX(position) FROM {}) AND {} = ?1",
            table, table, field,
        );
        let conn = self.conn();
        let mut stmt = conn.prepare(&query)?;
        let highest_position: i32 = stmt.query_row([equals], |r| r.get(0)).unwrap_or(-1);
        Ok(highest_position)
    }

    pub fn update_positions(&self, table: &str, old_position: i32) -> rusqlite::Result<()> {
        let update_position_query = format!(
            "UPDATE {} SET position = position - 1, updated_at = ?1 WHERE position > ?2",
            table
        );
        let conn = self.conn();
        let mut update_position_stmt = conn.prepare(&update_position_query)?;
        update_position_stmt.execute((DateTime::now(), old_position))?;
        Ok(())
    }

    pub fn last_row_id(&self, table: &str) -> rusqlite::Result<i32> {
        let query = format!(
            "SELECT id from {} WHERE id = (SELECT MAX(id) FROM {})",
            table, table
        );
        let conn = self.conn();
        let mut stmt = conn.prepare(&query).unwrap();
        let recent_id: i32 = stmt.query_row((), |r| r.get(0)).unwrap();
        Ok(recent_id)
    }

    pub fn int_to_bool(&self, integer: i32) -> bool {
        if integer == 1 {
            true
        } else if integer == 0 {
            false
        } else {
            trace_panic!("failed to convert integer to bool");
        }
    }
}
