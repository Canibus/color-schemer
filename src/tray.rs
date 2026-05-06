use log::info;
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
};

/// ID элементов меню
pub const MENU_QUIT: &str = "quit";
pub const MENU_NEXT: &str = "next_profile";
pub const MENU_PREV: &str = "prev_profile";
pub const MENU_RESET: &str = "reset";

pub struct TrayController {
    _tray: TrayIcon,
}

impl TrayController {
    pub fn new(profile_names: &[String]) -> Result<Self, String> {
        let menu = Menu::new();

        // Подменю профилей
        let profiles_submenu = Submenu::new("Профили", true);
        for (i, name) in profile_names.iter().enumerate() {
            let item = MenuItem::with_id(format!("profile_{}", i), name, true, None);
            profiles_submenu
                .append(&item)
                .map_err(|e| format!("Menu error: {}", e))?;
        }

        let next_item =
            MenuItem::with_id(MENU_NEXT, "Следующий профиль (Ctrl+Shift+F5)", true, None);
        let prev_item =
            MenuItem::with_id(MENU_PREV, "Предыдущий профиль (Ctrl+Shift+F6)", true, None);
        let reset_item = MenuItem::with_id(MENU_RESET, "Сброс (Ctrl+Shift+F7)", true, None);
        let quit_item = MenuItem::with_id(MENU_QUIT, "Выход", true, None);

        menu.append(&profiles_submenu).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| e.to_string())?;
        menu.append(&next_item).map_err(|e| e.to_string())?;
        menu.append(&prev_item).map_err(|e| e.to_string())?;
        menu.append(&reset_item).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator())
            .map_err(|e| e.to_string())?;
        menu.append(&quit_item).map_err(|e| e.to_string())?;

        // Создаём иконку (простая иконка из пикселей)
        let icon = create_default_icon()?;

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("NVIDIA Profile Switcher")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("Не удалось создать tray icon: {}", e))?;

        info!("Tray icon создан");

        Ok(Self { _tray: tray })
    }
}

/// Создание простой иконки 32x32 (зелёный квадрат с буквой N)
fn create_default_icon() -> Result<Icon, String> {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;

            // Фон - тёмно-зелёный
            if x >= 2 && x < size - 2 && y >= 2 && y < size - 2 {
                rgba[idx] = 0x76; // R
                rgba[idx + 1] = 0xB9; // G
                rgba[idx + 2] = 0x00; // B (NVIDIA green)
                rgba[idx + 3] = 255; // A
            } else {
                // Рамка
                rgba[idx] = 0x50;
                rgba[idx + 1] = 0x80;
                rgba[idx + 2] = 0x00;
                rgba[idx + 3] = 255;
            }
        }
    }

    // Рисуем букву "N" белым цветом (упрощённо)
    let letter_coords: Vec<(u32, u32)> = {
        let mut coords = Vec::new();
        // Левая вертикальная линия
        for y in 8..24 {
            for x in 9..13 {
                coords.push((x, y));
            }
        }
        // Правая вертикальная линия
        for y in 8..24 {
            for x in 20..24 {
                coords.push((x, y));
            }
        }
        // Диагональ
        for i in 0..16 {
            let x = 9 + i;
            let y = 8 + i;
            if x < 24 && y < 24 {
                coords.push((x, y));
                if x + 1 < 24 {
                    coords.push((x + 1, y));
                }
            }
        }
        coords
    };

    for (x, y) in letter_coords {
        if x < size && y < size {
            let idx = ((y * size + x) * 4) as usize;
            rgba[idx] = 255; // R
            rgba[idx + 1] = 255; // G
            rgba[idx + 2] = 255; // B
            rgba[idx + 3] = 255; // A
        }
    }

    Icon::from_rgba(rgba, size, size).map_err(|e| format!("Не удалось создать иконку: {}", e))
}
