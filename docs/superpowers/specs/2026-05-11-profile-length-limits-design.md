# Design Spec: Profile Name and Description Length Limits

This document outlines the design for enforcing character limits on profile names and descriptions in the Color Schemer application.

## Goals
- Prevent UI layout issues caused by excessively long profile names or descriptions.
- Ensure data consistency across the frontend and backend.
- Provide immediate user feedback in the GUI.

## Constraints
- **Profile Name:** Maximum 32 characters.
- **Profile Description:** Maximum 32 characters.
- **Encoding:** Must handle multi-byte UTF-8 characters (e.g., Cyrillic, Emojis) safely without splitting them.

## Proposed Changes

### 1. Frontend (Svelte)
In `src/lib/ProfileEditor.svelte`, add the `maxlength` attribute to the relevant input fields.

- **Profile Name Input:**
  ```html
  <input type="text" bind:value={edited.name} maxlength="32" />
  ```
- **Profile Description Input:**
  ```html
  <input type="text" bind:value={edited.description} maxlength="32" />
  ```

### 2. Backend (Rust)
In the Rust backend, we will enforce these limits at the data structure level and during configuration loading.

#### `src/profiles.rs`
- **`DisplayProfile` struct**: No structural changes, but we'll add logic to its methods.
- **`DisplayProfile::new`**: Update to truncate inputs.
  ```rust
  pub fn new(name: &str, description: &str, settings: DisplaySettings) -> Self {
      Self {
          name: name.chars().take(32).collect(),
          description: description.chars().take(32).collect(),
          // ... rest
      }
  }
  ```
- **`DisplayProfile::sanitize(&mut self)`**: Add a method to truncate existing strings.
  ```rust
  pub fn sanitize(&mut self) {
      if self.name.chars().count() > 32 {
          self.name = self.name.chars().take(32).collect();
      }
      if self.description.chars().count() > 32 {
          self.description = self.description.chars().take(32).collect();
      }
  }
  ```

#### `src/config.rs`
- **`AppConfig::load_from`**: Update the loading logic to sanitize all profiles after deserialization.
  ```rust
  // After parsing from TOML
  for profile in &mut config.profiles {
      profile.sanitize();
  }
  ```

## Verification Plan

### Automated Tests
1. **Rust Unit Tests**: 
   - Test `DisplayProfile::new` with strings longer than 32 characters.
   - Test `DisplayProfile::sanitize` with various UTF-8 strings.
   - Test that `AppConfig` sanitizes profiles on load.
2. **Frontend Component Tests** (if applicable):
   - Verify `maxlength` attribute is present on inputs.

### Manual Verification
1. Open the Profile Editor in the GUI.
2. Attempt to type a name longer than 32 characters.
3. Attempt to type a description longer than 32 characters.
4. Manually edit `config.toml` with long strings, restart the app, and verify they are truncated in the UI and then saved back as truncated strings.
