use crate::nvidia::DisplaySettings;
use crate::profiles::DisplayProfile;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Конфигурация горячих клавиш
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Клавиша для переключения на следующий профиль
    pub next_profile: String,
    /// Клавиша для переключения на предыдущий профиль
    pub prev_profile: String,
    /// Клавиша для сброса к дефолту
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

/// Основная конфигурация приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Горячие клавиши
    pub hotkeys: HotkeyConfig,
    /// Показывать уведомления при переключении
    pub show_notifications: bool,
    /// Запускать свёрнутым в трей
    pub start_minimized: bool,
    /// Профили
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
    /// Путь к файлу конфигурации
    fn config_path() -> PathBuf {
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        path.push("config.toml");
        path
    }

    /// Загрузить конфигурацию из файла
    pub fn load() -> Self {
        let path = Self::config_path();
        info!("Загрузка конфигурации из {:?}", path);

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    info!("Конфигурация загружена успешно");
                    config
                }
                Err(e) => {
                    warn!(
                        "Ошибка парсинга конфигурации: {}. Используются значения по умолчанию.",
                        e
                    );
                    let config = Self::default();
                    config.save();
                    config
                }
            },
            Err(_) => {
                info!("Файл конфигурации не найден. Создаётся новый.");
                let config = Self::default();
                config.save();
                config
            }
        }
    }

    /// Сохранить конфигурацию в файл
    pub fn save(&self) {
        let path = Self::config_path();
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    warn!("Не удалось сохранить конфигурацию: {}", e);
                } else {
                    info!("Конфигурация сохранена в {:?}", path);
                }
            }
            Err(e) => {
                warn!("Не удалось сериализовать конфигурацию: {}", e);
            }
        }
    }
}
