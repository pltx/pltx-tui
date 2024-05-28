use std::{fs, path::PathBuf};

/// Get the pltx config directory.
/// ### Linux
/// Value: `$XDG_CONFIG_HOME/.config/pltx`
/// Default: `/home/user/.config/pltx`
/// ### macOS
/// Value: `$HOME/Library/Application Support/pltx`
/// Default: `/Users/User/Library/Application Support/pltx`
/// ### Windows
/// Value: `{FOLDERID_RoamingAppData}/pltx`
/// Default: `C:\Users\User\AppData\Roaming\pltx`
pub fn config_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("failed to get the config directory")
        .join("pltx");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("failed to create the config directory")
    }
    config_dir
}

/// Get the pltx data directory.
/// ### Linux
/// Value: `$XDG_CONFIG_HOME/.local/share/pltx`
/// Default: `/home/user/.local/share/pltx`
/// ### macOS
/// Value: `$HOME/Library/Application Support/pltx`
/// Default: `/Users/User/Library/Application Support/pltx`
/// ### Windows
/// Value: `{FOLDERID_RoamingAppData}/pltx`
/// Default: `C:\Users\User\AppData\Roaming\pltx`
pub fn data_dir() -> PathBuf {
    let data_dir = dirs::data_dir()
        .expect("failed to get the data directory")
        .join("pltx");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("failed to create the data directory")
    }
    data_dir
}

/// Get the pltx cache directory.
/// ### Linux
/// Value: `$XDG_CACHE_HOME/.cache/pltx`
/// Default: `/home/user/.cache/pltx`
/// ### macOS
/// Value: `$HOME/Library/Caches/pltx`
/// Default: `/Users/User/Library/Caches/pltx`
/// ### Windows
/// Value: `{FOLDERID_LocalAppData}/pltx`
/// Default: `C:\Users\User\AppData\Local\pltx`
pub fn cache_dir() -> PathBuf {
    let cache_dir = dirs::cache_dir()
        .expect("failed to get the cache directory")
        .join("pltx");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).expect("failed to create the cache directory")
    }
    cache_dir
}
