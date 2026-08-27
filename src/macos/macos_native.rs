use objc2::AnyThread;
use objc2::rc::Retained;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ffi::c_void;
use std::ops::Deref;
use std::path::PathBuf;

use objc2::runtime::AnyObject;
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBitmapImageFileType, NSBitmapImageRep, NSEvent,
    NSFloatingWindowLevel, NSScreen, NSView, NSWindowCollectionBehavior, NSWorkspace,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSBundle, NSDictionary, NSNotification, NSNotificationCenter,
    NSObject, NSObjectProtocol, NSPoint, NSRect, NSSearchPathDirectory, NSSearchPathDomainMask,
    NSSearchPathForDirectoriesInDomains, NSSize, NSString,
};
use tracing::{debug, warn};

use crate::gui::screen::{Point, Rect};

// cursor position + usable screen area, so the popup shows up where you clicked
pub fn mouse_position_and_work_area() -> (Point, Rect) {
    let mtm = MainThreadMarker::new().expect("must be called from the main thread");
    let mouse_location = NSEvent::mouseLocation();
    let screens = NSScreen::screens(mtm);

    let primary_height = screens
        .firstObject()
        .map(|s| s.frame().size.height)
        .unwrap_or(0.0);

    let mut visible_frame: Option<NSRect> = None;
    for screen in screens.iter() {
        let frame = screen.frame();
        let contains = mouse_location.x >= frame.origin.x
            && mouse_location.x <= frame.origin.x + frame.size.width
            && mouse_location.y >= frame.origin.y
            && mouse_location.y <= frame.origin.y + frame.size.height;
        if contains {
            visible_frame = Some(screen.visibleFrame());
            break;
        }
    }

    let visible_frame = visible_frame.unwrap_or_else(|| {
        screens
            .firstObject()
            .map(|s| s.visibleFrame())
            .unwrap_or(NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(1920.0, 1080.0)))
    });

    let point = Point {
        x: mouse_location.x as f32,
        y: (primary_height - mouse_location.y) as f32,
    };

    let rect = Rect {
        x0: visible_frame.origin.x as f32,
        y0: (primary_height - (visible_frame.origin.y + visible_frame.size.height)) as f32,
        x1: (visible_frame.origin.x + visible_frame.size.width) as f32,
        y1: (primary_height - visible_frame.origin.y) as f32,
    };

    (point, rect)
}

// always-on-top isn't documented for macOS in Slint, so we do it natively here (see gui::app::run)
// not setting CanJoinAllSpaces on purpose, the popup should stay on the Space it was triggered
// from and not follow you around
pub fn make_window_floating(ns_view_ptr: *mut c_void) {
    unsafe {
        let ns_view = &*(ns_view_ptr as *const NSView);
        if let Some(window) = ns_view.window() {
            window.setLevel(NSFloatingWindowLevel);
            window.setCollectionBehavior(NSWindowCollectionBehavior::FullScreenAuxiliary);
        } else {
            warn!("could not find NSWindow for view to make it floating");
        }
    }
}

pub struct EventBridgeIvars {
    resign_active: RefCell<Option<Box<dyn Fn()>>>,
    // (sender bundle id, if known; url)
    open_urls: RefCell<Option<Box<dyn Fn(Option<String>, String)>>>,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - EventBridge does not implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = EventBridgeIvars]
    pub struct EventBridge;

    unsafe impl NSObjectProtocol for EventBridge {}

    impl EventBridge {
        #[unsafe(method(handleAppResignActive:))]
        fn handle_app_resign_active(&self, _notification: &NSNotification) {
            if let Some(cb) = self.ivars().resign_active.borrow().as_ref() {
                cb();
            }
        }

        #[unsafe(method(handleGetURLEvent:withReplyEvent:))]
        fn handle_get_url_event(&self, event: *mut AnyObject, _reply: *mut AnyObject) {
            // event is an NSAppleEventDescriptor*; paramDescriptorForKeyword: keyDirectObject
            // (four-char code '----') gives the URL as a UTF-8 string param.
            const KEY_DIRECT_OBJECT: u32 = 0x2d2d2d2d; // '----'
            unsafe {
                let desc: *mut AnyObject =
                    msg_send![event, paramDescriptorForKeyword: KEY_DIRECT_OBJECT];
                if desc.is_null() {
                    return;
                }
                let ns_string: *mut NSString = msg_send![desc, stringValue];
                if ns_string.is_null() {
                    return;
                }
                let url_string = (*ns_string).to_string();
                let sender_bundle_id = sender_bundle_id(event);
                if let Some(cb) = self.ivars().open_urls.borrow().as_ref() {
                    cb(sender_bundle_id, url_string);
                }
            }
        }
    }
);

// bundle id of whoever sent us this GetURL event, for source_app-based opening rules
fn sender_bundle_id(event: *mut AnyObject) -> Option<String> {
    const KEY_ADDRESS_ATTR: u32 = 0x61646472; // 'addr'
    const TYPE_APPLICATION_BUNDLE_ID: u32 = 0x62756e64; // 'bund'
    unsafe {
        let sender: *mut AnyObject =
            msg_send![event, attributeDescriptorForKeyword: KEY_ADDRESS_ATTR];
        if sender.is_null() {
            return None;
        }
        let coerced: *mut AnyObject =
            msg_send![sender, coerceToDescriptorType: TYPE_APPLICATION_BUNDLE_ID];
        if coerced.is_null() {
            return None;
        }
        let ns_string: *mut NSString = msg_send![coerced, stringValue];
        if ns_string.is_null() {
            return None;
        }
        Some((*ns_string).to_string())
    }
}

impl EventBridge {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(EventBridgeIvars {
            resign_active: RefCell::new(None),
            open_urls: RefCell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }

    pub fn set_resign_active_handler(&self, f: impl Fn() + 'static) {
        *self.ivars().resign_active.borrow_mut() = Some(Box::new(f));
    }

    // f gets (sender bundle id if we could resolve it, url)
    pub fn set_open_urls_handler(&self, f: impl Fn(Option<String>, String) + 'static) {
        *self.ivars().open_urls.borrow_mut() = Some(Box::new(f));
    }
}

// hides the Dock icon via activation policy Accessory - needed alongside Info.plist's
// LSUIElement, which only kicks in once packaged as a .app, so this also covers the raw dev binary
pub fn hide_dock_icon() {
    let Some(mtm) = MainThreadMarker::new() else {
        warn!("hide_dock_icon must be called from the main thread");
        return;
    };
    NSApplication::sharedApplication(mtm)
        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
}

// native dark mode check - works before any window exists, unlike Slint's Palette.color-scheme.
// only used for the very first paint, see theme::detect_system_is_dark for why
pub fn is_dark_mode() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        warn!("is_dark_mode must be called from the main thread");
        return false;
    };
    let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();

    // https://developer.apple.com/documentation/appkit/nsappearance/name-swift.struct
    appearance.name().to_string().contains("Dark")
}

// sets up native hooks for app-resign-active (quit_on_lost_focus) and GetURL apple events.
// doesn't touch NSApplication's delegate, so it won't step on winit's.
//
// call this before anything else in main() - on a cold launch via URL scheme, macOS can deliver
// GetURL before our own setup runs, and if nothing's registered yet the event is just gone.
// keep the returned EventBridge alive for the life of the process (only weak refs held
// elsewhere), and wire up the real callbacks later with
// set_resign_active_handler/set_open_urls_handler
pub fn register_event_bridge() -> Option<Retained<EventBridge>> {
    let mtm = match MainThreadMarker::new() {
        Some(mtm) => mtm,
        None => {
            warn!("register_event_bridge must be called from the main thread");
            return None;
        }
    };

    let bridge = EventBridge::new(mtm);

    unsafe {
        let center = NSNotificationCenter::defaultCenter();
        center.addObserver_selector_name_object(
            &bridge,
            sel!(handleAppResignActive:),
            Some(objc2_app_kit::NSApplicationDidResignActiveNotification),
            None,
        );

        let manager: *mut AnyObject =
            msg_send![objc2::class!(NSAppleEventManager), sharedAppleEventManager];
        const CLASS_INTERNET: u32 = 0x4755524c; // 'GURL'
        const EVENT_ID_GET_URL: u32 = 0x4755524c; // 'GURL'
        let _: () = msg_send![
            manager,
            setEventHandler: &*bridge,
            andSelector: sel!(handleGetURLEvent:withReplyEvent:),
            forEventClass: CLASS_INTERNET,
            andEventID: EVENT_ID_GET_URL
        ];
    }

    Some(bridge)
}

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
    bundle
        .name() // Info.plist -> CFBundleName (optional)
        .unwrap_or_else(|| {
            bundle_path
                .lastPathComponent()
                .stringByDeletingPathExtension() // SomeBrowser.app -> SomeBrowser
        })
        .to_string()
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
