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
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
        fn EnumWindows(
            lp_enum_func: unsafe extern "system" fn(*mut c_void, isize) -> i32,
            l_param: isize,
        ) -> i32;
        fn IsWindowVisible(hwnd: *mut c_void) -> i32;
        fn GetWindowTextW(hwnd: *mut c_void, lp_string: *mut u16, n_max_count: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: *mut c_void) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut c_void;
        fn CloseHandle(h_object: *mut c_void) -> i32;
    }

    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetModuleFileNameExW(
            h_process: *mut c_void,
            h_module: *mut c_void,
            lp_filename: *mut u16,
            n_size: u32,
        ) -> u32;
    }

    const PM_REMOVE: u32 = 0x0001;
    const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
    const PROCESS_VM_READ: u32 = 0x0010;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ProcessInfo {
        pub name: String,
        pub title: String,
    }

    /// Get the executable name of the process owning the foreground window.
    pub fn get_foreground_process_name() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            let mut process_id = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == 0 {
                return None;
            }

            get_process_name_from_id(process_id)
        }
    }

    fn get_process_name_from_id(process_id: u32) -> Option<String> {
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);
            if handle.is_null() {
                return None;
            }

            let mut buffer = [0u16; 1024];
            let size = GetModuleFileNameExW(handle, std::ptr::null_mut(), buffer.as_mut_ptr(), 1024);
            CloseHandle(handle);

            if size > 0 {
                let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                // Return only the filename
                std::path::Path::new(&full_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        }
    }

    /// Get a list of running applications with visible windows.
    pub fn get_running_apps() -> Vec<ProcessInfo> {
        let mut apps = Vec::new();

        unsafe extern "system" fn enum_windows_callback(hwnd: *mut c_void, l_param: isize) -> i32 {
            let apps = unsafe { &mut *(l_param as *mut Vec<ProcessInfo>) };

            if unsafe { IsWindowVisible(hwnd) } != 0 {
                let length = unsafe { GetWindowTextLengthW(hwnd) };
                if length > 0 {
                    let mut buffer = vec![0u16; length as usize + 1];
                    let size = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), length + 1) };
                    let title = String::from_utf16_lossy(&buffer[..size as usize]);

                    let mut process_id = 0;
                    unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
                    
                    if let Some(name) = get_process_name_from_id(process_id) {
                        apps.push(ProcessInfo { name, title });
                    }
                }
            }
            1
        }

        unsafe {
            EnumWindows(enum_windows_callback, &mut apps as *mut _ as isize);
        }

        apps
    }

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
        fn EnumDisplayMonitors(
            hdc: *mut c_void,
            lprc_clip: *const c_void,
            lpfn_enum: unsafe extern "system" fn(*mut c_void, *mut c_void, *mut RECT, isize) -> i32,
            dw_data: isize,
        ) -> i32;
        fn GetMonitorInfoW(h_monitor: *mut c_void, lpmi: *mut MONITORINFOEXW) -> i32;
        fn EnumDisplayDevicesW(
            lp_device: *const u16,
            i_dev_num: u32,
            lp_display_device: *mut DISPLAY_DEVICEW,
            dw_flags: u32,
        ) -> i32;
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct DISPLAY_DEVICEW {
        pub cb: u32,
        pub DeviceName: [u16; 32],
        pub DeviceString: [u16; 128],
        pub StateFlags: u32,
        pub DeviceID: [u16; 128],
        pub DeviceKey: [u16; 128],
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn SetDeviceGammaRamp(hdc: *mut c_void, lpRamp: *const c_void) -> i32;
        fn CreateDCW(
            lp_driver: *const u16,
            lp_device: *const u16,
            lp_output: *const u16,
            lp_init_data: *const c_void,
        ) -> *mut c_void;
        fn DeleteDC(hdc: *mut c_void) -> i32;
    }

    #[repr(C)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct MONITORINFOEXW {
        pub cbSize: u32,
        pub rcMonitor: RECT,
        pub rcWork: RECT,
        pub dwFlags: u32,
        pub szDevice: [u16; 32],
    }

    pub struct MonitorData {
        pub device_id: String,
        pub friendly_name: String,
        pub is_primary: bool,
    }

    pub fn enumerate_monitors() -> Vec<MonitorData> {
        let mut monitors = Vec::new();

        unsafe extern "system" fn enum_callback(
            h_monitor: *mut c_void,
            _hdc_monitor: *mut c_void,
            _lprc_monitor: *mut RECT,
            dw_data: isize,
        ) -> i32 {
            let monitors = unsafe { &mut *(dw_data as *mut Vec<MonitorData>) };
            let mut info = MONITORINFOEXW {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                dwFlags: 0,
                szDevice: [0u16; 32],
            };

            if unsafe { GetMonitorInfoW(h_monitor, &mut info) } != 0 {
                let device_id = String::from_utf16_lossy(&info.szDevice)
                    .trim_matches('\0')
                    .to_string();
                
                let mut friendly_name = device_id.clone();
                let mut device = DISPLAY_DEVICEW {
                    cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                    DeviceName: [0u16; 32],
                    DeviceString: [0u16; 128],
                    StateFlags: 0,
                    DeviceID: [0u16; 128],
                    DeviceKey: [0u16; 128],
                };

                if unsafe { EnumDisplayDevicesW(info.szDevice.as_ptr(), 0, &mut device, 0) } != 0 {
                    let name = String::from_utf16_lossy(&device.DeviceString)
                        .trim_matches('\0')
                        .to_string();
                    if !name.is_empty() {
                        friendly_name = name;
                    }
                }

                monitors.push(MonitorData {
                    device_id,
                    friendly_name,
                    is_primary: (info.dwFlags & 1) != 0,
                });
            }
            1
        }

        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null(),
                enum_callback,
                &mut monitors as *mut _ as isize,
            );
        }

        monitors
    }

    pub fn set_device_gamma_ramp(ramp: &[[u16; 256]; 3], device_name: Option<&str>) -> Result<(), String> {
        unsafe {
            let hdc = if let Some(name) = device_name {
                let name_u16: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
                let hdc = CreateDCW(
                    std::ptr::null(),
                    name_u16.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                );
                if hdc.is_null() {
                    return Err(format!("CreateDCW failed for device: {}", name));
                }
                hdc
            } else {
                let hdc = GetDC(std::ptr::null_mut());
                if hdc.is_null() {
                    return Err("GetDC failed".to_string());
                }
                hdc
            };

            let ok = SetDeviceGammaRamp(hdc, ramp.as_ptr() as *const c_void);
            
            if device_name.is_some() {
                DeleteDC(hdc);
            } else {
                ReleaseDC(std::ptr::null_mut(), hdc);
            }

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
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ProcessInfo {
        pub name: String,
        pub title: String,
    }

    pub fn get_foreground_process_name() -> Option<String> {
        None
    }

    pub fn get_running_apps() -> Vec<ProcessInfo> {
        Vec::new()
    }

    pub fn pump_messages() {}

    pub fn set_device_gamma_ramp(_ramp: &[[u16; 256]; 3]) -> Result<(), String> {
        Err("Gamma ramp is only supported on Windows".to_string())
    }

    pub fn show_notification(_title: &str, _message: &str) {}

    pub fn update_auto_start(_enabled: bool) -> Result<(), String> {
        Ok(())
    }
}

