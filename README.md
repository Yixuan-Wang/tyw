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
# Build for aarch64-darwin
env GOOS=darwin GOARCH=arm64 go build -o tyw

# Build for amd64-linux
env GOOS=linux GOARCH=amd64 go build -o tyw
# Build for aarch64-linux
env GOOS=linux GOARCH=arm64 go build -o tyw
```
