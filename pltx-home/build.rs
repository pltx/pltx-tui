use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Document {
    filename: String,
    frontmatter: Frontmatter,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Frontmatter {
    title: String,
    last_updated: String,
}

const IGNORE_FILES: [&str; 2] = ["README", "REPO_README"];
const DOCS_DIR: &str = "../docs";
const DOCS_DEST: &str = "src/generated_docs.rs";

fn main() -> io::Result<()> {
    let dest_path = PathBuf::from(DOCS_DEST);
    let mut dest_file = File::create(dest_path).expect("failed to create generated_docs.rs");

    let start_of_file = "
        use crate::help::{Document,Frontmatter};
        #[allow(private_interfaces)]
        \n"
    .split('\n')
    .map(|s| s.trim_start())
    .collect::<Vec<&str>>()
    .join("\n");

    write!(dest_file, "{}", start_of_file)?;

    writeln!(dest_file, "pub static DOCUMENTS: &[Document] = &[")?;

    for file in fs::read_dir(DOCS_DIR).expect("failed to read the docs directory") {
        let path = file.expect("failed to get the file path").path();

        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap();

            if IGNORE_FILES.contains(&&(*filename.replace(".md", ""))) || !filename.contains(".md")
            {
                continue;
            }

            let contents = fs::read_to_string(&path)?;

            let parts: Vec<&str> = contents.splitn(2, "\n---\n").collect();
            if parts.len() != 2 {
                panic!(
                    "invalid file format for {}, does it include the correct yaml frontmatter?",
                    filename
                );
            }

            let frontmatter: Frontmatter = serde_yaml::from_str(parts[0])
                .unwrap_or_else(|_| panic!("failed to parse frontmatter for {}", filename));

            let document = Document {
                filename: filename.replace(".md", ""),
                frontmatter,
                content: String::from(parts[1]),
            };

            writeln!(dest_file, "{:?},", document)?;
        }
    }
    writeln!(dest_file, "];")?;

    Ok(())
}
