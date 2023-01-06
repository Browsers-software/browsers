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

pub fn get_app_icon_path() -> PathBuf {
    return get_resources_basedir().join("icons/512x512/software.Browsers.png");
}

// This is the {base_dir} of the path {base_dir}/{locale}/{resource},
// where '{locale}' is a valid BCP47 language tag, and {resource} is a file with .ftl extension.
pub fn get_localizations_basedir() -> PathBuf {
    return get_resources_basedir().join("i18n");
}

// on macOS basedir should be "/Applications/Browsers.app/Contents/Resources/"
#[cfg(target_os = "macos")]
pub fn get_resources_basedir() -> PathBuf {
    let app_bundle_dir = macos_utils::get_this_app_bundle_dir();
    return app_bundle_dir.join("Contents").join("Resources");
}

// TODO: find the best path for resources of a linux app
#[cfg(target_os = "linux")]
pub fn get_resources_basedir() -> PathBuf {
    let basedir = "./resources".to_string();
    PathBuf::from(basedir.as_str())
}
