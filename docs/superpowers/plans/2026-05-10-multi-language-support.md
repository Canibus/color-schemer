# Multi-language Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add English and Russian support with system locale detection and user settings.

**Architecture:** One-time locale detection in Rust, Svelte 5 runes for frontend i18n, and shared `AppConfig` to persist settings.

**Tech Stack:** Rust (`sys-locale`), Svelte 5, Tauri.

---

### Task 1: Backend Dependencies & Config Update

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/config.rs`
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add `sys-locale` dependency**
Run: `cargo add sys-locale`

- [ ] **Step 2: Update `AppConfig` and `AppConfig::default()` in `src/config.rs`**
```rust
// src/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String, // Add this
    pub hotkeys: HotkeyConfig,
    pub show_notifications: bool,
    pub start_minimized: bool,
    pub profiles: Vec<DisplayProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        let lang = if locale.starts_with("ru") { "ru" } else { "en" };

        let profiles = if lang == "ru" {
            vec![
                DisplayProfile::new("Стандарт", "Стандартные настройки", DisplaySettings::default()),
                DisplayProfile::new("Игровой", "Игровой профиль", DisplaySettings { brightness: 1.1, contrast: 1.15, gamma: 0.95, digital_vibrance: 63 }),
                DisplayProfile::new("Ночной", "Ночной режим", DisplaySettings { brightness: 0.7, contrast: 0.9, gamma: 1.2, digital_vibrance: 0 }),
            ]
        } else {
            vec![
                DisplayProfile::new("Default", "Standard settings", DisplaySettings::default()),
                DisplayProfile::new("Gaming", "Gaming profile", DisplaySettings { brightness: 1.1, contrast: 1.15, gamma: 0.95, digital_vibrance: 63 }),
                DisplayProfile::new("Night", "Night mode", DisplaySettings { brightness: 0.7, contrast: 0.9, gamma: 1.2, digital_vibrance: 0 }),
            ]
        };

        Self {
            language: lang.to_string(),
            hotkeys: HotkeyConfig::default(),
            show_notifications: true,
            start_minimized: false,
            profiles,
        }
    }
}
```

- [ ] **Step 3: Update TypeScript types in `src/lib/types.ts`**
```typescript
// src/lib/types.ts

export type AppConfig = {
    language: string; // Add this
    hotkeys: HotkeyConfig;
    show_notifications: boolean;
    start_minimized: boolean;
    profiles: DisplayProfile[];
};
```

- [ ] **Step 4: Verify compilation**
Run: `cargo check`

- [ ] **Step 5: Commit**
```bash
git add Cargo.toml src/config.rs src/lib/types.ts
git commit -m "feat: add language to config and implement locale detection"
```

---

### Task 2: Backend Translation Helper

**Files:**
- Create: `src/i18n.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create `src/i18n.rs`**
```rust
// src/i18n.rs

pub fn t(lang: &str, key: &str) -> &'static str {
    match lang {
        "ru" => match key {
            "tray.quit" => "Выход",
            "tray.show" => "Показать интерфейс",
            "notif.title" => "Профиль изменен",
            "notif.body" => "Активный профиль: {}",
            _ => key,
        },
        _ => match key {
            "tray.quit" => "Quit",
            "tray.show" => "Show UI",
            "notif.title" => "Profile Switched",
            "notif.body" => "Active profile: {}",
            _ => key,
        },
    }
}
```

- [ ] **Step 2: Export `i18n` in `src/lib.rs`**
Add `pub mod i18n;` to `src/lib.rs`.

- [ ] **Step 3: Commit**
```bash
git add src/i18n.rs src/lib.rs
git commit -m "feat: add backend translation helper"
```

---

### Task 3: Frontend i18n Infrastructure

**Files:**
- Create: `src/lib/i18n.svelte.ts`

- [ ] **Step 1: Create `src/lib/i18n.svelte.ts`**
```typescript
// src/lib/i18n.svelte.ts
import { derived } from 'svelte/store'; // Not using runes for the whole store yet to keep it simple, or use $state
import { AppConfig } from './types';

type Translations = { [key: string]: { [key: string]: string } };

const translations: Translations = {
  en: {
    "nav.profiles": "Profiles",
    "nav.settings": "Settings",
    "profiles.active": "Active",
    "profiles.apply": "Apply",
    "profiles.edit": "Edit",
    "editor.name": "Name",
    "editor.description": "Description",
    "editor.brightness": "Brightness",
    "editor.contrast": "Contrast",
    "editor.gamma": "Gamma",
    "editor.vibrance": "Digital Vibrance",
    "editor.save": "Save",
    "editor.cancel": "Cancel",
    "settings.hotkeys": "Hotkeys",
    "settings.language": "Language",
    "settings.notifications": "Show Notifications",
    "settings.minimized": "Start Minimized",
    "settings.save": "Save Changes",
    "hotkey.next": "Next Profile",
    "hotkey.prev": "Previous Profile",
    "hotkey.reset": "Reset to Default",
    "hotkey.recording": "Recording... Press keys",
  },
  ru: {
    "nav.profiles": "Профили",
    "nav.settings": "Настройки",
    "profiles.active": "Активен",
    "profiles.apply": "Применить",
    "profiles.edit": "Изменить",
    "editor.name": "Название",
    "editor.description": "Описание",
    "editor.brightness": "Яркость",
    "editor.contrast": "Контраст",
    "editor.gamma": "Гамма",
    "editor.vibrance": "Насыщенность (NV)",
    "editor.save": "Сохранить",
    "editor.cancel": "Отмена",
    "settings.hotkeys": "Горячие клавиши",
    "settings.language": "Язык",
    "settings.notifications": "Уведомления",
    "settings.minimized": "Запуск свернутым",
    "settings.save": "Сохранить изменения",
    "hotkey.next": "Следующий профиль",
    "hotkey.prev": "Предыдущий профиль",
    "hotkey.reset": "Сброс",
    "hotkey.recording": "Запись... Нажмите клавиши",
  }
};

class I18nManager {
  currentLang = $state("en");

  setLanguage(lang: string) {
    this.currentLang = lang;
  }

  t(key: string): string {
    return translations[this.currentLang]?.[key] || key;
  }
}

export const i18n = new I18nManager();
```

- [ ] **Step 2: Commit**
```bash
git add src/lib/i18n.svelte.ts
git commit -m "feat: add frontend i18n infrastructure"
```

---

### Task 4: Refactor Components to use i18n

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/lib/ProfileList.svelte`
- Modify: `src/lib/ProfileEditor.svelte`
- Modify: `src/lib/SettingsTab.svelte`

- [ ] **Step 1: Update `src/App.svelte` to initialize language**
Initialize `i18n.setLanguage(config.language)` in the `onMount` or after `get_config`.

- [ ] **Step 2: Replace strings in `src/App.svelte`**
Replace "Profiles" and "Settings" tabs with `{i18n.t('nav.profiles')}` etc.

- [ ] **Step 3: Replace strings in `src/lib/ProfileList.svelte`**
Replace "Active", "Apply", "Edit" etc.

- [ ] **Step 4: Replace strings in `src/lib/ProfileEditor.svelte`**
Replace field labels and buttons.

- [ ] **Step 5: Replace strings and add selector in `src/lib/SettingsTab.svelte`**
Add the `<select>` for language and update existing labels.

- [ ] **Step 6: Commit**
```bash
git add src/App.svelte src/lib/ProfileList.svelte src/lib/ProfileEditor.svelte src/lib/SettingsTab.svelte
git commit -m "feat: refactor UI components to use translations"
```

---

### Task 5: Update Tray and Notifications in Rust

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Update Tray Menu creation**
Use `color_schemer::i18n::t` with the current config language.

- [ ] **Step 2: Update notification logic**
Use `color_schemer::i18n::t` for notification title and body.

- [ ] **Step 3: Verify overall functionality**
Run: `bun run tauri dev`
Test language switching and system tray labels.

- [ ] **Step 4: Commit**
```bash
git add src-tauri/src/main.rs
git commit -m "feat: localized tray and notifications"
```
