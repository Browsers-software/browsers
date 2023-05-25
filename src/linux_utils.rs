use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use dirs::home_dir;
use tracing::{info, warn};

use glib::prelude::AppInfoExt;
use glib::AppInfo;
use gtk::prelude::*;
use gtk::{gio, IconLookupFlags, IconTheme};

use crate::{InstalledBrowser, SupportedAppRepository};

const XDG_NAME: &'static str = "software.Browsers";

pub struct OsHelper {
    app_repository: SupportedAppRepository,
    icon_theme: Arc<Mutex<IconTheme>>,
}

unsafe impl Send for OsHelper {}

impl OsHelper {
    // must be initialized in main thread (because of gtk requirements)
    pub fn new() -> OsHelper {
        let _result = gtk::init();
        let icon_theme = gtk::IconTheme::default().unwrap();
        let app_repository = SupportedAppRepository::new();
        Self {
            app_repository: app_repository,
            icon_theme: Arc::new(Mutex::new(icon_theme)),
            // unsandboxed_home_dir: unsandboxed_home_dir().unwrap(), probably needed if snap pkg
        }
    }

    pub fn get_app_repository(&self) -> &SupportedAppRepository {
        return &self.app_repository;
    }

    pub fn get_installed_browsers(
        &self,
        schemes: Vec<(String, Vec<String>)>,
    ) -> Vec<InstalledBrowser> {
        let mut browsers: Vec<InstalledBrowser> = Vec::new();

        let cache_root_dir = get_this_app_cache_root_dir();
        let icons_root_dir = cache_root_dir.join("icons");
        fs::create_dir_all(icons_root_dir.as_path()).unwrap();

        let app_infos: Vec<(AppInfo, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domains)| {
                (
                    AppInfo::all_for_type(format!("x-scheme-handler/{scheme}").as_str()),
                    domains,
                )
            })
            .flat_map(|(app_infos, domains)| {
                let app_info_and_domains: Vec<(AppInfo, Vec<String>)> = app_infos
                    .iter()
                    .map(|app_info| (app_info.clone(), domains.clone()))
                    .collect();
                app_info_and_domains
            })
            .collect();

        for (app_info, domains) in app_infos {
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
        app_info: AppInfo,
        icons_root_dir: &Path,
        restricted_domains: Vec<String>,
    ) -> Option<InstalledBrowser> {
        let option = app_info.commandline();
        // uses %u or %U, see https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s07.html
        // need to use command(), because executable() is not fine, as snap apps just have "env" there
        let command_with_field_codes = option.unwrap();
        let command_str = command_with_field_codes.to_str().unwrap();

        let command_parts: Vec<String> = shell_words::split(&command_str)
            .expect("failed to parse Exec value in the .desktop file");

        if command_parts.is_empty() {
            warn!("Exec line is empty! This browser won't work");
            return None;
        }

        // check if it's snap package in a bit naive way
        // "env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/firefox_firefox.desktop /snap/bin/firefox %u"
        let is_snap = command_parts
            .iter()
            .any(|part| part.starts_with("/snap/bin"));

        //let snap_root_path = self.snap_base.clone();
        //let snap_linux_config_dir_relative_path = PathBuf::from(snap_name)
        //    .join("common")
        //    .join(linux_config_dir_relative);
        //let config_dir_absolute = snap_root_path.join(snap_linux_config_dir_relative_path);

        // we need executable path for two reasons:
        //  - to uniquely identify apps
        //  - to identify which Firefox profiles are allowed for firefox instance, they hash the binary path
        let executable_path_best_guess = command_parts
            .iter()
            .rfind(|component| !component.starts_with("%") && !component.starts_with("-"))
            .map(|path_perhaps| Path::new(path_perhaps))
            .unwrap_or(Path::new("unknown"));

        // TODO: get correct path for firefox snap, which one is actually used to calculate installation id in profiles.ini
        // let command_dir = executable_path.parent();
        // let binary_dir = command_dir.and_then(|p| p.to_str()).unwrap_or("").to_string();

        // env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/firefox_firefox.desktop

        let name = app_info.name().to_string();
        /*let icon_maybe = app_info.icon();
        let icon: Icon = icon_maybe.unwrap();*/

        let id_maybe = app_info.id();
        if id_maybe.is_none() {
            println!("no id found for {}", name);
            return None;
        }

        let id_gstring = id_maybe.unwrap();
        let id = id_gstring.as_str().to_string();
        // "google-chrome-beta.desktop"

        if id == "software.Browsers.desktop" {
            // this is us, skip
            return None;
        }

        let supported_app = self
            .app_repository
            .get_or_generate(id.as_str(), &restricted_domains);

        let icon_filename = id.to_string() + ".png";
        let full_stored_icon_path = icons_root_dir.join(icon_filename);
        let icon_path_str = full_stored_icon_path.display().to_string();
        if let Some(icon) = app_info.icon() {
            create_icon_for_app(&self.icon_theme, &icon, icon_path_str.as_str())
        }

        let string1 = app_info.display_name();
        let display_name = string1.as_str();
        let _string = app_info.to_string();
        //println!("app_info: {}", id);

        let profiles = supported_app.find_profiles(executable_path_best_guess.clone(), is_snap);

        let browser = InstalledBrowser {
            command: command_parts.clone(),
            executable_path: executable_path_best_guess.to_str().unwrap().to_string(),
            display_name: display_name.to_string(),
            bundle: supported_app.get_app_id().to_string(),
            user_dir: supported_app
                .get_app_config_dir_absolute(is_snap)
                .to_string(),
            icon_path: icon_path_str.clone(),
            profiles: profiles,
            restricted_domains: restricted_domains,
        };
        return Some(browser);
    }
}

fn create_icon_for_app(
    icon_theme: &Arc<Mutex<IconTheme>>,
    icon: &impl IsA<gio::Icon>,
    to_icon_path: &str,
) {
    // icon.to_string() returns either file path or icon name in theme
    // so not using that
    // https://lazka.github.io/pgi-docs/Gio-2.0/interfaces/Icon.html#Gio.Icon.to_string
    //let icon_str = IconExt::to_string(&icon);
    //let icon_gstr = icon_str.unwrap();
    //let string2 = icon_gstr.to_string();

    let icon_theme = Arc::clone(&icon_theme);
    let icon_theme2 = icon_theme.lock().unwrap();

    let icon_info = icon_theme2
        .lookup_by_gicon(icon, 48, IconLookupFlags::USE_BUILTIN)
        .unwrap();

    // to support scaled resolutions
    //let icon_info = icon_theme.lookup_by_gicon_for_scale(&icon, 128, 1,IconLookupFlags::USE_BUILTIN).unwrap();

    // or load_icon() to get PixBuf
    let original_icon_filepath = icon_info.filename().unwrap();
    let original_icon_path_str = original_icon_filepath
        .as_path()
        .to_str()
        .unwrap()
        .to_string();

    let icon_pixbuf_result = icon_info.load_icon();
    if icon_pixbuf_result.is_err() {
        return;
    }
    let pixbuf = icon_pixbuf_result.unwrap();
    let result = pixbuf.savev(to_icon_path, "png", &[]);
    if result.is_err() {
        return;
    }

    info!("icon: from {} to {}", original_icon_path_str, to_icon_path);
}

// $HOME/.config/software.Browsers
pub fn get_this_app_config_root_dir() -> PathBuf {
    return get_this_app_xdg_config_dir();
}

// $HOME/.local/share/software.Browsers/resources
pub fn get_this_app_resources_dir() -> PathBuf {
    return get_this_app_data_dir().join("resources");
}

// $HOME/.local/share/software.Browsers
pub fn get_this_app_data_dir() -> PathBuf {
    return get_this_app_xdg_data_dir();
}

// /run/user/1001/software.Browsers/
pub fn get_this_app_runtime_dir() -> PathBuf {
    // Either $XDG_RUNTIME_DIR (/run/user/1001/)
    // or $XDG_CACHE_HOME
    // or $HOME/.cache
    dirs::runtime_dir()
        .or_else(|| dirs::cache_dir())
        .unwrap()
        .join(XDG_NAME)
}

// $HOME/.config/software.Browsers
fn get_this_app_xdg_config_dir() -> PathBuf {
    // $XDG_CONFIG_HOME or $HOME/.config
    return dirs::config_dir().unwrap().join(XDG_NAME);
}

// $HOME/.local/share/software.Browsers
fn get_this_app_xdg_data_dir() -> PathBuf {
    // $XDG_DATA_HOME or $HOME/.local/share
    return dirs::data_dir().unwrap().join(XDG_NAME);
}

// $HOME/.cache/software.Browsers
pub fn get_this_app_cache_root_dir() -> PathBuf {
    // $XDG_CACHE_HOME or $HOME/.cache
    return dirs::cache_dir().unwrap().join(XDG_NAME);
}

// $HOME/.local/state/software.Browsers/logs
pub fn get_this_app_logs_root_dir() -> PathBuf {
    return get_this_app_xdg_state_dir().join("logs");
}

// $HOME/.local/state/software.Browsers
fn get_this_app_xdg_state_dir() -> PathBuf {
    // $XDG_STATE_HOME or $HOME/.local/state
    let state_dir = dirs::state_dir().unwrap();
    return state_dir.join(XDG_NAME);
}

pub fn linux_get_unsandboxed_config_dir() -> PathBuf {
    // TODO: escape sandbox if Browsers is running in snap/flatpak
    return dirs::config_dir().unwrap();
}

pub fn get_snap_root_dir() -> PathBuf {
    // TODO: escape sandbox if Browsers is running in snap/flatpak
    let home_dir = dirs::home_dir().unwrap();
    let buf1 = home_dir.join("snap");
    return buf1;
    //let buf = buf1.join("chromium").join("common");
    //return buf;
}

pub fn linux_get_unsandboxed_home_dir() -> PathBuf {
    // TODO: escape sandbox if in snap/flatpak
    return dirs::home_dir().unwrap();
}
