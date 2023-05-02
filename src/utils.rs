use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, io, u32};

use druid::image;
use druid::image::imageops::FilterType;
use druid::image::{ImageFormat, Rgba};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use url::Url;

#[cfg(target_os = "linux")]
use crate::linux_utils;
#[cfg(target_os = "macos")]
use crate::macos_utils;
use crate::{paths, InstalledBrowser, SupportedAppRepository};

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
        if let Some(hidden_profile_index) = hidden_profile_index_maybe {
            self.hidden_profiles.remove(hidden_profile_index);
        }
    }

    pub fn hide_profile(&mut self, profile_id: &str) {
        let hidden_profile_index_maybe = self
            .hidden_profiles
            .iter()
            .position(|unique_id| unique_id == profile_id);
        if let Some(_) = hidden_profile_index_maybe {
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
        let schemes = vec![
            ("figma", vec!["figma.com", "www.figma.com"]),
            ("linear", vec!["linear.app"]),
            ("notion", vec!["notion.so", "www.notion.so"]),
            ("spotify", vec!["open.spotify.com"]),
            ("tg", vec!["t.me"]), // telegram
            (
                "zoommtg",
                vec![
                    "zoom.us",
                    "eu01web.zoom.us",
                    "us02web.zoom.us",
                    "us03web.zoom.us",
                    "us04web.zoom.us",
                    "us05web.zoom.us",
                    "us06web.zoom.us",
                    "us07web.zoom.us",
                ],
            ),
            ("https", vec![]),
        ];
        let schemes_vec: Vec<(String, Vec<String>)> = schemes
            .iter()
            .map(|(scheme, domains)| {
                (
                    scheme.to_string(),
                    domains.iter().map(|d| d.to_string()).collect(),
                )
            })
            .collect();

        return self.inner.get_installed_browsers(schemes_vec);
    }

    pub(crate) fn get_app_repository(&self) -> &SupportedAppRepository {
        return self.inner.get_app_repository();
    }

    pub(crate) fn save_installed_browsers_config(&self, config: &Config) {
        let config_root_dir = paths::get_config_root_dir();
        fs::create_dir_all(config_root_dir.as_path()).unwrap();
        let config_json_path = paths::get_config_json_path();
        let buffer = File::create(config_json_path).unwrap();
        serde_json::to_writer_pretty(buffer, config).unwrap();
    }

    pub(crate) fn get_installed_browsers_config(&self) -> Config {
        let config_root_dir = paths::get_config_root_dir();
        fs::create_dir_all(config_root_dir.as_path()).unwrap();
        let config_json_path = paths::get_config_json_path();
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

/*const fn create_circular_mask_radius<const N: usize>() -> [[bool; N]; N] {
    let mut mask = [[true; N]; N];

    let mut x: usize = 0;
    while x < N as usize {
        let mut y: usize = 0;
        while y < N as usize {
            let w = x.abs_diff(N / 2);
            let h = y.abs_diff(N / 2);
            let a = w.pow(2) + h.pow(2);

            let sq = ct_sqrt(a as u32, 1, a as u32);
            let distance = sq + 1;

            // if distance to center is > 16, then put transparent pixel
            let is_visible = distance <= N as u32 / 2;
            mask[x][y] = is_visible;

            y += 1;
        }
        x += 1;
    }

    return mask;
}*/

// https://baptiste-wicht.com/posts/2014/07/compile-integer-square-roots-at-compile-time-in-cpp.html
/*const fn ct_sqrt(res: u32, l: u32, r: u32) -> u32 {
    return if l == r {
        r
    } else {
        let mid = (r + l) / 2;

        if mid * mid >= res {
            0
            // too high recursion, but don't need this branch, so just returning 0
            //return ct_sqrt(res, l, mid);
        } else {
            ct_sqrt(res, mid + 1, r)
        }
    };
}*/

//const CIRCULAR_MASK_32: [[bool; 64]; 64] = create_circular_mask_radius();
const CIRCULAR_RADIUS: usize = 64;

lazy_static! {
    static ref CIRCULAR_MASK_32_LAZY: [[bool; CIRCULAR_RADIUS]; CIRCULAR_RADIUS] = {
        const N: usize = CIRCULAR_RADIUS;

        let mut mask = [[true; N]; N];

        let mut x: usize = 0;
        while x < N as usize {
            let mut y: usize = 0;
            while y < N as usize {
                let w = x.abs_diff(N / 2);
                let h = y.abs_diff(N / 2);
                let a = w.pow(2) + h.pow(2);

                let sq = (a as f64).sqrt() as i64;
                let distance = sq + 1;

                // if distance to center is > 16, then put transparent pixel
                let is_visible = distance <= N as i64 / 2;
                mask[x][y] = is_visible;

                y += 1;
            }
            x += 1;
        }

        return mask;
    };
}

pub fn save_as_circular(image_bytes: Vec<u8>, to_image_path: &Path) {
    let vec = image_bytes;
    let result1 = image::load_from_memory(vec.as_slice());
    let image1 = result1.unwrap();
    let image1 = image1.resize_exact(
        CIRCULAR_RADIUS as u32,
        CIRCULAR_RADIUS as u32,
        FilterType::Nearest,
    );
    let mut image_with_alpha = image1.to_rgba16();

    //for (x, row) in CIRCULAR_MASK_32.iter().enumerate() {
    for (x, row) in CIRCULAR_MASK_32_LAZY.iter().enumerate() {
        for (y, mask) in row.iter().enumerate() {
            if !mask {
                image_with_alpha.put_pixel(x as u32, y as u32, Rgba([122, 0, 0, 122]));
            }
        }
    }

    let png_file_path = to_image_path.to_path_buf();

    image_with_alpha
        .save_with_format(png_file_path.as_path(), ImageFormat::Png)
        .unwrap();

    debug!("WROTE TO : {:?}", png_file_path.as_path());
}
