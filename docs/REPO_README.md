[![crates.io](https://img.shields.io/crates/v/pltx.svg)](https://crates.io/crates/pltx)
[![Tests status](https://github.com/pltx/pltx-tui/actions/workflows/tests.yaml/badge.svg?branch=main)](https://github.com/pltx/pltx-tui/actions)
[![Latest release](https://img.shields.io/github/v/tag/pltx/pltx-tui?label=Release)](https://github.com/pltx/pltx-tui/releases)
[![Crates.io installs](https://img.shields.io/crates/d/pltx?label=cargo%20installs)](https://crates.io/crates/pltx)
[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
![Apache 2.0 or MIT license](https://img.shields.io/badge/License-Apache_2.0_or_MIT-blue.svg)

**pltx** _(Privacy Life Tracker X)_ is a tool for tracking and managing your life in a private and secure manner.<br>
**pltx-tui** is the default interface that provides this capability in the terminal!

![pltx-tui preview](./.github/assets/preview.png)

## Features

- [See all the modules here](#modules).
- Vim-like keybinds for navigation and editing.
- Fully customizable profiles, colors, limits, and more.
- ~~Data is encrypted by default with a passphrase (coming soon).~~
- Data can be stored locally ~~or on a [server](https://github.com/pltx/server) (coming soon)~~.

## Documentation

- [Installation](#installation)
- [Usage](#usage)
- [Modules](#modules)
- [Configuration](#configuration)

## Installation

Whilst being in early development, pltx-tui can only be manually installed via cargo.

**Linux**

```sh
git clone https://github.com/pltx/pltx-tui
cargo install --path .
```

## Usage

1. Run `pltx` to start the application.
2. You will see the dashboard. Press `}` twice to go two tabs to the right.
3. These are the help pages. Move up with **`j`** and down with **`k`**. Select "navigation" and press **`<enter>`**.
4. Here you'll find all the information you need to navigate pltx.

These pages are generated from the README files in the `/docs` directory, so you can also [view them on GitHub](https://github.com/pltx/pltx-tui/blob/main/docs).

Press **`[`** to go back and **`:`** to open the command prompt where you can type **`q`** + **`<enter>`** to quit (quit should auto-complete). You can also use the help command to go to the help pages from anywhere in the application.

## Modules

- **[Home](./docs/home.md):** Includes the dashboard, settings, and help pages.
- **[Project Management](./docs/project-management.md):** Manage project or general tasks. Similar to Trello or GitHub projects.
- **More coming soon!**

## Configuration

You can edit the config in your platforms config directory:

| Platform | Location                                                                                                                                            |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Linux    | <pre><code>$XDG_CONFIG_HOME/.config/pltx/config.toml</code><br><code>/home/user/.config/pltx/config.toml</code></pre>                               |
| macOS    | <pre><code>$HOME/Library/Application Support/pltx/config.toml</code><br><code>/Users/User/Library/Application Support/pltx/config.toml</code></pre> |
| Windows  | <pre><code>{FOLDERID_RoamingAppData}\pltx\config.toml</code><br><code>C:\Users\User\AppData\Roaming\pltx\config.toml</code></pre>                   |

```toml
{default_config}
```

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Similar Projects

- [kdheepak/taskwarrior-tui](https://github.com/kdheepak/taskwarrior-tui) (project management)
- [PlankCipher/kabmat](https://github.com/PlankCipher/kabmat) (project management)
- [Zaloog/kanban-python](https://github.com/Zaloog/kanban-python) (project management)
- [topydo/topydo](https://github.com/topydo/topydo) (project management)
