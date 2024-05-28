[![Tests status](https://github.com/pltx/tui/actions/workflows/tests.yaml/badge.svg?branch=main)](https://github.com/pltx/tui/actions)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![GPL 3.0 license](https://img.shields.io/badge/License-GPL_3.0-blue.svg)](/LICENSE)

**pltx** _(Privacy Life Tracker X)_ is a tool for tracking and managing your personal life privately and securely.

![pltx-tui preview](./.github/assets/preview.png)

## Features

- Vim-like keybinds for navigation and editing.
- Fully configurable at `~/.config/pltx/config.toml`.
- ~~Data is encrypted by default with a passphrase (coming soon).~~
- Data is stored locally ~~or on a [server](https://github.com/pltx/server) (coming soon)~~.
- Modules: Project Management + more coming soon.

> [!IMPORTANT]
> This software is designed for use on Linux. However, porting for use on other OSs would not be a difficult task if there is a demand.

## Documentation

- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)

## Installation

Whilst being in early development, pltx-tui can only be manually installed via cargo.

**Linux**

```sh
git clone https://github.com/pltx/tui
cargo install --path .
```

pltx can also be uninstalled with `cargo uninstall pltx-tui`.

## Usage

Run `pltx` to start the application.

<!-- TODO: Add links to docs when docs are available -->

~~Pressing **`?`** will take you to the help pages with all the information on how to use pltx-tui (coming soon).~~

## Configuration

Edit the configuration in `~/.config/pltx/config.toml`.

```toml
# Controls the log level that outputs to `~/.cache/pltx/debug.log`.
# Available options: debug, info, warn, error
log_level = "info"
# This should be set to the name of a profile. There is no default profile by default.
# default_profile = ""

[colors]
# The default color preset. Defined colors will still override the preset colors.
preset = "default"
fg = "#c0caf5"
secondary_fg = "#7f87ac"
tertiary_fg = "#2c344d"
highlight_fg = "#61a4ff"
bg = "#11121D"
primary = "#9556f7"
success = "#85f67a"
warning = "#ff9382"
danger = "#ff4d66"
date_fg = "#9293b8"
time_fg = "#717299"
input_fg = "#c0caf5"
input_bg = "#232b44"
input_focus_fg = "#c0caf5"
input_focus_bg = "#2c344d"
input_cursor_fg = "#000000"
input_cursor_bg = "#7f87ac"
input_cursor_insert_fg = "#000000"
input_cursor_insert_bg = "#c0caf5"
active_fg = "#373f58"
active_bg = "#00FFFF"
border = "#373f58"
border_active = "#4d556e"
border_insert = "#00FFFF"
popup_bg = "#111111"
popup_border = "#A485DD"
keybind_key = "#A485DD"
keybind_fg = "#6698FF"
title_bar_bg = "#373f58"
title_bar_fg = "#CCCCCC"
tab_fg = "#7f87ac"
tab_active_fg = "#c0caf5"
tab_border = "#373f58"
status_bar_bg = "#232b44"
status_bar_fg = "#7f87ac"
status_bar_normal_mode_bg = "#9bff46"
status_bar_normal_mode_fg = "#232b44"
status_bar_insert_mode_bg = "#00ffff"
status_bar_insert_mode_fg = "#232b44"
status_bar_interactive_mode_bg = "#ffff32"
status_bar_interactive_mode_fg = "#232b44"
status_bar_delete_mode_bg = "#ff6069"
status_bar_delete_mode_fg = "#232b44"

[modules.home]
dashboard_title = "Privacy Life Tracker X"
dashboard_message = "Manage your personal life privately and securely."

[modules.project_management]
# The maximum number of lists allowed in a project.
max_lists = 5
# Days before the due date that a card should be considered due soon.
due_soon_days = 3
completed_char = "✅"
overdue_char = "🚫"
due_soon_char = "⏰"
in_progress_char = "🌐"
important_char = "⭐"
default_char = " "

# Create a separate profile. The profiles shown below are included by default. You can override it by changing the values or create new ones entirely.
[[profiles]]
name = "default"
config_file = "config.toml"
db_file = "data.db"
log_file = "debug.log"

[[profiles]]
name = "dev"
config_file = "dev.toml"
db_file = "dev.db"
log_file = "dev.log"
```

## Contributing

All contributions to the project are welcome! Please read the [Contributing Guidelines](./CONTRIBUTING.md) for more details.

## License

This project is licensed under the [GPL-3.0](./LICENSE) license.
