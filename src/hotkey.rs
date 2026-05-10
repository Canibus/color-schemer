use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use log::info;

use crate::config::HotkeyConfig;

/// Идентификаторы действий
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    NextProfile,
    PrevProfile,
    Reset,
}

/// Менеджер горячих клавиш
pub struct HotkeyController {
    _manager: GlobalHotKeyManager,
    next_id: u32,
    prev_id: u32,
    reset_id: u32,
}

impl HotkeyController {
    pub fn new(config: &HotkeyConfig) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Не удалось создать менеджер горячих клавиш: {}", e))?;

        let mut registered_ids = std::collections::HashSet::new();

        let mut register = |label: &str, input: &str| -> Result<Option<u32>, String> {
            if input.trim().is_empty() {
                return Ok(None);
            }
            let hk = parse_hotkey(input).map_err(|e| format!("{}: {}", label, e))?;
            let id = hk.id();
            if registered_ids.contains(&id) {
                info!("Горячая клавиша '{}' уже зарегистрирована, пропускаем дубликат для '{}'", input, label);
                return Ok(Some(id));
            }
            manager.register(hk).map_err(|e| {
                format!("Не удалось зарегистрировать '{}' для '{}': {}", input, label, e)
            })?;
            registered_ids.insert(id);
            Ok(Some(id))
        };

        let next_id = register("next_profile", &config.next_profile)?.unwrap_or(0);
        let prev_id = register("prev_profile", &config.prev_profile)?.unwrap_or(0);
        let reset_id = register("reset", &config.reset)?.unwrap_or(0);

        info!("Горячие клавиши зарегистрированы:");
        if !config.next_profile.is_empty() {
            info!("  {} — следующий профиль", config.next_profile);
        }
        if !config.prev_profile.is_empty() {
            info!("  {} — предыдущий профиль", config.prev_profile);
        }
        if !config.reset.is_empty() {
            info!("  {} — сброс к стандартным", config.reset);
        }

        Ok(Self {
            _manager: manager,
            next_id,
            prev_id,
            reset_id,
        })
    }

    /// Определить действие по ID горячей клавиши
    pub fn get_action(&self, id: u32) -> Option<HotkeyAction> {
        if id == self.next_id {
            Some(HotkeyAction::NextProfile)
        } else if id == self.prev_id {
            Some(HotkeyAction::PrevProfile)
        } else if id == self.reset_id {
            Some(HotkeyAction::Reset)
        } else {
            None
        }
    }

    /// Получить receiver для событий горячих клавиш
    pub fn receiver() -> &'static crossbeam_channel::Receiver<GlobalHotKeyEvent> {
        GlobalHotKeyEvent::receiver()
    }
}

fn parse_hotkey(input: &str) -> Result<HotKey, String> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err("hotkey string is empty".to_string());
    }

    let mut mods = Modifiers::empty();
    let mut key: Option<Code> = None;

    for part in raw.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let token = part.to_ascii_lowercase();

        match token.as_str() {
            "ctrl" | "control" => {
                mods |= Modifiers::CONTROL;
                continue;
            }
            "shift" => {
                mods |= Modifiers::SHIFT;
                continue;
            }
            "alt" => {
                mods |= Modifiers::ALT;
                continue;
            }
            // On Windows this maps to the Win/Super key.
            "win" | "meta" | "super" => {
                mods |= Modifiers::META;
                continue;
            }
            _ => {}
        }

        if key.is_some() {
            return Err(format!(
                "multiple key codes found in '{}'; expected one key plus optional modifiers",
                input
            ));
        }

        key = Some(parse_code(part).ok_or_else(|| {
            format!(
                "unsupported key '{}'. Supported: F1..F24, A..Z, 0..9 plus modifiers Ctrl/Shift/Alt/Win",
                part
            )
        })?);
    }

    let code = key.ok_or_else(|| format!("missing key code in '{}'", input))?;
    let modifiers = if mods.is_empty() { None } else { Some(mods) };
    Ok(HotKey::new(modifiers, code))
}

fn parse_code(token: &str) -> Option<Code> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }

    let lower = t.to_ascii_lowercase();

    if let Some(rest) = lower.strip_prefix('f') {
        if let Ok(n) = rest.parse::<u8>() {
            return match n {
                1 => Some(Code::F1),
                2 => Some(Code::F2),
                3 => Some(Code::F3),
                4 => Some(Code::F4),
                5 => Some(Code::F5),
                6 => Some(Code::F6),
                7 => Some(Code::F7),
                8 => Some(Code::F8),
                9 => Some(Code::F9),
                10 => Some(Code::F10),
                11 => Some(Code::F11),
                12 => Some(Code::F12),
                13 => Some(Code::F13),
                14 => Some(Code::F14),
                15 => Some(Code::F15),
                16 => Some(Code::F16),
                17 => Some(Code::F17),
                18 => Some(Code::F18),
                19 => Some(Code::F19),
                20 => Some(Code::F20),
                21 => Some(Code::F21),
                22 => Some(Code::F22),
                23 => Some(Code::F23),
                24 => Some(Code::F24),
                _ => None,
            };
        }
    }

    if lower.len() == 1 {
        let c = lower.chars().next()?;
        return match c {
            'a' => Some(Code::KeyA),
            'b' => Some(Code::KeyB),
            'c' => Some(Code::KeyC),
            'd' => Some(Code::KeyD),
            'e' => Some(Code::KeyE),
            'f' => Some(Code::KeyF),
            'g' => Some(Code::KeyG),
            'h' => Some(Code::KeyH),
            'i' => Some(Code::KeyI),
            'j' => Some(Code::KeyJ),
            'k' => Some(Code::KeyK),
            'l' => Some(Code::KeyL),
            'm' => Some(Code::KeyM),
            'n' => Some(Code::KeyN),
            'o' => Some(Code::KeyO),
            'p' => Some(Code::KeyP),
            'q' => Some(Code::KeyQ),
            'r' => Some(Code::KeyR),
            's' => Some(Code::KeyS),
            't' => Some(Code::KeyT),
            'u' => Some(Code::KeyU),
            'v' => Some(Code::KeyV),
            'w' => Some(Code::KeyW),
            'x' => Some(Code::KeyX),
            'y' => Some(Code::KeyY),
            'z' => Some(Code::KeyZ),
            '0' => Some(Code::Digit0),
            '1' => Some(Code::Digit1),
            '2' => Some(Code::Digit2),
            '3' => Some(Code::Digit3),
            '4' => Some(Code::Digit4),
            '5' => Some(Code::Digit5),
            '6' => Some(Code::Digit6),
            '7' => Some(Code::Digit7),
            '8' => Some(Code::Digit8),
            '9' => Some(Code::Digit9),
            _ => None,
        };
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_hotkey() {
        let hk = parse_hotkey("Ctrl+Shift+F5").unwrap();
        let expected =
            HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F5);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn parses_letters_and_digits() {
        let hk = parse_hotkey("Alt+Z").unwrap();
        let expected = HotKey::new(Some(Modifiers::ALT), Code::KeyZ);
        assert_eq!(hk.id(), expected.id());

        let hk = parse_hotkey("Ctrl+1").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL), Code::Digit1);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn rejects_multiple_keys() {
        assert!(parse_hotkey("Ctrl+A+B").is_err());
    }

    #[test]
    fn handles_duplicate_registration_gracefully() {
        let config = HotkeyConfig {
            next_profile: "Ctrl+A".to_string(),
            prev_profile: "Ctrl+A".to_string(), // Duplicate
            reset: "Ctrl+R".to_string(),
        };
        
        let controller = HotkeyController::new(&config).expect("Should succeed now");
        assert_eq!(controller.next_id, controller.prev_id);
        
        // Verify action mapping (should return the first match)
        let action = controller.get_action(controller.next_id).unwrap();
        assert_eq!(action, HotkeyAction::NextProfile);
    }
}
