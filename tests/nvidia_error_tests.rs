use color_schemer::nvidia::{NvError, NVAPI_NVIDIA_DEVICE_NOT_FOUND, NVAPI_INVALID_HANDLE};

#[test]
fn test_nv_error_display() {
    let err = NvError::Status(NVAPI_NVIDIA_DEVICE_NOT_FOUND);
    assert_eq!(format!("{}", err), "NVAPI Error (-6): NVIDIA device not found");

    let err = NvError::Status(NVAPI_INVALID_HANDLE);
    assert_eq!(format!("{}", err), "NVAPI Error (-8): Invalid handle");

    let err = NvError::Status(999);
    assert_eq!(format!("{}", err), "NVAPI Error (999): Unknown NVAPI error");
}
