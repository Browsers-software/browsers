use std::collections::HashMap;
use std::path::{Path, PathBuf};

use url::Url;

use crate::ProfileIcon::NoIcon;
use crate::{chromium_profiles_parser, firefox_profiles_parser, paths, InstalledBrowserProfile};

// Holds list of custom SupportedApp configurations
// All other apps will be the "default" supported app implementation
pub struct SupportedAppRepository {
    snap_base: PathBuf,
    chromium_user_dir_base: PathBuf,
    firefox_user_dir_base: PathBuf,
    supported_apps: HashMap<String, SupportedApp>,
}

impl SupportedAppRepository {
    pub fn new() -> Self {
        let mut repository = Self {
            snap_base: paths::get_snap_root(),
            chromium_user_dir_base: paths::get_chrome_user_dir_root(),
            firefox_user_dir_base: paths::get_firefox_user_dir_root(),
            supported_apps: HashMap::new(),
        };
        repository.generate_app_id_to_supported_app();
        return repository;
    }

    pub fn get_or_generate(&self, app_id_str: &str) -> SupportedApp {
        return self
            .supported_apps
            .get(app_id_str)
            .map(|app| app.to_owned())
            .unwrap_or_else(|| {
                let app_id = AppIdentifier::new_for_os(app_id_str.to_string());
                SupportedAppRepository::generic_app(app_id)
            });
    }

    fn add(&mut self, supported_app: SupportedApp) -> &mut SupportedAppRepository {
        self.supported_apps
            .insert(supported_app.get_app_id().to_string(), supported_app);
        return self;
    }

    fn generate_app_id_to_supported_app(&mut self) {
        self.start()
            .add_chromium_based_app(
                "company.thebrowser.Browser",
                "TODOTODOTODO",
                "Arc/User Data",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.google.Chrome",
                "google-chrome.desktop",
                "Google/Chrome",
                "google-chrome",
            )
            .add_chromium_based_app(
                "com.google.Chrome.beta",
                "google-chrome-beta.desktop",
                "Google/Chrome Beta",
                "google-chrome-beta",
            )
            .add_chromium_based_app(
                "com.google.Chrome.dev",
                "google-chrome-dev.desktop",
                "Google/Chrome Dev",
                "google-chrome-dev",
            )
            .add_chromium_based_app(
                "com.google.Chrome.canary",
                "google-chrome-canary.desktop",
                "Google/Chrome Canary",
                "google-chrome-canary",
            )
            .add_chromium_based_app_with_snap(
                "org.chromium.Chromium",
                "chromium_chromium.desktop",
                "chromium",
                "Chromium",
                "chromium",
            )
            .add_chromium_based_app(
                "com.avast.browser",
                "TODOTODOTODO",
                "AVAST Software/Browser",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.whisttechnologies.whist",
                "TODOTODOTODO",
                "Whist/Whist-Browser",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.bookry.wavebox",
                "TODOTODOTODO",
                "WaveboxApp",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.coccoc.Coccoc",
                "TODOTODOTODO",
                "Coccoc",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "net.qihoo.360browser",
                "TODOTODOTODO",
                "360Chrome",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "ru.yandex.desktop.yandex-browser",
                "TODOTODOTODO",
                "Yandex/YandexBrowser",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.microsoft.edgemac",
                "TODOTODOTODO",
                "Microsoft Edge",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.microsoft.edgemac.Beta",
                "TODOTODOTODO",
                "Microsoft Edge Beta",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.microsoft.edgemac.Dev",
                "microsoft-edge-dev.desktop",
                "Microsoft Edge Dev",
                "microsoft-edge-dev",
            )
            .add_chromium_based_app(
                "com.microsoft.edgemac.Canary",
                "TODOTODOTODO",
                "Microsoft Edge Canary",
                "TODOTODOTODO",
            )
            .add_chromium_based_app_with_snap(
                "com.brave.Browser",
                "brave-browser.desktop",
                "brave",
                "BraveSoftware/Brave-Browser",
                "BraveSoftware/Brave-Browser",
            )
            .add_chromium_based_app(
                "com.brave.Browser.beta",
                "TODOTODOTODO",
                "BraveSoftware/Brave-Browser-Beta",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.brave.Browser.nightly",
                "TODOTODOTODO",
                "BraveSoftware/Brave-Browser-Nightly",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.pushplaylabs.sidekick",
                "TODOTODOTODO",
                "Sidekick",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.vivaldi.Vivaldi",
                "vivaldi-stable.desktop",
                "Vivaldi",
                "vivaldi",
            )
            .add_chromium_based_app(
                "com.vivaldi.Vivaldi.snapshot",
                "TODOTODOTODO",
                "Vivaldi Snapshot",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "com.naver.Whale",
                "TODOTODOTODO",
                "Naver/Whale",
                "TODOTODOTODO",
            )
            .add_chromium_based_app(
                "de.iridiumbrowser",
                "TODOTODOTODO",
                "Iridium",
                "TODOTODOTODO",
            )
            .add_firefox_based_app_with_snap(
                "org.mozilla.firefox",
                "firefox_firefox.desktop",
                "firefox",
                "Firefox",
                ".mozilla/firefox",
            )
            .add_firefox_based_app(
                "org.mozilla.firefoxdeveloperedition",
                "TODOTODOTODO",
                "Firefox",
                ".mozilla/firefox",
            )
            .add_firefox_based_app(
                "org.mozilla.nightly",
                "TODOTODOTODO",
                "Firefox",
                ".mozilla/firefox",
            )
            .add_firefox_based_app("org.mozilla.floorp", "TODOTODOTODO", "Floorp", "")
            .add_firefox_based_app(
                "org.torproject.torbrowser",
                "TODOTODOTODO",
                "TorBrowser-Data/Browser",
                "TODOTODOTODO",
            )
            .add_firefox_based_app(
                "org.mozilla.librewolf",
                "TODOTODOTODO",
                "LibreWolf",
                "TODOTODOTODO",
            )
            .add_firefox_based_app(
                "net.waterfox.waterfox",
                "TODOTODOTODO",
                "Waterfox",
                "TODOTODOTODO",
            )
            .add(Self::notion_app())
            .add(Self::spotify_app())
            .add(Self::zoom_app());
    }

    fn add_firefox_based_app(
        &mut self,
        mac_bundle_id: &str,
        linux_desktop_id: &str,
        mac_config_dir_relative: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        return self.add_firefox_based_app_with_snap(
            mac_bundle_id,
            linux_desktop_id,
            "",
            mac_config_dir_relative,
            linux_config_dir_relative,
        );
    }

    fn add_firefox_based_app_with_snap(
        &mut self,
        mac_bundle_id: &str,
        linux_desktop_id: &str,
        linux_snap_id: &str,
        mac_config_dir_relative: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_id = AppIdentifier {
            mac_bundle_id: mac_bundle_id.to_string(),
            linux_desktop_id: linux_desktop_id.to_string(),
        };
        let app_config_dir = AppConfigDir {
            root_path: self.firefox_user_dir_base.clone(),
            mac_config_dir_relative: PathBuf::from(mac_config_dir_relative),
            linux_config_dir_relative: PathBuf::from(linux_config_dir_relative),
        };

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        let app = Self::firefox_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            snap_app_config_dir_absolute,
        );
        return self.add(app);
    }

    fn start(&mut self) -> &mut SupportedAppRepository {
        return self;
    }

    fn add_chromium_based_app(
        &mut self,
        mac_bundle_id: &str,
        linux_desktop_id: &str,
        mac_config_dir_relative: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        return self.add_chromium_based_app_with_snap(
            mac_bundle_id,
            linux_desktop_id,
            "",
            mac_config_dir_relative,
            linux_config_dir_relative,
        );
    }

    fn add_chromium_based_app_with_snap(
        &mut self,
        mac_bundle_id: &str,
        linux_desktop_id: &str,
        linux_snap_id: &str,
        mac_config_dir_relative: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_id = AppIdentifier {
            mac_bundle_id: mac_bundle_id.to_string(),
            linux_desktop_id: linux_desktop_id.to_string(),
        };
        let app_config_dir = AppConfigDir {
            root_path: self.chromium_user_dir_base.clone(),
            mac_config_dir_relative: PathBuf::from(mac_config_dir_relative),
            linux_config_dir_relative: PathBuf::from(linux_config_dir_relative),
        };

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        let app = Self::chromium_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            snap_app_config_dir_absolute,
        );
        return self.add(app);
    }

    fn snap_config_dir_absolute_path(
        &self,
        snap_name: &str,
        linux_config_dir_relative: &str,
    ) -> PathBuf {
        let snap_root_path = self.snap_base.clone();
        let snap_linux_config_dir_relative_path = PathBuf::from(snap_name)
            .join("common")
            .join(linux_config_dir_relative);
        let config_dir_absolute = snap_root_path.join(snap_linux_config_dir_relative_path);
        return config_dir_absolute;
    }

    fn chromium_based_app(
        app_id: AppIdentifier,
        app_config_dir_absolute: PathBuf,
        snap_app_config_dir_absolute: PathBuf,
    ) -> SupportedApp {
        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: app_config_dir_absolute,
            snap_app_config_dir_absolute: snap_app_config_dir_absolute,
            find_profiles_fn: Some(chromium_profiles_parser::find_chromium_profiles),
            restricted_domains: vec![],
            profile_args_fn: |profile_cli_arg_value| {
                vec![format!("--profile-directory={}", profile_cli_arg_value)]
            },
            incognito_args: vec!["-incognito".to_string()],
            url_transform_fn: |profile_cli_container_name, url| url.to_string(),
            url_as_first_arg: true,
        }
    }

    fn firefox_based_app(
        app_id: AppIdentifier,
        app_config_dir_absolute: PathBuf,
        snap_app_config_dir_absolute: PathBuf,
    ) -> SupportedApp {
        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: app_config_dir_absolute,
            snap_app_config_dir_absolute: snap_app_config_dir_absolute,
            find_profiles_fn: Some(firefox_profiles_parser::find_firefox_profiles),
            restricted_domains: vec![],
            profile_args_fn: |profile_cli_arg_value| {
                vec!["-P".to_string(), profile_cli_arg_value.to_string()]
            },
            incognito_args: vec!["-private".to_string()],
            url_transform_fn: |container_name_maybe, url| {
                return if container_name_maybe.is_some() {
                    let container_name = container_name_maybe.unwrap();
                    let fake_url = "ext+container:name=".to_string() + container_name;
                    let full_url = fake_url + "&url=" + url.clone();
                    full_url.to_string()
                } else {
                    url.to_string()
                };
            },
            url_as_first_arg: true,
        }
    }

    pub fn generic_app(app_id: AppIdentifier) -> SupportedApp {
        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: PathBuf::new(),
            snap_app_config_dir_absolute: PathBuf::new(),
            find_profiles_fn: None,
            restricted_domains: vec![],
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: |profile_cli_container_name, url| url.to_string(),
            url_as_first_arg: false,
        }
    }

    fn notion_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "notion.id".to_string(),
            linux_desktop_id: "NOLINUXAPPEXISTS.desktop".to_string(),
        };

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: PathBuf::new(),
            snap_app_config_dir_absolute: PathBuf::new(),
            find_profiles_fn: None,
            restricted_domains: vec!["https://notion.so".to_string(), "https://www.notion.so".to_string()],
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: |profile_cli_container_name, url| url.to_string(),
            url_as_first_arg: false,
        }
    }

    fn spotify_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "com.spotify.client".to_string(),
            linux_desktop_id: "spotify_spotify.desktop".to_string(),
        };

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: PathBuf::new(),
            snap_app_config_dir_absolute: PathBuf::new(),
            find_profiles_fn: None,
            restricted_domains: vec!["https://open.spotify.com".to_string()],
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: convert_spotify_uri,
            url_as_first_arg: true,
        }
    }

    fn zoom_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "us.zoom.xos".to_string(),
            linux_desktop_id: "Zoom.desktop".to_string(),
        };

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: PathBuf::new(),
            snap_app_config_dir_absolute: PathBuf::new(),
            find_profiles_fn: None,
            restricted_domains: vec!["https://zoom.us".to_string()],
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: |profile_cli_container_name, url| url.to_string(),
            url_as_first_arg: true,
        }
    }
}

#[derive(Clone)]
pub struct SupportedApp {
    app_id: AppIdentifier,
    app_config_dir_absolute: PathBuf,
    snap_app_config_dir_absolute: PathBuf,
    restricted_domains: Vec<String>,
    find_profiles_fn: Option<
        fn(
            app_config_dir_absolute: &Path,
            binary_path: &Path,
            app_id: &str,
        ) -> Vec<InstalledBrowserProfile>,
    >,
    profile_args_fn: fn(profile_cli_arg_value: &str) -> Vec<String>,
    incognito_args: Vec<String>,
    url_transform_fn: fn(profile_cli_container_name: Option<&String>, url: &str) -> String,
    url_as_first_arg: bool,
}

impl SupportedApp {
    pub fn get_app_id(&self) -> &str {
        return self.app_id.app_id();
    }

    pub fn get_app_config_dir_abs(&self, is_snap: bool) -> &Path {
        return if is_snap {
            &self.snap_app_config_dir_absolute.as_path()
        } else {
            &self.app_config_dir_absolute.as_path()
        };
    }

    pub fn get_app_config_dir_absolute(&self, is_snap: bool) -> &str {
        return self.get_app_config_dir_abs(is_snap).to_str().unwrap();
    }

    pub fn get_restricted_domains(&self) -> &Vec<String> {
        return &self.restricted_domains;
    }

    pub fn find_profiles(&self, binary_path: &Path, is_snap: bool) -> Vec<InstalledBrowserProfile> {
        return if self.find_profiles_fn.is_some() {
            let app_config_dir_abs = self.get_app_config_dir_abs(is_snap);
            (self.find_profiles_fn.unwrap())(app_config_dir_abs, binary_path, self.get_app_id())
        } else {
            Self::find_placeholder_profiles()
        };
    }

    fn find_placeholder_profiles() -> Vec<InstalledBrowserProfile> {
        let mut browser_profiles: Vec<InstalledBrowserProfile> = Vec::new();

        browser_profiles.push(InstalledBrowserProfile {
            profile_cli_arg_value: "".to_string(),
            profile_cli_container_name: None,
            profile_name: "".to_string(),
            profile_icon: None,
        });

        return browser_profiles;
    }

    pub fn supports_profiles(&self) -> bool {
        return self.find_profiles_fn.is_some();
    }

    pub fn get_profile_args(&self, profile_cli_arg_value: &str) -> Vec<String> {
        return (self.profile_args_fn)(profile_cli_arg_value);
    }

    pub fn supports_incognito(&self) -> bool {
        return !self.incognito_args.is_empty();
    }

    pub fn get_incognito_args(&self) -> &Vec<String> {
        return &self.incognito_args;
    }

    pub fn get_transformed_url(
        &self,
        profile_cli_container_name: Option<&String>,
        url: &str,
    ) -> String {
        return (self.url_transform_fn)(profile_cli_container_name, url);
    }

    pub fn is_url_as_first_arg(&self) -> bool {
        return self.url_as_first_arg;
    }
}

#[derive(Clone)]
pub struct AppIdentifier {
    mac_bundle_id: String,
    linux_desktop_id: String,
}

impl AppIdentifier {
    fn new(mac_bundle_id: String, linux_desktop_id: String) -> Self {
        Self {
            mac_bundle_id: mac_bundle_id,
            linux_desktop_id: linux_desktop_id,
        }
    }

    pub fn new_for_os(app_id: String) -> Self {
        #[cfg(target_os = "macos")]
        return Self::new_mac(app_id);

        #[cfg(target_os = "linux")]
        return Self::new_linux(app_id);
    }

    pub fn new_mac(mac_bundle_id: String) -> Self {
        Self {
            mac_bundle_id: mac_bundle_id,
            linux_desktop_id: "".to_string(),
        }
    }

    fn new_linux(linux_desktop_id: String) -> Self {
        Self {
            mac_bundle_id: "".to_string(),
            linux_desktop_id: linux_desktop_id,
        }
    }

    fn app_id(&self) -> &str {
        #[cfg(target_os = "macos")]
        return self.mac_bundle_id.as_str();

        #[cfg(target_os = "linux")]
        return self.linux_desktop_id.as_str();
    }
}

struct AppConfigDir {
    root_path: PathBuf,
    mac_config_dir_relative: PathBuf,
    linux_config_dir_relative: PathBuf,
}

impl AppConfigDir {
    fn config_dir_relative(&self) -> &Path {
        #[cfg(target_os = "macos")]
        return self.mac_config_dir_relative.as_path();

        #[cfg(target_os = "linux")]
        return self.linux_config_dir_relative.as_path();
    }

    fn config_dir_absolute(&self) -> PathBuf {
        return self.root_path.join(self.config_dir_relative());
    }
}

fn convert_spotify_uri(profile_cli_container_name: Option<&String>, url: &str) -> String {
    let unknown = "spotify:track:2QFvsZEjbketrpCgCNC9Zp".to_string();

    let result = Url::parse(url);
    if result.is_err() {
        return unknown;
    }

    let url1 = result.unwrap();
    //let domain_maybe = url1.domain();
    // verify it's "open.spotify.com" ?
    //let x = url1.path();

    let segments_maybe = url1.path_segments().map(|c| c.collect::<Vec<_>>());
    if segments_maybe.is_none() {
        return unknown;
    }

    let segments = segments_maybe.unwrap();
    let type_maybe = segments.get(0);
    let id_maybe = segments.get(1);

    let uri_maybe = type_maybe
        .zip(id_maybe)
        .map(|(resource_type, resource_id)| format!("spotify:{}:{}", resource_type, resource_id));

    return uri_maybe.unwrap_or_else(|| unknown);

    //var data=document.URL.match(/[\/\&](track|playlist|album|artist|show|episode)\/([^\&\#\/\?]+)/i);
    //console.log("This is a "+data[1]+" with id:"+data[2]+"\nAttempting to redirect");
    //window.location.replace('spotify:'+data[1]+':'+data[2]);

    // TODO: translate https://open.spotify.com/track/2QFvsZEjbketrpCgCNC9Zp?si=4dc7c9cdc3b84286
    //       to spotify arg

    //let path = Path::new(url);
    //let path = "https://open.spotify.com/track/2QFvsZEjbketrpCgCNC9Zp?si=4dc7c9cdc3b84286";

    //info!(path);

    // 1: "track"
    // 2: "2QFvsZEjbketrpCgCNC9Zp"
}
