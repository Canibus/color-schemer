# Project: Color Schemer

## Overview
Color Schemer is a Windows-based utility designed to manage and switch between different display color profiles. It specifically targets NVIDIA GPU users, allowing adjustments to brightness, contrast, gamma, and NVIDIA's Digital Vibrance.

## Core Features
- **Profile Management:** Define multiple display profiles in `config.toml`.
- **NVIDIA Integration:** Direct control over display settings via NVAPI.
- **Global Hotkeys:** Switch profiles using customizable keyboard shortcuts (e.g., `Ctrl+Shift+E` for next profile).
- **System Tray:** Access profiles and settings via a tray icon.
- **Notifications:** Windows native notifications on profile change.
- **Tauri GUI (In Progress):** Transitioning towards a Svelte 5 based graphical interface.

## Tech Stack
- **Backend:** Rust (2024 edition)
  - `nvapi` / `libloading`: Interface with NVIDIA drivers.
  - `global-hotkey`: Global system-wide keyboard shortcuts.
  - `tray-icon`: System tray management.
  - `serde` / `toml`: Configuration parsing.
  - `winrt-notification`: Native Windows notifications.
- **Frontend:** Svelte 5, TypeScript, Vite.
- **Desktop Framework:** Tauri (v1.6.0).

## Project Structure
- `src/`: Core logic and standalone tray application.
  - `main.rs`: Standalone tray app entry point and event loop.
  - `lib.rs`: Shared library components.
  - `nvidia.rs`: NVAPI implementation details.
  - `config.rs`: Configuration loading and structure.
  - `profiles.rs`: Profile switching logic.
  - `hotkey.rs`: Hotkey management.
  - `tray.rs`: Tray icon and menu implementation.
  - `platform.rs`: Windows-specific OS integration (notifications, message pump).
- `src-tauri/`: Tauri configuration and GUI entry point.
- `tests/`: Rust unit and integration tests.
- `config.toml`: User-defined profiles and hotkeys.

## Development Workflows
- **Running the Tray App:** `cargo run --feature app`
- **Developing the GUI:** `bun run tauri dev`
- **Building for Release:** `bun run tauri build`
- **Testing:** `cargo test`

## Code Style
**Rust:**

- Run `cargo fmt` and `cargo clippy` before committing
- Handle errors explicitly (avoid unwrap in production)
- Use descriptive names, add doc comments for public APIs

**TypeScript:**
- Svelte 5 components with TypeScript
- Strict TypeScript, avoid `any` types
- Functional components with hooks
- Path aliases: `@/` → `./src/`

**Style:**
- Prefer explicit composition over complex inheritance.
