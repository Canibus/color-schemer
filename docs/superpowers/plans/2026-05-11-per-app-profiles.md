# Per-Application Profile Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable automatic display profile switching based on the active application, including a "Pick from list" feature in the GUI.

**Architecture:** 
1. Update configuration to store application lists per profile.
2. Implement WinAPI-based active window detection and process enumeration in the platform layer.
3. Introduce an `AutoSwitchManager` to manage the transition between manual and automatic profiles.
4. Integrate `SetWinEventHook` in the main loop for efficient foreground window tracking.
5. Add Tauri commands for process enumeration to support the GUI picker.

**Tech Stack:** Rust (2024), WinAPI, Tauri, Svelte 5.

---

### Task 1: Update Data Model and Configuration

**Files:**
- Modify: `src/profiles.rs`
- Modify: `src/config.rs`
- Modify: `config.toml`
- Test: `tests/profile_tests.rs`

- [ ] **Step 1: Update `DisplayProfile` struct**
Add `applications` field to `DisplayProfile` in `src/profiles.rs`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayProfile {
    pub name: String,
    pub description: String,
    pub settings: DisplaySettings,
    #[serde(default)]
    pub target_displays: Vec<String>,
    #[serde(default)]
    pub applications: Vec<String>, // Added field
}
```

- [ ] **Step 2: Update `DisplayProfile::new`**
Update the constructor in `src/profiles.rs`.

```rust
impl DisplayProfile {
    pub fn new(name: &str, description: &str, settings: DisplaySettings) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            settings,
            target_displays: Vec::new(),
            applications: Vec::new(),
        }
    }
}
```

- [ ] **Step 3: Update Default Configuration**
Add dummy applications to default profiles in `src/config.rs` for testing.

```rust
// In AppConfig::default() for "Gaming" profile
DisplayProfile {
    name: "Gaming".to_string(),
    applications: vec!["Cyberpunk2077.exe".to_string()],
    // ...
}
```

- [ ] **Step 4: Update `config.toml`**
Add an application entry to one of the profiles in `config.toml`.

```toml
[[profiles]]
name = "Gaming"
applications = ["Notepad.exe"]
# ...
```

- [ ] **Step 5: Verify via Tests**
Run `cargo test profile_tests` to ensure serialization/deserialization still works.

---

### Task 2: Implement Windows Process Detection

**Files:**
- Modify: `src/platform.rs`

- [ ] **Step 1: Add WinAPI Imports**
Add necessary external functions to `src/platform.rs` under `windows` module.

```rust
#[link(name = "user32")]
extern "system" {
    fn GetForegroundWindow() -> *mut c_void;
    fn GetWindowThreadProcessId(hwnd: *mut c_void, pid: *mut u32) -> u32;
    fn EnumWindows(lp_enum_func: extern "system" fn(*mut c_void, isize) -> i32, l_param: isize) -> i32;
    fn IsWindowVisible(hwnd: *mut c_void) -> i32;
    fn GetWindowTextW(hwnd: *mut c_void, lp_string: *mut u16, n_max_count: i32) -> i32;
}

#[link(name = "psapi")]
extern "system" {
    fn GetModuleFileNameExW(h_process: *mut c_void, h_module: *mut c_void, lp_filename: *mut u16, n_size: u32) -> u32;
}

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(dw_desired_access: u32, b_inherit_handle: i32, dw_process_id: u32) -> *mut c_void;
    fn CloseHandle(h_object: *mut c_void) -> i32;
}

const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
const PROCESS_VM_READ: u32 = 0x0010;
```

- [ ] **Step 2: Implement `get_foreground_process_name`**
Implement the helper in `src/platform.rs`.

```rust
pub fn get_foreground_process_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() { return None; }

        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return None; }

        let h_process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if h_process.is_null() { return None; }

        let mut buffer = [0u16; 512];
        let len = GetModuleFileNameExW(h_process, std::ptr::null_mut(), buffer.as_mut_ptr(), 512);
        CloseHandle(h_process);

        if len == 0 { return None; }
        
        let full_path = String::from_utf16_lossy(&buffer[..len as usize]);
        std::path::Path::new(&full_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }
}
```

- [ ] **Step 3: Implement `get_running_apps`**
Implement process enumeration for the GUI picker.

```rust
pub struct ProcessInfo {
    pub name: String,
    pub title: String,
}

pub fn get_running_apps() -> Vec<ProcessInfo> {
    // Implementation using EnumWindows, checking IsWindowVisible, 
    // and resolving process names for windows with titles.
}
```

---

### Task 3: Implement Auto-Switch Logic

**Files:**
- Create: `src/auto_switch.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Define `AutoSwitchManager`**
Create `src/auto_switch.rs` to handle switching logic.

```rust
pub struct AutoSwitchManager {
    manual_index: usize,
    active_auto_index: Option<usize>,
    last_process: String,
}

impl AutoSwitchManager {
    pub fn new(initial_index: usize) -> Self {
        Self {
            manual_index: initial_index,
            active_auto_index: None,
            last_process: String::new(),
        }
    }

    pub fn handle_manual_switch(&mut self, index: usize) {
        self.manual_index = index;
    }

    pub fn evaluate_focus_change(&mut self, process_name: &str, profiles: &[DisplayProfile]) -> Option<usize> {
        if process_name == self.last_process { return None; }
        self.last_process = process_name.to_string();

        let matched_index = profiles.iter().position(|p| {
            p.applications.iter().any(|app| app.eq_ignore_ascii_case(process_name))
        });

        if matched_index != self.active_auto_index {
            self.active_auto_index = matched_index;
            Some(matched_index.unwrap_or(self.manual_index))
        } else {
            None
        }
    }
}
```

- [ ] **Step 2: Register module**
Add `pub mod auto_switch;` to `src/lib.rs`.

---

### Task 4: Integrate Event Hook in Main Loop

**Files:**
- Modify: `src/main.rs`
- Modify: `src/platform.rs`

- [ ] **Step 1: Add WinEventHook to `platform.rs`**
Add hook registration and message handling logic.

```rust
// In src/platform.rs
pub type WinEventProc = unsafe extern "system" fn(*mut c_void, u32, *mut c_void, i32, i32, u32, u32);

#[link(name = "user32")]
extern "system" {
    fn SetWinEventHook(event_min: u32, event_max: u32, h_module: *mut c_void, proc: WinEventProc, id_process: u32, id_thread: u32, flags: u32) -> *mut c_void;
    fn UnhookWinEvent(h_win_event_hook: *mut c_void) -> i32;
}

const EVENT_SYSTEM_FOREGROUND: u32 = 0x0003;
const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
```

- [ ] **Step 2: Update Main Loop in `src/main.rs`**
Integrate `AutoSwitchManager` and the hook. Use a static or Arc/Mutex for the callback to communicate back to the loop (or just poll `get_foreground_process_name` if hook callback is too complex for first iteration, but spec asked for Event-based).

*Note: Since `SetWinEventHook` requires a callback that might be hard to route to the main loop's channel, we'll use the hook to simply `PostMessage` or trigger a small wakeup, then the main loop will call `get_foreground_process_name()`.*

---

### Task 5: Implement GUI Picker (Tauri/Svelte)

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src/lib/types.ts`
- Modify: `src/lib/ProfileEditor.svelte`

- [ ] **Step 1: Add Tauri Command**
Expose `get_running_apps` to the frontend.

```rust
#[tauri::command]
fn get_running_apps() -> Vec<ProcessInfo> {
    platform::windows::get_running_apps()
}
```

- [ ] **Step 2: Update Frontend Types**
Update `DisplayProfile` interface in `src/lib/types.ts`.

- [ ] **Step 3: Add Application List to `ProfileEditor.svelte`**
Add a UI section to manage the `applications` array and a button to open the picker.

---

### Task 6: Final Verification and Testing

- [ ] **Step 1: Manual Test - Basic Switching**
Open Notepad.exe (if configured). Verify profile switches to Gaming. Close Notepad. Verify it reverts.
- [ ] **Step 2: Manual Test - Manual Override**
While Notepad is open, manually switch to "Night" via Tray. It should stay "Night". Alt-Tab out and back. It should switch back to "Gaming".
- [ ] **Step 3: Verification**
Run `cargo test` and `bun run tauri build` to ensure everything compiles.
