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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Ru,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Ru => "ru",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
        if locale.to_lowercase().starts_with("ru") {
            Language::Ru
        } else {
            Language::En
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub language: Language,
    pub hotkeys: HotkeyConfig,
    pub start_minimized: bool,
    pub auto_start: bool,
    pub profiles: Vec<DisplayProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let language = Language::default();
        let first_profile_name = if language == Language::Ru {
            "Standard"
        } else {
            "Default"
        };

        let profiles = vec![
            DisplayProfile::new(first_profile_name, "Standard settings", DisplaySettings::default()),
            DisplayProfile::new(
                "Gaming",
                "Gaming profile",
                DisplaySettings {
                    brightness: 1.1,
                    contrast: 1.15,
                    gamma: 0.95,
                    digital_vibrance: 63,
                },
            ),
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
        ];

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
    pub fn config_path() -> PathBuf {
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
        if let Some(proj_dirs) = directories::ProjectDirs::from("io.github.canibus", "canibus", "ColorSchemer") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            // Fallback to current directory
            PathBuf::from("config.toml")
        }
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
                    warn!("Parse error: {}. Backing up corrupted config and using defaults.", e);
                    if let Err(err) = fs::rename(path, path.with_extension("toml.bak")) {
                        warn!("Failed to back up corrupted config: {}", err);
                    }
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
    pub fn save_to(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
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
