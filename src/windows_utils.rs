use std::{
    fs,
    path::{Path, PathBuf},
};

use tracing::info;
use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

use crate::{browser_repository::SupportedAppRepository, InstalledBrowser};

#[derive(Clone)]
struct AppInfoHolder {
    registry_key: String,
    name: String,
    icon_path: String,
    binary_path: String,
}

pub struct OsHelper {
    app_repository: SupportedAppRepository,
}

unsafe impl Send for OsHelper {}

impl OsHelper {
    pub fn new() -> OsHelper {
        let app_repository = SupportedAppRepository::new();
        Self {
            app_repository: app_repository,
        }
    }

    pub fn get_app_repository(&self) -> &SupportedAppRepository {
        return &self.app_repository;
    }

    fn find_applications_for_url_scheme(scheme: &str) -> Vec<AppInfoHolder> {
        if scheme != "https" {
            return vec![];
        }
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let start_menu_internet = hklm
            .open_subkey("SOFTWARE\\Clients\\StartMenuInternet")
            .unwrap();
        let bundle_ids = start_menu_internet.enum_keys();

        let mut apps: Vec<AppInfoHolder> = bundle_ids
            .map(|result| result.unwrap())
            .map(|browser_key_name| {
                let browser_reg_key = start_menu_internet
                    .open_subkey(browser_key_name.as_str())
                    .unwrap();
                let browser_name: String = browser_reg_key.get_value("").unwrap();

                let command_reg_key = browser_reg_key.open_subkey("shell\\open\\command").unwrap();
                let binary_path: String = command_reg_key.get_value("").unwrap();
                // remove surrounding quotes if there are any
                let binary_path = binary_path.trim_start_matches("\"");
                let binary_path = binary_path.trim_end_matches("\"");

                let binary_path_path = Path::new(binary_path);
                let binary_path_str = binary_path_path.to_str().unwrap();
                info!("path is {}", binary_path_str);

                // Either Capabilities->ApplicationIcon
                // or DefaultIcon->""
                let default_icon_reg_key = browser_reg_key.open_subkey("DefaultIcon").unwrap();
                // e.g `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe,0`
                let default_icon_path: String = default_icon_reg_key.get_value("").unwrap();

                AppInfoHolder {
                    registry_key: browser_key_name,
                    name: browser_name.to_string(),
                    icon_path: default_icon_path.to_string(),
                    binary_path: binary_path_str.to_string(),
                }
            })
            .collect::<Vec<_>>();

        //apps.sort_by_key(|a|a.name);
        return apps;
    }

    pub fn get_installed_browsers(
        &self,
        schemes: Vec<(String, Vec<String>)>,
    ) -> Vec<InstalledBrowser> {
        let mut browsers: Vec<InstalledBrowser> = Vec::new();

        let cache_root_dir = get_this_app_cache_root_dir();
        let icons_root_dir = cache_root_dir.join("icons");
        fs::create_dir_all(icons_root_dir.as_path()).unwrap();

        let app_infos_and_domains: Vec<(AppInfoHolder, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domains)| (Self::find_applications_for_url_scheme(scheme), domains))
            .flat_map(|(app_infos, domains)| {
                let app_info_and_domains: Vec<(AppInfoHolder, Vec<String>)> = app_infos
                    .iter()
                    .map(|app_info| (app_info.clone(), domains.clone()))
                    .collect();
                app_info_and_domains
            })
            .collect();

        for (app_info, domains) in app_infos_and_domains {
            let browser_maybe =
                self.to_installed_browser(app_info, icons_root_dir.as_path(), domains);
            if let Some(browser) = browser_maybe {
                browsers.push(browser);
            }
        }
        return browsers;
    }

    fn to_installed_browser(
        &self,
        app_info: AppInfoHolder,
        icons_root_dir: &Path,
        restricted_domains: Vec<String>,
    ) -> Option<InstalledBrowser> {
        let display_name = app_info.name.to_string();

        // Using the name as the unique id,
        // because registry_key can differ based on Firefox install path,
        // but we need to just identify that it is Firefox
        // We do use path for uniqueness, so it should be fine if there are duplicate names
        let app_id = app_info.name.to_string();

        let supported_app = self
            .app_repository
            .get_or_generate(app_id.as_str(), &restricted_domains);

        let icon_filename = app_id.to_string() + ".png";
        let full_stored_icon_path = icons_root_dir.join(icon_filename);
        let icon_path_str = full_stored_icon_path.display().to_string();
        create_icon_for_app(app_info.icon_path.as_str(), icon_path_str.as_str());

        let command_str = app_info.binary_path;
        let executable_path = Path::new(command_str.as_str());

        let profiles = supported_app.find_profiles(executable_path, false);

        let browser = InstalledBrowser {
            executable_path: command_str.to_string(),
            display_name: display_name.to_string(),
            bundle: app_id.to_string(),
            user_dir: supported_app.get_app_config_dir_absolute(false).to_string(),
            icon_path: icon_path_str.clone(),
            profiles: profiles,
            restricted_domains: restricted_domains,
        };
        return Some(browser);
    }
}

pub fn create_icon_for_app(full_path_and_index: &str, icon_path: &str) {
    // e.g `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe,0`
    let split: Vec<&str> = full_path_and_index.split(",").collect();
    let path = split[0];
    let index_str = split[1];
    let index = index_str.parse::<i32>().unwrap();

    // TODO
    /*let hicon = ExtractIconA(
        hInst: HINSTANCE,
        pszExeFileName: LPCSTR,
        nIconIndex: UINT,
    ) -> HICON;*/

    // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticona
}

// PATHS
const APP_DIR_NAME: &'static str = "software.Browsers";
const APP_BUNDLE_ID: &'static str = "software.Browsers";

// C:\Users\Alice\AppData\Local\software.Browsers\cache\runtime
pub fn get_this_app_runtime_dir() -> PathBuf {
    return get_this_app_cache_root_dir().join("runtime");
}

// C:\Users\Alice\AppData\Local\software.Browsers\cache
pub fn get_this_app_cache_root_dir() -> PathBuf {
    return get_this_app_config_local_dir().join("cache");
}

// C:\Users\Alice\AppData\Local\software.Browsers\logs
pub fn get_this_app_logs_root_dir() -> PathBuf {
    return get_this_app_config_local_dir().join("logs");
}

// C:\Users\Alice\AppData\Local\software.Browsers\config
pub fn get_this_app_config_root_dir() -> PathBuf {
    return get_this_app_config_local_dir().join("config");
}

// For resources (e.g translations)
// C:\Users\Alice\AppData\Local\software.Browsers\data
pub fn get_this_app_data_dir() -> PathBuf {
    return get_this_app_config_local_dir().join("data");
}

// C:\Users\Alice\AppData\Local\software.Browsers
fn get_this_app_config_local_dir() -> PathBuf {
    return get_config_local_dir().join(APP_DIR_NAME);
}

// C:\Users\Alice\AppData\Local
fn get_config_local_dir() -> PathBuf {
    return dirs::config_local_dir().unwrap();
}

// To access config dirs of other apps aka %localappdata%
// C:\Users\Alice\AppData\Local
pub fn get_unsandboxed_local_config_dir() -> PathBuf {
    return dirs::config_local_dir().unwrap();
}

// To access config dirs of other apps aka %appdata%
// C:\Users\Alice\AppData\Roaming
pub fn get_unsandboxed_roaming_config_dir() -> PathBuf {
    return dirs::config_dir().unwrap();
}

// To access home dir of other apps
// C:\Users\Alice
fn get_unsandboxed_home_dir() -> PathBuf {
    return dirs::home_dir().unwrap();
}
