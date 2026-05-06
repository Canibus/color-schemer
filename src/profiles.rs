use crate::nvidia::DisplaySettings;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProfile {
    pub name: String,
    pub description: String,
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

pub struct ProfileManager {
    profiles: Vec<DisplayProfile>,
    current_index: AtomicUsize,
}

impl ProfileManager {
    pub fn new(profiles: Vec<DisplayProfile>) -> Self {
        assert!(!profiles.is_empty(), "Нужен хотя бы один профиль");
        Self {
            profiles,
            current_index: AtomicUsize::new(0),
        }
    }

    pub fn current_profile(&self) -> &DisplayProfile {
        let idx = self.current_index.load(Ordering::Relaxed);
        &self.profiles[idx]
    }

    pub fn current_index(&self) -> usize {
        self.current_index.load(Ordering::Relaxed)
    }

    pub fn next_profile(&self) -> &DisplayProfile {
        let current = self.current_index.load(Ordering::Relaxed);
        let next = (current + 1) % self.profiles.len();
        self.current_index.store(next, Ordering::Relaxed);
        info!(
            "Переключение на профиль: {} ({})",
            self.profiles[next].name, self.profiles[next].description
        );
        &self.profiles[next]
    }

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

    pub fn set_profile(&self, index: usize) -> Option<&DisplayProfile> {
        if index < self.profiles.len() {
            self.current_index.store(index, Ordering::Relaxed);
            Some(&self.profiles[index])
        } else {
            None
        }
    }

    pub fn profiles(&self) -> &[DisplayProfile] {
        &self.profiles
    }

    pub fn count(&self) -> usize {
        self.profiles.len()
    }

    pub fn add_profile(&mut self, profile: DisplayProfile) {
        self.profiles.push(profile);
    }

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
