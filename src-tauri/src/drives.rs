use serde::Serialize;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod linux;

#[cfg(target_os = "macos")]
// Re-export helpers for tests; some items are only used in drives_tests.rs.
#[allow(unused_imports)]
pub(crate) use macos::{
    classify_volume, find_field_value, parse_filesystem_from_info, parse_size_str, VolumeKind,
};

#[derive(Debug, Serialize, Clone)]
pub struct RemovableDrive {
    pub name: String,
    pub mount_path: String,
    pub size_bytes: Option<u64>,
    pub filesystem: Option<String>,
    pub available_bytes: Option<u64>,
}

/// Trait abstracting platform-specific drive detection and formatting.
///
/// Each platform module (`macos`, `windows`, `linux`) provides a detector
/// struct that implements this trait. The public API functions at the bottom
/// of this file delegate to the compile-time selected implementation via
/// `#[cfg]` gating — no runtime dispatch overhead.
pub trait DriveDetector {
    fn list(&self) -> Result<Vec<RemovableDrive>, String>;
    fn format(&self, mount_path: &str, volume_name: &str) -> Result<(), String>;
}

/// List removable drives detected on this system.
/// Delegates to the platform-specific `DriveDetector` implementation.
#[cfg(target_os = "macos")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    macos::MacOSDetector.list()
}

#[cfg(target_os = "windows")]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    windows::WindowsDetector.list()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn list_removable_drives() -> Result<Vec<RemovableDrive>, String> {
    linux::LinuxDetector.list()
}

/// Format a drive on this system.
/// Delegates to the platform-specific `DriveDetector` implementation.
#[cfg(target_os = "macos")]
pub fn format_drive(mount_path: &str, volume_name: &str) -> Result<(), String> {
    macos::MacOSDetector.format(mount_path, volume_name)
}

#[cfg(target_os = "windows")]
pub fn format_drive(mount_path: &str, volume_name: &str) -> Result<(), String> {
    windows::WindowsDetector.format(mount_path, volume_name)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn format_drive(mount_path: &str, volume_name: &str) -> Result<(), String> {
    linux::LinuxDetector.format(mount_path, volume_name)
}

#[cfg(test)]
#[path = "drives_tests.rs"]
mod tests;
