# OCD (OpenCode Desktop)

**Generated:** 2026-05-07  
**Commit:** c3a75d9  
**Branch:** main

## OVERVIEW

Minimal Tauri v2 desktop wrapper around the `opencode serve` web server. Launches the server as a child process, displays the UI in a native window, and injects an in-app settings panel for port/network/auth configuration.

**Core stack:** Rust (Tauri v2 backend) + static HTML/CSS/JS (frontend shell).

## STRUCTURE

```
.
├── src/                    # Frontend shell (static HTML, no bundler)
│   ├── index.html          # Loading spinner shown before server ready
│   └── assets/             # SVG assets
├── src-tauri/              # Rust backend + Tauri configuration
│   ├── src/
│   │   ├── main.rs         # Binary entry point
│   │   ├── lib.rs          # Core logic: server mgmt, commands, JS injection
│   │   └── config.rs       # Config struct with serde load/save
│   ├── capabilities/       # Tauri v2 capability definitions
│   ├── permissions/        # Custom permission tomls (3 commands)
│   ├── icons/              # Platform app icons
│   ├── Cargo.toml          # Rust manifest
│   └── tauri.conf.json     # App metadata, bundle config
├── .github/workflows/      # CI: automated release builds
└── package.json            # npm manifest (Tauri CLI only)
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Change server behavior, commands, or JS injection | `src-tauri/src/lib.rs` | Contains `INJECT_SCRIPT` constant (lines 130–295) |
| Change config fields, defaults, or validation | `src-tauri/src/config.rs` | `Config` struct; defaults: port=4096, expose_network=false |
| Change loading screen | `src/index.html` | Static HTML/CSS spinner |
| Change app metadata, window size, bundle targets | `src-tauri/tauri.conf.json` | Identifier: `org.amethystlab.ocd` |
| Change CI/release behavior | `.github/workflows/release-tauri.yml` | Custom AppImage stripping logic |
| Change Tauri permissions/capabilities | `src-tauri/capabilities/default.json` | Also check `permissions/*.toml` |

## CONVENTIONS

- **No frontend build tool.** `src/` is raw static HTML/CSS/JS. No Vite, Webpack, or bundler.
- **No frontend framework.** The UI is a single `index.html` loading screen. The actual UI comes from the external `opencode serve` process.
- **Dynamic UI injection.** The settings panel is injected into the loaded web page at runtime via `window.eval(INJECT_SCRIPT)` in `lib.rs`.
- **Process wrapper architecture.** The Rust backend spawns and manages `opencode serve` as a child process rather than embedding a local HTTP server.
- **Custom Tauri permissions.** Each command (`get_config`, `save_config`, `restart_server`) has its own `.toml` permission file in `src-tauri/permissions/`.
- **Validation rule.** If `expose_network` is enabled, both `username` and `password` are required (enforced in Rust backend and injected JS).

## ANTI-PATTERNS (THIS PROJECT)

- **Panic on error.** `.expect()` call in `lib.rs` (Tauri build) remains; `config.rs` now falls back gracefully with logged warnings.
- ~~Silent error swallowing~~ **Fixed.** `stop_server` and `window.eval` calls now log warnings on failure.
- ~~Silent config corruption~~ **Fixed.** `config.rs` now logs parse/read errors before falling back to defaults.
- **No tests.** Zero `#[cfg(test)]` modules, `#[test]` functions, or test infrastructure.
- **Never remove the Windows subsystem attribute.** `main.rs:1–2` (`#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`) is critical for Windows release builds.

## UNIQUE STYLES

- **Minimal npm footprint.** Node.js is only used for the Tauri CLI (`@tauri-apps/cli`). The frontend has zero npm dependencies.
- **Single-window, programmatic creation.** `tauri.conf.json` declares `"windows": []`; the window is created in `lib.rs` setup with custom `on_page_load` hook.
- **AppImage stripping.** The Ubuntu CI workflow manually strips bundled Wayland/EGL libraries from the AppImage to avoid compatibility issues.
- **Library naming constraint.** The Cargo `[lib]` name must be `opencode_desktop_lib` (with `_lib` suffix) to avoid Windows linker conflicts (see cargo#8519).

## COMMANDS

```bash
# Development
cd src-tauri && cargo run

# Production build
cd src-tauri && cargo build --release

# Run via Tauri CLI (from root)
npx tauri dev
npx tauri build
```

## NOTES

- Config is stored as JSON in the OS app data directory (`~/.config/OCD/config.json` on Linux).
- The app polls TCP port availability for up to 30s before navigating the webview.
- The webview is restricted to `http://localhost:*` and `http://127.0.0.1:*` URLs via the capability `remote` config.
- VS Code: install recommended extensions (`tauri-apps.tauri-vscode`, `rust-lang.rust-analyzer`).
