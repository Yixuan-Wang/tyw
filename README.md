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

- [Python](pkg/py/README.md)
- [Telegram](pkg/tg/README.md)

## Build

```bash
# Build for the current platform
cargo build --release

# Build the release targets
rustup target add x86_64-unknown-linux-gnu aarch64-apple-darwin
cargo build --locked --release --target x86_64-unknown-linux-gnu
cargo build --locked --release --target aarch64-apple-darwin
```

GitHub CI will automatically build for `x86_64-unknown-linux-gnu` and `aarch64-apple-darwin`.
