# Design: Production Console Hiding

## Overview
Currently, the standalone tray application (`src/main.rs`) opens a console window when executed. For a production-ready application, this console should be suppressed in release builds while remaining available for debugging in development builds.

## Goals
- Suppress the Windows console window in production (`--release`) builds.
- Retain console visibility in development builds for logging.

## Proposed Changes
### Backend (Rust)
Add the `windows_subsystem` attribute to the crate root of the standalone application.

**File:** `src/main.rs`
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

## Data Flow
This is a compile-time attribute. When the Rust compiler (rustc) targets Windows:
1. If `debug_assertions` are NOT enabled (Release mode), the linker flag `/SUBSYSTEM:WINDOWS` is used.
2. If `debug_assertions` ARE enabled (Debug mode), the default `/SUBSYSTEM:CONSOLE` is used.

## Testing Strategy
1. **Verification of Debug Mode:** Run `cargo run`. The console window should still appear.
2. **Verification of Release Mode:** Run `cargo run --release`. The console window should NOT appear (the app should only be visible in the system tray).

## Success Criteria
- Standalone app runs without a visible console window when built in release mode.
- Tauri app behavior remains unchanged (already correctly configured).
