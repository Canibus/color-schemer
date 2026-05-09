#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use color_schemer::config::AppConfig;
use color_schemer::nvidia::{DisplaySettings, GpuController, NvidiaController};
use color_schemer::profiles::{DisplayProfile, ProfileManager};
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfilesState {
    profiles: Vec<DisplayProfile>,
    active_index: usize,
}

struct AppState {
    nvidia: Arc<NvidiaController>,
    pm: Arc<Mutex<ProfileManager>>,
    config: Arc<Mutex<AppConfig>>,
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
    let pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(index)
        .ok_or_else(|| format!("invalid profile index: {}", index))?;
    state.nvidia.apply_display_settings(&profile.settings)
}

#[tauri::command]
fn next_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pm = lock_pm(&state.pm);
    let profile = pm.next_profile();
    state.nvidia.apply_display_settings(&profile.settings)
}

#[tauri::command]
fn prev_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pm = lock_pm(&state.pm);
    let profile = pm.prev_profile();
    state.nvidia.apply_display_settings(&profile.settings)
}

#[tauri::command]
fn reset_profile(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let pm = lock_pm(&state.pm);
    let profile = pm
        .set_profile(0)
        .ok_or_else(|| "no profiles available".to_string())?;
    state.nvidia.apply_display_settings(&profile.settings)
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
    state.nvidia.apply_display_settings(&settings)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if !cfg!(windows) {
        eprintln!("color-schemer is currently Windows-only");
        std::process::exit(1);
    }

    let config = AppConfig::load();
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

    tauri::Builder::default()
        .manage(AppState {
            nvidia,
            pm,
            config: Arc::new(Mutex::new(config)),
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

