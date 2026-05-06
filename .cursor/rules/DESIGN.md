---
alwaysApply: true
---

# Design: `color-schemer`

## Overview

`color-schemer` is a **Windows tray application** for **switching display color profiles** using:

- **Gamma ramp** changes (brightness/contrast/gamma) applied via **WinAPI GDI** (`SetDeviceGammaRamp`).
- **NVIDIA Digital Vibrance** changes applied via **NVAPI** (`nvapi64.dll` / `nvapi.dll`) loaded dynamically.

The app runs primarily as a background utility with:

- A **system tray icon** with a menu to pick profiles / move next/prev / reset / quit.
- **Global hotkeys** to switch profiles without focusing the app.
- Optional **toast notifications** on profile changes.

The codebase is split so the core pieces (config, profile management, GPU controller API, gamma ramp computation) are reusable from the library crate and testable with a mock GPU controller.

## Core responsibilities (high level)

- **Configuration**: load/save `config.toml` next to the executable; provide safe defaults.
- **Profiles**: maintain a list of profiles and a “current profile index”; cycle next/prev; allow direct selection.
- **Display application**:
  - Set **digital vibrance** using NVAPI.
  - Set **gamma ramp** using `SetDeviceGammaRamp`.
  - Clamp/validate settings to safe ranges.
- **UX**: tray icon + menu + hotkeys + (Windows) toast notifications.
- **Main loop**: a lightweight Windows message pump loop that drives hotkey/tray event receivers without a GUI framework event loop.

## Architecture and module map

### Crate layout

- `src/main.rs`: Windows-only binary entrypoint; wires config → profile manager → NVIDIA controller → tray/hotkeys; runs event loop. Imports modules from the library crate (`color_schemer::...`) to avoid duplication.
- `src/lib.rs`: library entrypoint; exports modules for tests and reuse.
- `src/config.rs`: config model + default config + load/save (`TOML` and serde-based formats).
- `src/profiles.rs`: profile model and `ProfileManager` (profile navigation/state).
- `src/nvidia.rs`: GPU abstraction trait + NVIDIA implementation + gamma-ramp computation.
- `src/mock_gpu.rs`: standalone mock implementation of `GpuController` for tests.
- `src/hotkey.rs`: registers global hotkeys from config and maps hotkey IDs to actions.
- `src/tray.rs`: builds tray icon/menu (including “profiles” submenu).
- `src/platform.rs`: platform helpers; contains the Windows message pump, gamma-ramp application, and toast notifications behind `platform::windows::*`.
- `tests/*.rs`: integration-style tests for config/profiles/nvidia math and end-to-end profile switching using the mock GPU.

### Key domain types

- `DisplaySettings` (`src/nvidia.rs`)
  - `brightness: f64`
  - `contrast: f64`
  - `gamma: f64`
  - `digital_vibrance: i32`

- `DisplayProfile` (`src/profiles.rs`)
  - `name: String`
  - `description: String`
  - `settings: DisplaySettings`

- `AppConfig` (`src/config.rs`)
  - `hotkeys: HotkeyConfig` (used to register global hotkeys)
  - `show_notifications: bool`
  - `start_minimized: bool` (controls whether a startup notification is shown)
  - `profiles: Vec<DisplayProfile>`

- `ProfileManager` (`src/profiles.rs`)
  - Holds `Vec<DisplayProfile>`
  - Tracks current index with `AtomicUsize`

- `GpuController` trait (`src/nvidia.rs`)
  - `apply_display_settings(&DisplaySettings)`
  - `set_digital_vibrance(i32)`
  - `get_digital_vibrance() -> NV_DISPLAY_DVC_INFO`
  - `reset()`

The “real” implementation is `NvidiaController`, while tests use `MockGpuController` (`src/mock_gpu.rs`).

## Runtime flow (binary)

### Startup

1. **Initialize logging** via `env_logger`.
2. **Load config**:
   - `AppConfig::load()` reads `config.toml` located next to the executable (`current_exe()/config.toml`).
   - If file missing or invalid TOML, the app uses `AppConfig::default()` (with three built-in profiles).
3. **Initialize NVIDIA controller**:
   - `NvidiaController::new()` dynamically loads `nvapi64.dll` (x86_64) or `nvapi.dll` (x86).
   - Uses `nvapi_QueryInterface` to resolve the NVAPI function pointers by ID.
   - Calls `NvAPI_Initialize` and finds the first display handle (`EnumNvidiaDisplayHandle` with index 0).
   - If this fails, the app logs and shows a Windows toast notification, then exits.
4. **Create profile manager**: `ProfileManager::new(config.profiles)`.
5. **Apply initial profile**: reads `current_profile()` (index 0 initially) and applies it to the display.
6. **Register global hotkeys** (main thread):
   - Config-driven hotkeys from `[hotkeys]` (defaults are `Ctrl+Shift+F5` / `Ctrl+Shift+F6` / `Ctrl+Shift+F7`).
7. **Create tray icon + menu** (main thread):
   - Menu includes a “Profiles” submenu with entries `profile_0`, `profile_1`, etc.
   - Also includes Next/Prev/Reset/Exit.

### Main event loop

The app uses a single tight loop rather than an external GUI event loop:

1. **Pump Windows messages** (non-blocking) using `PeekMessageW/TranslateMessage/DispatchMessageW`.
   - This is required for tray/hotkey plumbing to function reliably.
2. **Handle hotkey events** from `GlobalHotKeyEvent::receiver().try_recv()`.
   - Includes a small debounce/cooldown (300ms).
   - Maps hotkey ID → `HotkeyAction` and triggers the corresponding profile switch.
3. **Handle tray menu events** from `MenuEvent::receiver().try_recv()`.
   - Handles `quit`, `next_profile`, `prev_profile`, `reset`, and `profile_{index}`.
4. **Sleep 10ms** to reduce CPU usage.

### Shutdown behavior

On “quit” from the tray menu:

- The app attempts to **reset gamma ramp** to default.
- The app sets **digital vibrance to 0**.
- Then exits the loop and terminates.

`NvidiaController` also calls `NvAPI_Unload` from `Drop` if initialization succeeded.

## How settings are applied

### Validation / clamping

Before applying, `DisplaySettings::validated()` clamps values:

- `brightness`: \(0.0..=2.0\)
- `contrast`: \(0.0..=2.0\)
- `gamma`: \(0.1..=5.0\)
- `digital_vibrance`: \(-1024..=1023\)

This prevents extreme inputs from producing invalid gamma tables or out-of-range NVAPI calls.

### Gamma ramp

Gamma ramp is computed by `compute_gamma_ramp(settings) -> [[u16; 256]; 3]`.

For each input intensity \(x \in [0,1]\):

1. Gamma correction: \(x' = x^{\gamma}\)
2. Contrast around 0.5: \(c = (x' - 0.5)\cdot contrast + 0.5\)
3. Brightness scale: \(b = c \cdot brightness\)
4. Clamp to \(0..=1\), then map to `u16` \(0..=65535\)

All three channels (R/G/B) are set to the same values (this tool applies a neutral ramp, not per-channel grading).

The ramp is applied via `platform::windows::set_device_gamma_ramp()` (internally uses `GetDC(NULL)` → `SetDeviceGammaRamp` → `ReleaseDC(NULL, hdc)`).

### Digital vibrance (NVAPI)

Digital vibrance is set via NVAPI calls resolved at runtime:

- `NvAPI_SetDVCLevel(display_handle, outputId=0, level)`

The controller also implements `get_digital_vibrance()` via:

- `NvAPI_GetDVCInfo(display_handle, outputId=0, &mut NV_DISPLAY_DVC_INFO)`

Notes:

- The implementation currently targets **the first enumerated display handle** (index 0).
- Output/display selection is simplified (uses `0` for the output ID argument).

## Configuration format (`config.toml`)

The app looks for `config.toml` **next to the executable**.

Fields:

- `show_notifications: bool`
- `start_minimized: bool`
- `[hotkeys]` (strings like `"Ctrl+F1"`) — used to register hotkeys at startup. Supported: `Ctrl`/`Shift`/`Alt`/`Win` (aka `Meta`/`Super`) plus one key (`F1..F24`, `A..Z`, `0..9`).
- `[[profiles]]` list entries:
  - `name`, `description`
  - `[profiles.settings]`: `brightness`, `contrast`, `gamma`, `digital_vibrance`

If config is missing/invalid, defaults include three profiles:

- `Default` (neutral)
- `Gaming` (brighter, higher contrast, slight gamma adjustment, vibrance 63)
- `Night` (dimmer, lower contrast, higher gamma, vibrance 0)

## Testing strategy and guarantees

Tests are integration-style (in `tests/`) and focus on deterministic logic:

- **Config**:
  - Default values are correct.
  - TOML/JSON round-trips via serde work.
  - Loading invalid/nonexistent/empty config returns defaults.
  - Saving then reloading preserves values.

- **Profiles**:
  - `ProfileManager` cycles next/prev with wrap-around.
  - `set_profile` bounds are respected.
  - Add/remove behavior (cannot remove last remaining profile).

- **Display math** (`compute_gamma_ramp`):
  - Default is approximately linear (midpoint near ~32767).
  - Ramp is monotonic.
  - Channels are equal.
  - Brightness/gamma effects match expectations.

- **End-to-end switching**:
  - Uses `MockGpuController` to verify that switching results in the expected settings being applied.
  - Demonstrates that **GPU failure does not roll back profile index** (profile state changes even if apply fails).
  - Verifies rapid switching and basic concurrency behavior with shared `Arc<Mutex<ProfileManager>>`.

## Notable constraints / current limitations

- **Windows-only behavior**: gamma ramp and tray/hotkeys/notifications are Windows-specific in practice.
- **NVIDIA-only vibrance**: digital vibrance uses NVAPI and requires NVIDIA drivers + NVAPI DLL availability.
- **Single display handle**: selects only the first NVIDIA display handle (no multi-monitor selection UI).
- **No persistence of “last selected profile”**: current profile index is in-memory only; startup always applies index 0.

## Extension points

If you want to evolve the project, the most natural extension points are:

- **Multi-monitor selection**: enumerate display handles/outputs and map profiles to specific displays.
- **Profile persistence**: store the last active profile index in `config.toml` (or separate state file) on change.
- **Non-NVIDIA / generic paths**: keep gamma ramp as a generic path, and optionally implement vendor-specific enhancements behind `GpuController`.

