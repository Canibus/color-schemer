use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use log::info;

/// Идентификаторы действий
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    NextProfile,
    PrevProfile,
    Reset,
}

/// Менеджер горячих клавиш
pub struct HotkeyController {
    manager: GlobalHotKeyManager,
    next_id: u32,
    prev_id: u32,
    reset_id: u32,
}

impl HotkeyController {
    pub fn new() -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Не удалось создать менеджер горячих клавиш: {}", e))?;

        // Ctrl+Shift+F5 — следующий профиль
        let next_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F5);

        // Ctrl+Shift+F6 — предыдущий профиль
        let prev_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F6);

        // Ctrl+Shift+F7 — сброс
        let reset_hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::F7);

        let next_id = next_hotkey.id();
        let prev_id = prev_hotkey.id();
        let reset_id = reset_hotkey.id();

        manager
            .register(next_hotkey)
            .map_err(|e| format!("Не удалось зарегистрировать Ctrl+Shift+F5: {}", e))?;
        manager
            .register(prev_hotkey)
            .map_err(|e| format!("Не удалось зарегистрировать Ctrl+Shift+F6: {}", e))?;
        manager
            .register(reset_hotkey)
            .map_err(|e| format!("Не удалось зарегистрировать Ctrl+Shift+F7: {}", e))?;

        info!("Горячие клавиши зарегистрированы:");
        info!("  Ctrl+Shift+F5 — следующий профиль");
        info!("  Ctrl+Shift+F6 — предыдущий профиль");
        info!("  Ctrl+Shift+F7 — сброс к стандартным");

        Ok(Self {
            manager,
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
