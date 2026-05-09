use libloading::{Library, Symbol};
use log::info;
use serde;
use std::ffi::c_void;
use std::mem;
use thiserror::Error;

use crate::platform;

// ============================================================================
// NVAPI типы и константы
// ============================================================================

#[allow(non_camel_case_types)]
pub type NvAPI_Status = i32;
pub const NVAPI_OK: NvAPI_Status = 0;
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
// Errors
// ============================================================================

#[derive(Error, Debug)]
pub enum NvError {
    #[error("NVAPI error {0}")]
    Status(NvAPI_Status),
    #[error("Library load error: {0}")]
    Library(String),
    #[error("Function 0x{0:08X} not found")]
    NotFound(u32),
    #[error("OS error: {0}")]
    Os(String),
    #[error("Not supported on this platform")]
    NotSupported,
}

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
    fn apply_display_settings(&self, settings: &DisplaySettings) -> NvResult<()>;
    fn set_digital_vibrance(&self, level: i32) -> NvResult<()>;
    fn get_digital_vibrance(&self) -> NvResult<NV_DISPLAY_DVC_INFO>;
    fn reset(&self) -> NvResult<()>;
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
    display_handle: usize,
    initialized: bool,

    // Cached function pointers
    fn_unload: NvAPI_Unload_t,
    fn_set_dvc_level: NvAPI_SetDVCLevel_t,
    fn_get_dvc_info: NvAPI_GetDVCInfo_t,
}

// Безопасность: NvidiaController хранит указатели на функции из DLL,
// которые потокобезопасны по контракту NVAPI
unsafe impl Send for NvidiaController {}
unsafe impl Sync for NvidiaController {}

impl NvidiaController {
    pub fn new() -> NvResult<Self> {
        if !cfg!(windows) {
            return Err(NvError::NotSupported);
        }

        info!("Загрузка NVAPI...");

        let lib_name = if cfg!(target_arch = "x86_64") {
            "nvapi64.dll"
        } else {
            "nvapi.dll"
        };

        let library = unsafe {
            Library::new(lib_name)
                .map_err(|e| NvError::Library(format!("Не удалось загрузить {}: {}", lib_name, e)))?
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

        let enum_fn = Self::get_func_ptr::<NvAPI_EnumNvidiaDisplayHandle_t>(
            query_interface,
            NVAPI_ENUM_NVIDIA_DISPLAY_HANDLE,
        )?;
        let mut display_handle: usize = 0;
        let status = unsafe { enum_fn(0, &mut display_handle) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }

        let controller = Self {
            _library: library,
            display_handle,
            initialized: true,
            fn_unload: Self::get_func_ptr(query_interface, NVAPI_UNLOAD)?,
            fn_set_dvc_level: Self::get_func_ptr(query_interface, NVAPI_SET_DVC_LEVEL)?,
            fn_get_dvc_info: Self::get_func_ptr(query_interface, NVAPI_GET_DVC_INFO)?,
        };

        info!("NVAPI инициализирован. Display handle: 0x{:X}", display_handle);
        Ok(controller)
    }

    fn get_func_ptr<T>(qi: NvAPI_QueryInterface_t, id: u32) -> NvResult<T> {
        let ptr = unsafe { qi(id) };
        if ptr.is_null() {
            return Err(NvError::NotFound(id));
        }
        Ok(unsafe { mem::transmute_copy(&ptr) })
    }

    fn set_gamma_ramp(settings: &DisplaySettings) -> NvResult<()> {
        let ramp = compute_gamma_ramp(settings);
        platform::windows::set_device_gamma_ramp(&ramp).map_err(NvError::Os)
    }

    pub fn reset_gamma_ramp() -> NvResult<()> {
        Self::set_gamma_ramp(&DisplaySettings::default())
    }
}

impl GpuController for NvidiaController {
    fn apply_display_settings(&self, settings: &DisplaySettings) -> NvResult<()> {
        let validated = settings.validated();
        self.set_digital_vibrance(validated.digital_vibrance)?;
        Self::set_gamma_ramp(&validated)?;
        info!("Настройки применены: {:?}", validated);
        Ok(())
    }

    fn set_digital_vibrance(&self, level: i32) -> NvResult<()> {
        let status = unsafe { (self.fn_set_dvc_level)(self.display_handle, 0, level) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }
        info!("Digital Vibrance: {}", level);
        Ok(())
    }

    fn get_digital_vibrance(&self) -> NvResult<NV_DISPLAY_DVC_INFO> {
        let mut info = NV_DISPLAY_DVC_INFO::default();
        let status = unsafe { (self.fn_get_dvc_info)(self.display_handle, 0, &mut info) };
        if status != NVAPI_OK {
            return Err(NvError::Status(status));
        }
        Ok(info)
    }

    fn reset(&self) -> NvResult<()> {
        self.set_digital_vibrance(0)?;
        Self::reset_gamma_ramp()?;
        Ok(())
    }
}

impl Drop for NvidiaController {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                (self.fn_unload)();
            }
            info!("NVAPI выгружен");
        }
    }
}
