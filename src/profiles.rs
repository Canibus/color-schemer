use crate::nvidia::DisplaySettings;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProfile {
    pub name: String,
    pub description: String,
    pub settings: DisplaySettings,
    #[serde(default)]
    pub target_displays: Vec<String>,
    #[serde(default)]
    pub applications: Vec<String>,
}

impl DisplayProfile {
    pub fn new(name: &str, description: &str, settings: DisplaySettings) -> Self {
        Self {
            name: name.chars().take(32).collect(),
            description: description.chars().take(32).collect(),
            settings,
            target_displays: Vec::new(),
            applications: Vec::new(),
        }
    }

    pub fn sanitize(&mut self) {
        if self.name.chars().count() > 32 {
            self.name = self.name.chars().take(32).collect();
        }
        if self.description.chars().count() > 32 {
            self.description = self.description.chars().take(32).collect();
        }
    }
}

pub struct ProfileManager {
    profiles: Vec<DisplayProfile>,
    current_index: usize,
}

impl ProfileManager {
    pub fn new(profiles: Vec<DisplayProfile>) -> Self {
        assert!(!profiles.is_empty(), "At least one profile is required");
        Self {
            profiles,
            current_index: 0,
        }
    }

    pub fn current_profile(&self) -> &DisplayProfile {
        &self.profiles[self.current_index]
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn next_profile(&mut self) -> &DisplayProfile {
        let next = (self.current_index + 1) % self.profiles.len();
        self.current_index = next;
        info!(
            "Switching to profile: {} ({})",
            self.profiles[next].name, self.profiles[next].description
        );
        &self.profiles[next]
    }

    pub fn prev_profile(&mut self) -> &DisplayProfile {
        let prev = if self.current_index == 0 {
            self.profiles.len() - 1
        } else {
            self.current_index - 1
        };
        self.current_index = prev;
        &self.profiles[prev]
    }

    pub fn set_profile(&mut self, index: usize) -> Option<&DisplayProfile> {
        if index < self.profiles.len() {
            self.current_index = index;
            Some(&self.profiles[index])
        } else {
            None
        }
    }

    pub fn set_profile_by_name(&mut self, name: &str) -> Option<&DisplayProfile> {
        if let Some(index) = self.profiles.iter().position(|p| p.name == name) {
            self.current_index = index;
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
            if self.current_index >= self.profiles.len() - 1 {
                self.current_index = 0;
            }
            Some(self.profiles.remove(index))
        } else {
            None
        }
    }
}
