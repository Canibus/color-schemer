use color_schemer::nvidia::DisplaySettings;
use color_schemer::profiles::{DisplayProfile, ProfileManager};

fn make_profiles(count: usize) -> Vec<DisplayProfile> {
    (0..count)
        .map(|i| {
            DisplayProfile::new(
                &format!("Profile {}", i),
                &format!("Description {}", i),
                DisplaySettings {
                    brightness: 1.0 + (i as f64 * 0.1),
                    ..Default::default()
                },
            )
        })
        .collect()
}

// ============================================================
// Создание
// ============================================================

#[test]
fn test_create_manager() {
    let pm = ProfileManager::new(make_profiles(3));
    assert_eq!(pm.count(), 3);
    assert_eq!(pm.current_index(), 0);
}

#[test]
#[should_panic(expected = "Нужен хотя бы один профиль")]
fn test_create_empty_panics() {
    ProfileManager::new(vec![]);
}

#[test]
fn test_single_profile() {
    let pm = ProfileManager::new(make_profiles(1));
    assert_eq!(pm.count(), 1);
    assert_eq!(pm.current_profile().name, "Profile 0");
}

// ============================================================
// Навигация вперёд
// ============================================================

#[test]
fn test_next_cycles_through_all() {
    let mut pm = ProfileManager::new(make_profiles(3));

    assert_eq!(pm.current_index(), 0);

    pm.next_profile();
    assert_eq!(pm.current_index(), 1);

    pm.next_profile();
    assert_eq!(pm.current_index(), 2);

    pm.next_profile();
    assert_eq!(pm.current_index(), 0); // цикл
}

#[test]
fn test_next_returns_correct_profile() {
    let mut pm = ProfileManager::new(make_profiles(3));

    let p = pm.next_profile();
    assert_eq!(p.name, "Profile 1");

    let p = pm.next_profile();
    assert_eq!(p.name, "Profile 2");
}

#[test]
fn test_next_single_profile_stays() {
    let mut pm = ProfileManager::new(make_profiles(1));

    pm.next_profile();
    assert_eq!(pm.current_index(), 0);

    pm.next_profile();
    assert_eq!(pm.current_index(), 0);
}

// ============================================================
// Навигация назад
// ============================================================

#[test]
fn test_prev_wraps_to_last() {
    let mut pm = ProfileManager::new(make_profiles(3));

    let p = pm.prev_profile();
    assert_eq!(p.name, "Profile 2");
    assert_eq!(pm.current_index(), 2);
}

#[test]
fn test_prev_cycles_backward() {
    let mut pm = ProfileManager::new(make_profiles(3));

    pm.prev_profile(); // 0 → 2
    pm.prev_profile(); // 2 → 1
    pm.prev_profile(); // 1 → 0

    assert_eq!(pm.current_index(), 0);
}

// ============================================================
// Прямой выбор
// ============================================================

#[test]
fn test_set_profile_valid() {
    let mut pm = ProfileManager::new(make_profiles(3));

    let result = pm.set_profile(2);
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "Profile 2");
    assert_eq!(pm.current_index(), 2);
}

#[test]
fn test_set_profile_invalid() {
    let mut pm = ProfileManager::new(make_profiles(3));

    assert!(pm.set_profile(99).is_none());
    assert_eq!(pm.current_index(), 0); // не изменился
}

#[test]
fn test_set_profile_boundary() {
    let mut pm = ProfileManager::new(make_profiles(3));

    assert!(pm.set_profile(0).is_some());
    assert!(pm.set_profile(2).is_some());
    assert!(pm.set_profile(3).is_none());
}

#[test]
fn test_set_profile_by_name() {
    let mut pm = ProfileManager::new(make_profiles(3));

    assert!(pm.set_profile_by_name("Profile 1").is_some());
    assert_eq!(pm.current_index(), 1);

    assert!(pm.set_profile_by_name("Profile 2").is_some());
    assert_eq!(pm.current_index(), 2);

    assert!(pm.set_profile_by_name("Non-existent").is_none());
    assert_eq!(pm.current_index(), 2); // should not change
}

// ============================================================
// Добавление / удаление
// ============================================================

#[test]
fn test_add_profile() {
    let mut pm = ProfileManager::new(make_profiles(2));

    pm.add_profile(DisplayProfile::new(
        "New",
        "New",
        DisplaySettings::default(),
    ));
    assert_eq!(pm.count(), 3);
    assert_eq!(pm.profiles()[2].name, "New");
}

#[test]
fn test_remove_profile() {
    let mut pm = ProfileManager::new(make_profiles(3));

    let removed = pm.remove_profile(1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "Profile 1");
    assert_eq!(pm.count(), 2);
}

#[test]
fn test_remove_last_remaining_fails() {
    let mut pm = ProfileManager::new(make_profiles(1));

    assert!(pm.remove_profile(0).is_none());
    assert_eq!(pm.count(), 1);
}

#[test]
fn test_remove_resets_index() {
    let mut pm = ProfileManager::new(make_profiles(3));

    pm.set_profile(2);
    pm.remove_profile(1);
    assert_eq!(pm.current_index(), 0);
}

#[test]
fn test_remove_invalid_index() {
    let mut pm = ProfileManager::new(make_profiles(3));
    assert!(pm.remove_profile(99).is_none());
}

// ============================================================
// Полные циклы
// ============================================================

#[test]
fn test_full_cycle_forward() {
    let mut pm = ProfileManager::new(make_profiles(4));

    let names: Vec<String> = (0..8).map(|_| pm.next_profile().name.clone()).collect();

    assert_eq!(
        names,
        vec![
            "Profile 1",
            "Profile 2",
            "Profile 3",
            "Profile 0",
            "Profile 1",
            "Profile 2",
            "Profile 3",
            "Profile 0",
        ]
    );
}

#[test]
fn test_forward_backward_consistency() {
    let mut pm = ProfileManager::new(make_profiles(5));

    pm.next_profile(); // → 1
    pm.next_profile(); // → 2
    pm.next_profile(); // → 3
    pm.prev_profile(); // → 2
    pm.prev_profile(); // → 1

    assert_eq!(pm.current_index(), 1);
    assert_eq!(pm.current_profile().name, "Profile 1");
}

// ============================================================
// Данные
// ============================================================

#[test]
fn test_profile_settings_preserved() {
    let profiles = vec![DisplayProfile::new(
        "Custom",
        "Custom",
        DisplaySettings {
            brightness: 1.5,
            contrast: 0.8,
            gamma: 2.2,
            digital_vibrance: 63,
        },
    )];

    let pm = ProfileManager::new(profiles);
    let s = &pm.current_profile().settings;

    assert_eq!(s.brightness, 1.5);
    assert_eq!(s.contrast, 0.8);
    assert_eq!(s.gamma, 2.2);
    assert_eq!(s.digital_vibrance, 63);
}

#[test]
fn test_profile_serialization() {
    let profile = DisplayProfile::new(
        "Test",
        "Test",
        DisplaySettings {
            brightness: 1.2,
            ..Default::default()
        },
    );

    let json = serde_json::to_string(&profile).unwrap();
    let restored: DisplayProfile = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.name, "Test");
    assert_eq!(restored.settings.brightness, 1.2);
}

#[test]
fn test_profile_applications() {
    let mut profile = DisplayProfile::new(
        "Gaming",
        "Gaming settings",
        DisplaySettings::default(),
    );
    profile.applications.push("Notepad.exe".to_string());

    assert_eq!(profile.applications.len(), 1);
    assert_eq!(profile.applications[0], "Notepad.exe");

    let json = serde_json::to_string(&profile).unwrap();
    assert!(json.contains("\"applications\":[\"Notepad.exe\"]"));
    
    let restored: DisplayProfile = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.applications.len(), 1);
    assert_eq!(restored.applications[0], "Notepad.exe");
}

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
