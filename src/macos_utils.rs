use std::collections::HashSet;
use std::ffi::{CStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::{fs, mem, ptr};

use cocoa_foundation::base::{id, nil};
use cocoa_foundation::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString, NSURL};
use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_foundation::string::CFStringRef;
use objc::runtime::Object;
use objc::runtime::YES;
use objc::{class, msg_send, sel, sel_impl};
use tracing::{debug, info};

use crate::browser_repository::SupportedAppRepository;
use crate::{macos, InstalledBrowser};

const APP_DIR_NAME: &'static str = "software.Browsers";
const APP_BUNDLE_ID: &'static str = "software.Browsers";

/// Create a new NSString from a &str.
pub(crate) fn make_nsstring(s: &str) -> *mut Object {
    unsafe { NSString::alloc(nil).init_str(s).autorelease() }
}

pub(crate) fn from_nsstring(s: *mut Object) -> String {
    unsafe {
        let slice = std::slice::from_raw_parts(s.UTF8String() as *const _, s.len());
        let result = std::str::from_utf8_unchecked(slice);
        result.into()
    }
}

pub fn create_icon_for_app(full_path: id, icon_path: &str) {
    unsafe {
        let workspace = class!(NSWorkspace);
        let shared: *mut Object = msg_send![workspace, sharedWorkspace];

        // NSImage
        let icon: id = msg_send![shared, iconForFile: full_path];

        // resize to smaller
        let size = NSSize::new(64.0, 64.0);

        //this makes NSImage create a new representation
        let _a: id = msg_send![icon, setScalesWhenResized: true];
        let _a: id = msg_send![icon, setSize: size];

        let tiff: id = msg_send![icon, TIFFRepresentation];

        let ns_bitmap_image_rep = class!(NSBitmapImageRep);

        let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);

        // NSBitmapImageRep
        let rep_from_tiff: *mut Object = msg_send![ns_bitmap_image_rep, imageRepWithData: tiff];

        let _a: id = msg_send![icon, lockFocus];
        // NSBitmapImageRep
        let rep: id = msg_send![rep_from_tiff, initWithFocusedViewRect: rect];
        let _b: id = msg_send![icon, unlockFocus];

        let ns_png_file_type: i32 = 4;
        // NSData
        let icon_png: *mut Object =
            msg_send![rep, representationUsingType:ns_png_file_type properties: nil];

        //let width: NSInteger = msg_send![icon_png, pixelsWide];

        //let filename_str = format!("icons/{}.png", bundle_id);
        let filename = NSString::alloc(nil).init_str(icon_path);
        let _: () = msg_send![icon_png, writeToFile: filename atomically: YES];

        //let raw = from_nsdata(icon_png);

        //info!("ok");
        //return (raw, filename_str);
    }
}

// returns nsstring
pub fn get_bundle_url(bundle_id: &str) -> id {
    debug!("Getting url for bundle: {}", bundle_id);

    let bundle_id_nsstring = make_nsstring(bundle_id);

    unsafe {
        let workspace = class!(NSWorkspace);
        let shared: *mut Object = msg_send![workspace, sharedWorkspace];

        //let _: () = msg_send![obj, setArg1:1 arg2:2];

        let url: id = msg_send![
            shared,
            URLForApplicationWithBundleIdentifier: bundle_id_nsstring
        ];

        // fileSystemRepresentation is commented out :S
        let url_str = url.path();
        let path = from_nsstring(url_str);
        debug!("App path: {}", path);

        return url_str;

        //let url_string = from_nsstring(url_str);
    }
}

//noinspection RsConstNaming
const NS_CACHES_DIRECTORY: u64 = 13;

//noinspection RsConstNaming
const NS_APPLICATION_SUPPORT_DIRECTORY: u64 = 14;

// potentially sandboxed
const NS_LIBRARY_DIRECTORY: u64 = 5;

/// get macOS application support directory for this app, supports sandboxing
pub fn get_this_app_support_dir() -> PathBuf {
    return macos_get_application_support_dir_path().join(APP_DIR_NAME);
}

/// get macOS application support directory, supports sandboxing
pub fn macos_get_application_support_dir_path() -> PathBuf {
    macos_get_directory(NS_APPLICATION_SUPPORT_DIRECTORY)
}

/// get macOS application support directory, ignores sandboxing
/// e.g $HOME/Library/Application Support
pub fn macos_get_unsandboxed_application_support_dir() -> PathBuf {
    let home_dir = macos::mac_paths::unsandboxed_home_dir().unwrap();
    return home_dir.join("Library").join("Application Support");
}

/// get macOS standard directory, supports sandboxing
pub fn macos_get_directory(directory: u64) -> PathBuf {
    let results = unsafe { NSSearchPathForDirectoriesInDomains(directory, 1, 1) };
    let results = unsafe { CFArray::<CFString>::wrap_under_get_rule(results) };

    let option = results.get(0);
    if option.is_none() {
        panic!("no")
    }

    let x = option.unwrap().to_string();

    return PathBuf::from(x);
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub fn NSSearchPathForDirectoriesInDomains(
        directory: u64,
        domain_mask: u64,
        expand_tilde: i8,
    ) -> CFArrayRef;
}

#[link(name = "Foundation", kind = "framework")]
extern "C" {
    pub fn LSCopyAllHandlersForURLScheme(in_url_scheme: CFStringRef) -> CFArrayRef;
}

extern "C" {
    pub static kCFBundleNameKey: CFStringRef;
    pub static kCFBundleExecutableKey: CFStringRef;
}

fn get_app_name(bundle_path: id) -> String {
    let bundle = get_bundle(bundle_path);
    //bundleWithURL
    unsafe {
        // returns NSString
        let bundle_name: id = msg_send![bundle, objectForInfoDictionaryKey: kCFBundleNameKey];
        let bundle_name_str = from_nsstring(bundle_name);
        return bundle_name_str;
    }
}

fn get_app_executable(bundle_path: id) -> String {
    let bundle = get_bundle(bundle_path);
    //bundleWithURL
    unsafe {
        // returns NSString
        // apparently CFBundleExecutableKey is optional, but let's see if this ever causes issues
        // for apps
        let executable_name: id =
            msg_send![bundle, objectForInfoDictionaryKey: kCFBundleExecutableKey];
        let bundle_name_str = from_nsstring(executable_name);
        return bundle_name_str;
    }
}

// returns NSBundle
fn get_bundle(bundle_path: id) -> id {
    //bundleWithURL
    unsafe {
        // returns NSBundle
        let bundle: id = msg_send![class!(NSBundle), bundleWithPath: bundle_path];
        return bundle;
    }
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

        // to for each bundle id copy the domain
        let bundle_ids_and_domains: Vec<(String, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domains)| (find_bundle_ids_for_url_scheme(scheme), domains))
            .flat_map(|(bundle_ids, domains)| {
                let bundle_id_and_domains: Vec<(String, Vec<String>)> = bundle_ids
                    .iter()
                    .map(|bundle_id| (bundle_id.clone(), domains.clone()))
                    .collect();

                bundle_id_and_domains
            })
            .collect();

        for (bundle_id, domains) in bundle_ids_and_domains {
            let browser_maybe =
                self.to_installed_browser(bundle_id.as_str(), icons_root_dir.as_path(), domains);
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
        restricted_domains: Vec<String>,
    ) -> Option<InstalledBrowser> {
        if bundle_id == "software.Browsers" {
            // this is us, skip
            return None;
        }

        let supported_app = self
            .app_repository
            .get_or_generate(bundle_id, &restricted_domains);
        let icon_filename = bundle_id.to_string() + ".png";
        let full_stored_icon_path = icons_root_dir.join(icon_filename);

        let bundle_url = get_bundle_url(bundle_id);

        let bundle_path = from_nsstring(bundle_url);
        let display_name = get_app_name(bundle_url);
        let executable_name = get_app_executable(bundle_url);
        let executable_path = Path::new(bundle_path.as_str())
            .join("Contents")
            .join("MacOS")
            .join(executable_name.as_str());

        let icon_path_str = full_stored_icon_path.display().to_string();
        create_icon_for_app(bundle_url, icon_path_str.as_str());

        let browser = InstalledBrowser {
            executable_path: executable_path.to_str().unwrap().to_string(),
            display_name: display_name.to_string(),
            bundle: supported_app.get_app_id().to_string(),
            user_dir: supported_app.get_app_config_dir_absolute(false).to_string(),
            icon_path: icon_path_str.clone(),
            profiles: supported_app.find_profiles(executable_path.as_path(), false),
            restricted_domains: restricted_domains,
        };

        return Some(browser);
    }
}

// e.g /Applications/Browsers.app/
pub fn get_this_app_bundle_dir() -> PathBuf {
    get_bundle_path(APP_BUNDLE_ID)
}

// e.g /Applications/<bundle>/
fn get_bundle_path(bundle_id: &str) -> PathBuf {
    let bundle_url = get_bundle_url(bundle_id);
    let bundle_path = from_nsstring(bundle_url);
    return PathBuf::from(bundle_path.as_str());
}

pub fn get_this_app_cache_root_dir() -> PathBuf {
    let cache_dir_root = macos_get_caches_dir();
    return cache_dir_root.join(APP_DIR_NAME);
}

/// get macOS caches directory, supports sandboxing
pub fn macos_get_caches_dir() -> PathBuf {
    macos_get_directory(NS_CACHES_DIRECTORY)
}

/// get macOS logs directory for this app, supports sandboxing
pub fn get_this_app_logs_root_dir() -> PathBuf {
    return macos_get_logs_dir().join(APP_DIR_NAME);
}

/// get macOS logs directory, supports sandboxing
pub fn macos_get_logs_dir() -> PathBuf {
    return macos_get_library_dir().join("Logs");
}

/// get macOS library directory, supports sandboxing
pub fn macos_get_library_dir() -> PathBuf {
    macos_get_directory(NS_LIBRARY_DIRECTORY)
}

pub fn get_this_app_config_root_dir() -> PathBuf {
    return get_this_app_support_dir();
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
    let bundle_ids = get_bundle_ids_for_url_scheme(scheme);
    let mut vec = bundle_ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    vec.sort();
    return vec;
}

// check schemes from an apps Info.plist CFBundleUrlTypes.CFBundleURLSchemes
pub fn get_bundle_ids_for_url_scheme(scheme: &str) -> Vec<String> {
    let app_ids: HashSet<String> = unsafe {
        // https scheme has some apps which are not browsers, e.g iterm2, Folx
        let handlers_https =
            LSCopyAllHandlersForURLScheme(CFString::new(scheme).as_concrete_TypeRef());
        if handlers_https.is_null() {
            return Vec::new();
        }

        let handlers_https: CFArray<CFString> = TCFType::wrap_under_create_rule(handlers_https);

        let mut vec = handlers_https
            .iter()
            .map(|h| String::from(h.to_string()))
            .collect::<Vec<_>>();

        vec.sort();

        let bundles_https = vec
            .iter()
            .map(|h| String::from(h.to_string()))
            .collect::<HashSet<_>>();
        bundles_https
    };

    Vec::from_iter(app_ids)
}

// returns true if it was already default web browser (then nothing was done)
pub fn set_default_web_browser() -> bool {
    if is_default_web_browser() {
        return true;
    }

    let bundle_id = "software.Browsers";
    let bundle_id = CFString::new(bundle_id);
    let bundle_id_ref = bundle_id.as_concrete_TypeRef();

    let https_scheme = CFString::new("https");
    let https_scheme_ref = https_scheme.as_concrete_TypeRef();

    let http_scheme = CFString::new("http");
    let http_scheme_ref = http_scheme.as_concrete_TypeRef();

    unsafe {
        LSSetDefaultHandlerForURLScheme(https_scheme_ref, bundle_id_ref);
        LSSetDefaultHandlerForURLScheme(http_scheme_ref, bundle_id_ref);
    }

    return false;
}

pub fn is_default_web_browser() -> bool {
    let bundle_id = "software.Browsers";

    let https_scheme = CFString::new("https");
    let https_scheme_ref = https_scheme.as_concrete_TypeRef();

    let http_scheme = CFString::new("http");
    let http_scheme_ref = http_scheme.as_concrete_TypeRef();

    let https_bundle = unsafe { LSCopyDefaultHandlerForURLScheme(https_scheme_ref) };
    let https_bundle = from_nsstring(https_bundle);

    let http_bundle = unsafe { LSCopyDefaultHandlerForURLScheme(http_scheme_ref) };
    let http_bundle = from_nsstring(http_bundle);

    return https_bundle == bundle_id && http_bundle == bundle_id;
}

#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn LSSetDefaultHandlerForURLScheme(scheme: CFStringRef, bundle_id: CFStringRef);

    // returns bundle id
    fn LSCopyDefaultHandlerForURLScheme(scheme: CFStringRef) -> id;
}
