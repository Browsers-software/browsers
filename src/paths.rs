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

#[cfg(target_os = "macos")]
pub fn get_chrome_user_dir_root() -> PathBuf {
    return macos_utils::macos_get_unsandboxed_application_support_dir();
}

#[cfg(target_os = "linux")]
pub fn get_chrome_user_dir_root() -> PathBuf {
    return linux_utils::linux_get_unsandboxed_config_dir();
}

#[cfg(target_os = "macos")]
pub fn get_firefox_user_dir_root() -> PathBuf {
    return macos_utils::macos_get_unsandboxed_application_support_dir();
}

#[cfg(target_os = "linux")]
pub fn get_firefox_user_dir_root() -> PathBuf {
    return linux_utils::linux_get_unsandboxed_home_dir();
}

#[cfg(target_os = "macos")]
pub fn get_snap_root() -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "linux")]
pub fn get_snap_root() -> PathBuf {
    return linux_utils::get_snap_root_dir();
}

// This directory should be of the structure base_dir/{locale}/{resource},
// where '{locale}' is a valid BCP47 language tag, and {resource} is a .ftl included in resources.

// on macOS basedir should be "/Applications/Browsers.app/Contents/Resources/i18n/"
#[cfg(target_os = "macos")]
pub fn get_localizations_basedir() -> PathBuf {
    let app_bundle_dir = macos_utils::get_this_app_bundle_dir();
    let buf = app_bundle_dir
        .join("Contents")
        .join("Resources")
        .join("i18n");
    return buf;
}

#[cfg(target_os = "linux")]
pub fn get_localizations_basedir() -> PathBuf {
    let basedir = "./resources/i18n".to_string();
    return PathBuf::from_str(basedir.as_str()).unwrap();
}
