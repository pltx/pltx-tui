//! Configuration should not be more than three levels deep, e.g.,
//! `config.one.two.three`.

use std::str::FromStr;

use color_eyre::Result;
use pltx_utils::dirs;
use ratatui::style::Color;

const COLOR_PRESETS: [&str; 1] = ["default"];

mod config;

include!("generated_config.rs");

pub use config::*;
use serde::{Deserialize, Serialize};

/// The main base/merged config.
#[derive(Clone, Deserialize, Serialize)]
pub struct DefaultConfig {
    pub log_level: &'static str,
    pub default_profile: &'static str,
    pub colors: ColorsConfig<&'static str, &'static str>,
    pub modules: ModulesConfig<&'static str>,
    pub profiles: [ProfileConfig<&'static str>; 2],
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub log_level: String,
    pub default_profile: String,
    pub colors: ColorsConfig,
    pub modules: ModulesConfig,
    pub profiles: Vec<ProfileConfig>,
}

impl From<DefaultConfig> for Config {
    fn from(value: DefaultConfig) -> Self {
        let serialized = serde_json::to_string(&value).unwrap();
        serde_json::from_str(&serialized).unwrap()
    }
}

fn read_config_file(filename: &str) -> Result<Option<ConfigFile>> {
    let config_file = dirs::config_dir().join(filename);
    let config_contents: Option<String> = std::fs::read_to_string(config_file).ok();
    let config_toml: Option<ConfigFile> = match config_contents {
        Some(contents) => toml::from_str(&contents).expect("the config is invalid"),
        None => None,
    };
    Ok(config_toml)
}

/// Get a ratatui compatible color from a hex color.
fn get_color(color: &str) -> Color {
    Color::from_str(color).expect("failed to get color from string")
}

/// Call the `get_color()` function if a color is provided (from user config),
/// otherwise return the base config value.
fn color_op(color_op: Option<String>, base_config_color: Color) -> Color {
    match color_op {
        Some(color) => get_color(&color),
        None => base_config_color,
    }
}

// TODO: Optimisation. There is lots of clones to reduce the level of nesting.
// Try to not nest too deeply to keep the code easier to read and maintain.
/// Merge the user config with the base config.
fn merge_config(user_config: ConfigFile, base_config: Config) -> Config {
    let colors = user_config.colors.map(|a| {
        let b = base_config.colors.clone();
        ColorsConfig {
            preset: a
                .preset
                .filter(|p| COLOR_PRESETS.iter().any(|&cp| cp == p))
                .unwrap_or(b.preset),
            fg: color_op(a.fg, b.fg),
            secondary_fg: color_op(a.secondary_fg, b.secondary_fg),
            tertiary_fg: color_op(a.tertiary_fg, b.tertiary_fg),
            highlight_fg: color_op(a.highlight_fg, b.highlight_fg),
            bg: color_op(a.bg, b.bg),
            primary: color_op(a.primary, b.primary),
            success: color_op(a.success, b.success),
            warning: color_op(a.warning, b.warning),
            danger: color_op(a.danger, b.danger),
            date_fg: color_op(a.date_fg, b.date_fg),
            time_fg: color_op(a.time_fg, b.time_fg),
            input_fg: color_op(a.input_fg, b.input_fg),
            input_bg: color_op(a.input_bg, b.input_bg),
            input_focus_fg: color_op(a.input_focus_fg, b.input_focus_fg),
            input_focus_bg: color_op(a.input_focus_bg, b.input_focus_bg),
            input_cursor_fg: color_op(a.input_cursor_fg, b.input_cursor_fg),
            input_cursor_bg: color_op(a.input_cursor_bg, b.input_cursor_bg),
            input_cursor_insert_fg: color_op(a.input_cursor_insert_fg, b.input_cursor_insert_fg),
            input_cursor_insert_bg: color_op(a.input_cursor_insert_bg, b.input_cursor_insert_bg),
            active_fg: color_op(a.active_fg, b.active_fg),
            active_bg: color_op(a.active_bg, b.active_bg),
            border: color_op(a.border, b.border),
            border_active: color_op(a.border_active, b.border_active),
            border_insert: color_op(a.border_insert, b.border_insert),
            popup_bg: color_op(a.popup_bg, b.popup_bg),
            popup_border: color_op(a.popup_border, b.popup_border),
            keybind_key: color_op(a.keybind_key, b.keybind_key),
            keybind_fg: color_op(a.keybind_fg, b.keybind_fg),
            title_bar_bg: color_op(a.title_bar_bg, b.title_bar_bg),
            title_bar_fg: color_op(a.title_bar_fg, b.title_bar_fg),
            tab_fg: color_op(a.tab_fg, b.tab_fg),
            tab_active_fg: color_op(a.tab_active_fg, b.tab_active_fg),
            tab_border: color_op(a.tab_border, b.tab_border),
            status_bar_bg: color_op(a.status_bar_bg, b.status_bar_bg),
            status_bar_fg: color_op(a.status_bar_fg, b.status_bar_fg),
            status_bar_normal_mode_bg: color_op(
                a.status_bar_normal_mode_bg,
                b.status_bar_normal_mode_bg,
            ),
            status_bar_normal_mode_fg: color_op(
                a.status_bar_normal_mode_fg,
                b.status_bar_normal_mode_fg,
            ),
            status_bar_insert_mode_bg: color_op(
                a.status_bar_insert_mode_bg,
                b.status_bar_insert_mode_bg,
            ),
            status_bar_insert_mode_fg: color_op(
                a.status_bar_insert_mode_fg,
                b.status_bar_insert_mode_fg,
            ),
            status_bar_interactive_mode_bg: color_op(
                a.status_bar_interactive_mode_bg,
                b.status_bar_interactive_mode_bg,
            ),
            status_bar_interactive_mode_fg: color_op(
                a.status_bar_interactive_mode_fg,
                b.status_bar_interactive_mode_fg,
            ),
            status_bar_delete_mode_bg: color_op(
                a.status_bar_delete_mode_bg,
                b.status_bar_delete_mode_bg,
            ),
            status_bar_delete_mode_fg: color_op(
                a.status_bar_delete_mode_fg,
                b.status_bar_delete_mode_fg,
            ),
        }
    });

    let modules = user_config.modules.map(|modules| {
        let bcm = base_config.modules.clone();

        let home = modules.home.map(|a| {
            let b = bcm.home.clone();
            HomeModule {
                dashboard_title: a.dashboard_title.unwrap_or(b.dashboard_title),
                dashboard_message: a.dashboard_message.unwrap_or(b.dashboard_message),
            }
        });

        let project_management = modules.project_management.map(|a| {
            let b = bcm.project_management.clone();
            ProjectManagementModule {
                max_lists: a.max_lists.unwrap_or(b.max_lists),
                due_soon_days: a.due_soon_days.unwrap_or(b.due_soon_days),
                completed_char: a.completed_char.unwrap_or(b.completed_char),
                overdue_char: a.overdue_char.unwrap_or(b.overdue_char),
                due_soon_char: a.due_soon_char.unwrap_or(b.due_soon_char),
                in_progress_char: a.in_progress_char.unwrap_or(b.in_progress_char),
                important_char: a.important_char.unwrap_or(b.important_char),
                default_char: a.default_char.unwrap_or(b.default_char),
            }
        });

        ModulesConfig {
            home: home.unwrap_or(bcm.home),
            project_management: project_management.unwrap_or(bcm.project_management),
        }
    });

    let profiles = user_config.profiles.map(|a| {
        a.iter()
            .map(|profile| ProfileConfig {
                name: profile
                    .name
                    .as_ref()
                    .expect("profile name not provided")
                    .clone(),
                config_file: profile
                    .config_file
                    .as_ref()
                    .expect("profile config file not provided")
                    .clone(),
                db_file: profile
                    .db_file
                    .as_ref()
                    .expect("profile db file not provided")
                    .clone(),
                log_file: profile
                    .log_file
                    .as_ref()
                    .expect("profile log file not provided")
                    .clone(),
            })
            .collect()
    });

    Config {
        log_level: user_config.log_level.unwrap_or(base_config.log_level),
        default_profile: user_config
            .default_profile
            .unwrap_or(base_config.default_profile),
        colors: colors.unwrap_or(base_config.colors),
        modules: modules.unwrap_or(base_config.modules),
        profiles: profiles.unwrap_or(base_config.profiles),
    }
}

/// Read, parse, and marge the configuration.
pub fn init_config(profile: Option<String>) -> Result<(Config, ProfileConfig)> {
    let mut base_config = base_config();
    let default_profile_name = base_config.default_profile;
    let default_profile = base_config
        .profiles
        .as_ref()
        .iter()
        .find(|p| p.name == default_profile_name)
        .expect("failed to get default profile")
        .clone();

    if profile.as_ref().is_some_and(|p| p.as_str() == "dev") {
        base_config.log_level = "debug";
        base_config.modules.home.dashboard_title = "DEVELOPER PROFILE ENABLED";
        base_config.modules.home.dashboard_message = "All data is separate from the main profile.";
    }

    if let Some(profile_name) = profile {
        let profile = base_config
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .unwrap_or_else(|| panic!("no profile \"{}\" in config.toml", profile_name))
            .to_owned();

        let profile_config_file = read_config_file(profile.config_file);
        let profile_config = match profile_config_file? {
            Some(user_config) => merge_config(user_config, base_config.into()),
            None => base_config.into(),
        };
        Ok((profile_config, profile.into()))
    } else {
        Ok((base_config.into(), (default_profile).into()))
    }
}
