# OCD (OpenCode Desktop)

A minimal Tauri v2 wrapper around [OpenCode](https://github.com/opencode-ai/opencode)'s web server. Launches `opencode serve`, displays the UI in a native desktop window, and provides an in-app settings panel.

## Features

- Runs `opencode serve` automatically on startup
- Displays the OpenCode web interface in a native window
- Settings panel (bottom-right gear icon) with:
  - Configurable port
  - Local network exposure toggle (`--hostname 0.0.0.0`)
  - Username / password for server auth
  - Save and restart server from the UI

## Requirements (For Build)

- [Rust](https://rust-lang.org)
- [Node.js](https://nodejs.org) (for Tauri CLI)
- `opencode` installed and available in your `PATH`

## Build

```bash
cd src-tauri
cargo build --release
```

Or during development:

```bash
cd src-tauri
cargo run
```

## Configuration

Settings are stored in the OS app data directory (`~/.config/OCD/` on Linux) as `config.json`.

| Setting | Default | Description |
|---------|---------|-------------|
| `port` | 4096 | Port the server listens on |
| `expose_network` | false | Bind to `0.0.0.0` (requires auth) |
| `hostname` | `0.0.0.0` | Hostname to bind to when exposed |
| `username` | `""` | Server auth username |
| `password` | `""` | Server auth password |

When `expose_network` is enabled, both `username` and `password` are required.

## Donations

If you want to support me, it would be appreciated :)

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/W7W61L0JLW)

