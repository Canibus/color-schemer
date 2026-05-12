# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-05-12

### ✨ Added
- **New GUI:** Complete graphical interface built with Svelte 5 and Tauri, featuring a modern "NVIDIA Neon" aesthetic.
- **NVIDIA NVAPI Core:** Robust backend implementation for direct GPU control.
- **Profile Management:** Create, edit, and delete display profiles with support for:
  - Brightness, Contrast, and Gamma.
  - NVIDIA Digital Vibrance.
  - Per-display target selection.
- **Auto-Switching Engine:** Link profiles to specific executables (e.g., `Cyberpunk2077.exe`) to activate them automatically on focus change.
- **Global Hotkeys:** Customizable system-wide shortcuts for Next/Prev/Reset actions.
- **System Tray:** Background operation with a context menu for quick actions.
- **Hardware Info:** Real-time feedback on detected GPU and NVAPI status in Settings.
- **Maintenance Tools:** "Reset to Factory Defaults" to easily restore original settings.
- **Internationalization:** Full support for English and Russian languages.
- **Single Instance:** Prevents multiple copies of the app from running simultaneously.
- **Auto-Start:** Option to launch with Windows (managed via registry).

### 🛠 Changed
- Refactored core logic into a shared library for better testability.
- Optimized NVAPI handle caching for faster switching.
- Improved window dragging and custom title bar controls.

### 🐛 Fixed
- Fixed an issue where hotkeys would trigger twice due to message pump timing.
- Resolved a race condition in the auto-switch focus hook.
- Handled non-NVIDIA systems gracefully with a mock fallback and clear UI notification.

---
*Note: This is the initial public release of Color Schemer.*
