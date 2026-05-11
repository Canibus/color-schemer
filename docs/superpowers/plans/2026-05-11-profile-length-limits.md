# Profile Name and Description Length Limits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce a 32-character limit for profile names and descriptions in both the Rust backend and Svelte frontend to ensure data integrity and UI consistency.

**Architecture:** Hybrid enforcement. The Svelte frontend uses `maxlength` attributes for immediate feedback. The Rust backend truncates strings during creation and loading (sanitization) to handle manual config edits and ensure safety.

**Tech Stack:** Rust (2024 edition), Svelte 5, TypeScript.

---

### Task 1: Backend Implementation (Rust)

**Files:**
- Modify: `src/profiles.rs`
- Test: `tests/profile_tests.rs`

- [ ] **Step 1: Write failing tests for truncation**

Add these tests to `tests/profile_tests.rs`:

```rust
#[test]
fn test_profile_name_truncation() {
    let settings = DisplaySettings::default();
    let long_name = "A".repeat(33);
    let profile = DisplayProfile::new(&long_name, "desc", settings);
    assert_eq!(profile.name.chars().count(), 32);
}

#[test]
fn test_profile_description_truncation() {
    let settings = DisplaySettings::default();
    let long_desc = "B".repeat(33);
    let profile = DisplayProfile::new("name", &long_desc, settings);
    assert_eq!(profile.description.chars().count(), 32);
}

#[test]
fn test_profile_sanitize_utf8() {
    let mut profile = DisplayProfile::new("name", "desc", DisplaySettings::default());
    // 33 Cyrillic characters
    profile.name = "Ф".repeat(33); 
    profile.sanitize();
    assert_eq!(profile.name.chars().count(), 32);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test profile_tests`
Expected: Compilation error (missing `sanitize`) or FAIL (no truncation in `new`).

- [ ] **Step 3: Update `DisplayProfile` implementation**

Modify `src/profiles.rs`:

```rust
impl DisplayProfile {
    pub fn new(name: &str, description: &str, settings: DisplaySettings) -> Self {
        Self {
            name: name.chars().take(32).collect(),
            description: description.chars().take(32).collect(),
            settings,
            target_displays: Vec::new(),
            applications: Vec::new(),
        }
    }

    pub fn sanitize(&mut self) {
        if self.name.chars().count() > 32 {
            self.name = self.name.chars().take(32).collect();
        }
        if self.description.chars().count() > 32 {
            self.description = self.description.chars().take(32).collect();
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test profile_tests`
Expected: PASS

- [ ] **Step 5: Commit backend changes**

```bash
git add src/profiles.rs tests/profile_tests.rs
git commit -m "feat(backend): enforce 32-char limit on profile name and description"
```

---

### Task 2: Config Loading Integration

**Files:**
- Modify: `src/config.rs`
- Test: `tests/config_tests.rs`

- [ ] **Step 1: Write failing test for config sanitization**

Add this test to `tests/config_tests.rs`:

```rust
#[test]
fn test_config_load_sanitizes_profiles() {
    let mut config = AppConfig::default();
    config.profiles[0].name = "A".repeat(40);
    
    // Create a temp file to simulate loading
    let temp_path = std::env::temp_dir().join("test_config_sanitize.toml");
    config.save_to(&temp_path).unwrap();
    
    let loaded_config = AppConfig::load_from(&temp_path);
    assert_eq!(loaded_config.profiles[0].name.chars().count(), 32);
    
    let _ = std::fs::remove_file(temp_path);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_tests`
Expected: FAIL (profile name remains 40 chars)

- [ ] **Step 3: Update `AppConfig::load_from`**

Modify `src/config.rs`:

```rust
pub fn load_from(path: &std::path::Path) -> Self {
    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(mut config) => {
                info!("Конфигурация загружена из {:?}", path);
                // Sanitize all profiles after loading
                for profile in &mut config.profiles {
                    profile.sanitize();
                }
                config
            }
            // ... rest
        }
        // ... rest
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test config_tests`
Expected: PASS

- [ ] **Step 5: Commit config changes**

```bash
git add src/config.rs tests/config_tests.rs
git commit -m "feat(config): sanitize profiles on config load"
```

---

### Task 3: Frontend Enforcement (Svelte)

**Files:**
- Modify: `src/lib/ProfileEditor.svelte`

- [ ] **Step 1: Add maxlength to inputs**

Modify `src/lib/ProfileEditor.svelte` around lines 104-107:

```svelte
      <label>{i18n.t('editor.name')}
          <input type="text" bind:value={edited.name} maxlength="32" />
      </label>
      <label>{i18n.t('editor.description')}
          <input type="text" bind:value={edited.description} maxlength="32" />
      </label>
```

- [ ] **Step 2: Verify visually (if possible) or via build**

Run: `bun run tauri build` (or `bun run check`) to ensure no regressions.

- [ ] **Step 3: Commit frontend changes**

```bash
git add src/lib/ProfileEditor.svelte
git commit -m "feat(frontend): add maxlength limit to profile name and description inputs"
```

---

### Task 4: Final Verification

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 2: Manual Check (Optional but recommended)**

1. Start the app via `bun run tauri dev`.
2. Open Profile Editor.
3. Try to type > 32 chars in Name/Description. Verify it stops at 32.
4. Manually edit `config.toml` to have a long name.
5. Restart app. Verify name is truncated in UI.
