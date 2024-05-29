use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

pub fn sql_init() -> io::Result<()> {
    println!("cargo:rerun-if-changed=init.sql");

    let file_path = PathBuf::from("init.sql");
    let file_contents =
        fs::read_to_string(file_path).expect("failed to find init.sql in crate directory");

    let dest_path = PathBuf::from("src/generated_sql.rs");
    let mut dest_file = File::create(dest_path).expect("failed to create generated_sql.rs");

    write!(dest_file, "const SQL: &str = \"{}\";", file_contents)?;

    Ok(())
}
