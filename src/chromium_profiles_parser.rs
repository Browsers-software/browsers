use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{Map, Value};
use tracing::{debug, info};
use url::Url;

use crate::{paths, utils, InstalledBrowserProfile};

pub fn find_chromium_profiles(
    chromium_user_dir: &Path,
    _binary_path: &Path,
    app_id: &str,
) -> Vec<InstalledBrowserProfile> {
    let mut browser_profiles: Vec<InstalledBrowserProfile> = Vec::new();

    let local_state_file = chromium_user_dir.join("Local State");
    debug!("Chrome Local State Path: {:?}", local_state_file);

    if !local_state_file.exists() {
        info!("Could not find {}", local_state_file.display());
        return browser_profiles;
    }

    let info_cache_map = ChromeInfoCacheMap::new_from_local_state(local_state_file.as_path());
    let profiles = info_cache_map.parse_chrome_local_state_profiles();
    for profile in profiles {
        let profile_name = profile.name;

        let profile_icon_path = profile
            .image_url
            .map(|url| {
                let cache_root_dir = paths::get_cache_root_dir();
                let profiles_icons_root_dir = cache_root_dir.join("icons").join("profiles");
                fs::create_dir_all(profiles_icons_root_dir.as_path()).unwrap();
                let profiles_icons_root = profiles_icons_root_dir.join(app_id);
                fs::create_dir_all(profiles_icons_root.as_path()).unwrap();
                let profile_icon_path_without_extension =
                    profiles_icons_root.join(profile_name.to_string());

                return Url::parse(url.as_str())
                    .ok()
                    .and_then(|remote_url| {
                        utils::download_profile_images(
                            &remote_url,
                            profile_icon_path_without_extension.as_path(),
                        )
                        .ok()
                    })
                    .map(|path| path.to_str().unwrap().to_string());
            })
            .flatten();

        let profile_dir_name = profile.profile_dir_name;
        browser_profiles.push(InstalledBrowserProfile {
            profile_cli_arg_value: profile_dir_name.to_string(),
            profile_cli_container_name: None,
            profile_name: profile_name,
            profile_icon: profile_icon_path,
        })
    }

    return browser_profiles;
}

pub struct ChromeInfoCacheMap {
    info_cache_map: Map<String, Value>,
}

impl ChromeInfoCacheMap {
    pub fn new_from_local_state(local_state_file_path: &Path) -> ChromeInfoCacheMap {
        Self {
            info_cache_map: Self::profiles_info_cache_map(local_state_file_path),
        }
    }

    fn profiles_info_cache_map(local_state_file_path: &Path) -> Map<String, Value> {
        // Open the file in read-only mode with buffer.
        let file = File::open(local_state_file_path).unwrap();
        let reader = BufReader::new(file);
        let v: Value = serde_json::from_reader(reader).unwrap();
        let profiles = &v["profile"];
        let info_cache = &profiles["info_cache"];
        let info_cache_map = info_cache.as_object().unwrap();
        return info_cache_map.to_owned();
    }

    pub fn parse_chrome_local_state_profiles(self) -> Vec<ChromeProfilePreferences> {
        let info_cache_map = self.info_cache_map;
        let profiles_count = info_cache_map.len();

        let mut entries: Vec<ChromeProfileAttributesEntry> = Vec::with_capacity(profiles_count);
        for (dir_name, profile) in info_cache_map {
            let entry = ChromeProfileAttributesEntry::new(dir_name.as_str(), &profile);
            entries.push(entry);
        }

        let mut profiles_vec: Vec<ChromeProfilePreferences> = Vec::with_capacity(profiles_count);
        for entry in &entries {
            profiles_vec.push(entry.get_profile_info(&entries));
        }
        // constant ordering (well based on name)
        profiles_vec.sort_by(|p1, p2| p1.name.cmp(&p2.name));
        return profiles_vec;
    }
}

pub struct ChromeProfileAttributesEntry {
    profile_dir: String,
    profile: Value,
}

// GAIA - Google Accounts and ID Administration
impl ChromeProfileAttributesEntry {
    fn new(profile_dir: &str, profile_map: &Value) -> ChromeProfileAttributesEntry {
        Self {
            profile_dir: profile_dir.to_string(),
            profile: profile_map.clone(),
        }
    }

    // https://chromium.googlesource.com/chromium/src/+/lkgr/chrome/browser/profiles/profile_attributes_entry.cc#208
    fn get_profile_info(
        &self,
        all_entries: &Vec<ChromeProfileAttributesEntry>,
    ) -> ChromeProfilePreferences {
        let best_name = self.get_name(all_entries);
        let image_url_maybe = self.get_last_downloaded_gaia_picture_url_with_size();

        return ChromeProfilePreferences {
            profile_dir_name: self.profile_dir.to_string(),
            name: best_name.to_string(),
            image_url: image_url_maybe,
        };
    }

    // chrome/browser/profiles/profile_attributes_entry.cc#ProfileAttributesEntry::GetName()
    fn get_name(&self, all_entries: &Vec<ChromeProfileAttributesEntry>) -> String {
        let chrome_name_form = self.get_name_form(all_entries);

        return match chrome_name_form {
            ChromeNameForm::GaiaName => self.get_gaia_name_to_display(),
            ChromeNameForm::LocalName => self.get_local_profile_name(),
            ChromeNameForm::GaiaAndLocalName => {
                format!(
                    "{} ({})",
                    self.get_gaia_name_to_display(),
                    self.get_local_profile_name()
                )
            }
        };
    }

    fn get_name_form(&self, all_entries: &Vec<ChromeProfileAttributesEntry>) -> ChromeNameForm {
        let name_to_display = self.get_gaia_name_to_display();

        if name_to_display.is_empty() {
            return ChromeNameForm::LocalName;
        }

        if !self.should_show_profile_local_name(name_to_display.as_str(), all_entries) {
            return ChromeNameForm::GaiaName;
        }

        return ChromeNameForm::GaiaAndLocalName;
    }

    // Replicates ProfileAttributesEntry::ShouldShowProfileLocalName behaviour
    fn should_show_profile_local_name(
        &self,
        gaia_name_to_display: &str,
        all_entries: &Vec<ChromeProfileAttributesEntry>,
    ) -> bool {
        // Never show the profile name if it is equal to GAIA given name,
        // e.g. Matt (Matt), in that case we should only show the GAIA name.
        if gaia_name_to_display == self.get_local_profile_name() {
            return false;
        }

        // Customized profile name that is not equal to Gaia name, e.g. Matt (Work).
        if !self.is_using_default_name() {
            return true;
        }
        // The profile local name is a default profile name : Person n.
        // check other profile names...
        for other in all_entries {
            // TODO: this check maybe doesn't work
            if other.profile_dir == self.profile_dir {
                continue;
            }

            let other_gaia_name_to_display = other.get_gaia_name_to_display();
            if other_gaia_name_to_display.is_empty()
                || other_gaia_name_to_display != gaia_name_to_display
            {
                continue;
            }

            // Another profile with the same GAIA name.
            let other_profile_name_equal_gaia_name = other_gaia_name_to_display
                .eq_ignore_ascii_case(other.get_local_profile_name().as_str());

            // If for the other profile, the profile name is equal to GAIA name then it
            // will not be shown. For disambiguation, show for the current profile the
            // profile name even if it is Person n.
            if other_profile_name_equal_gaia_name {
                return true;
            }
            // Both profiles have a default profile name,
            // e.g. Matt (Person 1), Matt (Person 2).
            if other.is_using_default_name() {
                return true;
            }
        }
        return false;
    }

    fn get_gaia_name_to_display(&self) -> String {
        let gaia_given_name = self.get_gaia_given_name();

        if gaia_given_name.is_empty() {
            self.get_gaia_name()
        } else {
            gaia_given_name
        }
    }

    fn get_gaia_given_name(&self) -> String {
        self.profile["gaia_given_name"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    fn get_gaia_name(&self) -> String {
        self.profile["gaia_name"].as_str().unwrap_or("").to_string()
    }

    fn get_local_profile_name(&self) -> String {
        self.profile["name"].as_str().unwrap_or("").to_string()
    }

    fn is_using_default_name(&self) -> bool {
        self.profile["is_using_default_name"]
            .as_bool()
            .unwrap_or(false)
    }

    fn get_last_downloaded_gaia_picture_url_with_size(&self) -> Option<String> {
        self.profile["last_downloaded_gaia_picture_url_with_size"]
            .as_str()
            .map(|a| a.to_string())
    }
}

enum ChromeNameForm {
    GaiaName,
    LocalName,
    GaiaAndLocalName,
}

pub struct ChromeProfilePreferences {
    pub profile_dir_name: String,
    pub name: String,
    pub image_url: Option<String>,
}
