use std::path::PathBuf;

#[cfg(target_os = "linux")]
use crate::linux_utils;
#[cfg(target_os = "macos")]
use crate::macos_utils;

#[cfg(target_os = "macos")]
pub fn get_cache_root_dir() -> PathBuf {
    return macos_utils::get_this_app_cache_root_dir();
}

#[cfg(target_os = "linux")]
pub fn get_cache_root_dir() -> PathBuf {
    return linux_utils::get_this_app_cache_root_dir();
}

#[cfg(target_os = "macos")]
pub fn get_logs_root_dir() -> PathBuf {
    return macos_utils::get_this_app_logs_root_dir();
}

#[cfg(target_os = "linux")]
pub fn get_logs_root_dir() -> PathBuf {
    return linux_utils::get_this_app_logs_root_dir();
}


#[cfg(target_os = "macos")]
pub fn get_config_root_dir() -> PathBuf {
    return macos_utils::get_this_app_config_root_dir();
}

#[cfg(target_os = "linux")]
pub fn get_config_root_dir() -> PathBuf {
    return linux_utils::get_this_app_config_root_dir();
}