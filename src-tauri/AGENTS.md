# src-tauri

## OVERVIEW

Rust backend and Tauri v2 application configuration for OCD.

## STRUCTURE

```
src-tauri/
├── src/
│   ├── main.rs         # Binary entry point (6 lines)
│   ├── lib.rs          # Core logic: server lifecycle, commands, JS injection (424 lines)
│   └── config.rs       # Config struct with serde persistence (57 lines)
├── capabilities/
│   └── default.json    # Tauri capability for main window
├── permissions/        # Custom permission definitions (3 files)
├── icons/              # Platform app icons
├── Cargo.toml          # Rust manifest
├── tauri.conf.json     # Tauri app config
└── build.rs            # Tauri build script
```

## WHERE TO LOOK

| Task | Location |
|------|----------|
| Server spawn/kill/restart logic | `src/lib.rs` |
| Injected settings UI (JS/CSS) | `src/lib.rs` — `INJECT_SCRIPT` constant |
| Tauri commands (`get_config`, `save_config`, `restart_server`) | `src/lib.rs` |
| Config struct, load/save, defaults | `src/config.rs` |
| App identifier, bundle, window config | `tauri.conf.json` |
| Command permissions | `capabilities/default.json` + `permissions/*.toml` |
| Binary entry point | `src/main.rs` |

## CONVENTIONS

- **Custom permissions per command.** Each Tauri command has its own `.toml` file in `permissions/`.
- **Capability references permissions by identifier.** `capabilities/default.json` lists permission IDs, not inline definitions.
- **Library name suffix.** The `[lib]` name in `Cargo.toml` must include `_lib` suffix to avoid Windows linker conflicts.
- **Empty windows array in config.** The window is created programmatically in `lib.rs` setup, not declared in `tauri.conf.json`.

## ANTI-PATTERNS

- `.expect()` in `lib.rs` (Tauri build) is now logged before panic; `config.rs` falls back gracefully.
- ~~`let _ =` silently discards errors~~ **Fixed.** All process and eval errors now log warnings.
- ~~`Config::load` silently returns defaults~~ **Fixed.** Parse/read errors are now logged.
- No `#[cfg(test)]` modules or `#[test]` functions anywhere.

## NOTES

- `main.rs:1–2`: The `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` attribute prevents a console window on Windows release builds. **Do not remove.**
- `lib.rs` injects the settings panel only when the page URL starts with `http://localhost:` or `http://127.0.0.1:`.
- Server readiness is polled via `TcpStream::connect("127.0.0.1:{port}")` with 500ms interval, 30s timeout.
