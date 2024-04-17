use std::{fs, path::PathBuf, str::FromStr};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

// File config structs with all properties optional, as the user might not
// provide them.
#[derive(Deserialize, Serialize)]
struct ColorsConfigFile {
    primary: Option<String>,
    active: Option<String>,
    secondary: Option<String>,
    bg: Option<String>,
    fg: Option<String>,
    border: Option<String>,
    popup_bg: Option<String>,
    popup_border: Option<String>,

    title_bar_bg: Option<String>,
    title_bar_fg: Option<String>,

    status_bar_bg: Option<String>,
    status_bar_fg: Option<String>,
    status_bar_navigation_mode_bg: Option<String>,
    status_bar_navigation_mode_fg: Option<String>,
    status_bar_popup_mode_bg: Option<String>,
    status_bar_popup_mode_fg: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct ConfigFile {
    colors: Option<ColorsConfigFile>,
}

// Config structs with all properties provided.
pub struct ColorsConfig {
    pub primary: Color,
    pub active: Color,
    pub secondary: Color,
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub popup_bg: Color,
    pub popup_border: Color,

    pub title_bar_bg: Color,
    pub title_bar_fg: Color,

    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub status_bar_navigation_mode_bg: Color,
    pub status_bar_navigation_mode_fg: Color,
    pub status_bar_popup_mode_bg: Color,
    pub status_bar_popup_mode_fg: Color,
}

/// The main config struct where all properties are provided.
pub struct Config {
    pub colors: ColorsConfig,
}

fn get_base_config() -> Config {
    Config {
        colors: ColorsConfig {
            primary: get_color("#AF5FFF"),
            active: get_color("#00FFFF"),
            secondary: get_color("#999999"),
            fg: get_color("#FFFFFF"),
            bg: get_color("#000000"),
            border: get_color("#777777"),
            popup_bg: get_color("#000000"),
            popup_border: get_color("#AF5FFF"),

            title_bar_bg: get_color("#AF5FFF"),
            title_bar_fg: get_color("#FFFFFF"),

            status_bar_bg: get_color("#333333"),
            status_bar_fg: get_color("#CCCCCC"),
            status_bar_navigation_mode_bg: get_color("#99ce48"),
            status_bar_navigation_mode_fg: get_color("#000000"),
            status_bar_popup_mode_bg: get_color("#8d91ff"),
            status_bar_popup_mode_fg: get_color("#000000"),
        },
    }
}

/// Read the config file if it exists.
fn read_config_file() -> Option<ConfigFile> {
    let home_dir = match home::home_dir() {
        Some(path) => path,
        None => {
            panic!("failed to find home directory");
        }
    };

    let config_dir_str = format!("{}/.config/pltx", home_dir.to_str().unwrap());
    let config_dir = PathBuf::from(config_dir_str);

    // Create the directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
            panic!(
                "Failed to create directory: {}",
                &config_dir.to_str().unwrap()
            )
        });
    }

    let config_filename = "config.toml";
    let config_contents: Option<String> =
        std::fs::read_to_string(config_dir.join(config_filename)).ok();
    let config_file: Option<ConfigFile> = match config_contents {
        Some(contents) => {
            toml::from_str(&contents).unwrap_or_else(|_| panic!("the config is not valid toml"))
        }
        None => None,
    };

    config_file
}

/// Get the ratatui compatible color from a hex color.
fn get_color(color: &str) -> Color {
    // TODO: Add color validation
    Color::from_str(color).unwrap()
}

/// Call the `get_color()` function if provided (from user config), otherwise
/// return `Err`.
fn get_color_op(color_op: Option<String>) -> Result<Color, ()> {
    match color_op {
        Some(color) => Ok(get_color(&color)),
        None => Err(()),
    }
}

/// Merge the user config with the base config.
fn merge_config(user_config: ConfigFile, base_config: Config) -> Config {
    Config {
        colors: match user_config.colors {
            Some(colors) => ColorsConfig {
                primary: get_color_op(colors.primary).unwrap_or(base_config.colors.primary),
                active: get_color_op(colors.active).unwrap_or(base_config.colors.active),
                secondary: get_color_op(colors.secondary).unwrap_or(base_config.colors.secondary),
                fg: get_color_op(colors.fg).unwrap_or(base_config.colors.fg),
                bg: get_color_op(colors.bg).unwrap_or(base_config.colors.bg),
                border: get_color_op(colors.border).unwrap_or(base_config.colors.border),
                popup_bg: get_color_op(colors.popup_bg).unwrap_or(base_config.colors.popup_bg),
                popup_border: get_color_op(colors.popup_border)
                    .unwrap_or(base_config.colors.popup_border),

                title_bar_bg: get_color_op(colors.title_bar_bg)
                    .unwrap_or(base_config.colors.title_bar_bg),
                title_bar_fg: get_color_op(colors.title_bar_fg)
                    .unwrap_or(base_config.colors.title_bar_fg),

                status_bar_bg: get_color_op(colors.status_bar_bg)
                    .unwrap_or(base_config.colors.status_bar_bg),
                status_bar_fg: get_color_op(colors.status_bar_fg)
                    .unwrap_or(base_config.colors.status_bar_fg),
                status_bar_navigation_mode_bg: get_color_op(colors.status_bar_navigation_mode_bg)
                    .unwrap_or(base_config.colors.status_bar_navigation_mode_bg),
                status_bar_navigation_mode_fg: get_color_op(colors.status_bar_navigation_mode_fg)
                    .unwrap_or(base_config.colors.status_bar_navigation_mode_fg),
                status_bar_popup_mode_bg: get_color_op(colors.status_bar_popup_mode_bg)
                    .unwrap_or(base_config.colors.status_bar_popup_mode_bg),
                status_bar_popup_mode_fg: get_color_op(colors.status_bar_popup_mode_fg)
                    .unwrap_or(base_config.colors.status_bar_popup_mode_fg),
            },
            None => base_config.colors,
        },
    }
}

pub fn get_config() -> Config {
    let config_file = read_config_file();

    let base_config = get_base_config();
    match config_file {
        Some(user_config) => merge_config(user_config, base_config),
        None => base_config,
    }
}
