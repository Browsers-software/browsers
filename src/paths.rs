use std::path::PathBuf;

#[cfg(target_os = "linux")]
use crate::linux::linux_utils;

#[cfg(target_os = "macos")]
use crate::macos::macos_utils;

#[cfg(target_os = "windows")]
use crate::windows::windows_utils;

pub fn get_cache_root_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::get_this_app_cache_root_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::get_this_app_cache_root_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_this_app_cache_root_dir();
}

pub fn get_logs_root_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::get_this_app_logs_root_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::get_this_app_logs_root_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_this_app_logs_root_dir();
}

pub fn get_repository_toml_path() -> PathBuf {
    return get_repository_basedir().join("application-repository.toml");
}

pub fn get_config_json_path() -> PathBuf {
    return get_config_root_dir().join("config.json");
}

pub fn get_config_root_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::get_this_app_config_root_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::get_this_app_config_root_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_this_app_config_root_dir();
}

pub fn get_chrome_user_dir_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::macos_get_unsandboxed_application_support_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::linux_get_unsandboxed_config_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_unsandboxed_local_config_dir();
}

pub fn get_firefox_user_dir_root() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::macos_get_unsandboxed_application_support_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::linux_get_unsandboxed_home_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_unsandboxed_roaming_config_dir();
}

// ~/Library/Application Support/
#[cfg(target_os = "macos")]
pub fn get_user_home_for_unsandboxed_app() -> PathBuf {
    return macos_utils::macos_get_unsandboxed_home_dir();
}

// ~/
#[cfg(target_os = "macos")]
pub fn get_user_home_for_sandboxed_app(app_id: &str) -> PathBuf {
    return macos_utils::macos_get_sandboxed_home_dir(app_id);
}

#[cfg(target_os = "linux")]
pub fn get_user_home_for_unsandboxed_app() -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "linux")]
pub fn get_user_home_for_sandboxed_app(app_id: &str) -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "windows")]
pub fn get_user_home_for_unsandboxed_app() -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "windows")]
pub fn get_user_home_for_sandboxed_app(app_id: &str) -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "macos")]
pub fn get_snap_root() -> PathBuf {
    return PathBuf::new();
}

#[cfg(target_os = "linux")]
pub fn get_snap_root() -> PathBuf {
    return linux_utils::get_snap_root_dir();
}

#[cfg(target_os = "windows")]
pub fn get_snap_root() -> PathBuf {
    return PathBuf::new();
}

pub fn get_app_icon_path() -> PathBuf {
    return get_resources_basedir().join("icons/512x512/software.Browsers.png");
}

// This is the {base_dir} of the path {base_dir}/{locale}/{resource},
// where '{locale}' is a valid BCP47 language tag, and {resource} is a file with .ftl extension.
pub fn get_localizations_basedir() -> PathBuf {
    return get_resources_basedir().join("i18n");
}

pub fn get_repository_basedir() -> PathBuf {
    return get_resources_basedir().join("repository");
}

pub fn get_resources_basedir() -> PathBuf {
    #[cfg(debug_assertions)]
    // if running Browsers in debug mode
    return PathBuf::from("./resources");

    #[cfg(target_os = "macos")]
    // /Applications/Browsers.app/Contents/Resources/
    return macos_utils::get_this_app_resources_dir();

    #[cfg(target_os = "linux")]
    // $HOME/.local/share/software.Browsers/resources
    return linux_utils::get_this_app_resources_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_this_app_resources_dir();
}

pub fn get_runtime_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    return macos_utils::get_this_app_runtime_dir();

    #[cfg(target_os = "linux")]
    return linux_utils::get_this_app_runtime_dir();

    #[cfg(target_os = "windows")]
    return windows_utils::get_this_app_runtime_dir();
}
