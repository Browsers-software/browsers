use objc2::AnyThread;
use objc2::rc::Retained;
use std::collections::HashSet;
use std::ops::Deref;
use std::path::PathBuf;

use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
use objc2_foundation::{
    NSArray, NSBundle, NSDictionary, NSPoint, NSRect, NSSearchPathDirectory,
    NSSearchPathDomainMask, NSSearchPathForDirectoriesInDomains, NSSize, NSString,
};
use tracing::debug;

pub fn create_icon_for_app(full_path: &NSString, icon_path: &str) {
    unsafe {
        let shared_workspace = NSWorkspace::sharedWorkspace();

        let size = NSSize::new(64.0, 64.0);

        // NSImage
        let icon = shared_workspace.iconForFile(full_path);
        // resize to smaller
        icon.setSize(size);

        icon.lockFocus();

        let tiff = icon.TIFFRepresentation().unwrap();

        let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
        let rep_from_tiff = NSBitmapImageRep::imageRepWithData(&tiff).unwrap();

        // draws icon into the rectangle
        rep_from_tiff.drawInRect(rect);

        let rect_as_image =
            NSBitmapImageRep::initWithFocusedViewRect(NSBitmapImageRep::alloc(), rect).unwrap();
        icon.unlockFocus();

        let icon_png = rect_as_image
            .representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
            .unwrap();

        icon_png.writeToFile_atomically(&NSString::from_str(icon_path), true);
    }
}

pub fn get_bundle_url(bundle_id: &str) -> Option<Retained<NSString>> {
    debug!("Getting url for bundle: {}", bundle_id);

    unsafe {
        let shared_workspace = NSWorkspace::sharedWorkspace();

        // The URL of the app, or None if no app has the bundle identifier.
        shared_workspace
            .URLForApplicationWithBundleIdentifier(&NSString::from_str(bundle_id))
            .and_then(|url| url.relativePath())
    }
}

/// get macOS application support directory, supports sandboxing
pub fn macos_get_application_support_dir_path() -> PathBuf {
    macos_get_directory(NSSearchPathDirectory::ApplicationSupportDirectory)
}

/// get macOS caches directory, supports sandboxing
pub fn macos_get_caches_dir() -> PathBuf {
    macos_get_directory(NSSearchPathDirectory::CachesDirectory)
}

/// get macOS library directory, supports sandboxing
pub fn macos_get_library_dir() -> PathBuf {
    // LibraryDirectory is potentially sandboxed
    macos_get_directory(NSSearchPathDirectory::LibraryDirectory)
}

/// get macOS standard directory, supports sandboxing
pub fn macos_get_directory(directory: NSSearchPathDirectory) -> PathBuf {
    let results = unsafe {
        NSSearchPathForDirectoriesInDomains(directory, NSSearchPathDomainMask::UserDomainMask, true)
    };

    //let results = unsafe { CFArray::<CFString>::wrap_under_get_rule(results) };

    let option = results.firstObject();
    if option.is_none() {
        panic!("no")
    }

    let x = option.unwrap().to_string();

    PathBuf::from(x)
}

pub(crate) fn get_app_name(bundle_path: &NSString) -> String {
    let bundle = get_bundle(bundle_path);
    //bundleWithURL
    let bundle_name = bundle.name().unwrap();
    bundle_name.to_string()
}

pub(crate) fn get_app_executable_path(bundle_path: &NSString) -> String {
    let bundle = get_bundle(bundle_path);

    //bundleWithURL
    unsafe {
        let executable_path = bundle.executablePath().unwrap();
        executable_path.to_string()
    }
}

// returns NSBundle
fn get_bundle(bundle_path: &NSString) -> Retained<NSBundle> {
    //bundleWithURL
    unsafe {
        let bundle = NSBundle::bundleWithPath(bundle_path).unwrap();
        bundle
    }
}

// check schemes from an apps Info.plist CFBundleUrlTypes.CFBundleURLSchemes
pub fn get_bundle_ids_for_url_scheme(scheme: &str) -> Vec<String> {
    let scheme = NSString::from_str(scheme);

    let mut scheme_handlers = unsafe {
        // https scheme has some apps which are not browsers, e.g iterm2, Folx
        let scheme_handlers = LSCopyAllHandlersForURLScheme(&scheme);

        if scheme_handlers.is_none() {
            return Vec::new();
        }

        scheme_handlers
            .unwrap()
            .iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
    };

    scheme_handlers.sort();

    let app_ids = scheme_handlers
        .iter()
        .map(|h| String::from(h.to_string()))
        .collect::<HashSet<_>>();

    Vec::from_iter(app_ids)
}

pub fn set_default_web_browser() -> bool {
    let bundle_id = "software.Browsers";
    let bundle_id = NSString::from_str(bundle_id);
    let bundle_id = bundle_id.deref();

    let https_scheme = NSString::from_str("https");
    let https_scheme = https_scheme.deref();

    let http_scheme = NSString::from_str("http");
    let http_scheme = http_scheme.deref();

    unsafe {
        LSSetDefaultHandlerForURLScheme(https_scheme, bundle_id);
        LSSetDefaultHandlerForURLScheme(http_scheme, bundle_id);
    }

    return false;
}

pub fn is_default_web_browser() -> bool {
    let bundle_id = "software.Browsers";
    //let bundle_id = NSString::from_str(bundle_id);
    //let bundle_id = bundle_id.deref();

    let https_scheme = NSString::from_str("https");
    let https_scheme = https_scheme.deref();

    let http_scheme = NSString::from_str("http");
    let http_scheme = http_scheme.deref();

    let https_bundle = unsafe { LSCopyDefaultHandlerForURLScheme(https_scheme) };
    let https_bundle = https_bundle.to_string();

    let http_bundle = unsafe { LSCopyDefaultHandlerForURLScheme(http_scheme) };
    let http_bundle = http_bundle.to_string();

    return https_bundle == bundle_id && http_bundle == bundle_id;
}

fn has_sandbox_entitlement2(bundle_url: &NSString) -> bool {
    unsafe {
        let is_sandboxed = false;
        //SecStaticCodeCreateWithPath(bundle_url, 0, nil)
        //CFUrlRef *bundleURL = [[NSBundle mainBundle] bundleURL];

        // Can use https://stackoverflow.com/a/42244464/752697
        /*
        BOOL isSandboxed = NO;

        SecStaticCodeRef staticCode = NULL;
        NSURL *bundleURL = [[NSBundle mainBundle] bundleURL];

        if (SecStaticCodeCreateWithPath((__bridge CFURLRef)bundleURL, kSecCSDefaultFlags, &staticCode) == errSecSuccess) {
            if (SecStaticCodeCheckValidityWithErrors(staticCode, kSecCSBasicValidateOnly, NULL, NULL) == errSecSuccess) {
                SecRequirementRef sandboxRequirement;
                if (SecRequirementCreateWithString(CFSTR("entitlement[\"com.apple.security.app-sandbox\"] exists"), kSecCSDefaultFlags,
                                               &sandboxRequirement) == errSecSuccess)
                {
                    OSStatus codeCheckResult = SecStaticCodeCheckValidityWithErrors(staticCode, kSecCSBasicValidateOnly, sandboxRequirement, NULL);
                    if (codeCheckResult == errSecSuccess) {
                        isSandboxed = YES;
                    }
                }
            }
            CFRelease(staticCode);
        }
        */
    }

    return false;

    // Or use codesign utility:
    // codesign - d - -entitlements - --xml "/Applications/Slack.app"

    // TODO: check if "com.apple.security.app-sandbox" key exists and if it's value is true
    /*
    Executable=/Applications/Slack.app/Contents/MacOS/Slack
    <?xml version="1.0" encoding="UTF-8"?><!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
        "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
        <dict>
            <key>com.apple.security.app-sandbox</key>
            <true/>
            <key>com.apple.security.application-groups</key>
            <array>
                <string>BQR82RBBHL.com.tinyspeck.slackmacgap</string>
                <string>BQR82RBBHL.slack</string>
            </array>
            <key>com.apple.security.device.camera</key>
            <true/>
            <key>com.apple.security.device.microphone</key>
            <true/>
            <key>com.apple.security.device.usb</key>
            <true/>
            <key>com.apple.security.files.bookmarks.app-scope</key>
            <true/>
            <key>com.apple.security.files.downloads.read-write</key>
            <true/>
            <key>com.apple.security.files.user-selected.read-write</key>
            <true/>
            <key>com.apple.security.network.client</key>
            <true/>
            <key>com.apple.security.network.server</key>
            <true/>
            <key>com.apple.security.print</key>
            <true/>
        </dict>
    </plist>
     */
}

#[link(name = "CoreServices", kind = "framework")]
unsafe extern "C" {
    fn LSSetDefaultHandlerForURLScheme(scheme: &NSString, bundle_id: &NSString);

    // returns bundle id
    fn LSCopyDefaultHandlerForURLScheme(scheme: &NSString) -> Retained<NSString>;
}

#[link(name = "Foundation", kind = "framework")]
unsafe extern "C" {
    pub fn LSCopyAllHandlersForURLScheme(
        in_url_scheme: &NSString,
    ) -> Option<Retained<NSArray<NSString>>>;
}
