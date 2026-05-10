use color_schemer::nvidia::{NvError, NVAPI_NVIDIA_DEVICE_NOT_FOUND, NVAPI_INVALID_HANDLE, NVAPI_NOT_SUPPORTED, NVAPI_API_NOT_INITIALIZED, NVAPI_LIBRARY_NOT_FOUND, NVAPI_INVALID_ARGUMENT};

#[test]
fn test_nv_error_display() {
    let err = NvError::Status(NVAPI_NVIDIA_DEVICE_NOT_FOUND);
    assert_eq!(format!("{}", err), "NVAPI Error (-6): NVIDIA device not found");

    let err = NvError::Status(NVAPI_INVALID_HANDLE);
    assert_eq!(format!("{}", err), "NVAPI Error (-8): Invalid handle");

    let err = NvError::Status(NVAPI_NOT_SUPPORTED);
    assert_eq!(format!("{}", err), "NVAPI Error (-148): Not supported");

    let err = NvError::Status(NVAPI_API_NOT_INITIALIZED);
    assert_eq!(format!("{}", err), "NVAPI Error (-4): NVAPI not initialized");

    let err = NvError::Status(NVAPI_LIBRARY_NOT_FOUND);
    assert_eq!(format!("{}", err), "NVAPI Error (-2): NVAPI library not found");

    let err = NvError::Status(NVAPI_INVALID_ARGUMENT);
    assert_eq!(format!("{}", err), "NVAPI Error (-5): Invalid argument");

    let err = NvError::Status(999);
    assert_eq!(format!("{}", err), "NVAPI Error (999): Unknown NVAPI error");
}

#[test]
fn test_nv_error_variants() {
    let err = NvError::Library("DLL not found".to_string());
    assert_eq!(format!("{}", err), "Library load error: DLL not found");

    let err = NvError::NotFound(0x12345678);
    assert_eq!(format!("{}", err), "Function 0x12345678 not found");

    let err = NvError::Os("Access denied".to_string());
    assert_eq!(format!("{}", err), "OS error: Access denied");

    let err = NvError::NotSupported;
    assert_eq!(format!("{}", err), "Not supported on this platform");
}
