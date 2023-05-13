use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
extern crate winres;

#[cfg(target_os = "macos")]
fn main() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");
    let template_info_plist_path: PathBuf = Path::new(ROOT_DIR)
        .join("extra")
        .join("macos")
        .join("Info.plist");

    let target_info_plist_dir_path: PathBuf = Path::new(ROOT_DIR)
        .join("target")
        .join("universal-apple-darwin")
        .join("meta");

    let target_info_plist_path: PathBuf = target_info_plist_dir_path.join("Info.plist");

    fs::create_dir_all(target_info_plist_dir_path).unwrap();

    let info_plist_content = fs::read_to_string(template_info_plist_path).unwrap();
    let new_info_plist_content = info_plist_content
        .replace(
            "<string>$CFBundleShortVersion$</string>",
            format!("<string>{VERSION}</string>").as_str(),
        )
        .replace(
            "<string>$CFBundleVersion$</string>",
            format!("<string>{VERSION}</string>").as_str(),
        );

    let mut file = File::create(target_info_plist_path.as_path()).unwrap();
    file.write_all(new_info_plist_content.as_bytes()).unwrap();
}

#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("extra/windows/icons/browsers.ico");
    res.compile().unwrap();
}

#[cfg(target_os = "linux")]
fn main() {}
