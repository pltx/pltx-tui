use std::{fs, path::PathBuf};

use rusqlite::Connection;

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

pub fn database_connection() -> Connection {
    let db_path = get_db_path();
    Connection::open(db_path).unwrap()
}
