mod config;
mod hotkey;
mod nvidia;
mod profiles;
mod tray;

use crate::config::AppConfig;
use crate::hotkey::{HotkeyAction, HotkeyController};
use crate::nvidia::NvidiaController;
use crate::profiles::ProfileManager;
use crate::tray::TrayController;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tray_icon::menu::MenuEvent;

// Windows message pump
#[cfg(windows)]
mod win_msg {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default)]
    pub struct MSG {
        pub hwnd: *mut c_void,
        pub message: u32,
        pub wparam: usize,
        pub lparam: isize,
        pub time: u32,
        pub pt_x: i32,
        pub pt_y: i32,
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn PeekMessageW(
            msg: *mut MSG,
            hwnd: *mut c_void,
            filter_min: u32,
            filter_max: u32,
            remove: u32,
        ) -> i32;
        pub fn TranslateMessage(msg: *const MSG) -> i32;
        pub fn DispatchMessageW(msg: *const MSG) -> isize;
    }

    pub const PM_REMOVE: u32 = 0x0001;

    /// Обработать все ожидающие Windows-сообщения без блокировки
    pub fn pump_messages() {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

fn show_notification(title: &str, message: &str) {
    #[cfg(windows)]
    {
        use winrt_notification::Toast;
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(message)
            .duration(winrt_notification::Duration::Short)
            .show();
    }
}

fn handle_action(
    action: HotkeyAction,
    nvidia: &NvidiaController,
    profile_manager: &Arc<Mutex<ProfileManager>>,
    show_notifications: bool,
) {
    let pm = profile_manager.lock().unwrap();

    let (profile_name, settings) = match action {
        HotkeyAction::NextProfile => {
            let profile = pm.next_profile();
            (profile.name.clone(), profile.settings.clone())
        }
        HotkeyAction::PrevProfile => {
            let profile = pm.prev_profile();
            (profile.name.clone(), profile.settings.clone())
        }
        HotkeyAction::Reset => {
            if let Some(profile) = pm.set_profile(0) {
                (profile.name.clone(), profile.settings.clone())
            } else {
                return;
            }
        }
    };

    drop(pm);

    match nvidia.apply_display_settings(&settings) {
        Ok(()) => {
            info!("Профиль '{}' применён", profile_name);
            if show_notifications {
                show_notification(
                    "Профиль переключён",
                    &format!("Активный профиль: {}", profile_name),
                );
            }
        }
        Err(e) => {
            error!("Ошибка применения профиля '{}': {}", profile_name, e);
        }
    }
}

fn handle_menu_event(
    id: &str,
    nvidia: &NvidiaController,
    pm: &Arc<Mutex<ProfileManager>>,
    show_notif: bool,
) -> bool {
    match id {
        "quit" => {
            info!("Выход из приложения");
            let _ = NvidiaController::reset_gamma_ramp();
            let _ = nvidia.set_digital_vibrance(0);
            return true; // сигнал на выход
        }
        "next_profile" => {
            handle_action(HotkeyAction::NextProfile, nvidia, pm, show_notif);
        }
        "prev_profile" => {
            handle_action(HotkeyAction::PrevProfile, nvidia, pm, show_notif);
        }
        "reset" => {
            handle_action(HotkeyAction::Reset, nvidia, pm, show_notif);
        }
        other if other.starts_with("profile_") => {
            if let Ok(index) = other.trim_start_matches("profile_").parse::<usize>() {
                let mut pm_lock = pm.lock().unwrap();
                if let Some(profile) = pm_lock.set_profile(index) {
                    let name = profile.name.clone();
                    let settings = profile.settings.clone();
                    drop(pm_lock);

                    match nvidia.apply_display_settings(&settings) {
                        Ok(()) => {
                            info!("Профиль применён: {}", name);
                            if show_notif {
                                show_notification(
                                    "Профиль изменён",
                                    &format!("Активный профиль: {}", name),
                                );
                            }
                        }
                        Err(e) => error!("Ошибка: {}", e),
                    }
                }
            }
        }
        _ => {}
    }
    false // не выходим
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("=== NVIDIA Profile Switcher ===");

    let config = AppConfig::load();
    info!("Загружено {} профилей", config.profiles.len());

    let nvidia = match NvidiaController::new() {
        Ok(ctrl) => {
            info!("NVIDIA контроллер инициализирован");
            Arc::new(ctrl)
        }
        Err(e) => {
            error!("Ошибка инициализации NVIDIA: {}", e);
            show_notification("Ошибка", &format!("NVIDIA init failed: {}", e));
            return;
        }
    };

    let profile_manager = Arc::new(Mutex::new(ProfileManager::new(config.profiles)));

    // Применяем начальный профиль
    {
        let pm = profile_manager.lock().unwrap();
        let profile = pm.current_profile();
        info!("Начальный профиль: {}", profile.name);
        if let Err(e) = nvidia.apply_display_settings(&profile.settings) {
            warn!("Не удалось применить начальный профиль: {}", e);
        }
    }

    let show_notif = config.show_notifications;

    // Регистрируем хоткеи В ГЛАВНОМ ПОТОКЕ
    let hotkey_controller = match HotkeyController::new() {
        Ok(hk) => {
            info!("Горячие клавиши зарегистрированы");
            hk
        }
        Err(e) => {
            error!("Ошибка регистрации горячих клавиш: {}", e);
            return;
        }
    };

    // Создаём tray В ГЛАВНОМ ПОТОКЕ
    let profile_names: Vec<String> = {
        let pm = profile_manager.lock().unwrap();
        pm.profiles().iter().map(|p| p.name.clone()).collect()
    };

    let _tray = match TrayController::new(&profile_names) {
        Ok(t) => t,
        Err(e) => {
            error!("Ошибка создания tray icon: {}", e);
            return;
        }
    };

    info!("Приложение запущено.");
    info!("Ctrl+Shift+F5 — следующий профиль");
    info!("Ctrl+Shift+F6 — предыдущий профиль");
    info!("Ctrl+Shift+F7 — сброс");

    // =========================================================
    // Единый цикл: свой message pump вместо tao event loop
    // =========================================================
    let mut last_trigger = Instant::now();
    let cooldown = Duration::from_millis(300);

    loop {
        // 1. Прокачиваем Windows-сообщения (нужно для хоткеев и трея)
        #[cfg(windows)]
        win_msg::pump_messages();

        // 2. Обработка горячих клавиш
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            let now = Instant::now();

            if event.state() == HotKeyState::Pressed && now.duration_since(last_trigger) > cooldown
            {
                last_trigger = now;
                if let Some(action) = hotkey_controller.get_action(event.id()) {
                    handle_action(action, &nvidia, &profile_manager, show_notif);
                }
            }
        }

        // 3. Обработка событий меню трея
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = event.id().0.as_str();
            let should_quit = handle_menu_event(id, &nvidia, &profile_manager, show_notif);
            if should_quit {
                break;
            }
        }

        // 4. Небольшая пауза чтобы не грузить CPU
        std::thread::sleep(Duration::from_millis(10));
    }

    info!("Приложение завершено");
}
