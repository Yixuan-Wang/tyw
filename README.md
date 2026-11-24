# `tyw`

[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)


<code><strong>t</strong>yw</code> for **Y**ixuan's **W**orkflow is Yixuan's personal command line helper.

> [!WARNING]
> This is for personal usage.
> Breaking changes may happen.
> Notice that `tyw` is not supporting Windows.
>
> Previously non-POSIX shell such as `fish` is not supported, but now it is experimentally supported.

## Usage

Global flags:

- `--config <PATH>` — Path to config file (defaults to `$XDG_CONFIG_HOME/tyw.toml` or `~/.config/tyw.toml`)
- `-v`, `--verbose` — Increase logging verbosity

### `tyw py`

- `tyw py list` — List Python virtual environments under `py.env.home`
- `tyw py use [NAME] [--shell <SHELL>]` — Print activation command for a named venv or auto-detect from current directory tree
- `tyw py sel [--shell <SHELL>]` — Interactively select a venv via `fzf` and print its activation command

### `tyw tg`

- `tyw tg text [MESSAGE] [-f|--format <FORMAT>]` — Send a text message
- `tyw tg ping [TEXT] [-f|--format <FORMAT>] [-t|--timeout <DURATION>] [--transient[=true|false]]` — Send a ping and wait for a message or reaction
- `tyw tg ask [TEXT] [-f|--format <FORMAT>] [-t|--timeout <DURATION>]` — Send a message and print the text reply
- `tyw tg pick [TEXT] (-o|--option <OPTION>... | -C|--confirm) [-f|--format <FORMAT>] [-t|--timeout <DURATION>]` — Send inline buttons and print the chosen option

## Build

```bash
# Build for the current platform
cargo build --release

# Build the release targets
rustup target add x86_64-unknown-linux-musl aarch64-apple-darwin
cargo build --locked --release --target x86_64-unknown-linux-musl
cargo build --locked --release --target aarch64-apple-darwin
```

GitHub CI will automatically build for `x86_64-unknown-linux-musl` and `aarch64-apple-darwin`.
