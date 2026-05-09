#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_schemer::config::{AppConfig, HotkeyConfig};
use color_schemer::hotkey::{HotkeyAction, HotkeyController};
use color_schemer::nvidia::{DisplaySettings, GpuController, NvidiaController};
use color_schemer::profiles::{DisplayProfile, ProfileManager};
use crossbeam_channel::{unbounded, Sender};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesState {
    profiles: Vec<DisplayProfile>,
    active_index: usize,
}

enum HotkeyMsg {
    UpdateConfig(HotkeyConfig),
}

struct AppState {
    nvidia: Arc<NvidiaController>,
    pm: Arc<Mutex<ProfileManager>>,
    config: Arc<Mutex<AppConfig>>,
    hotkey_tx: Sender<HotkeyMsg>,
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
fn apply_profile(state: tauri::State<'_, AppState>, index: usize) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(index)
        .ok_or_else(|| format!("invalid profile index: {}", index))?;
    state.nvidia.apply_display_settings(&profile.settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn next_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm.next_profile();
    state.nvidia.apply_display_settings(&profile.settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn prev_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm.prev_profile();
    state.nvidia.apply_display_settings(&profile.settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn reset_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(0)
        .ok_or_else(|| "no profiles available".to_string())?;
    state.nvidia.apply_display_settings(&profile.settings).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<AppConfig, String> {
    let cfg = lock_cfg(&state.config);
    Ok(cfg.clone())
}

#[tauri::command]
fn save_config(state: tauri::State<'_, AppState>, updated: AppConfig) -> Result<(), String> {
    // Persist to config.toml (next to executable) using core implementation.
    updated.save();

    // Signal background thread to update hotkeys.
    if let Err(e) = state.hotkey_tx.send(HotkeyMsg::UpdateConfig(updated.hotkeys.clone())) {
        error!("Failed to send hotkey update message: {}", e);
    }

    // Update in-memory config.
    {
        let mut cfg = lock_cfg(&state.config);
        *cfg = updated.clone();
    }

    // Replace profile manager profiles with the new config’s profiles.
    {
        let mut pm = lock_pm(&state.pm);
        *pm = ProfileManager::new(updated.profiles.clone());
    }

    Ok(())
}

#[tauri::command]
fn preview_settings(state: tauri::State<'_, AppState>, settings: DisplaySettings) -> Result<(), String> {
    state.nvidia.apply_display_settings(&settings).map_err(|e| e.to_string())
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

    let nvidia = match NvidiaController::new() {
        Ok(ctrl) => Arc::new(ctrl),
        Err(e) => {
            eprintln!("NVIDIA init failed: {e}");
            std::process::exit(1);
        }
    };

    // Apply initial profile (index 0).
    if let Ok(pm_lock) = pm.lock() {
        let _ = nvidia.apply_display_settings(&pm_lock.current_profile().settings);
    }

    // System Tray Setup
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show UI");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    let config_state = Arc::new(Mutex::new(config.clone()));
    let (hotkey_tx, hotkey_rx) = unbounded::<HotkeyMsg>();

    // Background hotkey handler
    {
        let nvidia_c = nvidia.clone();
        let pm_c = pm.clone();
        let config_c = config_state.clone(); 
        
        std::thread::spawn(move || {
            let mut hotkey_controller = HotkeyController::new(&config.hotkeys).ok();
            let mut last_trigger = Instant::now();
            let cooldown = Duration::from_millis(100);
            
            loop {
                // 1. Process Windows messages (crucial for global-hotkey on Windows)
                color_schemer::platform::windows::pump_messages();

                // 2. Check for config updates
                if let Ok(msg) = hotkey_rx.try_recv() {
                    match msg {
                        HotkeyMsg::UpdateConfig(new_cfg) => {
                            // Drop the old controller first to unregister hotkeys from Windows.
                            // If we don't do this, HotkeyController::new will fail because 
                            // the hotkeys are already registered by the previous manager.
                            hotkey_controller = None;
                            
                            match HotkeyController::new(&new_cfg) {
                                Ok(new_hk) => {
                                    hotkey_controller = Some(new_hk);
                                    info!("Hotkeys re-registered in background thread");
                                }
                                Err(e) => error!("Failed to re-register hotkeys: {}", e),
                            }
                        }
                    }
                }

                // Check for hotkey events
                while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
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
                                drop(pm);
                                
                                let _ = nvidia_c.apply_display_settings(&profile.settings);
                                info!("Hotkey triggered: applied profile '{}'", profile.name);

                                // Show notification if enabled
                                let show_notif = {
                                    let cfg = config_c.lock().unwrap();
                                    cfg.show_notifications
                                };
                                if show_notif {
                                    color_schemer::platform::windows::show_notification(
                                        "Profile Switched",
                                        &format!("Active profile: {}", profile.name),
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

    let nvidia_for_quit = nvidia.clone();

    tauri::Builder::default()
        .manage(AppState {
            nvidia,
            pm,
            config: config_state,
            hotkey_tx,
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
                    let _ = NvidiaController::reset_gamma_ramp();
                    let _ = nvidia_for_quit.set_digital_vibrance(0);
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
            apply_profile,
            next_profile,
            prev_profile,
            reset_profile,
            get_config,
            save_config,
            preview_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

