use crate::profiles::DisplayProfile;

/// Manages automatic profile switching based on the active application.
///
/// It tracks the last manually selected profile and the currently active
/// automatically switched profile to avoid redundant switches and respect
/// manual overrides until the next focus change.
pub struct AutoSwitchManager {
    /// The index of the profile selected manually by the user.
    manual_index: usize,
    /// The index of the profile currently active due to an auto-switch.
    /// `None` if the manual profile is active.
    active_auto_index: Option<usize>,
    /// The name of the last processed application to prevent redundant evaluations.
    last_process: String,
}

impl AutoSwitchManager {
    /// Creates a new `AutoSwitchManager` starting with the given profile index.
    pub fn new(initial_index: usize) -> Self {
        Self {
            manual_index: initial_index,
            active_auto_index: None,
            last_process: String::new(),
        }
    }

    /// Updates the manual index and resets any active auto-switch.
    ///
    /// This should be called when the user manually changes the profile.
    pub fn handle_manual_switch(&mut self, index: usize) {
        self.manual_index = index;
        self.active_auto_index = None;
    }

    /// Evaluates if a profile switch is needed based on the current focused process.
    ///
    /// Returns `Some(index)` if a switch to a new profile is required, otherwise `None`.
    pub fn evaluate_focus_change(
        &mut self,
        process_name: &str,
        profiles: &[DisplayProfile],
    ) -> Option<usize> {
        if process_name == self.last_process {
            return None;
        }

        self.last_process = process_name.to_string();

        let matched_index = profiles.iter().position(|p| {
            p.applications
                .iter()
                .any(|app| app.eq_ignore_ascii_case(process_name))
        });

        if matched_index != self.active_auto_index {
            self.active_auto_index = matched_index;
            // If we found a match, switch to it. Otherwise, revert to the manual profile.
            Some(matched_index.unwrap_or(self.manual_index))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvidia::DisplaySettings;

    fn create_mock_profile(name: &str, apps: Vec<&str>) -> DisplayProfile {
        let mut profile = DisplayProfile::new(name, "desc", DisplaySettings::default());
        profile.applications = apps.into_iter().map(String::from).collect();
        profile
    }

    #[test]
    fn test_auto_switch_logic() {
        let profiles = vec![
            create_mock_profile("Default", vec![]),
            create_mock_profile("Game", vec!["game.exe"]),
            create_mock_profile("Browser", vec!["chrome.exe", "firefox.exe"]),
        ];

        let mut manager = AutoSwitchManager::new(0);

        // Focus on game
        assert_eq!(
            manager.evaluate_focus_change("game.exe", &profiles),
            Some(1)
        );
        assert_eq!(manager.active_auto_index, Some(1));

        // Focus on something else -> revert to default (manual)
        assert_eq!(
            manager.evaluate_focus_change("notepad.exe", &profiles),
            Some(0)
        );
        assert_eq!(manager.active_auto_index, None);

        // Focus on browser (case insensitive)
        assert_eq!(
            manager.evaluate_focus_change("CHROME.EXE", &profiles),
            Some(2)
        );
        assert_eq!(manager.active_auto_index, Some(2));

        // Same process again -> no change
        assert_eq!(manager.evaluate_focus_change("chrome.exe", &profiles), None);
    }

    #[test]
    fn test_manual_override() {
        let profiles = vec![
            create_mock_profile("Default", vec![]),
            create_mock_profile("Game", vec!["game.exe"]),
        ];

        let mut manager = AutoSwitchManager::new(0);

        // Switch to Game automatically
        manager.evaluate_focus_change("game.exe", &profiles);

        // User manually switches to Default while in game
        manager.handle_manual_switch(0);
        assert_eq!(manager.active_auto_index, None);

        // Changing focus to something else (e.g. desktop)
        // It's already at manual_index 0, and matched_index is None.
        // active_auto_index was None, and will remain None.
        // matched_index (None) == active_auto_index (None), so it returns None.
        assert_eq!(
            manager.evaluate_focus_change("explorer.exe", &profiles),
            None
        );

        // Changing focus back to game
        assert_eq!(
            manager.evaluate_focus_change("game.exe", &profiles),
            Some(1)
        );
    }
}
