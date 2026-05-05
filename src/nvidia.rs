//! Модуль для взаимодействия с NVIDIA NVAPI
//!
//! NVAPI — это проприетарный API NVIDIA для управления GPU.
//! Мы используем динамическую загрузку nvapi64.dll (или nvapi.dll для 32-bit)

use libloading::{Library, Symbol};
use log::{debug, error, info, warn};
use std::ffi::c_void;
use std::mem;
use std::ptr;
use windows_sys::Win32::Graphics::Gdi::{GetDC, ReleaseDC};

#[link(name = "gdi32")]
unsafe extern "system" {
    fn SetDeviceGammaRamp(hdc: *mut c_void, lpRamp: *const c_void) -> i32;
}

// ============================================================================
// NVAPI Типы и константы
// ============================================================================

type NvAPI_Status = i32;
const NVAPI_OK: NvAPI_Status = 0;
const NVAPI_ERROR: NvAPI_Status = -1;
const NVAPI_MAX_PHYSICAL_GPUS: usize = 64;
const NVAPI_MAX_DISPLAYS: usize = 32;

// Версии структур
const NV_DISPLAY_DVC_INFO_VER: u32 = 0x10010; // Version 1

/// Структура информации о Digital Vibrance Control
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NV_DISPLAY_DVC_INFO {
    version: u32,
    current_level: i32,
    min_level: i32,
    max_level: i32,
    default_level: i32,
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

/// Gamma Ramp — таблица гамма-коррекции для 256 значений * 3 канала
#[repr(C)]
#[derive(Clone)]
struct NV_GAMMA_RAMP {
    red: [u16; 256],
    green: [u16; 256],
    blue: [u16; 256],
}

impl Default for NV_GAMMA_RAMP {
    fn default() -> Self {
        let mut ramp = Self {
            red: [0u16; 256],
            green: [0u16; 256],
            blue: [0u16; 256],
        };
        // Линейная гамма по умолчанию
        for i in 0..256 {
            let value = ((i as u32 * 65535) / 255) as u16;
            ramp.red[i] = value;
            ramp.green[i] = value;
            ramp.blue[i] = value;
        }
        ramp
    }
}

// ============================================================================
// Типы функций NVAPI
// ============================================================================

type NvAPI_QueryInterface_t = unsafe extern "C" fn(u32) -> *const c_void;
type NvAPI_Initialize_t = unsafe extern "C" fn() -> NvAPI_Status;
type NvAPI_Unload_t = unsafe extern "C" fn() -> NvAPI_Status;
type NvAPI_EnumPhysicalGPUs_t =
    unsafe extern "C" fn(*mut [usize; NVAPI_MAX_PHYSICAL_GPUS], *mut u32) -> NvAPI_Status;
type NvAPI_EnumNvidiaDisplayHandle_t = unsafe extern "C" fn(u32, *mut usize) -> NvAPI_Status;
type NvAPI_GetDVCInfo_t =
    unsafe extern "C" fn(usize, u32, *mut NV_DISPLAY_DVC_INFO) -> NvAPI_Status;
type NvAPI_SetDVCLevel_t = unsafe extern "C" fn(usize, u32, i32) -> NvAPI_Status;

// Function IDs для NvAPI_QueryInterface
const NVAPI_INITIALIZE: u32 = 0x0150E828;
const NVAPI_UNLOAD: u32 = 0xD22BDD7E;
const NVAPI_ENUM_PHYSICAL_GPUS: u32 = 0xE5AC921F;
const NVAPI_ENUM_NVIDIA_DISPLAY_HANDLE: u32 = 0x9ABDD40D;
const NVAPI_GET_DVC_INFO: u32 = 0x4085DE45;
const NVAPI_SET_DVC_LEVEL: u32 = 0x172409B4;

// ============================================================================
// Параметры профиля изображения
// ============================================================================

/// Настройки изображения для профиля
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DisplaySettings {
    /// Яркость: 0.0 - 2.0 (1.0 = нормальная)
    pub brightness: f64,
    /// Контраст: 0.0 - 2.0 (1.0 = нормальный)
    pub contrast: f64,
    /// Гамма: 0.1 - 5.0 (1.0 = нормальная, 2.2 = sRGB типичная)
    pub gamma: f64,
    /// Digital Vibrance: -1024 до 1023 (0 = нормальный)
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

// ============================================================================
// NvidiaController — основной контроллер
// ============================================================================

pub struct NvidiaController {
    _library: Library,
    query_interface: NvAPI_QueryInterface_t,
    display_handle: usize,
    initialized: bool,
}

impl NvidiaController {
    /// Создание и инициализация контроллера NVIDIA
    pub fn new() -> Result<Self, String> {
        info!("Загрузка NVAPI...");

        // Загружаем nvapi64.dll (или nvapi.dll для 32-bit)
        let lib_name = if cfg!(target_arch = "x86_64") {
            "nvapi64.dll"
        } else {
            "nvapi.dll"
        };

        let library = unsafe {
            Library::new(lib_name)
                .map_err(|e| format!("Не удалось загрузить {}: {}", lib_name, e))?
        };

        // Получаем NvAPI_QueryInterface
        let query_interface: NvAPI_QueryInterface_t = unsafe {
            let sym: Symbol<NvAPI_QueryInterface_t> = library
                .get(b"nvapi_QueryInterface\0")
                .map_err(|e| format!("Не найдена функция nvapi_QueryInterface: {}", e))?;
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

    /// Инициализация NVAPI
    fn initialize(&mut self) -> Result<(), String> {
        // NvAPI_Initialize
        let init_fn = self.get_function::<NvAPI_Initialize_t>(NVAPI_INITIALIZE)?;
        let status = unsafe { init_fn() };
        if status != NVAPI_OK {
            return Err(format!("NvAPI_Initialize failed: {}", status));
        }
        self.initialized = true;
        info!("NVAPI инициализирован успешно");

        // Получаем display handle
        self.display_handle = self.get_display_handle(0)?;
        info!("Display handle: 0x{:X}", self.display_handle);

        Ok(())
    }

    /// Получение функции через QueryInterface
    fn get_function<T>(&self, id: u32) -> Result<T, String> {
        let ptr = unsafe { (self.query_interface)(id) };
        if ptr.is_null() {
            return Err(format!("Функция с ID 0x{:08X} не найдена", id));
        }
        Ok(unsafe { mem::transmute_copy(&ptr) })
    }

    /// Получение display handle по индексу
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

    /// Получение текущего уровня Digital Vibrance
    pub fn get_digital_vibrance(&self) -> Result<NV_DISPLAY_DVC_INFO, String> {
        let get_fn = self.get_function::<NvAPI_GetDVCInfo_t>(NVAPI_GET_DVC_INFO)?;

        let mut info = NV_DISPLAY_DVC_INFO::default();
        let status = unsafe { get_fn(self.display_handle, 0, &mut info) };
        if status != NVAPI_OK {
            return Err(format!("GetDVCInfo failed: {}", status));
        }

        debug!(
            "DVC Info: current={}, min={}, max={}, default={}",
            info.current_level, info.min_level, info.max_level, info.default_level
        );
        Ok(info)
    }

    /// Установка уровня Digital Vibrance
    pub fn set_digital_vibrance(&self, level: i32) -> Result<(), String> {
        let set_fn = self.get_function::<NvAPI_SetDVCLevel_t>(NVAPI_SET_DVC_LEVEL)?;

        let status = unsafe { set_fn(self.display_handle, 0, level) };
        if status != NVAPI_OK {
            return Err(format!("SetDVCLevel failed: {}", status));
        }

        info!("Digital Vibrance установлен: {}", level);
        Ok(())
    }

    /// Применение настроек через Windows GDI Gamma Ramp
    /// Это позволяет менять яркость, контраст и гамму
    pub fn apply_display_settings(&self, settings: &DisplaySettings) -> Result<(), String> {
        // Digital Vibrance через NVAPI
        self.set_digital_vibrance(settings.digital_vibrance)?;

        // Gamma/Brightness/Contrast через Windows GDI
        Self::set_gamma_ramp(settings)?;

        info!("Настройки применены: {:?}", settings);
        Ok(())
    }

    /// Установка gamma ramp через Windows GDI
    /// Формула для каждого канала:
    /// output = clamp(((input / 255.0)^(1/gamma) * contrast + (brightness - 1.0)) * 65535)
    fn set_gamma_ramp(settings: &DisplaySettings) -> Result<(), String> {
        let mut ramp = [[0u16; 256]; 3];

        for i in 0..256 {
            let normalized = i as f64 / 255.0;
            let gamma_corrected = normalized.powf(1.0 / settings.gamma);
            let contrasted = (gamma_corrected - 0.5) * settings.contrast + 0.5;
            let bright = contrasted * settings.brightness;
            let clamped = bright.clamp(0.0, 1.0);
            let value = (clamped * 65535.0) as u16;

            ramp[0][i] = value; // Red
            ramp[1][i] = value; // Green
            ramp[2][i] = value; // Blue
        }

        unsafe {
            let hdc = GetDC(std::ptr::null_mut());
            if hdc.is_null() {
                return Err("GetDC failed".to_string());
            }

            let result = SetDeviceGammaRamp(hdc, ramp.as_ptr() as *const c_void);

            ReleaseDC(std::ptr::null_mut(), hdc);

            if result == 0 {
                return Err("SetDeviceGammaRamp failed".to_string());
            }
        }

        Ok(())
    }

    /// Сброс гамма-таблицы к значениям по умолчанию
    pub fn reset_gamma_ramp() -> Result<(), String> {
        let default_settings = DisplaySettings::default();
        Self::set_gamma_ramp(&default_settings)
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
