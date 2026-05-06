use libloading::{Library, Symbol};
use log::info;
use serde;
use std::ffi::c_void;
use std::mem;

use crate::platform;

// ============================================================================
// NVAPI типы и константы
// ============================================================================

#[allow(non_camel_case_types)]
type NvAPI_Status = i32;
const NVAPI_OK: NvAPI_Status = 0;
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
    /// Валидация настроек — возвращает исправленную копию
    pub fn validated(&self) -> Self {
        Self {
            brightness: self.brightness.clamp(0.0, 2.0),
            contrast: self.contrast.clamp(0.0, 2.0),
            gamma: self.gamma.clamp(0.1, 5.0),
            digital_vibrance: self.digital_vibrance.clamp(-1024, 1023),
        }
    }

    /// Проверить, являются ли настройки дефолтными
    pub fn is_default(&self) -> bool {
        (self.brightness - 1.0).abs() < f64::EPSILON
            && (self.contrast - 1.0).abs() < f64::EPSILON
            && (self.gamma - 1.0).abs() < f64::EPSILON
            && self.digital_vibrance == 0
    }
}

// ============================================================================
// Трейт для абстракции GPU-контроллера (для тестирования)
// ============================================================================

pub trait GpuController: Send + Sync {
    fn apply_display_settings(&self, settings: &DisplaySettings) -> Result<(), String>;
    fn set_digital_vibrance(&self, level: i32) -> Result<(), String>;
    fn get_digital_vibrance(&self) -> Result<NV_DISPLAY_DVC_INFO, String>;
    fn reset(&self) -> Result<(), String>;
}

// ============================================================================
// Вычисление gamma ramp (публичное для тестирования)
// ============================================================================

/// Вычислить gamma ramp таблицу из настроек
/// Возвращает [[u16; 256]; 3] — Red, Green, Blue каналы
pub fn compute_gamma_ramp(settings: &DisplaySettings) -> [[u16; 256]; 3] {
    let mut ramp = [[0u16; 256]; 3];

    for i in 0..256 {
        let normalized = i as f64 / 255.0;
        let gamma_corrected = normalized.powf(settings.gamma);
        let contrasted = (gamma_corrected - 0.5) * settings.contrast + 0.5;
        let bright = contrasted * settings.brightness;
        let clamped = bright.clamp(0.0, 1.0);
        let value = (clamped * 65535.0) as u16;

        ramp[0][i] = value; // Red
        ramp[1][i] = value; // Green
        ramp[2][i] = value; // Blue
    }

    ramp
}

// ============================================================================
// NvidiaController — реальная реализация
// ============================================================================

pub struct NvidiaController {
    _library: Library,
    query_interface: NvAPI_QueryInterface_t,
    display_handle: usize,
    initialized: bool,
}

// Безопасность: NvidiaController хранит указатели на функции из DLL,
// которые потокобезопасны по контракту NVAPI
unsafe impl Send for NvidiaController {}
unsafe impl Sync for NvidiaController {}

impl NvidiaController {
    pub fn new() -> Result<Self, String> {
        if !cfg!(windows) {
            return Err("NvidiaController is only supported on Windows".to_string());
        }

        info!("Загрузка NVAPI...");

        let lib_name = if cfg!(target_arch = "x86_64") {
            "nvapi64.dll"
        } else {
            "nvapi.dll"
        };

        let library = unsafe {
            Library::new(lib_name)
                .map_err(|e| format!("Не удалось загрузить {}: {}", lib_name, e))?
        };

        let query_interface: NvAPI_QueryInterface_t = unsafe {
            let sym: Symbol<NvAPI_QueryInterface_t> = library
                .get(b"nvapi_QueryInterface\0")
                .map_err(|e| format!("nvapi_QueryInterface not found: {}", e))?;
            *sym
        };

        let mut controller = Self {
            _library: library,
            query_interface,
            display_handle: 0,
            initialized: false,
        };

        controller.initialize()?;
        Ok(controller)
    }

    fn initialize(&mut self) -> Result<(), String> {
        let init_fn = self.get_function::<NvAPI_Initialize_t>(NVAPI_INITIALIZE)?;
        let status = unsafe { init_fn() };
        if status != NVAPI_OK {
            return Err(format!("NvAPI_Initialize failed: {}", status));
        }
        self.initialized = true;
        info!("NVAPI инициализирован");

        self.display_handle = self.get_display_handle(0)?;
        info!("Display handle: 0x{:X}", self.display_handle);
        Ok(())
    }

    fn get_function<T>(&self, id: u32) -> Result<T, String> {
        let ptr = unsafe { (self.query_interface)(id) };
        if ptr.is_null() {
            return Err(format!("Function 0x{:08X} not found", id));
        }
        Ok(unsafe { mem::transmute_copy(&ptr) })
    }

    fn get_display_handle(&self, index: u32) -> Result<usize, String> {
        let enum_fn =
            self.get_function::<NvAPI_EnumNvidiaDisplayHandle_t>(NVAPI_ENUM_NVIDIA_DISPLAY_HANDLE)?;
        let mut handle: usize = 0;
        let status = unsafe { enum_fn(index, &mut handle) };
        if status != NVAPI_OK {
            return Err(format!("EnumNvidiaDisplayHandle failed: {}", status));
        }
        Ok(handle)
    }

    fn set_gamma_ramp(settings: &DisplaySettings) -> Result<(), String> {
        let ramp = compute_gamma_ramp(settings);
        platform::windows::set_device_gamma_ramp(&ramp)
    }

    pub fn reset_gamma_ramp() -> Result<(), String> {
        Self::set_gamma_ramp(&DisplaySettings::default())
    }
}

impl GpuController for NvidiaController {
    fn apply_display_settings(&self, settings: &DisplaySettings) -> Result<(), String> {
        let validated = settings.validated();
        self.set_digital_vibrance(validated.digital_vibrance)?;
        Self::set_gamma_ramp(&validated)?;
        info!("Настройки применены: {:?}", validated);
        Ok(())
    }

    fn set_digital_vibrance(&self, level: i32) -> Result<(), String> {
        let set_fn = self.get_function::<NvAPI_SetDVCLevel_t>(NVAPI_SET_DVC_LEVEL)?;
        let status = unsafe { set_fn(self.display_handle, 0, level) };
        if status != NVAPI_OK {
            return Err(format!("SetDVCLevel failed: {}", status));
        }
        info!("Digital Vibrance: {}", level);
        Ok(())
    }

    fn get_digital_vibrance(&self) -> Result<NV_DISPLAY_DVC_INFO, String> {
        let get_fn = self.get_function::<NvAPI_GetDVCInfo_t>(NVAPI_GET_DVC_INFO)?;
        let mut info = NV_DISPLAY_DVC_INFO::default();
        let status = unsafe { get_fn(self.display_handle, 0, &mut info) };
        if status != NVAPI_OK {
            return Err(format!("GetDVCInfo failed: {}", status));
        }
        Ok(info)
    }

    fn reset(&self) -> Result<(), String> {
        self.set_digital_vibrance(0)?;
        Self::reset_gamma_ramp()?;
        Ok(())
    }
}

impl Drop for NvidiaController {
    fn drop(&mut self) {
        if self.initialized {
            if let Ok(unload_fn) = self.get_function::<NvAPI_Unload_t>(NVAPI_UNLOAD) {
                unsafe {
                    unload_fn();
                }
                info!("NVAPI выгружен");
            }
        }
    }
}
