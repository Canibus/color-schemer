use color_schemer::mock_gpu::MockGpuController;
use color_schemer::nvidia::GpuController;
use color_schemer::nvidia::{DisplaySettings, compute_gamma_ramp};

// ============================================================
// DisplaySettings - defaults
// ============================================================

#[test]
fn test_display_settings_default() {
    let settings = DisplaySettings::default();
    assert_eq!(settings.brightness, 1.0);
    assert_eq!(settings.contrast, 1.0);
    assert_eq!(settings.gamma, 1.0);
    assert_eq!(settings.digital_vibrance, 0);
}

#[test]
fn test_display_settings_is_default() {
    assert!(DisplaySettings::default().is_default());

    let non_default = DisplaySettings {
        brightness: 1.5,
        ..Default::default()
    };
    assert!(!non_default.is_default());
}

// ============================================================
// DisplaySettings - validation
// ============================================================

#[test]
fn test_validation_clamps_high_values() {
    let extreme = DisplaySettings {
        brightness: 99.0,
        contrast: -5.0,
        gamma: 0.001,
        digital_vibrance: 9999,
    };
    let v = extreme.validated();

    assert_eq!(v.brightness, 2.0);
    assert_eq!(v.contrast, 0.0);
    assert_eq!(v.gamma, 0.1);
    assert_eq!(v.digital_vibrance, 1023);
}

#[test]
fn test_validation_clamps_negative_values() {
    let extreme = DisplaySettings {
        brightness: -1.0,
        contrast: -1.0,
        gamma: -1.0,
        digital_vibrance: -9999,
    };
    let v = extreme.validated();

    assert_eq!(v.brightness, 0.0);
    assert_eq!(v.contrast, 0.0);
    assert_eq!(v.gamma, 0.1);
    assert_eq!(v.digital_vibrance, -1024);
}

#[test]
fn test_validation_preserves_valid_values() {
    let normal = DisplaySettings {
        brightness: 1.5,
        contrast: 0.8,
        gamma: 2.2,
        digital_vibrance: 50,
    };
    let v = normal.validated();
    assert_eq!(v, normal);
}

// ============================================================
// DisplaySettings - serialization
// ============================================================

#[test]
fn test_display_settings_json_roundtrip() {
    let settings = DisplaySettings {
        brightness: 1.5,
        contrast: 0.8,
        gamma: 2.2,
        digital_vibrance: 63,
    };

    let json = serde_json::to_string(&settings).unwrap();
    let restored: DisplaySettings = serde_json::from_str(&json).unwrap();
    assert_eq!(settings, restored);
}

// ============================================================
// Gamma Ramp - basic checks
// ============================================================

#[test]
fn test_gamma_ramp_default_is_linear() {
    let ramp = compute_gamma_ramp(&DisplaySettings::default());

    assert_eq!(ramp[0][0], 0);
    assert_eq!(ramp[1][0], 0);
    assert_eq!(ramp[2][0], 0);
    assert_eq!(ramp[0][255], 65535);
    assert_eq!(ramp[1][255], 65535);
    assert_eq!(ramp[2][255], 65535);

    // Midpoint ≈ 32767
    let mid = ramp[0][128];
    assert!(
        mid >= 32500 && mid <= 33100,
        "Mid value {} not in expected range",
        mid
    );
}

#[test]
fn test_gamma_ramp_monotonically_increasing() {
    let settings = DisplaySettings {
        brightness: 1.2,
        contrast: 1.1,
        gamma: 0.9,
        digital_vibrance: 0,
    };
    let ramp = compute_gamma_ramp(&settings);

    for channel in 0..3 {
        for i in 1..256 {
            assert!(
                ramp[channel][i] >= ramp[channel][i - 1],
                "Channel {} not monotonic at {}: {} < {}",
                channel,
                i,
                ramp[channel][i],
                ramp[channel][i - 1]
            );
        }
    }
}

#[test]
fn test_gamma_ramp_channels_equal() {
    let settings = DisplaySettings {
        brightness: 1.5,
        contrast: 0.8,
        gamma: 1.2,
        digital_vibrance: 0,
    };
    let ramp = compute_gamma_ramp(&settings);

    for i in 0..256 {
        assert_eq!(ramp[0][i], ramp[1][i], "R != G at {}", i);
        assert_eq!(ramp[1][i], ramp[2][i], "G != B at {}", i);
    }
}

// ============================================================
// Gamma Ramp - parameter effects
// ============================================================

#[test]
fn test_gamma_ramp_high_brightness() {
    let ramp = compute_gamma_ramp(&DisplaySettings {
        brightness: 2.0,
        ..Default::default()
    });

    assert_eq!(ramp[0][255], 65535);
    // Midpoint higher than standard
    assert!(
        ramp[0][128] > 33000,
        "High brightness mid {} too low",
        ramp[0][128]
    );
}

#[test]
fn test_gamma_ramp_low_brightness() {
    let ramp = compute_gamma_ramp(&DisplaySettings {
        brightness: 0.5,
        ..Default::default()
    });

    assert!(
        ramp[0][255] < 65535,
        "Low brightness max {} should be < 65535",
        ramp[0][255]
    );
}

#[test]
fn test_gamma_ramp_high_gamma_darkens_midtones() {
    let default_ramp = compute_gamma_ramp(&DisplaySettings::default());
    let high_gamma_ramp = compute_gamma_ramp(&DisplaySettings {
        gamma: 2.2,
        ..Default::default()
    });

    assert!(
        high_gamma_ramp[0][128] < default_ramp[0][128],
        "High gamma should darken midtones"
    );
}

#[test]
fn test_gamma_ramp_low_gamma_brightens_midtones() {
    let default_ramp = compute_gamma_ramp(&DisplaySettings::default());
    let low_gamma_ramp = compute_gamma_ramp(&DisplaySettings {
        gamma: 0.5,
        ..Default::default()
    });

    assert!(
        low_gamma_ramp[0][128] > default_ramp[0][128],
        "Low gamma should brighten midtones"
    );
}

#[test]
fn test_gamma_ramp_values_always_clamped() {
    let extreme = DisplaySettings {
        brightness: 2.0,
        contrast: 2.0,
        gamma: 0.1,
        digital_vibrance: 0,
    };
    let ramp = compute_gamma_ramp(&extreme);

    // Verify the ramp is generated (the function doesn't panic)
    assert_eq!(ramp[0].len(), 256);
}

// ============================================================
// Mock GPU controller
// ============================================================

#[test]
fn test_mock_apply_records_call() {
    let mock = MockGpuController::new();

    let settings = DisplaySettings {
        brightness: 1.5,
        ..Default::default()
    };

    mock.apply_display_settings(None, &settings).unwrap();
    assert_eq!(mock.call_count(), 1);
    assert_eq!(mock.last_settings().unwrap().brightness, 1.5);
}

#[test]
fn test_mock_failure_mode() {
    let mock = MockGpuController::new();
    mock.set_should_fail(true);

    let result = mock.apply_display_settings(None, &DisplaySettings::default());
    assert!(result.is_err());
}

#[test]
fn test_mock_vibrance_tracking() {
    let mock = MockGpuController::new();

    mock.set_digital_vibrance(0, 50).unwrap();
    assert_eq!(mock.get_digital_vibrance(0).unwrap().current_level, 50);

    mock.set_digital_vibrance(0, -100).unwrap();
    assert_eq!(mock.get_digital_vibrance(0).unwrap().current_level, -100);
}

#[test]
fn test_mock_reset() {
    let mock = MockGpuController::new();

    mock.set_digital_vibrance(0, 50).unwrap();
    mock.reset(None).unwrap();

    assert_eq!(mock.get_digital_vibrance(0).unwrap().current_level, 0);
}

#[test]
fn test_mock_multiple_calls_tracked() {
    let mock = MockGpuController::new();

    mock.apply_display_settings(None, &DisplaySettings::default())
        .unwrap();
    mock.set_digital_vibrance(0, 10).unwrap();
    mock.reset(None).unwrap();

    assert_eq!(mock.call_count(), 3);
}
