use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// The base/merged colors config.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ColorsConfig<S = String, C = Color> {
    // TODO: color presets to be implemented
    pub preset: S,
    pub fg: C,
    pub bg: C,
    pub secondary_fg: C,
    pub tertiary_fg: C,
    pub highlight_fg: C,
    pub primary: C,
    pub success: C,
    pub warning: C,
    pub danger: C,
    pub date_fg: C,
    pub time_fg: C,
    pub input_fg: C,
    pub input_bg: C,
    pub input_focus_fg: C,
    pub input_focus_bg: C,
    pub input_cursor_fg: C,
    pub input_cursor_bg: C,
    pub input_cursor_insert_fg: C,
    pub input_cursor_insert_bg: C,
    pub active_fg: C,
    pub active_bg: C,
    pub border: C,
    pub border_active: C,
    pub border_insert: C,
    pub popup_bg: C,
    pub popup_border: C,
    pub keybind_key: C,
    pub keybind_fg: C,
    pub title_bar_bg: C,
    pub title_bar_fg: C,
    pub tab_fg: C,
    pub tab_active_fg: C,
    pub tab_border: C,
    pub status_bar_bg: C,
    pub status_bar_fg: C,
    pub status_bar_normal_mode_bg: C,
    pub status_bar_normal_mode_fg: C,
    pub status_bar_insert_mode_bg: C,
    pub status_bar_insert_mode_fg: C,
    pub status_bar_delete_mode_bg: C,
    pub status_bar_delete_mode_fg: C,
}

/// The base/merged home module config.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HomeModule<S> {
    pub dashboard_title: S,
    pub dashboard_message: S,
}

/// The base/merged project management config.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectManagementModule<N = i32, C = String> {
    pub max_lists: N,
    pub due_soon_days: N,
    pub completed_char: C,
    pub overdue_char: C,
    pub due_soon_char: C,
    pub in_progress_char: C,
    pub important_char: C,
    pub default_char: C,
}

/// The base/merged modules config.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModulesConfig<T = String> {
    pub home: HomeModule<T>,
    pub project_management: ProjectManagementModule<i32, T>,
}

/// The user modules config.
#[derive(Deserialize, Serialize)]
pub struct ModulesConfigFile {
    pub home: Option<HomeModule<Option<String>>>,
    pub project_management: Option<ProjectManagementModule<Option<i32>, Option<String>>>,
}

/// The base/merged profile config
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProfileConfig<S = String> {
    pub name: S,
    pub config_file: S,
    pub db_file: S,
    pub log_file: S,
}

impl From<ProfileConfig<&'static str>> for ProfileConfig<String> {
    fn from(config: ProfileConfig<&'static str>) -> Self {
        ProfileConfig {
            name: config.name.into(),
            config_file: config.config_file.into(),
            db_file: config.db_file.into(),
            log_file: config.log_file.into(),
        }
    }
}

/// The user config.
#[derive(Deserialize, Serialize)]
pub struct ConfigFile {
    pub log_level: Option<String>,
    pub default_profile: Option<String>,
    pub profiles: Option<Vec<ProfileConfig<Option<String>>>>,
    pub colors: Option<ColorsConfig<Option<String>, Option<String>>>,
    pub modules: Option<ModulesConfigFile>,
}
