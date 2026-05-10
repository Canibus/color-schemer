#[cfg(windows)]
pub mod windows {
    use std::ffi::c_void;

    // ==========================
    // WinAPI: message pump
    // ==========================

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
        fn PeekMessageW(
            msg: *mut MSG,
            hwnd: *mut c_void,
            filter_min: u32,
            filter_max: u32,
            remove: u32,
        ) -> i32;
        fn TranslateMessage(msg: *const MSG) -> i32;
        fn DispatchMessageW(msg: *const MSG) -> isize;
        fn WaitMessage() -> i32;
    }

    const PM_REMOVE: u32 = 0x0001;

    /// Process all pending Windows messages (non-blocking).
    pub fn pump_messages() {
        unsafe {
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    /// Wait for a new message to arrive in the queue (blocking).
    pub fn wait_message() {
        unsafe {
            WaitMessage();
        }
    }

    // ==========================
    // WinAPI: gamma ramp
    // ==========================

    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetDC(hwnd: *mut c_void) -> *mut c_void;
        fn ReleaseDC(hwnd: *mut c_void, hdc: *mut c_void) -> i32;
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn SetDeviceGammaRamp(hdc: *mut c_void, lpRamp: *const c_void) -> i32;
    }

    pub fn set_device_gamma_ramp(ramp: &[[u16; 256]; 3]) -> Result<(), String> {
        unsafe {
            let hdc = GetDC(std::ptr::null_mut());
            if hdc.is_null() {
                return Err("GetDC failed".to_string());
            }

            let ok = SetDeviceGammaRamp(hdc, ramp.as_ptr() as *const c_void);
            ReleaseDC(std::ptr::null_mut(), hdc);

            if ok == 0 {
                return Err("SetDeviceGammaRamp failed".to_string());
            }
        }

        Ok(())
    }

    // ==========================
    // Windows toast notification
    // ==========================

    #[cfg(feature = "app")]
    pub fn show_notification(title: &str, message: &str) {
        use winrt_notification::Toast;

        // Best-effort only: ignore failures to avoid crashing the resident app.
        let _ = Toast::new(Toast::POWERSHELL_APP_ID)
            .title(title)
            .text1(message)
            .duration(winrt_notification::Duration::Short)
            .show();
    }

    #[cfg(not(feature = "app"))]
    pub fn show_notification(_title: &str, _message: &str) {}

    // ==========================
    // Windows auto-start helper
    // ==========================

    pub fn update_auto_start(enabled: bool) -> Result<(), String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

        if enabled {
            let key = hkcu
                .open_subkey_with_flags(path, KEY_WRITE)
                .or_else(|_| hkcu.create_subkey(path).map(|(k, _)| k))
                .map_err(|e| format!("Failed to open/create Run key: {}", e))?;

            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get current exe path: {}", e))?;

            key.set_value("color-schemer", &exe_path.to_str().unwrap_or_default())
                .map_err(|e| format!("Failed to set registry value: {}", e))?;
        } else {
            if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
                let _ = key.delete_value("color-schemer");
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
pub mod windows {
    pub fn pump_messages() {}

    pub fn set_device_gamma_ramp(_ramp: &[[u16; 256]; 3]) -> Result<(), String> {
        Err("Gamma ramp is only supported on Windows".to_string())
    }

    pub fn show_notification(_title: &str, _message: &str) {}

    pub fn update_auto_start(_enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

