use crate::nvidia::DisplaySettings;
use crate::profiles::DisplayProfile;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub next_profile: String,
    pub prev_profile: String,
    pub reset: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            next_profile: "Ctrl+Shift+F5".to_string(),
            prev_profile: "Ctrl+Shift+F6".to_string(),
            reset: "Ctrl+Shift+F7".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: String,
    pub hotkeys: HotkeyConfig,
    pub start_minimized: bool,
    pub auto_start: bool,
    pub profiles: Vec<DisplayProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        let language = if locale.to_lowercase().starts_with("ru") {
            "ru".to_string()
        } else {
            "en".to_string()
        };

        let profiles = if language == "ru" {
            vec![
                DisplayProfile::new("Standard", "Standard settings", DisplaySettings::default()),
                {
                    let p = DisplayProfile::new(
                        "Gaming",
                        "Gaming profile",
                        DisplaySettings {
                            brightness: 1.1,
                            contrast: 1.15,
                            gamma: 0.95,
                            digital_vibrance: 63,
                        },
                    );
                    p
                },
                DisplayProfile::new(
                    "Night",
                    "Night mode",
                    DisplaySettings {
                        brightness: 0.7,
                        contrast: 0.9,
                        gamma: 1.2,
                        digital_vibrance: 0,
                    },
                ),
            ]
        } else {
            vec![
                DisplayProfile::new("Default", "Standard settings", DisplaySettings::default()),
                {
                    let p = DisplayProfile::new(
                        "Gaming",
                        "Gaming profile",
                        DisplaySettings {
                            brightness: 1.1,
                            contrast: 1.15,
                            gamma: 0.95,
                            digital_vibrance: 63,
                        },
                    );
                    p
                },
                DisplayProfile::new(
                    "Night",
                    "Night mode",
                    DisplaySettings {
                        brightness: 0.7,
                        contrast: 0.9,
                        gamma: 1.2,
                        digital_vibrance: 0,
                    },
                ),
            ]
        };

        Self {
            language,
            hotkeys: HotkeyConfig::default(),
            start_minimized: false,
            auto_start: true,
            profiles,
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        // 1. Try to find config next to .exe (portable mode)
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let portable_config = exe_dir.join("config.toml");
                if portable_config.exists() {
                    return portable_config;
                }
            }
        }

        // 2. Otherwise use standard AppData folder for installed application
        // On Windows this is usually C:\Users\<Name>\AppData\Local\color-schemer
        let mut path = if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
        } else {
            // Fallback to current directory if environment variable not found
            PathBuf::from(".")
        };

        path.push("color-schemer");

        // Create directory if it doesn't exist
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }

        path.push("config.toml");
        path
    }

    /// Load from arbitrary path (for testing)
    pub fn load_from(path: &std::path::Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(mut config) => {
                    info!("Configuration loaded from {:?}", path);
                    // Sanitize all profiles after loading
                    for profile in &mut config.profiles {
                        profile.sanitize();
                    }
                    config
                }
                Err(e) => {
                    warn!("Parse error: {}. Using default values.", e);
                    Self::default()
                }
            },
            Err(_) => {
                info!("File {:?} not found. Using default values.", path);
                Self::default()
            }
        }
    }

    /// Save to arbitrary path (for testing)
    pub fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(path, content).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    pub fn save(&self) {
        if let Err(e) = self.save_to(&Self::config_path()) {
            warn!("Failed to save configuration: {}", e);
        }
    }
}
