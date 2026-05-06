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
pub struct AppConfig {
    pub hotkeys: HotkeyConfig,
    pub show_notifications: bool,
    pub start_minimized: bool,
    pub profiles: Vec<DisplayProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hotkeys: HotkeyConfig::default(),
            show_notifications: true,
            start_minimized: false,
            profiles: vec![
                DisplayProfile::new(
                    "Default",
                    "Стандартные настройки",
                    DisplaySettings::default(),
                ),
                DisplayProfile::new(
                    "Gaming",
                    "Игровой профиль",
                    DisplaySettings {
                        brightness: 1.1,
                        contrast: 1.15,
                        gamma: 0.95,
                        digital_vibrance: 63,
                    },
                ),
                DisplayProfile::new(
                    "Night",
                    "Ночной режим",
                    DisplaySettings {
                        brightness: 0.7,
                        contrast: 0.9,
                        gamma: 1.2,
                        digital_vibrance: 0,
                    },
                ),
            ],
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        path.push("config.toml");
        path
    }

    /// Загрузить из произвольного пути (для тестирования)
    pub fn load_from(path: &std::path::Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Конфигурация загружена из {:?}", path);
                    config
                }
                Err(e) => {
                    warn!(
                        "Ошибка парсинга: {}. Используются значения по умолчанию.",
                        e
                    );
                    Self::default()
                }
            },
            Err(_) => {
                info!(
                    "Файл {:?} не найден. Используются значения по умолчанию.",
                    path
                );
                Self::default()
            }
        }
    }

    /// Сохранить в произвольный путь (для тестирования)
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
            warn!("Не удалось сохранить конфигурацию: {}", e);
        }
    }
}
