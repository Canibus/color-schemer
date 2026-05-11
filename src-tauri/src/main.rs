#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_schemer::auto_switch::AutoSwitchManager;
use color_schemer::config::{AppConfig, HotkeyConfig};
use color_schemer::hotkey::{HotkeyAction, HotkeyController};
use color_schemer::nvidia::{DisplaySettings, GpuController, NvidiaController};
use color_schemer::platform;
use color_schemer::profiles::{DisplayProfile, ProfileManager};
use crossbeam_channel::{unbounded, Sender};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesState {
    profiles: Vec<DisplayProfile>,
    active_index: usize,
}

#[derive(Clone, Serialize)]
struct ProfileChangedPayload {
    index: usize,
}

enum HotkeyMsg {
    UpdateConfig(HotkeyConfig),
    SetRecording(bool),
}

struct AppState {
    nvidia: Arc<dyn GpuController>,
    pm: Arc<Mutex<ProfileManager>>,
    asm: Arc<Mutex<AutoSwitchManager>>,
    config: Arc<Mutex<AppConfig>>,
    hotkey_tx: Sender<HotkeyMsg>,
    recording_mode: Arc<Mutex<bool>>,
}

#[tauri::command]
fn set_recording_mode(state: tauri::State<'_, AppState>, active: bool) {
    if let Ok(mut mode) = state.recording_mode.lock() {
        *mode = active;
        info!("Recording mode: {}", active);
        
        // Signal the hotkey thread to unregister/re-register
        if let Err(e) = state.hotkey_tx.send(HotkeyMsg::SetRecording(active)) {
            error!("Failed to send recording mode message: {}", e);
        }
    }
}

fn lock_pm(pm: &Arc<Mutex<ProfileManager>>) -> std::sync::MutexGuard<'_, ProfileManager> {
    match pm.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            error!("ProfileManager mutex poisoned; continuing with inner value");
            poisoned.into_inner()
        }
    }
}

fn lock_cfg(cfg: &Arc<Mutex<AppConfig>>) -> std::sync::MutexGuard<'_, AppConfig> {
    match cfg.lock() {
        Ok(g) => g,
        Err(poisoned) => {
            error!("AppConfig mutex poisoned; continuing with inner value");
            poisoned.into_inner()
        }
    }
}

#[tauri::command]
fn get_profiles_state(state: tauri::State<'_, AppState>) -> Result<ProfilesState, String> {
    let pm = lock_pm(&state.pm);
    Ok(ProfilesState {
        profiles: pm.profiles().to_vec(),
        active_index: pm.current_index(),
    })
}

#[tauri::command]
fn get_displays(state: tauri::State<'_, AppState>) -> Result<Vec<color_schemer::nvidia::DisplayInfo>, String> {
    state.nvidia.get_displays().map_err(|e| e.to_string())
}

#[tauri::command]
fn apply_profile(state: tauri::State<'_, AppState>, index: usize) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(index)
        .ok_or_else(|| format!("invalid profile index: {}", index))?;
    
    let settings = &profile.settings;
    if profile.target_displays.is_empty() {
        state.nvidia.apply_display_settings(None, settings).map_err(|e| e.to_string())?;
    } else {
        for id in &profile.target_displays {
            let _ = state.nvidia.apply_display_settings(Some(id), settings);
        }
    }
    Ok(())
}

#[tauri::command]
fn next_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm.next_profile().clone();
    drop(pm);

    if profile.target_displays.is_empty() {
        state.nvidia.apply_display_settings(None, &profile.settings).map_err(|e| e.to_string())
    } else {
        for id in &profile.target_displays {
            let _ = state.nvidia.apply_display_settings(Some(id), &profile.settings);
        }
        Ok(())
    }
}

#[tauri::command]
fn prev_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm.prev_profile().clone();
    drop(pm);

    if profile.target_displays.is_empty() {
        state.nvidia.apply_display_settings(None, &profile.settings).map_err(|e| e.to_string())
    } else {
        for id in &profile.target_displays {
            let _ = state.nvidia.apply_display_settings(Some(id), &profile.settings);
        }
        Ok(())
    }
}

#[tauri::command]
fn reset_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(0)
        .ok_or_else(|| "no profiles available".to_string())?;
    
    // Resetting usually means applying the default profile (index 0)
    // We should probably reset ALL displays if we're doing a global reset, 
    // or just the target displays of the default profile.
    // Let's assume reset_profile means "go to default profile".
    let settings = &profile.settings;
    if profile.target_displays.is_empty() {
        state.nvidia.apply_display_settings(None, settings).map_err(|e| e.to_string())?;
    } else {
        for id in &profile.target_displays {
            let _ = state.nvidia.apply_display_settings(Some(id), settings);
        }
    }
    Ok(())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = lock_cfg(&state.config);
    Ok(cfg.clone())
}

#[tauri::command]
fn save_config(state: tauri::State<'_, AppState>, app_handle: tauri::AppHandle, updated: AppConfig) -> Result<(), String> {
    // 1. Capture the name of the currently active profile
    let active_profile_name = {
        let pm = lock_pm(&state.pm);
        pm.current_profile().name.clone()
    };

    // 2. Get current config to check for changes
    let (lang_changed, auto_start_changed) = {
        let cfg = lock_cfg(&state.config);
        (cfg.language != updated.language, cfg.auto_start != updated.auto_start)
    };

    // Update registry if auto_start changed
    if auto_start_changed {
        color_schemer::platform::windows::update_auto_start(updated.auto_start)?;
    }

    // Persist to config.toml (next to executable) using core implementation.
    updated.save();

    // Update tray menu if language changed
    if lang_changed {
        let tray_handle = app_handle.tray_handle();
        let _ = tray_handle.get_item("quit").set_title(color_schemer::i18n::t(&updated.language, "tray.quit"));
        let _ = tray_handle.get_item("show").set_title(color_schemer::i18n::t(&updated.language, "tray.show"));
    }

    // Signal background thread to update hotkeys.
    if let Err(e) = state.hotkey_tx.send(HotkeyMsg::UpdateConfig(updated.hotkeys.clone())) {
        error!("Failed to send hotkey update message: {}", e);
    }

    // Update in-memory config.
    {
        let mut cfg = lock_cfg(&state.config);
        *cfg = updated.clone();
    }

    // 3. Replace profile manager profiles and restore active profile by name
    {
        let mut pm = lock_pm(&state.pm);
        *pm = ProfileManager::new(updated.profiles.clone());
        
        // Restore the previously active profile by name, or fallback to index 0
        pm.set_profile_by_name(&active_profile_name);
        
        let profile = pm.current_profile();
        let settings = &profile.settings;
        if profile.target_displays.is_empty() {
            state.nvidia.apply_display_settings(None, settings).map_err(|e| e.to_string())?;
        } else {
            for id in &profile.target_displays {
                let _ = state.nvidia.apply_display_settings(Some(id), settings);
            }
        }
        info!("Restored active profile '{}' after config save", profile.name);
    }

    Ok(())
}

#[tauri::command]
fn preview_settings(state: tauri::State<'_, AppState>, settings: DisplaySettings, display_ids: Vec<String>) -> Result<(), String> {
    if display_ids.is_empty() {
        state.nvidia.apply_display_settings(None, &settings).map_err(|e| e.to_string())
    } else {
        for id in &display_ids {
            let _ = state.nvidia.apply_display_settings(Some(id), &settings);
        }
        Ok(())
    }
}

#[tauri::command]
fn get_running_apps() -> Result<Vec<color_schemer::platform::windows::ProcessInfo>, String> {
    Ok(color_schemer::platform::windows::get_running_apps())
}

use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if !cfg!(windows) {
        eprintln!("color-schemer is currently Windows-only");
        std::process::exit(1);
    }

    let config = AppConfig::load();
    let start_minimized = config.start_minimized;
    let pm = Arc::new(Mutex::new(ProfileManager::new(config.profiles.clone())));
    let initial_index = pm.lock().unwrap().current_index();
    let asm = Arc::new(Mutex::new(AutoSwitchManager::new(initial_index)));

    let nvidia: Arc<dyn GpuController> = match NvidiaController::new() {
        Ok(ctrl) => Arc::new(ctrl),
        Err(e) => {
            error!("NVIDIA GPU not found or NVAPI init failed: {}. Falling back to mock controller.", e);
            Arc::new(color_schemer::mock_gpu::MockGpuController::new())
        }
    };

    // Apply initial profile (index 0).
    if let Ok(pm_lock) = pm.lock() {
        let profile = pm_lock.current_profile();
        if profile.target_displays.is_empty() {
            let _ = nvidia.apply_display_settings(None, &profile.settings);
        } else {
            for id in &profile.target_displays {
                let _ = nvidia.apply_display_settings(Some(id), &profile.settings);
            }
        }
    }

    // System Tray Setup
    let quit = CustomMenuItem::new("quit".to_string(), color_schemer::i18n::t(&config.language, "tray.quit"));
    let show = CustomMenuItem::new("show".to_string(), color_schemer::i18n::t(&config.language, "tray.show"));
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let config_state = Arc::new(Mutex::new(config.clone()));
    let (hotkey_tx, hotkey_rx) = unbounded::<HotkeyMsg>();
    let recording_mode = Arc::new(Mutex::new(false));

    // Установка хука на смену фокуса
    let hook = platform::windows::set_foreground_hook(win_event_proc);
    let hook_raw = hook as usize;

    let nvidia_for_quit = nvidia.clone();

    tauri::Builder::default()
        .manage(AppState {
            nvidia,
            pm,
            asm,
            config: config_state,
            hotkey_tx,
            recording_mode,
        })
        .system_tray(system_tray)
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                let _ = window.show();
                let _ = window.set_focus();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    info!("Quitting from tray menu");
                    let _ = nvidia_for_quit.reset(None);
                    if hook_raw != 0 {
                        platform::windows::unhook_event_hook(hook_raw as *mut std::ffi::c_void);
                    }
                    std::process::exit(0);
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                _ => {}
            },
            _ => {}
        })
        .setup(move |app| {
            let app_handle = app.handle();
            let state = app.state::<AppState>();
            
            // Background focus watcher thread
            {
                let handle = app_handle.clone();
                let nvidia_c = state.nvidia.clone();
                let pm_c = state.pm.clone();
                let asm_c = state.asm.clone();
                let config_c = state.config.clone();
                
                std::thread::spawn(move || {
                    loop {
                        if FOREGROUND_CHANGED.swap(false, Ordering::SeqCst) {
                            if let Some(process_name) = platform::windows::get_foreground_process_name() {
                                let mut asm = asm_c.lock().unwrap();
                                let pm = pm_c.lock().unwrap();
                                let profiles = pm.profiles().to_vec();
                                drop(pm);

                                if let Some(target_index) = asm.evaluate_focus_change(&process_name, &profiles) {
                                    drop(asm);
                                    
                                    let pm = pm_c.lock().unwrap();
                                    if let Some(profile) = pm.profiles().get(target_index) {
                                        let settings = profile.settings.clone();
                                        let target_displays = profile.target_displays.clone();
                                        drop(pm);
                                        
                                        if target_displays.is_empty() {
                                            let _ = nvidia_c.apply_display_settings(None, &settings);
                                        } else {
                                            for id in &target_displays {
                                                let _ = nvidia_c.apply_display_settings(Some(id), &settings);
                                            }
                                        }
                                        info!("Auto-switched to profile index {} for {}", target_index, process_name);
                                        
                                        // Notify frontend
                                        let _ = handle.emit_all("profile-changed", ProfileChangedPayload { index: target_index });
                                        
                                        let (show_notif, lang) = {
                                            let cfg = config_c.lock().unwrap();
                                            (cfg.show_notifications, cfg.language.clone())
                                        };
                                        if show_notif {
                                            color_schemer::platform::windows::show_notification(
                                                color_schemer::i18n::t(&lang, "notif.title_auto"),
                                                &color_schemer::i18n::t(&lang, "notif.body").replace("{}", &process_name),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(50));
                    }
                });
            }

            // Background hotkey handler
            {
                let handle = app_handle.clone();
                let nvidia_c = state.nvidia.clone();
                let pm_c = state.pm.clone();
                let asm_c = state.asm.clone();
                let config_c = state.config.clone(); 
                let recording_mode_c = state.recording_mode.clone();
                let hotkey_rx = hotkey_rx;
                
                std::thread::spawn(move || {
                    let initial_hotkeys = {
                        let cfg = config_c.lock().unwrap();
                        cfg.hotkeys.clone()
                    };
                    let mut hotkey_controller = HotkeyController::new(&initial_hotkeys).ok();
                    let mut last_trigger = Instant::now();
                    let cooldown = Duration::from_millis(100);
                    
                    loop {
                        color_schemer::platform::windows::pump_messages();

                        if let Ok(msg) = hotkey_rx.try_recv() {
                            match msg {
                                HotkeyMsg::UpdateConfig(new_cfg) => {
                                    hotkey_controller = None;
                                    match HotkeyController::new(&new_cfg) {
                                        Ok(new_hk) => {
                                            hotkey_controller = Some(new_hk);
                                            info!("Hotkeys re-registered in background thread");
                                        }
                                        Err(e) => error!("Failed to re-register hotkeys: {}", e),
                                    }
                                }
                                HotkeyMsg::SetRecording(active) => {
                                    if active {
                                        hotkey_controller = None;
                                        info!("Hotkeys unregistered for recording mode");
                                    } else {
                                        let cfg = {
                                            let c = config_c.lock().unwrap();
                                            c.hotkeys.clone()
                                        };
                                        match HotkeyController::new(&cfg) {
                                            Ok(new_hk) => {
                                                hotkey_controller = Some(new_hk);
                                                info!("Hotkeys re-registered after recording mode");
                                            }
                                            Err(e) => error!("Failed to re-register hotkeys: {}", e),
                                        }
                                    }
                                }
                            }
                        }

                        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                            if let Ok(mode) = recording_mode_c.lock() {
                                if *mode {
                                    continue;
                                }
                            }

                            let now = Instant::now();
                            if event.state() == HotKeyState::Pressed && now.duration_since(last_trigger) > cooldown {
                                last_trigger = now;
                                
                                if let Some(hk) = hotkey_controller.as_ref() {
                                    if let Some(action) = hk.get_action(event.id()) {
                                        let mut pm = pm_c.lock().unwrap();
                                        let profile = match action {
                                            HotkeyAction::NextProfile => pm.next_profile().clone(),
                                            HotkeyAction::PrevProfile => pm.prev_profile().clone(),
                                            HotkeyAction::Reset => {
                                                if let Some(p) = pm.set_profile(0) {
                                                    p.clone()
                                                } else {
                                                    pm.current_profile().clone()
                                                }
                                            }
                                        };
                                        let index = pm.current_index();
                                        drop(pm);
                                        
                                        {
                                            let mut asm = asm_c.lock().unwrap();
                                            asm.handle_manual_switch(index);
                                        }
                                        
                                        let _ = if profile.target_displays.is_empty() {
                                            nvidia_c.apply_display_settings(None, &profile.settings)
                                        } else {
                                            for id in &profile.target_displays {
                                                let _ = nvidia_c.apply_display_settings(Some(id), &profile.settings);
                                            }
                                            Ok(())
                                        };
                                        info!("Hotkey triggered: applied profile '{}'", profile.name);

                                        // Notify frontend
                                        let _ = handle.emit_all("profile-changed", ProfileChangedPayload { index });

                                        let (show_notif, lang) = {
                                            let cfg = config_c.lock().unwrap();
                                            (cfg.show_notifications, cfg.language.clone())
                                        };
                                        if show_notif {
                                            color_schemer::platform::windows::show_notification(
                                                color_schemer::i18n::t(&lang, "notif.title"),
                                                &color_schemer::i18n::t(&lang, "notif.body").replace("{}", &profile.name),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                });
            }

            if !start_minimized {
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                }
            }
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_profiles_state,
            get_displays,
            apply_profile,
            next_profile,
            prev_profile,
            reset_profile,
            get_config,
            save_config,
            preview_settings,
            set_recording_mode,
            get_running_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

