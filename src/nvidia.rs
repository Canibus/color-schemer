use libloading::{Library, Symbol};
use log::info;
use serde;
use std::ffi::c_void;
use std::mem;

use crate::platform;

// ============================================================================
// NVAPI types and constants
// ============================================================================

#[allow(non_camel_case_types)]
pub type NvAPI_Status = i32;
pub const NVAPI_OK: NvAPI_Status = 0;
pub const NVAPI_ERROR: NvAPI_Status = -1;
pub const NVAPI_LIBRARY_NOT_FOUND: NvAPI_Status = -2;
pub const NVAPI_NO_IMPLEMENTATION: NvAPI_Status = -3;
pub const NVAPI_API_NOT_INITIALIZED: NvAPI_Status = -4;
pub const NVAPI_INVALID_ARGUMENT: NvAPI_Status = -5;
pub const NVAPI_NVIDIA_DEVICE_NOT_FOUND: NvAPI_Status = -6;
pub const NVAPI_END_ENUMERATION: NvAPI_Status = -7;
pub const NVAPI_INVALID_HANDLE: NvAPI_Status = -8;
pub const NVAPI_INCOMPATIBLE_STRUCT_VERSION: NvAPI_Status = -9;
pub const NVAPI_HANDLE_INVALIDATED: NvAPI_Status = -10;
pub const NVAPI_OPENGL_CONTEXT_NOT_CURRENT: NvAPI_Status = -11;
pub const NVAPI_INVALID_POINTER: NvAPI_Status = -14;
pub const NVAPI_EXPECTED_LOGICAL_GPU_HANDLE: NvAPI_Status = -100;
pub const NVAPI_EXPECTED_PHYSICAL_GPU_HANDLE: NvAPI_Status = -101;
pub const NVAPI_EXPECTED_DISPLAY_HANDLE: NvAPI_Status = -102;
pub const NVAPI_NOT_SUPPORTED: NvAPI_Status = -148;
pub const NVAPI_INVALID_USER_PRIVILEGE: NvAPI_Status = -111;
pub const NVAPI_DATA_NOT_FOUND: NvAPI_Status = -133;
pub const NVAPI_SETTING_NOT_FOUND: NvAPI_Status = -152;
pub const NVAPI_MOSAIC_NOT_ACTIVE: NvAPI_Status = -112;
pub const NVAPI_ACCESS_DENIED: NvAPI_Status = -151;

pub fn nvapi_status_to_string(status: NvAPI_Status) -> &'static str {
    match status {
        NVAPI_OK => "Success",
        NVAPI_ERROR => "Generic error",
        NVAPI_LIBRARY_NOT_FOUND => "NVAPI library not found",
        NVAPI_NO_IMPLEMENTATION => "Function not implemented",
        NVAPI_API_NOT_INITIALIZED => "NVAPI not initialized",
        NVAPI_INVALID_ARGUMENT => "Invalid argument",
        NVAPI_NVIDIA_DEVICE_NOT_FOUND => "NVIDIA device not found",
        NVAPI_END_ENUMERATION => "End of enumeration",
        NVAPI_INVALID_HANDLE => "Invalid handle",
        NVAPI_INCOMPATIBLE_STRUCT_VERSION => "Incompatible structure version",
        NVAPI_HANDLE_INVALIDATED => "Handle invalidated",
        NVAPI_OPENGL_CONTEXT_NOT_CURRENT => "OpenGL context not current",
        NVAPI_INVALID_POINTER => "Invalid pointer",
        NVAPI_EXPECTED_LOGICAL_GPU_HANDLE => "Expected logical GPU handle",
        NVAPI_EXPECTED_PHYSICAL_GPU_HANDLE => "Expected physical GPU handle",
        NVAPI_EXPECTED_DISPLAY_HANDLE => "Expected display handle",
        NVAPI_NOT_SUPPORTED => "Not supported",
        NVAPI_INVALID_USER_PRIVILEGE => "Invalid user privilege",
        NVAPI_DATA_NOT_FOUND => "Data not found",
        NVAPI_SETTING_NOT_FOUND => "Setting not found",
        NVAPI_MOSAIC_NOT_ACTIVE => "Mosaic not active",
        NVAPI_ACCESS_DENIED => "Access denied",
        _ => "Unknown NVAPI error",
    }
}

const NV_DISPLAY_DVC_INFO_VER: u32 = 0x10010;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NV_DISPLAY_DVC_INFO {
    pub version: u32,
    pub current_level: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub default_level: i32,
}

impl Default for NV_DISPLAY_DVC_INFO {
    fn default() -> Self {
        Self {
            version: NV_DISPLAY_DVC_INFO_VER,
            current_level: 0,
            min_level: 0,
            max_level: 0,
            default_level: 0,
        }
    }
}

// NVAPI Function IDs
const NVAPI_INITIALIZE: u32 = 0x0150E828;
const NVAPI_UNLOAD: u32 = 0xD22BDD7E;
const NVAPI_ENUM_NVIDIA_DISPLAY_HANDLE: u32 = 0x9ABDD40D;
const NVAPI_GET_DVC_INFO: u32 = 0x4085DE45;
const NVAPI_SET_DVC_LEVEL: u32 = 0x172409B4;
const NVAPI_GET_ASSOCIATED_NVIDIA_DISPLAY_HANDLE: u32 = 0x35C29134;

#[allow(non_camel_case_types)]
type NvAPI_QueryInterface_t = unsafe extern "C" fn(u32) -> *const c_void;
#[allow(non_camel_case_types)]
type NvAPI_Initialize_t = unsafe extern "C" fn() -> NvAPI_Status;
#[allow(non_camel_case_types)]
type NvAPI_Unload_t = unsafe extern "C" fn() -> NvAPI_Status;
#[allow(non_camel_case_types)]
type NvAPI_EnumNvidiaDisplayHandle_t = unsafe extern "C" fn(u32, *mut usize) -> NvAPI_Status;
#[allow(non_camel_case_types)]
type NvAPI_GetDVCInfo_t =
    unsafe extern "C" fn(usize, u32, *mut NV_DISPLAY_DVC_INFO) -> NvAPI_Status;
#[allow(non_camel_case_types)]
type NvAPI_SetDVCLevel_t = unsafe extern "C" fn(usize, u32, i32) -> NvAPI_Status;
#[allow(non_camel_case_types)]
type NvAPI_GetAssociatedNvidiaDisplayHandle_t =
    unsafe extern "C" fn(*const i8, *mut usize) -> NvAPI_Status;

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug)]
pub enum NvError {
    Status(NvAPI_Status),
    Library(String),
    NotFound(u32),
    Os(String),
    NotSupported,
}

impl std::fmt::Display for NvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NvError::Status(s) => write!(f, "NVAPI Error ({}): {}", s, nvapi_status_to_string(*s)),
            NvError::Library(s) => write!(f, "Library load error: {}", s),
            NvError::NotFound(id) => write!(f, "Function 0x{:08X} not found", id),
            NvError::Os(s) => write!(f, "OS error: {}", s),
            NvError::NotSupported => write!(f, "Not supported on this platform"),
        }
    }
}

impl std::error::Error for NvError {}

pub type NvResult<T> = Result<T, NvError>;

// ============================================================================
// DisplaySettings
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DisplaySettings {
    pub brightness: f64,
    pub contrast: f64,
    pub gamma: f64,
    pub digital_vibrance: i32,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            contrast: 1.0,
            gamma: 1.0,
            digital_vibrance: 0,
        }
    }
}

impl DisplaySettings {
    /// Settings validation - returns a corrected copy
    pub fn validated(&self) -> Self {
        Self {
            brightness: self.brightness.clamp(0.0, 2.0),
            contrast: self.contrast.clamp(0.0, 2.0),
            gamma: self.gamma.clamp(0.1, 5.0),
            digital_vibrance: self.digital_vibrance.clamp(-1024, 1023),
        }
    }

    /// Check if settings are default
    pub fn is_default(&self) -> bool {
        (self.brightness - 1.0).abs() < f64::EPSILON
            && (self.contrast - 1.0).abs() < f64::EPSILON
            && (self.gamma - 1.0).abs() < f64::EPSILON
            && self.digital_vibrance == 0
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayInfo {
    pub id: String, // e.g. "\\.\DISPLAY1"
    pub name: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GpuInfo {
    pub name: String,
    pub is_mock: bool,
}

// ============================================================================
// Trait for GPU controller abstraction (for testing)
// ============================================================================

pub trait GpuController: Send + Sync {
    fn get_displays(&self) -> NvResult<Vec<DisplayInfo>>;
    fn apply_display_settings(
        &self,
        display_id: Option<&str>,
        settings: &DisplaySettings,
    ) -> NvResult<()>;
    fn set_digital_vibrance(&self, display_handle: usize, level: i32) -> NvResult<()>;
    fn get_digital_vibrance(&self, display_handle: usize) -> NvResult<NV_DISPLAY_DVC_INFO>;
    fn reset(&self, display_id: Option<&str>) -> NvResult<()>;
    fn get_info(&self) -> GpuInfo;
}

// ============================================================================
// Gamma ramp computation (public for testing)
// ============================================================================

/// Compute gamma ramp table from settings
/// Returns [[u16; 256]; 3] - Red, Green, Blue channels
pub fn compute_gamma_ramp(settings: &DisplaySettings) -> [[u16; 256]; 3] {
    let mut ramp = [[0u16; 256]; 3];

    for i in 0..256 {
        let normalized = i as f64 / 255.0;
        let gamma_corrected = normalized.powf(settings.gamma);
        let contrasted = (gamma_corrected - 0.5) * settings.contrast + 0.5;
        let bright = contrasted * settings.brightness;
        let clamped = bright.clamp(0.0, 1.0);
        let value = (clamped * 65535.0) as u16;

        for row in &mut ramp {
            row[i] = value;
        }
    }

    ramp
}

// ============================================================================
// NvidiaController - real implementation
// ============================================================================

pub struct NvidiaController {
    _library: Library,
    initialized: bool,
    handle_cache: std::sync::Mutex<std::collections::HashMap<String, usize>>,

    // Cached function pointers
    fn_unload: NvAPI_Unload_t,
    fn_set_dvc_level: NvAPI_SetDVCLevel_t,
    fn_get_dvc_info: NvAPI_GetDVCInfo_t,
    fn_get_associated_handle: NvAPI_GetAssociatedNvidiaDisplayHandle_t,
    fn_enum_display: NvAPI_EnumNvidiaDisplayHandle_t,
}

// Safety: NvidiaController stores function pointers from DLL,
// which are thread-safe by NVAPI contract
unsafe impl Send for NvidiaController {}
unsafe impl Sync for NvidiaController {}

impl NvidiaController {
    pub fn new() -> NvResult<Self> {
        if !cfg!(windows) {
            return Err(NvError::NotSupported);
        }

        info!("Loading NVAPI...");

        let lib_name = if cfg!(target_arch = "x86_64") {
            "nvapi64.dll"
        } else {
            "nvapi.dll"
        };

        let library = unsafe {
            Library::new(lib_name)
                .map_err(|e| NvError::Library(format!("Failed to load {}: {}", lib_name, e)))?
        };

        let query_interface: NvAPI_QueryInterface_t = unsafe {
            let sym: Symbol<NvAPI_QueryInterface_t> = library
                .get(b"nvapi_QueryInterface\0")
                .map_err(|e| NvError::Library(format!("nvapi_QueryInterface not found: {}", e)))?;
            *sym
        };

        // Resolve functions temporary to initialize
        let init_fn = Self::get_func_ptr::<NvAPI_Initialize_t>(query_interface, NVAPI_INITIALIZE)?;
        let status = unsafe { init_fn() };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }

        let controller = Self {
            _library: library,
            initialized: true,
            handle_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            fn_unload: Self::get_func_ptr(query_interface, NVAPI_UNLOAD)?,
            fn_set_dvc_level: Self::get_func_ptr(query_interface, NVAPI_SET_DVC_LEVEL)?,
            fn_get_dvc_info: Self::get_func_ptr(query_interface, NVAPI_GET_DVC_INFO)?,
            fn_get_associated_handle: Self::get_func_ptr(
                query_interface,
                NVAPI_GET_ASSOCIATED_NVIDIA_DISPLAY_HANDLE,
            )?,
            fn_enum_display: Self::get_func_ptr(query_interface, NVAPI_ENUM_NVIDIA_DISPLAY_HANDLE)?,
        };

        info!("NVAPI initialized.");
        Ok(controller)
    }

    fn get_func_ptr<T>(qi: NvAPI_QueryInterface_t, id: u32) -> NvResult<T> {
        let ptr = unsafe { qi(id) };
        if ptr.is_null() {
            return Err(NvError::NotFound(id));
        }
        Ok(unsafe { mem::transmute_copy(&ptr) })
    }

    fn set_gamma_ramp(settings: &DisplaySettings, device_name: Option<&str>) -> NvResult<()> {
        let ramp = compute_gamma_ramp(settings);
        platform::windows::set_device_gamma_ramp(&ramp, device_name).map_err(NvError::Os)
    }

    pub fn reset_gamma_ramp(device_name: Option<&str>) -> NvResult<()> {
        Self::set_gamma_ramp(&DisplaySettings::default(), device_name)
    }

    fn get_nv_handle(&self, display_id: &str) -> NvResult<usize> {
        if let Some(handle) = self
            .handle_cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(display_id).copied())
        {
            return Ok(handle);
        }

        let mut handle: usize = 0;
        let c_id = std::ffi::CString::new(display_id).map_err(|e| NvError::Os(e.to_string()))?;
        let status = unsafe { (self.fn_get_associated_handle)(c_id.as_ptr(), &mut handle) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }

        if let Ok(mut cache) = self.handle_cache.lock() {
            cache.insert(display_id.to_string(), handle);
        }

        Ok(handle)
    }
}

impl GpuController for NvidiaController {
    fn get_info(&self) -> GpuInfo {
        GpuInfo {
            name: "NVIDIA".to_string(),
            is_mock: false,
        }
    }

    fn get_displays(&self) -> NvResult<Vec<DisplayInfo>> {
        let monitors = platform::windows::enumerate_monitors();
        let mut displays = Vec::new();

        for m in monitors {
            // Check if it's an NVIDIA display by trying to get its handle
            if self.get_nv_handle(&m.device_id).is_ok() {
                displays.push(DisplayInfo {
                    id: m.device_id,
                    name: m.friendly_name,
                    is_primary: m.is_primary,
                });
            }
        }

        Ok(displays)
    }

    fn apply_display_settings(
        &self,
        display_id: Option<&str>,
        settings: &DisplaySettings,
    ) -> NvResult<()> {
        let validated = settings.validated();

        if let Some(id) = display_id {
            let handle = self.get_nv_handle(id)?;
            self.set_digital_vibrance(handle, validated.digital_vibrance)?;
            Self::set_gamma_ramp(&validated, Some(id))?;
        } else {
            // Apply to all NVIDIA displays
            let mut i = 0;
            let mut handle: usize = 0;
            while unsafe { (self.fn_enum_display)(i, &mut handle) } == NVAPI_OK {
                // Note: enum_display doesn't give us the WinAPI name easily,
                // but we can apply vibrance by handle.
                // For gamma ramp, if we don't have the name, we apply to default DC (all? or primary?).
                // Better to use get_displays and loop.
                self.set_digital_vibrance(handle, validated.digital_vibrance)?;
                i += 1;
            }
            // Apply gamma ramp to default (usually primary or all depending on OS/driver)
            Self::set_gamma_ramp(&validated, None)?;
        }

        info!("Settings applied for {:?}: {:?}", display_id, validated);
        Ok(())
    }

    fn set_digital_vibrance(&self, display_handle: usize, level: i32) -> NvResult<()> {
        let status = unsafe { (self.fn_set_dvc_level)(display_handle, 0, level) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }
        info!(
            "Digital Vibrance set for handle 0x{:X}: {}",
            display_handle, level
        );
        Ok(())
    }

    fn get_digital_vibrance(&self, display_handle: usize) -> NvResult<NV_DISPLAY_DVC_INFO> {
        let mut info = NV_DISPLAY_DVC_INFO::default();
        let status = unsafe { (self.fn_get_dvc_info)(display_handle, 0, &mut info) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }
        Ok(info)
    }

    fn reset(&self, display_id: Option<&str>) -> NvResult<()> {
        if let Some(id) = display_id {
            let handle = self.get_nv_handle(id)?;
            self.set_digital_vibrance(handle, 0)?;
            Self::reset_gamma_ramp(Some(id))?;
        } else {
            let mut i = 0;
            let mut handle: usize = 0;
            while unsafe { (self.fn_enum_display)(i, &mut handle) } == NVAPI_OK {
                self.set_digital_vibrance(handle, 0)?;
                i += 1;
            }
            Self::reset_gamma_ramp(None)?;
        }
        Ok(())
    }
}

impl Drop for NvidiaController {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                (self.fn_unload)();
            }
            info!("NVAPI unloaded");
        }
    }
}
