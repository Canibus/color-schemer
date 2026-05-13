use crate::nvidia::{
    DisplaySettings, GpuController, GpuInfo, NV_DISPLAY_DVC_INFO, NvError, NvResult,
};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum GpuCall {
    ApplySettings(Option<String>, DisplaySettings),
    SetVibrance(usize, i32),
    GetVibrance(usize),
    Reset(Option<String>),
}

pub struct MockGpuController {
    pub calls: Mutex<Vec<GpuCall>>,
    pub should_fail: Mutex<bool>,
    pub vibrance_level: Mutex<i32>,
}

impl MockGpuController {
    pub fn new() -> Self {
        Self::default()
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
            if let GpuCall::ApplySettings(_, s) = c {
                Some(s.clone())
            } else {
                None
            }
        })
    }
}

impl Default for MockGpuController {
    fn default() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            should_fail: Mutex::new(false),
            vibrance_level: Mutex::new(0),
        }
    }
}

impl GpuController for MockGpuController {
    fn get_info(&self) -> GpuInfo {
        GpuInfo {
            name: "Mock GPU".to_string(),
            is_mock: true,
        }
    }

    fn get_displays(&self) -> NvResult<Vec<crate::nvidia::DisplayInfo>> {
        Ok(vec![crate::nvidia::DisplayInfo {
            id: "\\\\.\\DISPLAY1".to_string(),
            name: "Mock Display".to_string(),
            is_primary: true,
        }])
    }

    fn apply_display_settings(
        &self,
        display_id: Option<&str>,
        settings: &DisplaySettings,
    ) -> NvResult<()> {
        self.calls.lock().unwrap().push(GpuCall::ApplySettings(
            display_id.map(|s| s.to_string()),
            settings.clone(),
        ));
        if *self.should_fail.lock().unwrap() {
            Err(NvError::Os("Mock apply failed".to_string()))
        } else {
            Ok(())
        }
    }

    fn set_digital_vibrance(&self, display_handle: usize, level: i32) -> NvResult<()> {
        self.calls
            .lock()
            .unwrap()
            .push(GpuCall::SetVibrance(display_handle, level));
        if *self.should_fail.lock().unwrap() {
            Err(NvError::Os("Mock vibrance failed".to_string()))
        } else {
            *self.vibrance_level.lock().unwrap() = level;
            Ok(())
        }
    }

    fn get_digital_vibrance(&self, display_handle: usize) -> NvResult<NV_DISPLAY_DVC_INFO> {
        self.calls
            .lock()
            .unwrap()
            .push(GpuCall::GetVibrance(display_handle));
        Ok(NV_DISPLAY_DVC_INFO {
            current_level: *self.vibrance_level.lock().unwrap(),
            min_level: -1024,
            max_level: 1023,
            default_level: 0,
            version: 0x10010,
        })
    }

    fn reset(&self, display_id: Option<&str>) -> NvResult<()> {
        self.calls
            .lock()
            .unwrap()
            .push(GpuCall::Reset(display_id.map(|s| s.to_string())));
        *self.vibrance_level.lock().unwrap() = 0;
        Ok(())
    }
}
