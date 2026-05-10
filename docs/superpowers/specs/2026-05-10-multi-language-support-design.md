# Multi-language Support Design

**Goal:** Implement English and Russian support with one-time system locale detection and user-switchable settings.

## Backend (Rust)

### Locale Detection
- Use the `sys-locale` crate to detect the system language on the first launch (when `config.toml` does not exist).
- Default to `en` if detection fails or the locale is not `ru`.

### Configuration (`AppConfig`)
- Add a `language` field (string) to `AppConfig`.
- Update `AppConfig::default()` to:
  1. Detect system locale.
  2. Set the `language` field accordingly.
  3. Generate default profile names ("Standard", "Gaming", etc.) in the detected language.

### Backend Translations
- Implement a simple translation helper in Rust to handle strings that don't live in the frontend:
  - System Tray menu items ("Quit", "Show UI").
  - Windows Notifications ("Profile Switched").

## Frontend (Svelte 5)

### i18n Logic (`src/lib/i18n.ts`)
- Use a Svelte 5 `$state` rune to track the current `language`.
- Create a `$derived` translation function `t(key)` that looks up strings in a dictionary.
- Support nested keys (e.g., `settings.title`).

### UI Updates
- **`SettingsTab.svelte`:** Add a "Language" dropdown with English and Russian options.
- **Component Refactoring:** Replace hardcoded strings in `App.svelte`, `ProfileList.svelte`, `ProfileEditor.svelte`, and `SettingsTab.svelte` with `t()` calls.

## Data Flow
1. **Startup:** Rust backend detects locale (if first run), loads/creates `config.toml`, and initializes state.
2. **Frontend Init:** Frontend calls `get_config` and updates the `i18n` rune with the `language` from the config.
3. **Switching Language:**
   - User changes dropdown in Settings.
   - Frontend updates `i18n` rune (immediate UI update).
   - Frontend calls `save_config` to persist the change.
   - Rust backend receives `save_config` and can optionally update the System Tray menu dynamically.

## Implementation Steps
1. Add `sys-locale` dependency.
2. Update `AppConfig` and default profile generation in `src/config.rs`.
3. Implement `i18n.ts` in the frontend.
4. Add English and Russian translation dictionaries.
5. Update all UI components to use translations.
6. Add language selector to Settings.
7. Update backend (tray/notifications) to use the selected language.
