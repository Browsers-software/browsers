use std::fs;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use druid::piet::TextStorage;
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

        let profile_dir = chromium_user_dir.join(profile.profile_dir_name.as_str());

        let cache_root_dir = paths::get_cache_root_dir();
        let profiles_icons_root_dir = cache_root_dir.join("icons").join("profiles");
        fs::create_dir_all(profiles_icons_root_dir.as_path()).unwrap();
        let profiles_icons_root = profiles_icons_root_dir.join(app_id);
        fs::create_dir_all(profiles_icons_root.as_path()).unwrap();

        let profile_icon_path = profile
            .avatar_file_path_relative_to_config
            .map(|image_file_path_from_config| {
                let image_file_path = chromium_user_dir.join(image_file_path_from_config);
                if image_file_path.exists() {
                    let png_file = File::open(image_file_path.as_path()).unwrap();
                    let mut png_file_reader = BufReader::new(png_file);
                    let mut buffer = Vec::new();
                    // Read file into vector
                    let result = png_file_reader.read_to_end(&mut buffer);
                    if result.is_err() {
                        return None;
                    }

                    let to_filename = profile_name.to_string() + ".png";
                    let png_file_path = profiles_icons_root.join(to_filename);
                    utils::save_as_circular(buffer, png_file_path.as_path());

                    Some(png_file_path.to_str().unwrap().to_string())
                } else {
                    None
                }
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

        let profile_avatar_file_path = self.find_avatar_file_path();

        profile_avatar_file_path.clone().map(|a| {
            let x1 = a.to_str().unwrap();
            let x = info!("{}", x1);
        });

        return ChromeProfilePreferences {
            profile_dir_name: self.profile_dir.to_string(),
            name: best_name.to_string(),
            avatar_file_path_relative_to_config: profile_avatar_file_path,
        };
    }

    // avatar file path relative to chrome config dir
    fn find_avatar_file_path(&self) -> Option<PathBuf> {
        let is_using_gaia_picture = self.is_using_gaia_picture();
        let image_file_name_maybe = if is_using_gaia_picture {
            self.get_gaia_picture_file_name()
                .map(|filename| PathBuf::from(self.profile_dir.clone()).join(filename))
        } else {
            None
        };

        return if image_file_name_maybe.is_none() {
            self.get_builtin_avatar_filename()
                .map(|filename| PathBuf::from("Avatars").join(filename.as_str()))
        } else {
            image_file_name_maybe
        };
    }

    fn is_using_gaia_picture(&self) -> bool {
        if self.use_gaia_picture() {
            return true;
        }
        // Prefer the GAIA avatar over a non-customized avatar.
        // TODO: chrome also actually checks if file really exists
        return self.is_using_default_avatar() && self.get_gaia_picture_file_name().is_some();
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

    fn is_using_default_avatar(&self) -> bool {
        self.profile["is_using_default_avatar"]
            .as_bool()
            .unwrap_or(false)
    }

    fn use_gaia_picture(&self) -> bool {
        self.profile["use_gaia_picture"].as_bool().unwrap_or(false)
    }

    fn get_last_downloaded_gaia_picture_url_with_size(&self) -> Option<String> {
        self.profile["last_downloaded_gaia_picture_url_with_size"]
            .as_str()
            .map(|a| a.to_string())
    }

    fn get_gaia_picture_file_name(&self) -> Option<String> {
        self.profile["gaia_picture_file_name"]
            .as_str()
            .map(|a| a.to_string())
    }

    fn get_builtin_avatar_filename(&self) -> Option<String> {
        let index_maybe = self.get_avatar_icon_index();
        if index_maybe.is_none() {
            return None;
        }
        let index = index_maybe.unwrap();
        return if index == 26 {
            // no image exists
            None
        } else {
            let avatar_filename = BUILTIN_AVATARS.get(index);
            let option = avatar_filename.map(|(a, b)| b.to_string());
            option
        };
    }

    fn get_avatar_icon_index(&self) -> Option<usize> {
        let prefix = "chrome://theme/IDR_PROFILE_AVATAR_";
        let prefix_len = prefix.len();

        let avatar_icon_maybe = self.get_avatar_icon();
        if avatar_icon_maybe.is_none() {
            return None;
        }
        let avatar_icon = avatar_icon_maybe.unwrap();
        let avatar_icon_str = avatar_icon.as_str();
        info!("Avatar icon: {}", avatar_icon_str);
        let index_str = &avatar_icon_str[prefix_len..];
        let index_result = index_str.parse::<usize>();
        return index_result.ok();
    }

    // "chrome://theme/IDR_PROFILE_AVATAR_26" is the default
    // "chrome://theme/IDR_PROFILE_AVATAR_34"
    fn get_avatar_icon(&self) -> Option<String> {
        self.profile["avatar_icon"].as_str().map(|a| a.to_string())
    }

    // "default_avatar_fill_color": -15189734,
    // "profile_highlight_color": -15189734,
}

// This avatar does not exist on the server, the high res copy is in the build.
const K_NO_HIGH_RES_AVATAR: &str = "NothingToDownload";

const BUILTIN_AVATARS: [(i32, &str); 56] = [
    (0, "avatar_generic.png"),
    (1, "avatar_generic_aqua.png"),
    (2, "avatar_generic_blue.png"),
    (3, "avatar_generic_green.png"),
    (4, "avatar_generic_orange.png"),
    (5, "avatar_generic_purple.png"),
    (6, "avatar_generic_red.png"),
    (7, "avatar_generic_yellow.png"),
    (8, "avatar_secret_agent.png"),
    (9, "avatar_superhero.png"),
    (10, "avatar_volley_ball.png"),
    (11, "avatar_businessman.png"),
    (12, "avatar_ninja.png"),
    (13, "avatar_alien.png"),
    (14, "avatar_smiley.png"),
    (15, "avatar_flower.png"),
    (16, "avatar_pizza.png"),
    (17, "avatar_soccer.png"),
    (18, "avatar_burger.png"),
    (19, "avatar_cat.png"),
    (20, "avatar_cupcake.png"),
    (21, "avatar_dog.png"),
    (22, "avatar_horse.png"),
    (23, "avatar_margarita.png"),
    (24, "avatar_note.png"),
    (25, "avatar_sun_cloud.png"),
    (26, K_NO_HIGH_RES_AVATAR),
    // Modern avatar icons:
    (27, "avatar_origami_cat.png"),
    (28, "avatar_origami_corgi.png"),
    (29, "avatar_origami_dragon.png"),
    (30, "avatar_origami_elephant.png"),
    (31, "avatar_origami_fox.png"),
    (32, "avatar_origami_monkey.png"),
    (33, "avatar_origami_panda.png"),
    (34, "avatar_origami_penguin.png"),
    (35, "avatar_origami_pinkbutterfly.png"),
    (36, "avatar_origami_rabbit.png"),
    (37, "avatar_origami_unicorn.png"),
    (38, "avatar_illustration_basketball.png"),
    (39, "avatar_illustration_bike.png"),
    (40, "avatar_illustration_bird.png"),
    (41, "avatar_illustration_cheese.png"),
    (42, "avatar_illustration_football.png"),
    (43, "avatar_illustration_ramen.png"),
    (44, "avatar_illustration_sunglasses.png"),
    (45, "avatar_illustration_sushi.png"),
    (46, "avatar_illustration_tamagotchi.png"),
    (47, "avatar_illustration_vinyl.png"),
    (48, "avatar_abstract_avocado.png"),
    (49, "avatar_abstract_cappuccino.png"),
    (50, "avatar_abstract_icecream.png"),
    (51, "avatar_abstract_icewater.png"),
    (52, "avatar_abstract_melon.png"),
    (53, "avatar_abstract_onigiri.png"),
    (54, "avatar_abstract_pizza.png"),
    (55, "avatar_abstract_sandwich.png"),
];

enum ChromeNameForm {
    GaiaName,
    LocalName,
    GaiaAndLocalName,
}

pub struct ChromeProfilePreferences {
    pub profile_dir_name: String,
    pub name: String,
    pub avatar_file_path_relative_to_config: Option<PathBuf>,
}
