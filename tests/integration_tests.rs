use color_schemer::mock_gpu::MockGpuController;
use color_schemer::nvidia::{DisplaySettings, GpuController};
use color_schemer::profiles::{DisplayProfile, ProfileManager};
use std::sync::{Arc, Mutex};

fn test_profiles() -> Vec<DisplayProfile> {
    vec![
        DisplayProfile::new("Default", "Default", DisplaySettings::default()),
        DisplayProfile::new(
            "Gaming",
            "Gaming",
            DisplaySettings {
                brightness: 1.2,
                contrast: 1.1,
                gamma: 0.9,
                digital_vibrance: 63,
            },
        ),
        DisplayProfile::new(
            "Night",
            "Night",
            DisplaySettings {
                brightness: 0.7,
                contrast: 0.9,
                gamma: 1.2,
                digital_vibrance: 0,
            },
        ),
    ]
}

#[test]
fn test_switch_applies_correct_settings() {
    let gpu = MockGpuController::new();
    let mut pm = ProfileManager::new(test_profiles());

    // Начальный профиль
    gpu.apply_display_settings(&pm.current_profile().settings)
        .unwrap();
    assert!(gpu.last_settings().unwrap().is_default());

    // Gaming
    let p = pm.next_profile();
    gpu.apply_display_settings(&p.settings).unwrap();
    assert_eq!(gpu.last_settings().unwrap().brightness, 1.2);
    assert_eq!(gpu.last_settings().unwrap().digital_vibrance, 63);

    // Night
    let p = pm.next_profile();
    gpu.apply_display_settings(&p.settings).unwrap();
    assert_eq!(gpu.last_settings().unwrap().brightness, 0.7);
}

#[test]
fn test_full_cycle_count() {
    let gpu = MockGpuController::new();
    let mut pm = ProfileManager::new(test_profiles());

    for _ in 0..pm.count() {
        let p = pm.next_profile();
        gpu.apply_display_settings(&p.settings).unwrap();
    }

    assert_eq!(gpu.call_count(), 3);
}

#[test]
fn test_reset_to_default() {
    let gpu = MockGpuController::new();
    let mut pm = ProfileManager::new(test_profiles());

    // Переключаемся на Gaming
    pm.next_profile();
    gpu.apply_display_settings(&pm.current_profile().settings)
        .unwrap();
    assert_eq!(gpu.last_settings().unwrap().brightness, 1.2);

    // Сброс
    let p = pm.set_profile(0).unwrap();
    gpu.apply_display_settings(&p.settings).unwrap();
    assert!(gpu.last_settings().unwrap().is_default());
}

#[test]
fn test_gpu_failure_does_not_affect_profile_state() {
    let gpu = MockGpuController::new();
    let mut pm = ProfileManager::new(test_profiles());

    gpu.set_should_fail(true);

    let p = pm.next_profile();
    let result = gpu.apply_display_settings(&p.settings);

    assert!(result.is_err());
    // ProfileManager всё равно переключился
    assert_eq!(pm.current_index(), 1);
}

#[test]
fn test_rapid_switching() {
    let gpu = MockGpuController::new();
    let mut pm = ProfileManager::new(test_profiles());

    for _ in 0..100 {
        let p = pm.next_profile();
        gpu.apply_display_settings(&p.settings).unwrap();
    }

    assert_eq!(gpu.call_count(), 100);
    // 100 % 3 = 1
    assert_eq!(pm.current_index(), 1);
}

#[test]
fn test_concurrent_access() {
    let gpu = Arc::new(MockGpuController::new());
    let pm = Arc::new(Mutex::new(ProfileManager::new(test_profiles())));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let gpu = gpu.clone();
            let pm = pm.clone();
            std::thread::spawn(move || {
                for _ in 0..10 {
                    let settings = {
                        let mut pm = pm.lock().unwrap();
                        let p = pm.next_profile();
                        p.settings.clone()
                    };
                    gpu.apply_display_settings(&settings).unwrap();
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(gpu.call_count(), 100);
}
