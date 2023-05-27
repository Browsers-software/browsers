use std::{
    ffi::CString,
    fs,
    mem::{self, MaybeUninit},
    path::{Path, PathBuf},
    ptr::{self, null_mut},
};

use druid::image::{ImageFormat, RgbaImage};
use tracing::{info, warn};

use winapi::shared::windef::*;
use winapi::um::winuser::*;
use winapi::{
    shared::{
        minwindef::*,
        ntdef::{LPCSTR, VOID},
    },
    um::{
        shellapi::{ExtractIconA, ExtractIconExA},
        wingdi::{DeleteObject, GetBitmapBits, GetObjectW, BITMAP, BITMAPINFOHEADER},
    },
};
use winreg::{
    enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey,
};

use crate::{browser_repository::SupportedAppRepository, InstalledBrowser};

#[derive(Clone)]
struct AppInfoHolder {
    registry_key: String,
    name: String,
    icon_path: Option<String>,
    command: String,
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
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let mut hklm_apps = Self::find_applications_for_url_scheme_and_reg_root(scheme, hklm);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let mut hkcu_apps = Self::find_applications_for_url_scheme_and_reg_root(scheme, hkcu);

        hklm_apps.append(&mut hkcu_apps);

        if scheme != "https" {
            let classes = RegKey::predef(HKEY_CLASSES_ROOT);
            //let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            //let classes = hklm.open_subkey("SOFTWARE\\Classes").unwrap();

            //let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            //let classes = hkcu.open_subkey("SOFTWARE\\Classes").unwrap();

            let mut class_apps = Self::find_applications_for_class(scheme, classes);
            hklm_apps.append(&mut class_apps);
        }

        hklm_apps
    }

    fn find_applications_for_class(scheme: &str, classes: RegKey) -> Vec<AppInfoHolder> {
        let classes_keys = classes.enum_keys();
        let apps = classes_keys
            .map(|result| result.unwrap())
            .filter(|protocol| protocol == scheme)
            .map(|protocol| classes.open_subkey(protocol).unwrap())
            .filter(|protocol_key| protocol_key.get_value::<String, _>("URL Protocol").is_ok())
            .filter_map(|protocol_key| {
                let default_app_name = scheme.to_string();

                let app_name_result = protocol_key.get_value::<String, _>("");
                let app_name = app_name_result.unwrap_or(default_app_name);

                // "C:\Users\Browsers\AppData\Roaming\Spotify\Spotify.exe" --protocol-uri="%1"
                // "C:\Users\Browsers\AppData\Local\Programs\WorkFlowy\WorkFlowy.exe" "%1"
                let Ok(command) = protocol_key
                    .open_subkey("shell\\open\\command")
                    .and_then(|command_reg_key| command_reg_key.get_value::<String, _>("")) else { return None};

                let icon_path_maybe = protocol_key
                    .open_subkey("DefaultIcon")
                    .and_then(|icon_key| icon_key.get_value::<String, _>(""))
                    .ok();

                Some(AppInfoHolder {
                    registry_key: scheme.to_string(),
                    name: app_name.to_string(),
                    icon_path: icon_path_maybe,
                    command: command.to_string(),
                })
            })
            .collect::<Vec<_>>();

        apps
    }

    fn find_applications_for_url_scheme_and_reg_root(
        scheme: &str,
        root: RegKey,
    ) -> Vec<AppInfoHolder> {
        let start_menu_internet = root
            .open_subkey("SOFTWARE\\Clients\\StartMenuInternet")
            .unwrap();
        let bundle_ids = start_menu_internet.enum_keys();

        let apps: Vec<AppInfoHolder> = bundle_ids
            .map(|result| result.unwrap())
            .map(|browser_key_name| {
                let browser_reg_key = start_menu_internet
                    .open_subkey(browser_key_name.as_str())
                    .unwrap();
                let browser_name: String = browser_reg_key.get_value("").unwrap();

                let command_reg_key = browser_reg_key.open_subkey("shell\\open\\command").unwrap();
                let binary_path: String = command_reg_key.get_value("").unwrap();
                // remove surrounding quotes if there are any
                //let binary_path = binary_path.trim_start_matches("\"");
                //let binary_path = binary_path.trim_end_matches("\"");

                //let binary_path_path = Path::new(binary_path);
                //let binary_path_str = binary_path_path.to_str().unwrap();
                //info!("path is {}", binary_path_str);

                // Either Capabilities->ApplicationIcon
                // or DefaultIcon->""
                let default_icon_reg_key = browser_reg_key.open_subkey("DefaultIcon").unwrap();
                // e.g `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe,0`
                let default_icon_path: String = default_icon_reg_key.get_value("").unwrap();

                AppInfoHolder {
                    registry_key: browser_key_name,
                    name: browser_name.to_string(),
                    icon_path: Some(default_icon_path.to_string()),
                    command: binary_path.to_string(),
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

        if app_info.registry_key == "software.Browsers" {
            // this is us, skip
            return None;
        }

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

        app_info
            .icon_path
            .map(|icon_path| create_icon_for_app(icon_path.as_str(), icon_path_str.as_str()));

        // "C:\Users\Browsers\AppData\Local\Programs\WorkFlowy\WorkFlowy.exe" "%1"
        // "C:\Users\Browsers\AppData\Roaming\Spotify\Spotify.exe" --protocol-uri="%1"
        // "C:\Users\Browsers\AppData\Roaming\Spotify\Spotify.exe"
        let command_str = app_info.command;

        // "C:\Users\Browsers\AppData\Roaming\Spotify\Spotify.exe"
        // --protocol-uri="%1"

        // "C:\Users\Browsers\AppData\Local\Programs\WorkFlowy\WorkFlowy.exe"
        // "%1"
        let command_parts: Vec<String> =
            shell_words::split(&command_str).expect("failed to parse command");

        if command_parts.is_empty() {
            warn!("Command is empty! This browser won't work");
            return None;
        }

        // we need executable path for two reasons:
        //  - to uniquely identify apps
        //  - to identify which Firefox profiles are allowed for firefox instance, they hash the binary path
        let executable_path_best_guess = command_parts
            .iter()
            .map(|path_perhaps| remove_quotes(&path_perhaps))
            .rfind(|component| !component.starts_with("%") && !component.starts_with("-"))
            .map(|path_perhaps| Path::new(path_perhaps))
            .unwrap_or(Path::new("unknown"));

        let profiles = supported_app.find_profiles(executable_path_best_guess.clone(), false);

        let browser = InstalledBrowser {
            command: command_parts.clone(),
            executable_path: executable_path_best_guess.to_str().unwrap().to_string(),
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

fn remove_quotes(binary_path: &str) -> &str {
    // remove surrounding quotes if there are any
    if binary_path.starts_with("\"") && binary_path.ends_with("\"") {
        let binary_path = binary_path.trim_start_matches("\"");
        let binary_path = binary_path.trim_end_matches("\"");
        return binary_path;
    }
    return binary_path;
}

pub fn create_icon_for_app(full_path_and_index: &str, icon_path: &str) {
    // e.g `C:\Program Files (x86)\Google\Chrome\Application\chrome.exe,0`
    //  or `"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",0`
    let split: Vec<&str> = full_path_and_index.split(",").collect();
    let path = split[0].trim();
    let path = remove_quotes(path);

    let index_str = split[1].trim();
    let icon_index = index_str.parse::<i32>().unwrap();

    info!("Icon Path: '{}', index: {}", path, icon_index);
    // We could be (todo) certain that our string doesn't have 0 bytes in the middle,
    // so we can .expect()
    let c_to_print = CString::new(path).expect("CString::new failed");
    let a = c_to_print.as_ptr();
    let ok = a.cast::<LPCSTR>();

    unsafe {
        //let h_inst= "ok"; // hInst HINSTANCE
        //let exe_file_path = "ok"; // pszExeFileName LPCSTR
        //let icon_index = 0; // nIconIndex UINT
        // TODO
        // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-extracticona
        let hicon: HICON = ExtractIconA(0 as HINSTANCE, ok as LPCSTR, icon_index as UINT);

        //let mut large: *mut HICON = HICON
        let mut large: HICON = ptr::null_mut();
        let array_pointer: *mut HICON = &mut large;

        //array_pointer: *const libc::int32_t,
        //array_pointer as *const i32

        let ret = ExtractIconExA(
            ok as LPCSTR,
            icon_index as INT,
            array_pointer,
            null_mut(),
            3 as UINT,
        );

        /*SHDefExtractIcon(
            ok as LPCSTR,
            icon_index as INT,
            0 as UINT,
            array_pointer,
            null_mut(),
            48,
        );*/

        // &mut pnt as LPPOINT

        // TODO: destroy icon

        let total = ret as usize;
        info!("ret is {}", ret);

        let a = *array_pointer;
        let b = std::slice::from_raw_parts(array_pointer, total);
        let c = b[0];

        let image_buffer = convert_icon_to_image(c);
        let result = image_buffer.save_with_format(icon_path, ImageFormat::Png);
        if result.is_err() {
            warn!("Could not save image to {}", icon_path);
        }
    }
}

// from https://users.rust-lang.org/t/how-to-convert-hicon-to-png/90975/15
unsafe fn convert_icon_to_image(icon: HICON) -> RgbaImage {
    let bitmap_size_i32 = i32::try_from(mem::size_of::<BITMAP>()).unwrap();
    let biheader_size_u32 = u32::try_from(mem::size_of::<BITMAPINFOHEADER>()).unwrap();
    let mut info = ICONINFO {
        fIcon: 0,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: std::mem::size_of::<HBITMAP>() as HBITMAP,
        hbmColor: std::mem::size_of::<HBITMAP>() as HBITMAP,
    };
    GetIconInfo(icon, &mut info);

    DeleteObject(info.hbmMask as *mut VOID);
    let mut bitmap: MaybeUninit<BITMAP> = MaybeUninit::uninit();

    let result = GetObjectW(
        info.hbmColor as *mut VOID,
        bitmap_size_i32,
        bitmap.as_mut_ptr() as *mut VOID,
    );

    assert!(result == bitmap_size_i32);
    let bitmap = bitmap.assume_init_ref();

    info!(
        "width_usize={}, height_usize={}",
        bitmap.bmWidth, bitmap.bmHeight
    );

    let width_u32 = u32::try_from(bitmap.bmWidth).unwrap();
    let height_u32 = u32::try_from(bitmap.bmHeight).unwrap();
    let width_usize = usize::try_from(bitmap.bmWidth).unwrap();
    let height_usize = usize::try_from(bitmap.bmHeight).unwrap();

    let buf_size = width_usize
        .checked_mul(height_usize)
        .and_then(|size| size.checked_mul(4))
        .unwrap();
    let mut buf: Vec<u8> = Vec::with_capacity(buf_size);

    let dc: HDC = GetDC(0 as HWND);
    assert!(dc != (0 as HDC));

    let _bitmap_info = BITMAPINFOHEADER {
        biSize: biheader_size_u32,
        biWidth: bitmap.bmWidth,
        biHeight: -bitmap.bmHeight.abs(),
        biPlanes: 1,
        biBitCount: 32,
        biCompression: winapi::um::wingdi::BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let mut bmp: Vec<u8> = vec![0; buf_size];
    let _mr_right = GetBitmapBits(info.hbmColor, buf_size as i32, bmp.as_mut_ptr() as LPVOID);
    buf.set_len(bmp.capacity());
    let result = ReleaseDC(0 as HWND, dc);
    assert!(result == 1);
    DeleteObject(info.hbmColor as *mut VOID);

    for chunk in bmp.chunks_exact_mut(4) {
        let [b, _, r, _] = chunk else { unreachable!() };
        mem::swap(b, r);
    }
    RgbaImage::from_vec(width_u32, height_u32, bmp).unwrap()
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
// C:\Users\Alice\AppData\Local\Programs\software.Browsers\resources
pub fn get_this_app_resources_dir() -> PathBuf {
    return get_this_app_program_dir().join("resources");
}

// C:\Users\Alice\AppData\Local\Programs\software.Browsers
fn get_this_app_program_dir() -> PathBuf {
    return get_config_local_dir().join("Programs").join(APP_DIR_NAME);
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
