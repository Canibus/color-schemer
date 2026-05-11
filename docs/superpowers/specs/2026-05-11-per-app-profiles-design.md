# Design Spec: Per-Application Profile Support

## Overview
Enable automatic switching of display color profiles based on the currently active (focused) application. Users can associate specific executable names with a profile, and the application will automatically apply those settings when the target app is in focus.

## User Requirements
- **Identification**: Match applications by process name (e.g., `chrome.exe`).
- **Discovery**: Allow picking applications from a list of currently running windows in the GUI.
- **Fallback**: Revert to the previously active "manual" profile when a target application loses focus.
- **Manual Override**: Manual profile changes (via tray/hotkey) are temporary and will be respected until the next window focus change.

## Architecture

### 1. Data Model (`src/profiles.rs`)
Extend `DisplayProfile` to include an `applications` field.
```rust
pub struct DisplayProfile {
    // ... existing fields ...
    pub applications: Vec<String>,
}
```

### 2. Detection Logic (`src/platform.rs`)
Implement Windows-specific helpers to:
- Get the foreground window's process name.
- Enumerate visible windows and their process names for the "Pick from list" feature.

### 3. Event-Based Switching (`src/main.rs`)
- Use `SetWinEventHook` to listen for `EVENT_SYSTEM_FOREGROUND`.
- Implement an `AutoSwitchManager` (or similar state logic) to track:
    - `manual_profile_index`: The last profile chosen by the user.
    - `current_app_name`: To prevent redundant switches if the focus stays within the same app.
- On focus change:
    1. Resolve process name.
    2. Check if a profile matches the process.
    3. Apply the match OR revert to the `manual_profile_index`.

### 4. UI Implementation (Tauri/Svelte)
- **ProfileEditor.svelte**: Add a section for "Target Applications".
- **Pick Modal**: Implement a way to fetch and display running apps via a new Tauri command `get_running_apps`.

## Success Criteria
1. Profile "Gaming" (with `Cyberpunk2077.exe`) is applied instantly when the game is launched or focused.
2. Settings revert to "Default" (or previous manual choice) when Alt-Tabbing back to the Desktop.
3. Manual switch to "Night" while in-game works until the next Alt-Tab or refocus.
4. Users can select `discord.exe` from a list in the GUI without typing it.

## Testing Strategy
- **Unit Tests**: Test the `ProfileManager` logic for finding matches in the application list.
- **Integration Tests**: Mock foreground window changes to verify state transitions (Manual -> Auto -> Manual).
- **Manual Verification**: Run a few standard apps (Notepad, Chrome) and verify profile switching.
