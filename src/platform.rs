#[cfg(windows)]
pub mod windows {
    use std::ffi::c_void;
    use log::info;

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
        fn PostMessageW(hwnd: *mut c_void, msg: u32, wparam: usize, lparam: isize) -> i32;
        fn GetForegroundWindow() -> *mut c_void;
        fn GetWindowThreadProcessId(hwnd: *mut c_void, lpdw_process_id: *mut u32) -> u32;
        fn EnumWindows(
            lp_enum_func: unsafe extern "system" fn(*mut c_void, isize) -> i32,
            l_param: isize,
        ) -> i32;
        fn IsWindowVisible(hwnd: *mut c_void) -> i32;
        fn GetWindowTextW(hwnd: *mut c_void, lp_string: *mut u16, n_max_count: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: *mut c_void) -> i32;
        fn GetWindow(hwnd: *mut c_void, u_cmd: u32) -> *mut c_void;
        fn SetWinEventHook(
            event_min: u32,
            event_max: u32,
            h_module_win_event_hook: *mut c_void,
            lpfn_win_event_proc: unsafe extern "system" fn(
                *mut c_void,
                u32,
                *mut c_void,
                i32,
                i32,
                u32,
                u32,
            ),
            id_process: u32,
            id_thread: u32,
            dw_flags: u32,
        ) -> *mut c_void;
        fn UnhookWinEvent(h_win_event_hook: *mut c_void) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut c_void;
        fn CloseHandle(h_object: *mut c_void) -> i32;
        fn QueryFullProcessImageNameW(h_process: *mut c_void, dw_flags: u32, lp_exe_name: *mut u16, lpdw_size: *mut u32) -> i32;
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut c_void;
        fn Process32FirstW(h_snapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h_snapshot: *mut c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CreateMutexW(lp_mutex_attributes: *mut c_void, b_initial_owner: i32, lp_name: *const u16) -> *mut c_void;
        fn GetLastError() -> u32;
    }

    const ERROR_ALREADY_EXISTS: u32 = 183;

    pub struct SingleInstance {
        handle: *mut c_void,
    }

    impl SingleInstance {
        pub fn new(name: &str) -> Option<Self> {
            unsafe {
                // Ensure name has a prefix if it doesn't already. 
                // Using 'Local\' is safer for per-user apps as it doesn't require SeCreateGlobalPrivilege.
                let full_name = if name.starts_with("Global\\") || name.starts_with("Local\\") {
                    name.to_string()
                } else {
                    format!("Local\\{}", name)
                };

                let name_u16: Vec<u16> = full_name.encode_utf16().chain(std::iter::once(0)).collect();
                let handle = CreateMutexW(std::ptr::null_mut(), 1, name_u16.as_ptr());
                if handle.is_null() {
                    return None;
                }

                if GetLastError() == ERROR_ALREADY_EXISTS {
                    CloseHandle(handle);
                    return None;
                }

                Some(Self { handle })
            }
        }
    }

    impl Drop for SingleInstance {
        fn drop(&mut self) {
            unsafe {
                if !self.handle.is_null() {
                    CloseHandle(self.handle);
                }
            }
        }
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct PROCESSENTRY32W {
        pub dwSize: u32,
        pub cntUsage: u32,
        pub th32ProcessID: u32,
        pub th32DefaultHeapID: usize,
        pub th32ModuleID: u32,
        pub cntThreads: u32,
        pub th32ParentProcessID: u32,
        pub pcPriClassBase: i32,
        pub dwFlags: u32,
        pub szExeFile: [u16; 260],
    }

    const TH32CS_SNAPPROCESS: u32 = 0x00000002;
    const PM_REMOVE: u32 = 0x0001;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const GW_OWNER: u32 = 4;

    pub const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
    pub const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
    pub const WM_NULL: u32 = 0x0000;

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
        // First try Toolhelp32 snapshot (more resilient to Anti-Cheat/Permissions)
        if let Some(name) = get_process_name_via_snapshot(process_id) {
            return Some(name);
        }

        // Fallback to OpenProcess + QueryFullProcessImageNameW
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if handle.is_null() {
                return None;
            }

            let mut buffer = [0u16; 1024];
            let mut size = 1024u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(handle);

            if ok != 0 {
                let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
                std::path::Path::new(&full_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        }
    }

    fn get_process_name_via_snapshot(process_id: u32) -> Option<String> {
        unsafe {
            let h_snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if h_snapshot == (usize::MAX as *mut c_void) {
                return None;
            }

            let mut pe = PROCESSENTRY32W {
                dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
                cntUsage: 0,
                th32ProcessID: 0,
                th32DefaultHeapID: 0,
                th32ModuleID: 0,
                cntThreads: 0,
                th32ParentProcessID: 0,
                pcPriClassBase: 0,
                dwFlags: 0,
                szExeFile: [0u16; 260],
            };

            let mut ok = Process32FirstW(h_snapshot, &mut pe);
            let mut found_name = None;
            while ok != 0 {
                if pe.th32ProcessID == process_id {
                    let name = String::from_utf16_lossy(&pe.szExeFile);
                    found_name = Some(name.trim_matches('\0').to_string());
                    break;
                }
                ok = Process32NextW(h_snapshot, &mut pe);
            }
            CloseHandle(h_snapshot);
            found_name
        }
    }

    /// Get a list of running applications with visible windows.
    pub fn get_running_apps() -> Vec<ProcessInfo> {
        let mut apps = Vec::new();

        unsafe extern "system" fn enum_windows_callback(hwnd: *mut c_void, l_param: isize) -> i32 {
            let apps = unsafe { &mut *(l_param as *mut Vec<ProcessInfo>) };

            // We want windows that are visible
            if unsafe { IsWindowVisible(hwnd) } != 0 {
                let owner = unsafe { GetWindow(hwnd, GW_OWNER) };
                
                // Usually we want top-level windows (no owner), 
                // but some games might have a dummy owner.
                let mut process_id = 0;
                unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };
                
                if let Some(name) = get_process_name_from_id(process_id) {
                    let name_lower = name.to_lowercase();
                    // Filter noise
                    if name_lower == "explorer.exe" || name_lower == "shellexperiencehost.exe" || name_lower == "searchhost.exe" {
                        return 1;
                    }

                    let length = unsafe { GetWindowTextLengthW(hwnd) };
                    let title = if length > 0 {
                        let mut buffer = vec![0u16; length as usize + 1];
                        let size = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), length + 1) };
                        String::from_utf16_lossy(&buffer[..size as usize])
                    } else {
                        // If no title, use the process name as title
                        format!("[{}]", name)
                    };

                    // Only add if it's likely a "real" window (has a title OR no owner)
                    if length > 0 || owner.is_null() {
                        if !apps.iter().any(|a: &ProcessInfo| a.name == name && a.title == title) {
                            apps.push(ProcessInfo { name, title });
                        }
                    }
                }
            }
            1
        }

        unsafe {
            EnumWindows(enum_windows_callback, &mut apps as *mut _ as isize);
        }

        // Sort by title
        apps.sort_by(|a: &ProcessInfo, b: &ProcessInfo| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
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

    /// Post a null message to wake up the message loop.
    pub fn wake_message_loop() {
        unsafe {
            PostMessageW(std::ptr::null_mut(), WM_NULL, 0, 0);
        }
    }

    /// Set a window event hook.
    pub fn set_foreground_hook(
        callback: unsafe extern "system" fn(*mut c_void, u32, *mut c_void, i32, i32, u32, u32),
    ) -> *mut c_void {
        unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                std::ptr::null_mut(),
                callback,
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        }
    }

    /// Unhook a window event hook.
    pub fn unhook_event_hook(hook: *mut c_void) {
        unsafe {
            UnhookWinEvent(hook);
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
    // Windows auto-start helper
    // ==========================

    pub fn update_auto_start(enabled: bool) -> Result<(), String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

        if enabled {
            if cfg!(debug_assertions) {
                // In debug mode, we usually don't want to register for auto-start 
                // because it might point to a transient build and cause issues at boot.
                // We return Ok(()) here to avoid error popups during development.
                info!("Auto-start registration skipped in debug mode.");
                return Ok(());
            }

            let key = hkcu
                .open_subkey_with_flags(path, KEY_WRITE)
                .or_else(|_| hkcu.create_subkey(path).map(|(k, _)| k))
                .map_err(|e| format!("Failed to open/create Run key: {}", e))?;

            let exe_path = std::env::current_exe()
                .map_err(|e| format!("Failed to get current exe path: {}", e))?;

            let mut path_str = exe_path.to_str().unwrap_or_default().to_string();
            if !path_str.starts_with('"') && path_str.contains(' ') {
                path_str = format!("\"{}\"", path_str);
            }

            // Check if already correctly registered to avoid unnecessary writes
            let existing: String = key.get_value("color-schemer").unwrap_or_default();
            if existing == path_str {
                return Ok(());
            }

            key.set_value("color-schemer", &path_str)
                .map_err(|e| format!("Failed to set registry value: {}", e))?;
            
            info!("Auto-start registered: {}", path_str);
        } else {
            if let Ok(key) = hkcu.open_subkey_with_flags(path, KEY_WRITE) {
                if key.get_value::<String, _>("color-schemer").is_ok() {
                    let _ = key.delete_value("color-schemer");
                    info!("Auto-start unregistered.");
                }
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

    pub fn wait_message() {}

    pub fn wake_message_loop() {}

    pub fn set_device_gamma_ramp(_ramp: &[[u16; 256]; 3], _device_name: Option<&str>) -> Result<(), String> {
        Err("Gamma ramp is only supported on Windows".to_string())
    }

    pub fn update_auto_start(_enabled: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn enumerate_monitors() -> Vec<MonitorData> {
        Vec::new()
    }

    pub struct SingleInstance;
    impl SingleInstance {
        pub fn new(_name: &str) -> Option<Self> {
            Some(Self)
        }
    }
}
