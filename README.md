[![Tests status](https://github.com/pltx/tui/actions/workflows/tests.yaml/badge.svg?branch=main)](https://github.com/pltx/tui/actions)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![GPL 3.0 license](https://img.shields.io/badge/License-GPL_3.0-blue.svg)](/LICENSE)

**pltx** _(Privacy Life Tracker X)_ is a tool for tracking and managing your personal life privately and securely.

<!-- TODO: Add a screenshot here -->

## Features

- Vim-like keybinds for navigation and editing.
- Fully configurable at `~/.config/pltx/config.toml`.
- Data is encrypted by default with a passphrase.
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

[colors]
primary = "#AF5FFF"
secondary = "#AAAAAA"
fg = "#FFFFFF"
bg = "#000000"
input_fg = "#FFFFFF"
input_bg = "#333333"
input_focus_fg = "#FFFFFF"
input_focus_bg = "#666666"
input_cursor_fg = "#000000"
input_cursor_bg = "#BBBBBB"
input_cursor_insert_fg = "#000000"
input_cursor_insert_bg = "#FFFFFF"
active_fg = "#000000"
active_bg = "#00FFFF"
border = "#777777"
border_insert = "#00FFFF"
popup_bg = "#111111"
popup_border = "#AF5FFF"
keybind_key = "#AF5FFF"
keybind_fg = "#6698FF"
title_bar_bg = "#AF5FFF"
title_bar_fg = "#FFFFFF"
status_bar_bg = "#333333"
status_bar_fg = "#CCCCCC"
status_bar_navigation_mode_bg = "#99ce48"
status_bar_navigation_mode_fg = "#000000"
status_bar_insert_mode_bg = "#00ffff"
status_bar_insert_mode_fg = "#000000"
status_bar_popup_mode_bg = "#8d91ff"
status_bar_popup_mode_fg = "#000000"
status_bar_popup_insert_mode_bg = "#ff85ff"
status_bar_popup_insert_mode_fg = "#000000"
status_bar_delete_mode_bg = "#ff6069"
status_bar_delete_mode_fg = "#000000"
status_bar_command_mode_bg = "#ffff64"
status_bar_command_mode_fg = "#000000"
status_bar_command_insert_mode_bg = "#ffcb5f"
status_bar_command_insert_mode_fg = "#000000"

[modules.project_management]
# The maximum number of lists allowed in a project.
max_lists = 5
completed_char = "✅"
overdue_char = "🚫"
due_soon_char = "⏰"
in_progress_char = "🌐"
important_char = "⭐"
default_char = " "
```

## Contributing

All contributions to the project are welcome! Please read the [Contributing Guidelines](./CONTRIBUTING.md) for more details.

## License

This project is licensed under the [GPL-3.0](./LICENSE) license.
