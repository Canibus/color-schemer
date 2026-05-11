#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg(windows)]

use color_schemer::auto_switch::AutoSwitchManager;
use color_schemer::config::AppConfig;
use color_schemer::hotkey::{HotkeyAction, HotkeyController};
use color_schemer::nvidia::{GpuController, NvidiaController};
use color_schemer::platform;
use color_schemer::profiles::ProfileManager;
use color_schemer::tray::TrayController;

use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tray_icon::menu::MenuEvent;

static FOREGROUND_CHANGED: AtomicBool = AtomicBool::new(true);

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: *mut std::ffi::c_void,
    _event: u32,
    _hwnd: *mut std::ffi::c_void,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dw_ms_event_time: u32,
) {
    FOREGROUND_CHANGED.store(true, Ordering::SeqCst);
    platform::windows::wake_message_loop();
}

fn apply_profile_internal(
    index: usize,
    nvidia: &NvidiaController,
    pm_arc: &Arc<Mutex<ProfileManager>>,
    show_notif: bool,
    is_auto: bool,
) {
    let mut pm = lock_profile_manager(pm_arc);
    if let Some(profile) = pm.set_profile(index) {
        let name = profile.name.clone();
        let settings = profile.settings.clone();
        let target_displays = profile.target_displays.clone();
        drop(pm);

        let res = if target_displays.is_empty() {
            nvidia.apply_display_settings(None, &settings)
        } else {
            for id in &target_displays {
                let _ = nvidia.apply_display_settings(Some(id), &settings);
            }
            Ok(())
        };

        match res {
            Ok(()) => {
                info!(
                    "Профиль применён{}: {}",
                    if is_auto { " (авто)" } else { "" },
                    name
                );
                if show_notif {
                    platform::windows::show_notification(
                        if is_auto {
                            "Авто-переключение"
                        } else {
                            "Профиль изменён"
                        },
                        &format!("Активный профиль: {}", name),
                    );
                }
            }
            Err(e) => error!("Ошибка применения профиля '{}': {}", name, e),
        }
    }
}

fn handle_action(
    action: HotkeyAction,
    nvidia: &NvidiaController,
    profile_manager: &Arc<Mutex<ProfileManager>>,
    auto_switch_manager: &Arc<Mutex<AutoSwitchManager>>,
    show_notifications: bool,
) {
    let index = {
        let mut pm = lock_profile_manager(profile_manager);
        match action {
            HotkeyAction::NextProfile => {
                pm.next_profile();
                pm.current_index()
            }
            HotkeyAction::PrevProfile => {
                pm.prev_profile();
                pm.current_index()
            }
            HotkeyAction::Reset => {
                pm.set_profile(0);
                0
            }
        }
    };

    let mut asm = lock_auto_switch_manager(auto_switch_manager);
    asm.handle_manual_switch(index);
    drop(asm);

    apply_profile_internal(index, nvidia, profile_manager, show_notifications, false);
}

fn handle_menu_event(
    id: &str,
    nvidia: &NvidiaController,
    pm: &Arc<Mutex<ProfileManager>>,
    asm: &Arc<Mutex<AutoSwitchManager>>,
    show_notif: bool,
) -> bool {
    match id {
        "quit" => {
            info!("Выход из приложения");
            let _ = nvidia.reset(None);
            return true; // сигнал на выход
        }
        "next_profile" => {
            handle_action(HotkeyAction::NextProfile, nvidia, pm, asm, show_notif);
        }
        "prev_profile" => {
            handle_action(HotkeyAction::PrevProfile, nvidia, pm, asm, show_notif);
        }
        "reset" => {
            handle_action(HotkeyAction::Reset, nvidia, pm, asm, show_notif);
        }
        other if other.starts_with("profile_") => {
            if let Ok(index) = other.trim_start_matches("profile_").parse::<usize>() {
                let mut asm_lock = lock_auto_switch_manager(asm);
                asm_lock.handle_manual_switch(index);
                drop(asm_lock);

                apply_profile_internal(index, nvidia, pm, show_notif, false);
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

fn lock_auto_switch_manager(
    asm: &Arc<Mutex<AutoSwitchManager>>,
) -> MutexGuard<'_, AutoSwitchManager> {
    match asm.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("AutoSwitchManager mutex poisoned; continuing with inner value");
            poisoned.into_inner()
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Ensure only one instance is running
    let _instance = match platform::windows::SingleInstance::new("ColorSchemerMutex") {
        Some(inst) => inst,
        None => {
            error!("Another instance of color-schemer is already running.");
            return;
        }
    };

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
    let initial_index = {
        let pm = lock_profile_manager(&profile_manager);
        pm.current_index()
    };
    let auto_switch_manager = Arc::new(Mutex::new(AutoSwitchManager::new(initial_index)));

    // Применяем начальный профиль
    let initial_profile_name = {
        let pm = lock_profile_manager(&profile_manager);
        let profile = pm.current_profile();
        info!("Начальный профиль: {}", profile.name);

        let res = if profile.target_displays.is_empty() {
            nvidia.apply_display_settings(None, &profile.settings)
        } else {
            for id in &profile.target_displays {
                let _ = nvidia.apply_display_settings(Some(id), &profile.settings);
            }
            Ok(())
        };

        if let Err(e) = res {
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

    // Установка хука на смену фокуса
    let hook = platform::windows::set_foreground_hook(win_event_proc);

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

        // 2. Обработка авто-переключения
        if FOREGROUND_CHANGED.swap(false, Ordering::SeqCst) {
            if let Some(process_name) = platform::windows::get_foreground_process_name() {
                let mut asm = lock_auto_switch_manager(&auto_switch_manager);
                let pm = lock_profile_manager(&profile_manager);
                let profiles = pm.profiles().to_vec();
                drop(pm);

                if let Some(target_index) = asm.evaluate_focus_change(&process_name, &profiles) {
                    drop(asm);
                    apply_profile_internal(
                        target_index,
                        &nvidia,
                        &profile_manager,
                        show_notif,
                        true,
                    );
                }
            }
        }

        // 3. Обработка горячих клавиш
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            let now = Instant::now();

            if event.state() == HotKeyState::Pressed && now.duration_since(last_trigger) > cooldown {
                last_trigger = now;
                if let Some(action) = hotkey_controller.get_action(event.id()) {
                    handle_action(
                        action,
                        &nvidia,
                        &profile_manager,
                        &auto_switch_manager,
                        show_notif,
                    );
                }
            }
        }

        // 4. Обработка событий меню трея
        let mut should_quit = false;
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = event.id().0.as_str();
            if handle_menu_event(
                id,
                &nvidia,
                &profile_manager,
                &auto_switch_manager,
                show_notif,
            ) {
                should_quit = true;
                break;
            }
        }
        if should_quit {
            break;
        }

        // 5. Ждём следующее сообщение (0% CPU в простое)
        platform::windows::wait_message();
    }

    if !hook.is_null() {
        platform::windows::unhook_event_hook(hook);
    }

    info!("Приложение завершено");
}
