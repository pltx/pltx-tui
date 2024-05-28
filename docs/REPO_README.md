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
- [See all modules here](#modules).

> [!IMPORTANT]
> This software is designed for use on Linux. However, porting for use on other OSs would not be a difficult task if there is a demand.

## Documentation

- [Installation](#installation)
- [Usage](#usage)
- [Modules](#modules)
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

1. Run `pltx` to start the application.
2. You will see the dashboard. Press `}` twice to go two tabs to the right.
3. These are the help pages. Move up with **`j`** and down with **`k`**. Select "navigation" and press **`<enter>`**.
4. Here you'll find all the information you need to navigate pltx.

These pages are generated from the README files in the `/docs` directory, so you can also [view them on GitHub](https://github.com/pltx/tui/blob/main/docs).

Press **`[`** to go back and **`:`** to open the command prompt where you can type **`q`** + **`<enter>`** to quit (quit should auto-complete).

## Modules

- **Home:** Includes the dashboard, settings, and help pages.
- **Project Management:** Manage project or general tasks. Similar to Trello or GitHub projects.
- More coming soon!

## Configuration

Edit the configuration in `~/.config/pltx/config.toml`.

```toml
{default_config}
```

## Contributing

All contributions to the project are welcome! Please read the [Contributing Guidelines](./CONTRIBUTING.md) for more details.

## License

This project is licensed under the [GPL-3.0](./LICENSE) license.

## Similar Projects

- [`kdheepak/taskwarrior-tui`](https://github.com/kdheepak/taskwarrior-tui) (project management)
- [`PlankCipher/kabmat`](https://github.com/PlankCipher/kabmat) (project management)
- [`Zaloog/kanban-python`](https://github.com/Zaloog/kanban-python) (project management)
- [`topydo/topydo`](https://github.com/topydo/topydo) (project management)

Have a similar project? [Open an issue](https://github.com/pltx/tui/issues/new).
