use std::any::Any;
use std::env::temp_dir;
use std::fmt::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::{fs, io, u32};

use druid::image;
use druid::image::imageops::FilterType;
use druid::image::{GenericImage, ImageFormat, Rgb, Rgba};
use druid::widget::Image;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_subscriber::fmt::MakeWriter;
use url::Url;

#[cfg(target_os = "linux")]
use crate::linux_utils;
#[cfg(target_os = "macos")]
use crate::macos_utils;
use crate::macos_utils::get_this_app_cache_root_dir;
use crate::{
    paths, InstalledBrowser, InstalledBrowserProfile, ProfileIcon, SupportedAppRepository,
};

#[cfg(target_os = "macos")]
pub fn set_as_default_web_browser() -> bool {
    return macos_utils::set_default_web_browser();
}

#[cfg(not(target_os = "macos"))]
pub fn set_as_default_web_browser() -> bool {
    return true;
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Config {
    hidden_apps: Vec<String>,
    hidden_profiles: Vec<String>,
    profile_order: Vec<String>,
    rules: Vec<ConfigRule>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigRule {
    pub source_app: Option<String>,
    pub url_pattern: Option<String>,
    pub profile: String,
}

impl Config {
    pub fn get_hidden_apps(&self) -> &Vec<String> {
        &self.hidden_apps
    }

    pub fn get_hidden_profiles(&self) -> &Vec<String> {
        &self.hidden_profiles
    }

    pub fn restore_profile(&mut self, profile_id: &str) {
        let hidden_profile_index_maybe = self
            .hidden_profiles
            .iter()
            .position(|unique_id| unique_id == profile_id);
        if hidden_profile_index_maybe.is_some() {
            let hidden_profile_index = hidden_profile_index_maybe.unwrap();
            self.hidden_profiles.remove(hidden_profile_index);
        }
    }

    pub fn hide_profile(&mut self, profile_id: &str) {
        let hidden_profile_index_maybe = self
            .hidden_profiles
            .iter()
            .position(|unique_id| unique_id == profile_id);
        if hidden_profile_index_maybe.is_some() {
            // already exists, do nothing
            //let hidden_profile_index = hidden_profile_index_maybe.unwrap();
            //self.hidden_profiles.remove(hidden_profile_index);
            return;
        }

        self.hidden_profiles.push(profile_id.to_string());
    }

    pub fn hide_all_profiles(&mut self, profile_ids: &Vec<String>) {
        for profile_id in profile_ids {
            self.hide_profile(profile_id);
        }
    }

    pub fn get_profile_order(&self) -> &Vec<String> {
        &self.profile_order
    }

    pub fn set_profile_order(&mut self, profile_order: &Vec<String>) {
        self.profile_order = profile_order.clone();
    }

    pub fn get_rules(&self) -> &Vec<ConfigRule> {
        return &self.rules;
    }
}

pub struct OSAppFinder {
    #[cfg(target_os = "linux")]
    inner: linux_utils::OsHelper,

    #[cfg(target_os = "macos")]
    inner: macos_utils::OsHelper,
}

impl OSAppFinder {
    #[cfg(target_os = "linux")]
    pub fn new() -> Self {
        Self {
            inner: linux_utils::OsHelper::new(),
        }
    }

    #[cfg(target_os = "macos")]
    pub fn new() -> Self {
        Self {
            inner: macos_utils::OsHelper::new(),
        }
    }

    pub fn get_installed_browsers(&self) -> Vec<InstalledBrowser> {
        return self.inner.get_installed_browsers();
    }

    pub(crate) fn get_app_repository(&self) -> &SupportedAppRepository {
        return self.inner.get_app_repository();
    }

    pub(crate) fn save_installed_browsers_config(&self, config: &Config) {
        let config_root_dir = paths::get_config_root_dir();
        fs::create_dir_all(config_root_dir.as_path()).unwrap();
        let config_json_path = self.get_config_json_path(config_root_dir.as_path());
        let buffer = File::create(config_json_path).unwrap();
        serde_json::to_writer_pretty(buffer, config).unwrap();
    }

    pub(crate) fn get_installed_browsers_config(&self) -> Config {
        let config_root_dir = paths::get_config_root_dir();
        fs::create_dir_all(config_root_dir.as_path()).unwrap();
        let config_json_path = self.get_config_json_path(config_root_dir.as_path());
        info!("Config: {}", config_json_path.display());

        if config_json_path.exists() {
            // Open the file in read-only mode with buffer.
            let file = File::open(config_json_path.as_path()).unwrap();
            let reader = BufReader::new(file);
            let result: Result<Config, _> = serde_json::from_reader(reader);

            if result.is_err() {
                // we can't read in config as valid config,
                // just in case copy the config file for debugging

                let corrupted_config_json_path = config_root_dir.join("config.corrupted.json");
                fs::copy(config_json_path.as_path(), corrupted_config_json_path).ok();

                // just use empty config, but don't write it yet, it will be overwritten on first
                // change in config
                return Config::default();
            }
            let config = result.unwrap();
            return config;
        } else {
            let config = Config::default();
            let buffer = File::create(config_json_path.as_path()).unwrap();
            serde_json::to_writer_pretty(buffer, &config).unwrap();
            return config;
        }
    }

    // create_dirs: creates directories if missing
    fn get_config_json_path(&self, config_root_dir: &Path) -> PathBuf {
        return config_root_dir.join("config.json");
    }

    pub(crate) fn get_installed_browsers_cached(
        &self,
        force_reload: bool,
    ) -> Vec<InstalledBrowser> {
        let cache_root_dir = paths::get_cache_root_dir();
        fs::create_dir_all(cache_root_dir.as_path()).unwrap();

        let installed_browsers_json_path = cache_root_dir.join("installed_browsers.json");

        if !force_reload && installed_browsers_json_path.exists() {
            // Open the file in read-only mode with buffer.
            let file = File::open(installed_browsers_json_path).unwrap();
            let reader = BufReader::new(file);

            let a: Result<Vec<InstalledBrowser>, _> = serde_json::from_reader(reader);
            let installed_browsers_cached = a.unwrap_or_default();
            return installed_browsers_cached;
        } else {
            let installed_browsers = self.get_installed_browsers();

            let buffer = File::create(installed_browsers_json_path).unwrap();
            serde_json::to_writer_pretty(buffer, &installed_browsers).unwrap();
            return installed_browsers;
        }
    }
}

pub fn download_profile_images(
    remote_url: &Url,
    local_icon_path_without_extension: &Path,
) -> Result<PathBuf, io::Error> {
    let response = attohttpc::get(remote_url).send().unwrap();
    if response.is_success() {
        let vec = response.bytes().unwrap();
        let result1 = image::load_from_memory(vec.as_slice());
        let image1 = result1.unwrap();
        let image1 = image1.resize_exact(32, 32, FilterType::Nearest);
        let mut image_with_alpha = image1.to_rgba16();

        for x in 0u32..32 {
            for y in 0u32..32 {
                let w = x.abs_diff(16);
                let h = y.abs_diff(16);
                let a = (w.pow(2) + h.pow(2));
                let distance = f64::sqrt(a as f64) + 1.0;
                // if distance to center is > 16, then put transparent pixel
                if distance > 16.0 {
                    image_with_alpha.put_pixel(x, y, Rgba([122, 0, 0, 122]));
                }
            }
        }

        let png_file_path = local_icon_path_without_extension
            .to_path_buf()
            .with_extension("png");

        image_with_alpha
            .save_with_format(png_file_path.as_path(), ImageFormat::Png)
            .unwrap();

        info!("WROTE TO : {:?}", png_file_path.as_path());

        return Ok(png_file_path);
    }
    info!("PROFILE ICON: {}", remote_url);

    return Err(io::Error::new(
        ErrorKind::Other,
        "could not save profile image",
    ));
}
