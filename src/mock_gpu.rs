use crate::nvidia::{DisplaySettings, GpuController, NV_DISPLAY_DVC_INFO};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum GpuCall {
    ApplySettings(DisplaySettings),
    SetVibrance(i32),
    GetVibrance,
    Reset,
}

pub struct MockGpuController {
    pub calls: Mutex<Vec<GpuCall>>,
    pub should_fail: Mutex<bool>,
    pub vibrance_level: Mutex<i32>,
}

impl MockGpuController {
    pub fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            should_fail: Mutex::new(false),
            vibrance_level: Mutex::new(0),
        }
    }

    pub fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    pub fn get_calls(&self) -> Vec<GpuCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn last_settings(&self) -> Option<DisplaySettings> {
        let calls = self.calls.lock().unwrap();
        calls.iter().rev().find_map(|c| {
            if let GpuCall::ApplySettings(s) = c {
                Some(s.clone())
            } else {
                None
            }
        })
    }
}

impl GpuController for MockGpuController {
    fn apply_display_settings(&self, settings: &DisplaySettings) -> Result<(), String> {
        self.calls
            .lock()
            .unwrap()
            .push(GpuCall::ApplySettings(settings.clone()));
        if *self.should_fail.lock().unwrap() {
            Err("Mock apply failed".to_string())
        } else {
            Ok(())
        }
    }

    fn set_digital_vibrance(&self, level: i32) -> Result<(), String> {
        self.calls.lock().unwrap().push(GpuCall::SetVibrance(level));
        if *self.should_fail.lock().unwrap() {
            Err("Mock vibrance failed".to_string())
        } else {
            *self.vibrance_level.lock().unwrap() = level;
            Ok(())
        }
    }

    fn get_digital_vibrance(&self) -> Result<NV_DISPLAY_DVC_INFO, String> {
        self.calls.lock().unwrap().push(GpuCall::GetVibrance);
        Ok(NV_DISPLAY_DVC_INFO {
            current_level: *self.vibrance_level.lock().unwrap(),
            min_level: -1024,
            max_level: 1023,
            default_level: 0,
            version: 0x10010,
        })
    }

    fn reset(&self) -> Result<(), String> {
        self.calls.lock().unwrap().push(GpuCall::Reset);
        *self.vibrance_level.lock().unwrap() = 0;
        Ok(())
    }
}
