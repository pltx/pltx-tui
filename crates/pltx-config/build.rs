use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

const DEFAULT_CONFIG_FILE: &str = "../../docs/config.toml";
const CONFIG_DEST: &str = "src/generated_config.rs";

include!("src/config.rs");

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultConfig {
    pub log_level: String,
    pub default_profile: String,
    pub colors: ColorsConfig<String, String>,
    pub modules: ModulesConfig,
    pub profiles: Vec<ProfileConfig<String>>,
}

fn main() -> io::Result<()> {
    let dest_path = PathBuf::from(CONFIG_DEST);
    let mut dest_file = File::create(dest_path).expect("failed to create generated_config.rs");

    let file_path = PathBuf::from(DEFAULT_CONFIG_FILE);
    let file_contents = fs::read_to_string(file_path)?;
    let file_toml: DefaultConfig =
        toml::from_str(&file_contents).expect("failed to parse default config");

    write!(
        dest_file,
        "
pub fn base_config() -> DefaultConfig {{
{:?}
}}
",
        file_toml
    )?;

    Ok(())
}
