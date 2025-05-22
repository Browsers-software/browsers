use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};

use crate::browser_repository::SupportedAppRepository;
use crate::macos::macos_native;
use crate::{InstalledBrowser, macos};

const APP_DIR_NAME: &'static str = "software.Browsers";
const APP_BUNDLE_ID: &'static str = "software.Browsers";

/// get macOS application support directory for this app, supports sandboxing
pub fn get_this_app_support_dir() -> PathBuf {
    macos_native::macos_get_application_support_dir_path().join(APP_DIR_NAME)
}

/// get macOS application support directory, ignores sandboxing
/// e.g $HOME/Library/Application Support
pub fn macos_get_unsandboxed_application_support_dir() -> PathBuf {
    let home_dir = macos::mac_paths::unsandboxed_home_dir().unwrap();
    home_dir.join("Library").join("Application Support")
}

// ~/
pub fn macos_get_unsandboxed_home_dir() -> PathBuf {
    macos::mac_paths::unsandboxed_home_dir().unwrap()
}

// ~/Library/Containers/com.tinyspeck.slackmacgap/Data/
pub fn macos_get_sandboxed_home_dir(app_id: &str) -> PathBuf {
    let home_dir = macos::mac_paths::unsandboxed_home_dir().unwrap();
    home_dir
        .join("Library")
        .join("Containers")
        .join(app_id)
        .join("Data")
}

// bundle_path e.g "/Applications/Slack.app"
fn has_sandbox_entitlement(bundle_path: &str) -> bool {
    let mut command = Command::new("codesign");
    command
        .arg("-d")
        .arg("--entitlements")
        .arg("-")
        .arg("--xml")
        .arg(bundle_path);

    let result = command.output();
    if result.is_err() {
        warn!("Could not check if app is sandboxed or not, defaulting to not");
        return false;
    }
    let output = result.unwrap();
    let stdout = output.stdout;
    let cow = String::from_utf8_lossy(&stdout);
    let search = "<key>com.apple.security.app-sandbox</key><true/>";

    return cow.contains(search);
}

pub struct OsHelper {
    app_repository: SupportedAppRepository,
    //unsandboxed_home_dir: PathBuf,
}

unsafe impl Send for OsHelper {}

impl OsHelper {
    pub fn new() -> OsHelper {
        let app_repository = SupportedAppRepository::new();
        Self {
            app_repository: app_repository,
            //unsandboxed_home_dir: unsandboxed_home_dir().unwrap(),
        }
    }

    pub fn get_app_repository(&self) -> &SupportedAppRepository {
        &self.app_repository
    }

    pub fn get_installed_browsers(
        &self,
        schemes: Vec<(String, Vec<String>)>,
    ) -> Vec<InstalledBrowser> {
        let mut browsers: Vec<InstalledBrowser> = Vec::new();

        let cache_root_dir = get_this_app_cache_root_dir();
        let icons_root_dir = cache_root_dir.join("icons");
        fs::create_dir_all(icons_root_dir.as_path()).unwrap();

        // to for each bundle id copy the domain
        let bundle_ids_and_domain_patterns: Vec<(String, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domain_patterns)| {
                (find_bundle_ids_for_url_scheme(scheme), domain_patterns)
            })
            .flat_map(|(bundle_ids, domain_patterns)| {
                let bundle_id_and_domains: Vec<(String, Vec<String>)> = bundle_ids
                    .iter()
                    .map(|bundle_id| (bundle_id.clone(), domain_patterns.clone()))
                    .collect();

                bundle_id_and_domains
            })
            .collect();

        for (bundle_id, domain_patterns) in bundle_ids_and_domain_patterns {
            let browser_maybe = self.to_installed_browser(
                bundle_id.as_str(),
                icons_root_dir.as_path(),
                domain_patterns,
            );
            if let Some(browser) = browser_maybe {
                info!("Added app: {:?}", browser);
                browsers.push(browser);
            }
        }

        return browsers;
    }

    fn to_installed_browser(
        &self,
        bundle_id: &str,
        icons_root_dir: &Path,
        restricted_domain_patterns: Vec<String>,
    ) -> Option<InstalledBrowser> {
        if bundle_id == "software.Browsers" {
            // this is us, skip
            return None;
        }

        let supported_app = self
            .app_repository
            .get_or_generate(bundle_id, &restricted_domain_patterns);
        let icon_filename = bundle_id.to_string() + ".png";
        let full_stored_icon_path = icons_root_dir.join(icon_filename);

        let bundle_url_maybe = macos_native::get_bundle_url(bundle_id);
        if bundle_url_maybe.is_none() {
            return None;
        }
        let bundle_url = bundle_url_maybe.unwrap();

        let bundle_path = bundle_url.to_string();
        let display_name = macos_native::get_app_name(&bundle_url);
        let executable_path = macos_native::get_app_executable_path(&bundle_url);
        let executable_path = PathBuf::from(executable_path);

        let icon_path_str = full_stored_icon_path.display().to_string();
        macos_native::create_icon_for_app(&bundle_url, icon_path_str.as_str());

        let command_parts: Vec<String> = vec![executable_path.to_str().unwrap().to_string()];

        // TODO: check if "com.apple.security.app-sandbox" entitlement exists for the app
        // TODO: https://stackoverflow.com/questions/12177948/how-do-i-detect-if-my-app-is-sandboxed
        let is_macos_sandbox = has_sandbox_entitlement(bundle_path.as_str());
        let app_config_dir_abs = supported_app.get_app_config_dir_abs(false, is_macos_sandbox);

        let browser = InstalledBrowser {
            command: command_parts,
            executable_path: executable_path.to_str().unwrap().to_string(),
            display_name: display_name.to_string(),
            bundle: supported_app.get_app_id().to_string(),
            user_dir: app_config_dir_abs.to_str().unwrap().to_string(),
            icon_path: icon_path_str.clone(),
            profiles: supported_app.find_profiles(executable_path.as_path(), app_config_dir_abs),
            restricted_domains: restricted_domain_patterns,
        };

        return Some(browser);
    }
}

// e.g /Applications/Browsers.app/Contents/Resources/
pub fn get_this_app_resources_dir() -> PathBuf {
    let app_bundle_dir = get_this_app_bundle_dir();
    app_bundle_dir.join("Contents").join("Resources")
}

// e.g /Applications/Browsers.app/
pub fn get_this_app_bundle_dir() -> PathBuf {
    get_bundle_path(APP_BUNDLE_ID).unwrap_or_else(|| get_this_app_bundle_dir_fallback())
}

fn get_this_app_bundle_dir_fallback() -> PathBuf {
    // .../Browsers.app/Contents/MacOS/browsers
    let binary_file_path =
        fs::canonicalize(std::env::current_exe().expect("Can't find current executable"))
            .expect("Can't canonicalize current executable path");

    // .../Browsers.app/Contents/MacOS/
    let binary_dir_path = binary_file_path.parent().unwrap();

    // .../Browsers.app/Contents/
    let contents_dir_path = binary_dir_path.parent().unwrap();

    // .../Browsers.app/
    let bundle_dir_path = contents_dir_path.parent().unwrap();

    bundle_dir_path.to_path_buf()
}

// e.g /Applications/<bundle>/
fn get_bundle_path(bundle_id: &str) -> Option<PathBuf> {
    macos_native::get_bundle_url(bundle_id)
        .map(|bundle_url| bundle_url.to_string())
        .map(|bundle_path| PathBuf::from(bundle_path.as_str()))
}

// ~/Library/Caches/software.Browsers/runtime/
pub fn get_this_app_runtime_dir() -> PathBuf {
    get_this_app_cache_root_dir().join("runtime")
}

// ~/Library/Caches/software.Browsers/
pub fn get_this_app_cache_root_dir() -> PathBuf {
    let cache_dir_root = macos_native::macos_get_caches_dir();
    cache_dir_root.join(APP_DIR_NAME)
}

/// get macOS logs directory for this app, supports sandboxing
pub fn get_this_app_logs_root_dir() -> PathBuf {
    return macos_get_logs_dir().join(APP_DIR_NAME);
}

/// get macOS logs directory, supports sandboxing
pub fn macos_get_logs_dir() -> PathBuf {
    macos_native::macos_get_library_dir().join("Logs")
}

pub fn get_this_app_config_root_dir() -> PathBuf {
    get_this_app_support_dir()
}

/*pub fn find_bundle_ids_for_browsers() -> Vec<String> {
    let bundle_ids_for_https = get_bundle_ids_for_url_scheme("https");

    let c = bundle_ids_for_https;
    /*let bundles_content_type = bundle_ids_for_content_type();

    let c = bundle_ids_for_https
        .intersection(&bundles_content_type)
        .collect::<Vec<_>>();*/

    let mut vec = c.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    vec.sort();
    return vec;
}

pub fn bundle_ids_for_content_type() -> HashSet<String> {
    // kUTTypeHTML
    // not present for Firefox (ff uses deprecated CFBundleTypeExtensions)
    let content_type = CFString::new("public.html");
    //let in_content_type = cfs.as_concrete_TypeRef();
    let role = core_services::kLSRolesAll;

    unsafe {
        let handlers_content_type = core_services::LSCopyAllRoleHandlersForContentType(
            content_type.as_concrete_TypeRef(),
            role,
        );
        if handlers_content_type.is_null() {
            return HashSet::new();
        }

        let handlers_content_type: CFArray<CFString> =
            core_services::TCFType::wrap_under_create_rule(handlers_content_type);

        let bundles_content_type = handlers_content_type
            .iter()
            .map(|h| String::from(h.to_string()))
            .collect::<HashSet<_>>();

        return bundles_content_type;
    }
}*/

pub fn find_bundle_ids_for_url_scheme(scheme: &str) -> Vec<String> {
    let bundle_ids = macos_native::get_bundle_ids_for_url_scheme(scheme);
    let mut vec = bundle_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    vec.sort();
    return vec;
}

// returns true if it was already default web browser (then nothing was done)
pub(crate) fn set_default_web_browser() -> bool {
    if macos_native::is_default_web_browser() {
        return true;
    }

    macos_native::set_default_web_browser()
}

pub(crate) fn is_default_web_browser() -> bool {
    macos_native::is_default_web_browser()
}
