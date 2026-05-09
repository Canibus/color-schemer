#![cfg(windows)]

use color_schemer::config::AppConfig;
use color_schemer::hotkey::{HotkeyAction, HotkeyController};
use color_schemer::nvidia::{GpuController, NvidiaController};
use color_schemer::platform;
use color_schemer::profiles::ProfileManager;
use color_schemer::tray::TrayController;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{error, info, warn};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tray_icon::menu::MenuEvent;

fn handle_action(
    action: HotkeyAction,
    nvidia: &NvidiaController,
    profile_manager: &Arc<Mutex<ProfileManager>>,
    show_notifications: bool,
) {
    let mut pm = lock_profile_manager(profile_manager);

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
                platform::windows::show_notification(
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
                let mut pm_lock = lock_profile_manager(pm);
                if let Some(profile) = pm_lock.set_profile(index) {
                    let name = profile.name.clone();
                    let settings = profile.settings.clone();
                    drop(pm_lock);

                    match nvidia.apply_display_settings(&settings) {
                        Ok(()) => {
                            info!("Профиль применён: {}", name);
                            if show_notif {
                                platform::windows::show_notification(
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

fn lock_profile_manager(pm: &Arc<Mutex<ProfileManager>>) -> MutexGuard<'_, ProfileManager> {
    match pm.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("ProfileManager mutex poisoned; continuing with inner value");
            poisoned.into_inner()
        }
    }
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
            platform::windows::show_notification("Ошибка", &format!("NVIDIA init failed: {}", e));
            return;
        }
    };

    let profile_manager = Arc::new(Mutex::new(ProfileManager::new(config.profiles)));

    // Применяем начальный профиль
    let initial_profile_name = {
        let pm = lock_profile_manager(&profile_manager);
        let profile = pm.current_profile();
        info!("Начальный профиль: {}", profile.name);
        if let Err(e) = nvidia.apply_display_settings(&profile.settings) {
            warn!("Не удалось применить начальный профиль: {}", e);
        }
        profile.name.clone()
    };

    let show_notif = config.show_notifications;
    if show_notif && !config.start_minimized {
        platform::windows::show_notification(
            "NVIDIA Profile Switcher",
            &format!("Запущено. Активный профиль: {}", initial_profile_name),
        );
    }

    // Регистрируем хоткеи В ГЛАВНОМ ПОТОКЕ
    let hotkey_controller = match HotkeyController::new(&config.hotkeys) {
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
        let pm = lock_profile_manager(&profile_manager);
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
    // Единый цикл: блокирующее ожидание сообщений
    // =========================================================
    let mut last_trigger = Instant::now();
    let cooldown = Duration::from_millis(300);

    loop {
        // 1. Прокачиваем Windows-сообщения (нужно для хоткеев и трея)
        platform::windows::pump_messages();

        // 2. Обработка горячих клавиш
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
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
        let mut should_quit = false;
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = event.id().0.as_str();
            if handle_menu_event(id, &nvidia, &profile_manager, show_notif) {
                should_quit = true;
                break;
            }
        }
        if should_quit {
            break;
        }

        // 4. Ждём следующее сообщение (0% CPU в простое)
        platform::windows::wait_message();
    }

    info!("Приложение завершено");
}
