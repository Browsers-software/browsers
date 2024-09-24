use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::slice::Split;

use druid::image;
use druid::image::ImageFormat;
use tracing::{info, warn};

use freedesktop_desktop_entry::{default_paths, get_languages_from_env, DesktopEntry, Iter};
use freedesktop_icons;

use crate::{InstalledBrowser, SupportedAppRepository};

const XDG_NAME: &'static str = "software.Browsers";

#[derive(Clone)]
struct DesktopEntryHolder {
    app_id: String,
    display_name: String,
    icon: Option<String>,

    // uses %u or %U, see https://specifications.freedesktop.org/desktop-entry-spec/latest/ar01s07.html
    exec: String,
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
        let cache_root_dir = get_this_app_cache_root_dir();
        let icons_root_dir = cache_root_dir.join("icons");
        fs::create_dir_all(icons_root_dir.as_path()).unwrap();

        let desktop_entry_holders: Vec<(DesktopEntryHolder, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domains)| {
                let content_type = format!("x-scheme-handler/{scheme}");
                let desktop_entry_holders =
                    Self::freedesktop_find_all_desktop_entries(content_type.as_str());

                (desktop_entry_holders, domains)
            })
            .flat_map(|(desktop_entry_holders, domains)| {
                let app_info_and_domains: Vec<(DesktopEntryHolder, Vec<String>)> =
                    desktop_entry_holders
                        .iter()
                        .map(|app_info| (app_info.clone(), domains.clone()))
                        .collect();
                app_info_and_domains
            })
            .collect();

        let mut browsers: Vec<InstalledBrowser> = Vec::new();
        for (desktop_entry_holder, domains) in desktop_entry_holders {
            let browser_maybe =
                self.to_installed_browser(&desktop_entry_holder, icons_root_dir.as_path(), domains);
            if let Some(browser) = browser_maybe {
                browsers.push(browser);
            }
        }

        return browsers;
    }

    fn freedesktop_find_all_desktop_entries(content_type: &str) -> Vec<DesktopEntryHolder> {
        let unique_search_paths = default_paths()
            .into_iter()
            // remove duplicate entries (e.g XDG_DATA_DIRS sometimes has every path twice)
            .collect::<HashSet<PathBuf>>()
            .into_iter();

        // collect all .desktop file paths and map them by file name to remove duplicate files,
        // even if they exist in different directories
        let mut desktop_file_paths_by_filename: BTreeMap<_, PathBuf> =
            Iter::new(unique_search_paths)
                .filter_map(|file_path| {
                    file_path
                        .file_name()
                        .map(|a| a.to_owned())
                        .map(|file_name| (file_name, file_path))
                })
                .collect();

        let cleaned_desktop_file_paths: Vec<PathBuf> =
            desktop_file_paths_by_filename.into_values().collect();

        let locales = get_languages_from_env();

        let vec = cleaned_desktop_file_paths
            .iter()
            .filter_map(|desktop_file_path| {
                Self::read_desktop_entry_matching(
                    &desktop_file_path.as_path(),
                    content_type,
                    &locales,
                )
            })
            .collect();

        return vec;
    }

    fn read_desktop_entry_matching(
        desktop_file_path: &Path,
        content_type: &str,
        locales: &Vec<String>,
    ) -> Option<DesktopEntryHolder> {
        return fs::read_to_string(&desktop_file_path)
            .ok()
            .and_then(|file_content| {
                DesktopEntry::from_str(&desktop_file_path, &file_content, Some(locales))
                    .ok()
                    .and_then(|entry| {
                        let contains_mime_type = entry
                            .mime_type()
                            .filter(|mime_types| mime_types.contains(&content_type))
                            .is_some();

                        if contains_mime_type {
                            Self::freedesktop_desktop_entry_to_desktop_entry_holder(&entry, locales)
                        } else {
                            None
                        }
                    })
            });
    }

    fn freedesktop_desktop_entry_to_desktop_entry_holder(
        desktop_entry: &DesktopEntry,
        locales: &Vec<String>,
    ) -> Option<DesktopEntryHolder> {
        let app_id = desktop_entry.id();

        let name_maybe = desktop_entry.name(locales);
        if name_maybe.is_none() {
            warn!("no name found for {}", app_id);
            return None;
        }
        let display_name = name_maybe.unwrap().to_string();

        let exec = match desktop_entry.exec() {
            Some(b) => b,
            None => {
                return None;
            }
        };

        let icon_maybe = desktop_entry.icon().map(|icon| icon.to_string());

        return Some(DesktopEntryHolder {
            app_id: app_id.to_string(),
            display_name: display_name,
            icon: icon_maybe,
            exec: exec.to_string(),
        });
    }

    fn to_installed_browser(
        &self,
        desktop_entry_holder: &DesktopEntryHolder,
        icons_root_dir: &Path,
        restricted_domains: Vec<String>,
    ) -> Option<InstalledBrowser> {
        let id = desktop_entry_holder.app_id.as_str();
        if id == "software.Browsers" {
            // this is us, skip
            return None;
        }

        let command_str = desktop_entry_holder.exec.as_str();
        let command_parts: Vec<String> = shell_words::split(command_str)
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
            .rfind(|component| {
                !component.starts_with("%")
                    && !component.starts_with("-")
                    && !component.starts_with("@")
            })
            .map(|path_perhaps| Path::new(path_perhaps))
            .unwrap_or(Path::new("unknown"));

        // TODO: get correct path for firefox snap, which one is actually used to calculate installation id in profiles.ini
        // let command_dir = executable_path.parent();
        // let binary_dir = command_dir.and_then(|p| p.to_str()).unwrap_or("").to_string();

        // env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/firefox_firefox.desktop

        //let name = app_info.name().to_string();
        /*let icon_maybe = app_info.icon();
        let icon: Icon = icon_maybe.unwrap();*/

        let supported_app = self.app_repository.get_or_generate(id, &restricted_domains);

        let icon_filename = id.to_string() + ".png";
        let full_stored_icon_path = icons_root_dir.join(icon_filename);
        let icon_path_str = full_stored_icon_path.display().to_string();

        if let Some(icon_str) = desktop_entry_holder.icon.as_ref() {
            create_icon_for_app(icon_str.as_str(), icon_path_str.as_str())
        }

        let display_name = desktop_entry_holder.display_name.as_str();
        //let _string = app_info.to_string();
        //println!("app_info: {}", id);

        // new firefox actually doesn't refer to the snap binary in the desktop file,
        // so there is no clean way to check that
        // I think we should just try both
        if !is_snap {
            // look deeper!
        }

        let app_config_dir_abs = supported_app.get_app_config_dir_abs(is_snap, false);

        let profiles = supported_app.find_profiles(executable_path_best_guess, app_config_dir_abs);

        let browser = InstalledBrowser {
            command: command_parts.clone(),
            executable_path: executable_path_best_guess.to_str().unwrap().to_string(),
            display_name: display_name.to_string(),
            bundle: supported_app.get_app_id().to_string(),
            user_dir: app_config_dir_abs.to_str().unwrap().to_string(),
            icon_path: icon_path_str.clone(),
            profiles: profiles,
            restricted_domains: restricted_domains,
        };
        return Some(browser);
    }
}

fn create_icon_for_app(icon_str: &str, to_icon_path: &str) {
    let icon_path_maybe = find_icon_path_from_desktop_icon_value(icon_str);
    if icon_path_maybe.is_none() {
        warn!(
            "Could not get icon path from icon (destination icon={})",
            to_icon_path
        );
        return;
    }
    let image_file_path = icon_path_maybe.unwrap();
    let image_file_name_maybe = image_file_path
        .file_name()
        .map(|file_name| file_name.to_str().unwrap().to_string());
    if image_file_name_maybe.is_none() {
        warn!("File does not exist (destination icon={})", to_icon_path);
        return;
    }
    let image_file_name = image_file_name_maybe.unwrap();
    if !image_file_name.to_lowercase().ends_with(".png") {
        warn!(
            "Filename does not have .png extension: {} (destination icon={})",
            image_file_name.as_str(),
            to_icon_path
        );
        return;
    }

    let original_icon_path_str = image_file_path.as_path().to_str().unwrap().to_string();
    if !image_file_path.exists() {
        warn!(
            "File does not exist: {} (destination icon={})",
            original_icon_path_str, to_icon_path
        );
        return;
    }

    let result1 = image::open(image_file_path);
    if result1.is_err() {
        warn!(
            "File could not be read {} (destination icon={})",
            original_icon_path_str, to_icon_path
        );
        return;
    }
    let dynamic_image = result1.unwrap();

    let result2 = dynamic_image.save_with_format(to_icon_path, ImageFormat::Png);
    if result2.is_err() {
        return;
    }

    info!("icon: from {} to {}", original_icon_path_str, to_icon_path);
}

fn find_icon_path_from_desktop_icon_value(icon_str: &str) -> Option<PathBuf> {
    // Icon in .desktop file is either:
    return if icon_str.starts_with("/") {
        // 1) absolute path to a file
        Some(PathBuf::from(icon_str))
    } else {
        // 2) or name of icon in icon theme
        freedesktop_icons::lookup(icon_str).with_size(48).find()
    };
}

// $HOME/.config/software.Browsers
pub fn get_this_app_config_root_dir() -> PathBuf {
    return get_this_app_xdg_config_dir();
}

// $HOME/.local/share/software.Browsers/resources
// or /usr/local/share/software.Browsers/resources
pub fn get_this_app_resources_dir() -> PathBuf {
    return get_this_app_data_dir().join("resources");
}

// $HOME/.local/share/software.Browsers
// $XDG_DATA_HOME/software.Browsers
// /usr/local/share/software.Browsers
// /usr/share/software.Browsers
fn get_this_app_data_dir() -> PathBuf {
    //   # ~/.local/share/software.Browsers/bin/browsers
    //   $XDG_DATA_HOME/software.Browsers/bin/browsers
    //   /usr/local/share/software.Browsers/bin/browsers
    let binary_file_path =
        fs::canonicalize(std::env::current_exe().expect("Can't find current executable"))
            .expect("Can't canonicalize current executable path");

    //   # ~/.local/share/software.Browsers/bin/
    //   $XDG_DATA_HOME/software.Browsers/bin/
    //   /usr/local/share/software.Browsers/bin/
    let binary_dir_path = binary_file_path.parent().unwrap();

    //   # ~/.local/share/software.Browsers/
    //   $XDG_DATA_HOME/software.Browsers/
    //   /usr/local/share/software.Browsers/
    let data_dir_path = binary_dir_path.parent().unwrap();
    return data_dir_path.to_path_buf();
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

// $HOME/.config
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

// returns true if it was already default web browser (then nothing was done)
pub fn set_default_web_browser() -> bool {
    if is_default_web_browser() {
        return true;
    }

    let desktop_file_name = format!("{}.desktop", XDG_NAME);

    let result = Command::new("xdg-mime")
        .arg("default")
        .arg(desktop_file_name.as_str())
        .arg("x-scheme-handler/https")
        .arg("x-scheme-handler/http")
        .status();

    if result.is_err() {
        warn!("Could not set Browsers as default app");
    }

    return false;
}

pub fn is_default_web_browser() -> bool {
    let desktop_file_name = format!("{}.desktop", XDG_NAME);

    let https_default_app = query_default_app("x-scheme-handler/https").unwrap_or("".to_string());
    let http_default_app = query_default_app("x-scheme-handler/http").unwrap_or("".to_string());

    return https_default_app == desktop_file_name && http_default_app == desktop_file_name;
}

fn query_default_app(scheme: &str) -> Option<String> {
    let result = Command::new("xdg-mime")
        .arg("query")
        .arg("default")
        .arg(scheme)
        .stdout(Stdio::piped())
        .output();

    if result.is_err() {
        warn!("Could not query default app for scheme {}", scheme);
        return None;
    }

    let output = result.unwrap();

    // extract the raw bytes that we captured and interpret them as a string
    let default_app = String::from_utf8(output.stdout).unwrap().trim().to_string();
    info!("Default for {scheme} is '{default_app}'");
    return Some(default_app);
}
