use std::ffi::{CStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::{mem, ptr};

// https://stackoverflow.com/questions/10952225/is-there-any-way-to-give-my-sandboxed-mac-app-read-only-access-to-files-in-lib
pub fn unsandboxed_home_dir() -> Option<PathBuf> {
    let os_string_maybe = unsafe { pw_dir() };
    return os_string_maybe.map(|os_string| PathBuf::from(os_string));
}

unsafe fn pw_dir() -> Option<OsString> {
    let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
        n if n < 0 => 512 as usize,
        n => n as usize,
    };
    let mut buf = Vec::with_capacity(amt);
    let mut passwd: libc::passwd = mem::zeroed();
    let mut result = ptr::null_mut();

    // if running under macos sandbox,
    // home directory can be found via getpwuid(getuid()).pw_dir
    match libc::getpwuid_r(
        libc::getuid(),
        &mut passwd,
        buf.as_mut_ptr(),
        buf.capacity(),
        &mut result,
    ) {
        0 if !result.is_null() => {
            let ptr = passwd.pw_dir as *const _;
            let bytes = CStr::from_ptr(ptr).to_bytes();
            if bytes.is_empty() {
                None
            } else {
                Some(OsStringExt::from_vec(bytes.to_vec()))
            }
        }
        _ => None,
    }
}
