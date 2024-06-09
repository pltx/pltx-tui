//! Initializes a new connection pool to the database and provides utility
//! methods for convenience.

use std::{
    fs, thread,
    time::{Duration, Instant},
};

use color_eyre::Result;
use pltx_utils::{dirs, DateTime};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::ToSql;

mod init_sql;

pub use init_sql::*;
use tracing::info;

pub struct Database {
    pool: Pool<SqliteConnectionManager>,
    filename: String,
    session_started: bool,
    pub session_id: Option<i32>,
    pub started: Option<DateTime>,
}

impl Database {
    pub fn init(filename: String) -> Database {
        let db_file = dirs::data_dir().join(&filename);
        let manager = SqliteConnectionManager::file(db_file);
        let pool = Pool::new(manager).expect("failed to create database pool");

        Database {
            pool,
            filename,
            session_id: None,
            session_started: false,
            started: None,
        }
    }

    /// Access the pooled connection.
    pub fn conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.pool.get().expect("failed to get database pool")
    }

    pub fn execute<P: rusqlite::Params>(&self, query: &str, params: P) -> Result<usize> {
        let conn = self.conn();
        let mut stmt = conn.prepare(query)?;
        Ok(stmt.execute(params)?)
    }

    pub fn start_session(&mut self) -> Result<()> {
        let start = Instant::now();
        self.ensure_tables()?;
        let started = DateTime::new();
        self.execute(
            "INSERT INTO session (started, ended) VALUES (?1, ?2)",
            [started.into_db(), DateTime::now()],
        )?;
        self.session_id = Some(self.last_row_id("session")?);
        self.started = Some(started);
        self.session_started = true;
        self.create_sync_session_thread()?;
        info!("started session in {:?}", start.elapsed());
        Ok(())
    }

    /// Ensure that the tables needed in the database are created here. If they
    /// don't, then create them.
    /// Non-global modules, popups, etc, manage their own data initialization.
    fn ensure_tables(&self) -> Result<()> {
        self.execute(
            "CREATE TABLE IF NOT EXISTS session (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                started DATETIME NOT NULL,
                ended DATETIME NOT NULL
            )",
            (),
        )?;

        Ok(())
    }

    fn create_sync_session_thread(&self) -> Result<()> {
        let pool = self.pool.clone();
        let session_id = self.session_id;

        thread::spawn(move || loop {
            let conn = pool.get().expect("failed to get database pool");
            conn.execute(
                "UPDATE session SET ended = ?1 WHERE id = ?2",
                (DateTime::now(), session_id),
            )
            .expect("failed to sync session");
            thread::sleep(Duration::from_secs(1));
        });

        Ok(())
    }

    pub fn reset(&self) -> Result<()> {
        fs::remove_file(dirs::data_dir().join(&self.filename))?;
        Ok(())
    }

    pub fn get_position(&self, table: &str, id: i32) -> Result<i32> {
        let query = format!("SELECT position FROM {} WHERE id = ?1", table);
        let conn = self.conn();
        let position: i32 = conn.prepare(&query)?.query_row([id], |r| r.get(0))?;
        Ok(position)
    }

    pub fn get_highest_position(&self, table: &str) -> Result<i32> {
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

    pub fn get_highest_position_where<T>(&self, table: &str, field: &str, equals: T) -> Result<i32>
    where
        T: ToSql,
    {
        let conn = self.conn();
        let query = format!(
            "SELECT position from {} WHERE {} = ?1 AND position = (SELECT MAX(position) FROM {} \
             WHERE {} = ?1)",
            table, field, table, field
        );
        let mut stmt = conn.prepare(&query)?;
        let highest_position: i32 = stmt.query_row([equals], |r| r.get(0)).unwrap_or(-1);
        Ok(highest_position)
    }

    pub fn decrement_positions_after(&self, table: &str, old_position: i32) -> Result<()> {
        let query = format!(
            "UPDATE {} SET position = position - 1, updated_at = ?1 WHERE position > ?2",
            table
        );
        self.execute(&query, (DateTime::now(), old_position))?;
        Ok(())
    }

    pub fn decrement_positions_after_where<T>(
        &self,
        table: &str,
        old_position: i32,
        field: &str,
        equals: T,
    ) -> Result<()>
    where
        T: ToSql,
    {
        let query = format!(
            "UPDATE {} SET position = position - 1, updated_at = ?1 WHERE position > ?2 and {} = \
             ?3",
            table, field
        );
        self.execute(&query, (DateTime::now(), old_position, equals))?;
        Ok(())
    }

    pub fn increment_position(&self, table: &str, id: i32, next_id: i32) -> Result<()> {
        let query = format!(
            "UPDATE {} SET position = position + 1, updated_at = ?1 where id = ?2",
            table,
        );
        self.execute(&query, (DateTime::now(), id))?;

        let query_2 = format!(
            "UPDATE {} SET position = position - 1, updated_at = ?1 where id = ?2",
            table
        );
        self.execute(&query_2, (DateTime::now(), next_id))?;

        Ok(())
    }

    pub fn decrement_position(&self, table: &str, id: i32, prev_id: i32) -> Result<()> {
        let query = format!(
            "UPDATE {} SET position = position - 1, updated_at = ?1 where id = ?2",
            table,
        );
        self.execute(&query, (DateTime::now(), id))?;

        let query_2 = format!(
            "UPDATE {} SET position = position + 1, updated_at = ?1 where id = ?2",
            table
        );
        self.execute(&query_2, (DateTime::now(), prev_id))?;

        Ok(())
    }

    pub fn last_row_id(&self, table: &str) -> Result<i32> {
        let query = format!(
            "SELECT id from {} WHERE id = (SELECT MAX(id) FROM {})",
            table, table
        );
        let conn = self.conn();
        let mut stmt = conn.prepare(&query)?;
        let recent_id: i32 = stmt.query_row((), |r| r.get(0))?;
        Ok(recent_id)
    }
}
