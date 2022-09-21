use std::collections::HashMap;
use std::path::Path;

use configparser::ini::{Ini, IniDefault};
use naive_cityhash::cityhash64;
use tracing::{debug, info};

use crate::InstalledBrowserProfile;

pub fn find_firefox_profiles(
    firefox_profiles_dir: &Path,
    binary_path: &Path,
) -> Vec<InstalledBrowserProfile> {
    let binary_dir = binary_path.parent().and_then(|p| p.to_str()).unwrap_or("");
    // binary_dir is the path where binary is (without trailing slash)
    let install_dir_hash = hash_firefox_install_dir(binary_dir);
    //run_arg = r#"open -b com.google.Chrome -n --args --profile-directory="Profile 1" https://www.google.com"#

    let mut browser_profiles: Vec<InstalledBrowserProfile> = Vec::new();

    let profiles_ini_path = firefox_profiles_dir.join("profiles.ini");
    debug!("profiles_ini_path: {:?}", profiles_ini_path);

    let mut ini_default = IniDefault::default();
    ini_default.case_sensitive = true;
    let mut profiles_ini_config = Ini::new_from_defaults(ini_default);

    // You can easily load a file to get a clone of the map:
    let profiles_ini_map = profiles_ini_config.load(&profiles_ini_path).unwrap();
    //info!("{:?}", map);
    // You can also safely not store the reference and access it later with get_map_ref() or get a clone with get_map()

    // Make two runs over profiles.ini
    // if profile has a hash, but it's not our hash then skip it (except when disabling hash feature?)

    // profile locked to an firefox instance
    let mut locked_profile_path_and_hash: HashMap<String, String> = HashMap::new();

    // 1. find all default profiles,
    //    see https://support.mozilla.org/en-US/kb/understanding-depth-profile-installation
    for (install_key, install_values) in profiles_ini_map.into_iter() {
        // e.g [Install9F3C89D8F8FDBC89]
        if !install_key.starts_with("Install") {
            continue;
        }
        // e.g "9F3C89D8F8FDBC89"
        let installation_dir_hashed = install_key.strip_prefix("Install").unwrap();
        if !install_values.contains_key("Default") {
            continue;
        }
        if !install_values.contains_key("Locked") {
            continue;
        }
        // build a map of profile to installation hash?

        /*if installation_dir_hashed != install_dir_hash {
            // skip if not a rule for this browser
            continue;
        }*/

        // can be relative or absolute
        let default_profile_path = install_values
            .get("Default")
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .to_string();

        let default_locked = install_values
            .get("Locked")
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .to_string();
        let is_profile_locked = default_locked == "1";

        if is_profile_locked {
            locked_profile_path_and_hash.insert(
                default_profile_path.clone(),
                installation_dir_hashed.to_string(),
            );
        }
    }

    let profiles_ini_map = profiles_ini_config.load(&profiles_ini_path).unwrap();
    for (_profile_key, profile_values) in profiles_ini_map.into_iter() {
        if !profile_values.contains_key("Path") {
            continue;
        }
        // can be relative or absolute
        let profile_path = profile_values
            .get("Path")
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .to_string();

        let profile_hash_maybe = locked_profile_path_and_hash.get(profile_path.as_str());
        if profile_hash_maybe.is_some() {
            let profile_hash = profile_hash_maybe.unwrap();
            // if this profile has some other hash than current firefox, then skip the profile
            if profile_hash.to_string() != install_dir_hash {
                continue;
            }
        }

        let profile_dir = if !profile_path.starts_with("/") {
            firefox_profiles_dir.join(profile_path.as_str())
        } else {
            Path::new(profile_path.as_str()).to_path_buf()
        };
        if !profile_dir.exists() {
            info!(
                "Skipping profile directory '{}', because it does not exist",
                profile_dir.display()
            );
            continue;
        }

        let name_maybe = profile_values
            .get("Name")
            .and_then(|a| a.as_ref())
            .map(|b| b.to_string());

        let profile_name = name_maybe.unwrap();

        browser_profiles.push(InstalledBrowserProfile {
            profile_cli_arg_value: profile_name.to_string(),
            profile_name: profile_name.to_string(),
            profile_icon: Some(profile_name.to_string()),
        })
    }

    return browser_profiles;
}

fn hash_firefox_install_dir(ff_binary_dir: &str) -> String {
    // ff_binary_dir has to be absolute and not contain trailing slash
    // see https://github.com/mozilla/gecko-dev/blob/d36cf98aa85f24ceefd07521b3d16b9edd2abcb7/toolkit/mozapps/update/common/commonupdatedir.cpp#L761
    // e.g "/Applications/Firefox.app/Contents/MacOS" as that is where "firefox" binary lives

    let path_as_utf16_bytes: Vec<u8> = ff_binary_dir
        .encode_utf16()
        .flat_map(|a| a.to_le_bytes())
        .collect();

    let path_as_utf16_slice = path_as_utf16_bytes.as_slice();

    let hash_u64: u64 = cityhash64(path_as_utf16_slice);
    let hash_u64_str = format!("{:X}", hash_u64);
    return hash_u64_str;
}
