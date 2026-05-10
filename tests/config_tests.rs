use color_schemer::config::AppConfig;
use color_schemer::nvidia::DisplaySettings;
use color_schemer::profiles::DisplayProfile;
use std::io::Write;

fn temp_config(content: &str) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

// ============================================================
// Defaults
// ============================================================

#[test]
fn test_default_config() {
    let config = AppConfig::default();
    assert!(config.show_notifications);
    assert!(!config.start_minimized);
    assert_eq!(config.profiles.len(), 3);
    assert_eq!(config.hotkeys.next_profile, "Ctrl+Shift+F5");
    // This will fail to compile until 'language' field is added
    assert!(config.language == "en" || config.language == "ru");
}

#[test]
fn test_localized_defaults() {
    let config = AppConfig::default();
    if config.language == "ru" {
        assert_eq!(config.profiles[0].name, "Стандарт");
        assert_eq!(config.profiles[1].name, "Игровой");
        assert_eq!(config.profiles[2].name, "Ночной");
    } else {
        assert_eq!(config.profiles[0].name, "Default");
        assert_eq!(config.profiles[1].name, "Gaming");
        assert_eq!(config.profiles[2].name, "Night");
    }
}

#[test]
fn test_default_has_standard_profile() {
    let config = AppConfig::default();
    // Profile name depends on language now
    if config.language == "ru" {
        assert_eq!(config.profiles[0].name, "Стандарт");
    } else {
        assert_eq!(config.profiles[0].name, "Default");
    }
    assert!(config.profiles[0].settings.is_default());
}

// ============================================================
// Serialization
// ============================================================

#[test]
fn test_toml_roundtrip() {
    let config = AppConfig::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    let restored: AppConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(config.profiles.len(), restored.profiles.len());
    assert_eq!(config.show_notifications, restored.show_notifications);
}

#[test]
fn test_json_roundtrip() {
    let config = AppConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    let restored: AppConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.profiles.len(), restored.profiles.len());
}

// ============================================================
// Loading
// ============================================================

#[test]
fn test_load_valid_config() {
    let content = r#"
language = "en"
show_notifications = false
start_minimized = true

[hotkeys]
next_profile = "Ctrl+F1"
prev_profile = "Ctrl+F2"
reset = "Ctrl+F3"

[[profiles]]
name = "TestProfile"
description = "Test"

[profiles.settings]
brightness = 1.5
contrast = 0.8
gamma = 2.2
digital_vibrance = 42
"#;

    let file = temp_config(content);
    let config = AppConfig::load_from(file.path());

    assert_eq!(config.language, "en");
    assert!(!config.show_notifications);
    assert!(config.start_minimized);
    assert_eq!(config.profiles.len(), 1);
    assert_eq!(config.profiles[0].name, "TestProfile");
    assert_eq!(config.profiles[0].settings.brightness, 1.5);
    assert_eq!(config.profiles[0].settings.digital_vibrance, 42);
    assert_eq!(config.hotkeys.next_profile, "Ctrl+F1");
}

#[test]
fn test_load_invalid_toml_returns_default() {
    let file = temp_config("this is {{{{ not valid toml !@#$");
    let config = AppConfig::load_from(file.path());

    assert_eq!(config.profiles.len(), 3);
    assert!(config.show_notifications);
}

#[test]
fn test_load_nonexistent_returns_default() {
    let config = AppConfig::load_from(std::path::Path::new("/nonexistent/config.toml"));
    assert_eq!(config.profiles.len(), 3);
}

#[test]
fn test_load_empty_file_returns_default() {
    let file = temp_config("");
    let config = AppConfig::load_from(file.path());
    assert_eq!(config.profiles.len(), 3);
}

// ============================================================
// Saving
// ============================================================

#[test]
fn test_save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_config.toml");

    let mut config = AppConfig::default();
    config.show_notifications = false;
    config.profiles.push(DisplayProfile::new(
        "Extra",
        "Extra profile",
        DisplaySettings {
            brightness: 1.8,
            contrast: 0.5,
            gamma: 3.0,
            digital_vibrance: 100,
        },
    ));

    config.save_to(&path).unwrap();

    let loaded = AppConfig::load_from(&path);
    assert_eq!(loaded.language, config.language);
    assert!(!loaded.show_notifications);
    assert_eq!(loaded.profiles.len(), 4);
    assert_eq!(loaded.profiles[3].name, "Extra");
    assert_eq!(loaded.profiles[3].settings.brightness, 1.8);
}

// ============================================================
// Multiple profiles
// ============================================================

#[test]
fn test_multiple_profiles_in_config() {
    let content = r#"
language = "en"
show_notifications = true
start_minimized = false

[hotkeys]
next_profile = "Ctrl+Shift+F5"
prev_profile = "Ctrl+Shift+F6"
reset = "Ctrl+Shift+F7"

[[profiles]]
name = "A"
description = "First"
[profiles.settings]
brightness = 0.5
contrast = 0.5
gamma = 0.5
digital_vibrance = -100

[[profiles]]
name = "B"
description = "Second"
[profiles.settings]
brightness = 1.5
contrast = 1.5
gamma = 1.5
digital_vibrance = 100

[[profiles]]
name = "C"
description = "Third"
[profiles.settings]
brightness = 2.0
contrast = 2.0
gamma = 2.0
digital_vibrance = 200
"#;

    let file = temp_config(content);
    let config = AppConfig::load_from(file.path());

    assert_eq!(config.language, "en");
    assert_eq!(config.profiles.len(), 3);
    assert_eq!(config.profiles[0].name, "A");
    assert_eq!(config.profiles[1].name, "B");
    assert_eq!(config.profiles[2].name, "C");
    assert_eq!(config.profiles[2].settings.brightness, 2.0);
}
