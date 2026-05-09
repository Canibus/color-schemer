pub mod config;
pub mod mock_gpu;
pub mod nvidia;
pub mod platform;
pub mod profiles;

#[cfg(feature = "hotkeys")]
pub mod hotkey;

#[cfg(feature = "tray")]
pub mod tray;
