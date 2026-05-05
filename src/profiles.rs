use crate::nvidia::DisplaySettings;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Профиль отображения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProfile {
    /// Имя профиля
    pub name: String,
    /// Описание
    pub description: String,
    /// Настройки изображения
    pub settings: DisplaySettings,
}

impl DisplayProfile {
    pub fn new(name: &str, description: &str, settings: DisplaySettings) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            settings,
        }
    }
}

/// Менеджер профилей — хранит список и текущий активный профиль
pub struct ProfileManager {
    profiles: Vec<DisplayProfile>,
    current_index: AtomicUsize,
}

impl ProfileManager {
    pub fn new(profiles: Vec<DisplayProfile>) -> Self {
        Self {
            profiles,
            current_index: AtomicUsize::new(0),
        }
    }

    /// Создать менеджер с профилями по умолчанию
    pub fn with_defaults() -> Self {
        let profiles = vec![
            DisplayProfile::new(
                "Default",
                "Стандартные настройки",
                DisplaySettings {
                    brightness: 1.0,
                    contrast: 1.0,
                    gamma: 1.0,
                    digital_vibrance: 0,
                },
            ),
            DisplayProfile::new(
                "Gaming",
                "Игровой профиль - повышенная яркость и насыщенность",
                DisplaySettings {
                    brightness: 1.1,
                    contrast: 1.15,
                    gamma: 0.95,
                    digital_vibrance: 63, // ~50% Digital Vibrance
                },
            ),
            DisplayProfile::new(
                "Movie",
                "Кинопрофиль - тёплые тона, мягкий контраст",
                DisplaySettings {
                    brightness: 0.95,
                    contrast: 1.1,
                    gamma: 1.1,
                    digital_vibrance: 20,
                },
            ),
            DisplayProfile::new(
                "Night",
                "Ночной режим - сниженная яркость",
                DisplaySettings {
                    brightness: 0.7,
                    contrast: 0.9,
                    gamma: 1.2,
                    digital_vibrance: 0,
                },
            ),
            DisplayProfile::new(
                "Vibrant",
                "Максимальная насыщенность",
                DisplaySettings {
                    brightness: 1.0,
                    contrast: 1.2,
                    gamma: 0.9,
                    digital_vibrance: 100, // Максимальная вибрация
                },
            ),
        ];

        Self::new(profiles)
    }

    /// Получить текущий профиль
    pub fn current_profile(&self) -> &DisplayProfile {
        let idx = self.current_index.load(Ordering::Relaxed);
        &self.profiles[idx]
    }

    /// Получить индекс текущего профиля
    pub fn current_index(&self) -> usize {
        self.current_index.load(Ordering::Relaxed)
    }

    /// Переключиться на следующий профиль (циклически)
    pub fn next_profile(&self) -> &DisplayProfile {
        let current = self.current_index.load(Ordering::Relaxed);
        let next = (current + 1) % self.profiles.len();
        self.current_index.store(next, Ordering::Relaxed);

        let profile = &self.profiles[next];
        info!(
            "Переключение на профиль: {} ({})",
            profile.name, profile.description
        );
        profile
    }

    /// Переключиться на предыдущий профиль
    pub fn prev_profile(&self) -> &DisplayProfile {
        let current = self.current_index.load(Ordering::Relaxed);
        let prev = if current == 0 {
            self.profiles.len() - 1
        } else {
            current - 1
        };
        self.current_index.store(prev, Ordering::Relaxed);
        &self.profiles[prev]
    }

    /// Переключиться на профиль по индексу
    pub fn set_profile(&self, index: usize) -> Option<&DisplayProfile> {
        if index < self.profiles.len() {
            self.current_index.store(index, Ordering::Relaxed);
            Some(&self.profiles[index])
        } else {
            None
        }
    }

    /// Получить все профили
    pub fn profiles(&self) -> &[DisplayProfile] {
        &self.profiles
    }

    /// Количество профилей
    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    /// Добавить профиль
    pub fn add_profile(&mut self, profile: DisplayProfile) {
        self.profiles.push(profile);
    }

    /// Удалить профиль по индексу
    pub fn remove_profile(&mut self, index: usize) -> Option<DisplayProfile> {
        if index < self.profiles.len() && self.profiles.len() > 1 {
            let current = self.current_index.load(Ordering::Relaxed);
            if current >= self.profiles.len() - 1 {
                self.current_index.store(0, Ordering::Relaxed);
            }
            Some(self.profiles.remove(index))
        } else {
            None
        }
    }
}
