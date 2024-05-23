use std::{path::PathBuf, str::FromStr};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// The merged colors config.
#[derive(Deserialize, Serialize, Clone)]
pub struct ColorsConfig<T = Color> {
    pub primary: T,
    pub secondary: T,
    pub bg: T,
    pub fg: T,
    pub input_fg: T,
    pub input_bg: T,
    pub input_focus_fg: T,
    pub input_focus_bg: T,
    pub input_cursor_fg: T,
    pub input_cursor_bg: T,
    pub input_cursor_insert_fg: T,
    pub input_cursor_insert_bg: T,
    pub active_fg: T,
    pub active_bg: T,
    pub border: T,
    pub border_insert: T,
    pub popup_bg: T,
    pub popup_border: T,
    pub keybind_key: T,
    pub keybind_fg: T,
    pub title_bar_bg: T,
    pub title_bar_fg: T,
    pub status_bar_bg: T,
    pub status_bar_fg: T,
    pub status_bar_normal_mode_bg: T,
    pub status_bar_normal_mode_fg: T,
    pub status_bar_insert_mode_bg: T,
    pub status_bar_insert_mode_fg: T,
    pub status_bar_interactive_mode_bg: T,
    pub status_bar_interactive_mode_fg: T,
    pub status_bar_delete_mode_bg: T,
    pub status_bar_delete_mode_fg: T,
}

/// The user config.
#[derive(Deserialize, Serialize)]
struct ConfigFile {
    log_level: Option<String>,
    colors: Option<ColorsConfig<Option<String>>>,
    modules: Option<ModulesConfigFile>,
}

/// The merged project management config.
#[derive(Deserialize, Serialize, Clone)]
pub struct ProjectMangementModule<LimitType, CharType> {
    pub max_lists: LimitType,
    pub completed_char: CharType,
    pub overdue_char: CharType,
    pub due_soon_char: CharType,
    pub in_progress_char: CharType,
    pub important_char: CharType,
    pub default_char: CharType,
}

/// The merged modules config.
#[derive(Clone)]
pub struct ModulesConfig {
    pub project_management: ProjectMangementModule<i32, String>,
}

/// The user modules config.
#[derive(Deserialize, Serialize)]
pub struct ModulesConfigFile {
    pub project_management: Option<ProjectMangementModule<Option<i32>, Option<String>>>,
}

/// The main merged config.
#[derive(Clone)]
pub struct Config {
    pub log_level: String,
    pub colors: ColorsConfig,
    pub modules: ModulesConfig,
}

/// Default config values. Overridden if user config values are provided.
fn get_base_config() -> Config {
    // NOTE: Remember to update `README.md` with the default configuration values.
    Config {
        log_level: String::from("info"),
        colors: ColorsConfig {
            primary: get_color("#AF5FFF"),
            secondary: get_color("#AAAAAA"),
            fg: get_color("#FFFFFF"),
            bg: get_color("#000000"),
            input_fg: get_color("#FFFFFF"),
            input_bg: get_color("#333333"),
            input_focus_fg: get_color("#FFFFFF"),
            input_focus_bg: get_color("#666666"),
            input_cursor_fg: get_color("#000000"),
            input_cursor_bg: get_color("#BBBBBB"),
            input_cursor_insert_fg: get_color("#000000"),
            input_cursor_insert_bg: get_color("#FFFFFF"),
            active_fg: get_color("#000000"),
            active_bg: get_color("#00ffff"),
            border: get_color("#777777"),
            border_insert: get_color("#00FFFF"),
            popup_bg: get_color("#111111"),
            popup_border: get_color("#AF5FFF"),
            keybind_key: get_color("#AF5FFF"),
            keybind_fg: get_color("#6698FF"),
            title_bar_bg: get_color("#AF5FFF"),
            title_bar_fg: get_color("#FFFFFF"),
            status_bar_bg: get_color("#333333"),
            status_bar_fg: get_color("#CCCCCC"),
            status_bar_normal_mode_bg: get_color("#9bff46"),
            status_bar_normal_mode_fg: get_color("#000000"),
            status_bar_insert_mode_bg: get_color("#00ffff"),
            status_bar_insert_mode_fg: get_color("#000000"),
            status_bar_interactive_mode_bg: get_color("#ffff32"),
            status_bar_interactive_mode_fg: get_color("#000000"),
            status_bar_delete_mode_bg: get_color("#ff6069"),
            status_bar_delete_mode_fg: get_color("#000000"),
        },
        modules: ModulesConfig {
            project_management: ProjectMangementModule {
                max_lists: 5,
                completed_char: String::from("✅"),
                overdue_char: String::from("🚫"),
                due_soon_char: String::from("⏰"),
                in_progress_char: String::from("🌐"),
                important_char: String::from("⭐"),
                default_char: String::from(" "),
            },
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
    std::fs::create_dir_all(config_dir.clone()).unwrap();

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
/// return the base config value.
fn get_color_op(color_op: Option<String>, base_config_color: Color) -> Color {
    match color_op {
        Some(color) => get_color(&color),
        None => base_config_color,
    }
}

/// Merge the user config with the base config.
fn merge_config(user_config: ConfigFile, base_config: Config) -> Config {
    let bc = &base_config;
    Config {
        log_level: match user_config.log_level {
            Some(log_level) => log_level,
            None => base_config.log_level.clone(),
        },
        colors: match user_config.colors {
            Some(colors) => ColorsConfig {
                primary: get_color_op(colors.primary, bc.colors.primary),
                secondary: get_color_op(colors.secondary, bc.colors.secondary),
                fg: get_color_op(colors.fg, bc.colors.fg),
                bg: get_color_op(colors.bg, bc.colors.bg),
                input_fg: get_color_op(colors.input_fg, bc.colors.input_fg),
                input_bg: get_color_op(colors.input_bg, bc.colors.input_bg),
                input_focus_fg: get_color_op(colors.input_focus_fg, bc.colors.input_focus_fg),
                input_focus_bg: get_color_op(colors.input_focus_bg, bc.colors.input_focus_bg),
                input_cursor_fg: get_color_op(colors.input_cursor_fg, bc.colors.input_cursor_fg),
                input_cursor_bg: get_color_op(colors.input_cursor_bg, bc.colors.input_cursor_bg),
                input_cursor_insert_fg: get_color_op(
                    colors.input_cursor_insert_fg,
                    bc.colors.input_cursor_insert_fg,
                ),
                input_cursor_insert_bg: get_color_op(
                    colors.input_cursor_insert_bg,
                    bc.colors.input_cursor_insert_bg,
                ),
                active_fg: get_color_op(colors.active_fg, bc.colors.active_fg),
                active_bg: get_color_op(colors.active_bg, bc.colors.active_bg),
                border: get_color_op(colors.border, bc.colors.border),
                border_insert: get_color_op(colors.border_insert, bc.colors.border_insert),
                popup_bg: get_color_op(colors.popup_bg, bc.colors.popup_bg),
                popup_border: get_color_op(colors.popup_border, bc.colors.popup_border),
                keybind_key: get_color_op(colors.keybind_key, bc.colors.keybind_key),
                keybind_fg: get_color_op(colors.keybind_fg, bc.colors.keybind_fg),
                title_bar_bg: get_color_op(colors.title_bar_bg, bc.colors.title_bar_bg),
                title_bar_fg: get_color_op(colors.title_bar_fg, bc.colors.title_bar_fg),
                status_bar_bg: get_color_op(colors.status_bar_bg, bc.colors.status_bar_bg),
                status_bar_fg: get_color_op(colors.status_bar_fg, bc.colors.status_bar_fg),
                status_bar_normal_mode_bg: get_color_op(
                    colors.status_bar_normal_mode_bg,
                    bc.colors.status_bar_normal_mode_bg,
                ),
                status_bar_normal_mode_fg: get_color_op(
                    colors.status_bar_normal_mode_fg,
                    bc.colors.status_bar_normal_mode_fg,
                ),
                status_bar_insert_mode_bg: get_color_op(
                    colors.status_bar_insert_mode_bg,
                    bc.colors.status_bar_insert_mode_bg,
                ),
                status_bar_insert_mode_fg: get_color_op(
                    colors.status_bar_insert_mode_fg,
                    bc.colors.status_bar_insert_mode_fg,
                ),
                status_bar_interactive_mode_bg: get_color_op(
                    colors.status_bar_interactive_mode_bg,
                    bc.colors.status_bar_interactive_mode_bg,
                ),
                status_bar_interactive_mode_fg: get_color_op(
                    colors.status_bar_interactive_mode_fg,
                    bc.colors.status_bar_interactive_mode_fg,
                ),
                status_bar_delete_mode_bg: get_color_op(
                    colors.status_bar_delete_mode_bg,
                    bc.colors.status_bar_delete_mode_bg,
                ),
                status_bar_delete_mode_fg: get_color_op(
                    colors.status_bar_delete_mode_fg,
                    bc.colors.status_bar_delete_mode_fg,
                ),
            },
            None => base_config.colors,
        },
        modules: match user_config.modules {
            Some(modules) => ModulesConfig {
                project_management: match modules.project_management {
                    Some(project_management) => ProjectMangementModule {
                        max_lists: project_management
                            .max_lists
                            .unwrap_or(base_config.modules.project_management.max_lists),
                        completed_char: project_management
                            .completed_char
                            .unwrap_or(base_config.modules.project_management.completed_char),
                        overdue_char: project_management
                            .overdue_char
                            .unwrap_or(base_config.modules.project_management.overdue_char),
                        due_soon_char: project_management
                            .due_soon_char
                            .unwrap_or(base_config.modules.project_management.due_soon_char),
                        in_progress_char: project_management
                            .in_progress_char
                            .unwrap_or(base_config.modules.project_management.in_progress_char),
                        important_char: project_management
                            .important_char
                            .unwrap_or(base_config.modules.project_management.important_char),
                        default_char: project_management
                            .default_char
                            .unwrap_or(base_config.modules.project_management.default_char),
                    },
                    None => base_config.modules.project_management,
                },
            },
            None => base_config.modules,
        },
    }
}

/// Read, parse, and marge the configuration.
pub fn get_config() -> Config {
    let config_file = read_config_file();

    let base_config = get_base_config();
    match config_file {
        Some(user_config) => merge_config(user_config, base_config),
        None => base_config,
    }
}
