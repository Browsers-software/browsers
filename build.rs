#[cfg(target_os = "windows")]
extern crate winres;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

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

    // statically link vcruntime140.dll instead of requiring user to install the runtime
    static_vcruntime::metabuild();
}

#[cfg(target_os = "linux")]
fn main() {
    // x86_64, aarch64, arm, i686
    // https://doc.rust-lang.org/reference/conditional-compilation.html#target_arch
    let rust_target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    // e.g "/target/aarch64-unknown-linux-gnu/release/build/browsers-f4fff742057613df/out"
    //  or "/target/release/build/browsers-f4fff742057613df/out"
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = PathBuf::from(out_dir.as_str());

    // /target/aarch64-unknown-linux-gnu
    let arch_target_dir = out_dir
        .parent() // release/build/browsers/
        .map(|a| a.parent()) // target/.../release/build/
        .flatten()
        .map(|a| a.parent()) // target/.../release/
        .flatten()
        .map(|a| a.parent()) // target/
        .flatten()
        .unwrap();

    create_deb_control(rust_target_arch.as_str(), arch_target_dir);
    create_rpm_spec(rust_target_arch.as_str(), arch_target_dir);
}

#[cfg(target_os = "linux")]
fn create_deb_control(rust_target_arch: &str, arch_target_dir: &Path) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

    // https://wiki.debian.org/SupportedArchitectures
    let deb_arch = match rust_target_arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "arm" => "armhf",
        "i686" => "i386",
        _ => "unknown",
    };

    let template_deb_control_file_path: PathBuf = Path::new(ROOT_DIR)
        .join("extra")
        .join("linux")
        .join("deb")
        .join("DEBIAN")
        .join("template.control");

    // /target/aarch64-unknown-linux-gnu/meta/deb_control
    let target_deb_control_dir_path = arch_target_dir.join("meta").join("deb_control");

    // /target/aarch64-unknown-linux-gnu/meta/deb_control/control
    let target_deb_control_path: PathBuf = target_deb_control_dir_path.join("control");

    fs::create_dir_all(target_deb_control_dir_path).unwrap();

    // 7 MB estimate at the moment
    const INSTALLED_SIZE_KB: &str = "7168";

    let deb_control_content = fs::read_to_string(template_deb_control_file_path).unwrap();
    let new_deb_control_content = deb_control_content
        .replace("€Version€", VERSION)
        .replace("€Architecture€", deb_arch)
        .replace("€InstalledSize€", INSTALLED_SIZE_KB);

    let mut file = File::create(target_deb_control_path.as_path()).unwrap();
    file.write_all(new_deb_control_content.as_bytes()).unwrap();
}

#[cfg(target_os = "linux")]
fn create_rpm_spec(rust_target_arch: &str, arch_target_dir: &Path) {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");

    let rpm_arch = match rust_target_arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        "arm" => "armhfp",
        "i686" => "i386",
        _ => "unknown",
    };

    let template_rpm_control_file_path: PathBuf = Path::new(ROOT_DIR)
        .join("extra")
        .join("linux")
        .join("rpm")
        .join("SPECS")
        .join("template.browsers.spec");

    // /target/aarch64-unknown-linux-gnu/meta/rpm_spec/
    let target_rpm_spec_dir_path = arch_target_dir.join("meta").join("rpm_spec");

    // /target/aarch64-unknown-linux-gnu/meta/rpm_spec/browsers.spec
    let target_rpm_spec_file_path: PathBuf = target_rpm_spec_dir_path.join("browsers.spec");

    fs::create_dir_all(target_rpm_spec_dir_path).unwrap();

    // 7 MB estimate at the moment
    const INSTALLED_SIZE_KB: &str = "7168";

    let rpm_control_content = fs::read_to_string(template_rpm_control_file_path).unwrap();
    let new_rpm_control_content = rpm_control_content
        .replace("€Version€", VERSION)
        .replace("€Architecture€", rpm_arch)
        .replace("€InstalledSize€", INSTALLED_SIZE_KB);

    let mut file = File::create(target_rpm_spec_file_path.as_path()).unwrap();
    file.write_all(new_rpm_control_content.as_bytes()).unwrap();
}
