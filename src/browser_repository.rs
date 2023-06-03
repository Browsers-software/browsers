use std::collections::HashMap;
use std::path::{Path, PathBuf};

use druid::piet::TextStorage;
use tracing::info;
use url::form_urlencoded::Parse;
use url::Url;

use crate::ui::RESTORE_HIDDEN_PROFILE;
use crate::url_rule::UrlGlobMatcher;
use crate::{
    chromium_profiles_parser, firefox_profiles_parser, paths, slack_profiles_parser,
    slack_url_parser, url_rule, CommonBrowserProfile, InstalledBrowserProfile,
};

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

    pub fn get_or_generate(
        &self,
        app_id_str: &str,
        restricted_domain_patterns: &Vec<String>,
    ) -> SupportedApp {
        return self
            .supported_apps
            .get(app_id_str)
            .map(|app| app.to_owned())
            .unwrap_or_else(|| {
                let app_id = AppIdentifier::new_for_os(app_id_str);
                Self::generic_app(app_id, restricted_domain_patterns.clone())
            });
    }

    fn add(&mut self, supported_app: SupportedApp) -> &mut SupportedAppRepository {
        self.supported_apps
            .insert(supported_app.get_app_id().to_string(), supported_app);
        return self;
    }

    fn generate_app_id_to_supported_app(&mut self) {
        self.start()
            .add_chromium_based_mac(vec!["company.thebrowser.Browser"], "Arc/User Data")
            .add_chromium_based_mac(vec!["com.google.Chrome"], "Google/Chrome")
            .add_chromium_based_linux(vec!["google-chrome.desktop"], "", "google-chrome")
            .add_chromium_based_windows(vec!["Google Chrome"], "Google/Chrome/User Data")
            .add_chromium_based_mac(vec!["com.google.Chrome.beta"], "Google/Chrome Beta")
            .add_chromium_based_linux(vec!["google-chrome-beta.desktop"], "", "google-chrome-beta")
            .add_chromium_based_mac(vec!["com.google.Chrome.dev"], "Google/Chrome Dev")
            .add_chromium_based_linux(vec!["google-chrome-dev.desktop"], "", "google-chrome-dev")
            .add_chromium_based_mac(vec!["com.google.Chrome.canary"], "Google/Chrome Canary")
            .add_chromium_based_linux(
                vec!["google-chrome-canary.desktop"],
                "",
                "google-chrome-canary",
            )
            .add_chromium_based_mac(vec!["org.chromium.Chromium"], "Chromium")
            .add_chromium_based_linux(
                vec![
                    "chromium.desktop",
                    "chromium_chromium.desktop",
                    "chromium-browser.desktop",
                ],
                "chromium",
                "chromium",
            )
            .add_chromium_based_mac(vec!["com.avast.browser"], "AVAST Software/Browser")
            .add_chromium_based_mac(vec!["com.whisttechnologies.whist"], "Whist/Whist-Browser")
            .add_chromium_based_mac(vec!["com.bookry.wavebox"], "WaveboxApp")
            .add_chromium_based_mac(vec!["com.coccoc.Coccoc"], "Coccoc")
            .add_chromium_based_mac(vec!["net.qihoo.360browser"], "360Chrome")
            .add_chromium_based_mac(
                vec!["ru.yandex.desktop.yandex-browser"],
                "Yandex/YandexBrowser",
            )
            .add_chromium_based_mac(vec!["com.microsoft.edgemac"], "Microsoft Edge")
            .add_chromium_based_mac(vec!["com.microsoft.edgemac.Beta"], "Microsoft Edge Beta")
            .add_chromium_based_mac(vec!["com.microsoft.edgemac.Dev"], "Microsoft Edge Dev")
            .add_chromium_based_linux(vec!["microsoft-edge-dev.desktop"], "", "microsoft-edge-dev")
            .add_chromium_based_mac(vec!["com.microsoft.edgemac.Canary"], "Microsoft Edge Canary")
            .add_chromium_based_mac(vec!["com.brave.Browser"], "BraveSoftware/Brave-Browser")
            .add_chromium_based_linux(
                vec!["brave-browser.desktop"],
                "brave",
                "BraveSoftware/Brave-Browser",
            )
            .add_chromium_based_mac(
                vec!["com.brave.Browser.beta"],
                "BraveSoftware/Brave-Browser-Beta",
            )
            .add_chromium_based_mac(
                vec!["com.brave.Browser.nightly"],
                "BraveSoftware/Brave-Browser-Nightly",
            )
            .add_chromium_based_mac(vec!["com.pushplaylabs.sidekick"], "Sidekick")
            .add_chromium_based_mac(vec!["com.vivaldi.Vivaldi"], "Vivaldi")
            .add_chromium_based_linux(vec!["vivaldi-stable.desktop"], "", "vivaldi")
            .add_chromium_based_mac(vec!["com.vivaldi.Vivaldi.snapshot"], "Vivaldi Snapshot")
            .add_chromium_based_mac(vec!["com.naver.Whale"], "Naver/Whale")
            .add_chromium_based_mac(vec!["de.iridiumbrowser"], "Iridium")
            .add_firefox_based_mac(vec!["org.mozilla.firefox"], "Firefox")
            .add_firefox_based_linux(
                vec![
                    "firefox.desktop",
                    "firefox_firefox.desktop",
                    "firefox-esr.desktop",
                ],
                "firefox",
                ".mozilla/firefox",
            )
            .add_firefox_based_windows(vec!["Mozilla Firefox"], "Mozilla/Firefox")
            .add_firefox_based_mac(vec!["org.mozilla.firefoxdeveloperedition"], "Firefox")
            .add_firefox_based_windows(vec!["Firefox Developer Edition"], "Mozilla/Firefox")
            .add_firefox_based_mac(vec!["org.mozilla.nightly"], "Firefox")
            .add_firefox_based_mac(vec!["org.mozilla.floorp"], "Floorp")
            .add_firefox_based_mac(vec!["org.torproject.torbrowser"], "TorBrowser-Data/Browser")
            .add_firefox_based_mac(vec!["org.mozilla.librewolf"], "LibreWolf")
            .add_firefox_based_mac(vec!["net.waterfox.waterfox"], "Waterfox")
            .add_slack_mac("com.tinyspeck.slackmacgap", "Slack")
            .add(Self::linear_app())
            .add(Self::notion_app())
            .add(Self::spotify_app())
            .add(Self::telegram_app())
            .add(Self::workflowy_app())
            .add(Self::zoom_app());
    }

    fn add_firefox_based_windows(
        &mut self,
        bundle_ids: Vec<&str>,
        config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_windows(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(config_dir_relative),
        );

        for bundle_id in bundle_ids {
            let app_id = AppIdentifier::new_windows(bundle_id);
            let app = Self::firefox_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                PathBuf::from(""),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
    }

    fn add_firefox_based_mac(
        &mut self,
        mac_bundle_ids: Vec<&str>,
        mac_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_mac(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        for mac_bundle_id in mac_bundle_ids {
            let app_id = AppIdentifier::new_mac(mac_bundle_id);
            let app = Self::firefox_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                PathBuf::from(""),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
    }

    fn add_firefox_based_linux(
        &mut self,
        linux_desktop_ids: Vec<&str>,
        linux_snap_id: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_linux(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(linux_config_dir_relative),
        );

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        for linux_desktop_id in linux_desktop_ids {
            let app_id = AppIdentifier::new_linux(linux_desktop_id);
            let app = Self::firefox_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                snap_app_config_dir_absolute.clone(),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
    }

    fn start(&mut self) -> &mut SupportedAppRepository {
        return self;
    }

    fn add_slack_mac(
        &mut self,
        mac_bundle_id: &str,
        mac_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let user_home_for_unsandboxed_app = paths::get_user_home_for_unsandboxed_app();
        let unsandboxed_user_dir_root = user_home_for_unsandboxed_app
            .join("Library")
            .join("Application Support");
        let unsandboxed_app_config_dir = AppConfigDir::new_mac(
            unsandboxed_user_dir_root.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        let user_home_for_sandboxed_app = paths::get_user_home_for_sandboxed_app(mac_bundle_id);
        let sandboxed_user_dir_root = user_home_for_sandboxed_app
            .join("Library")
            .join("Application Support");
        let sandboxed_app_config_dir = AppConfigDir::new_mac(
            sandboxed_user_dir_root.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        let app_id = AppIdentifier::new_mac(mac_bundle_id);
        let app = Self::slack_app(
            app_id,
            unsandboxed_app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            sandboxed_app_config_dir.config_dir_absolute(),
        );
        self.add(app);

        return self;
    }

    fn add_chromium_based_mac(
        &mut self,
        mac_bundle_ids: Vec<&str>,
        mac_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_mac(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        for mac_bundle_id in mac_bundle_ids {
            let app_id = AppIdentifier::new_mac(mac_bundle_id);
            let app = Self::chromium_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                PathBuf::from(""),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
    }

    fn add_chromium_based_windows(
        &mut self,
        bundle_ids: Vec<&str>,
        config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_windows(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(config_dir_relative),
        );

        for bundle_id in bundle_ids {
            let app_id = AppIdentifier::new_windows(bundle_id);
            let app = Self::chromium_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                PathBuf::from(""),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
    }

    fn add_chromium_based_linux(
        &mut self,
        linux_desktop_ids: Vec<&str>,
        linux_snap_id: &str,
        linux_config_dir_relative: &str,
    ) -> &mut SupportedAppRepository {
        let app_config_dir = AppConfigDir::new_linux(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(linux_config_dir_relative),
        );

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        for linux_desktop_id in linux_desktop_ids {
            let app_id = AppIdentifier::new_linux(linux_desktop_id);
            let app = Self::chromium_based_app(
                app_id,
                app_config_dir.config_dir_absolute(),
                snap_app_config_dir_absolute.clone(),
                PathBuf::from(""),
            );
            self.add(app);
        }

        return self;
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
        macos_sandbox_app_config_dir_absolute: PathBuf,
    ) -> SupportedApp {
        let chromium_url_transform_fn: UrlTransformFn = |_, url| url.to_string();

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: app_config_dir_absolute,
            snap_app_config_dir_absolute: snap_app_config_dir_absolute,
            macos_sandbox_app_config_dir_absolute: macos_sandbox_app_config_dir_absolute,
            find_profiles_fn: Some(chromium_profiles_parser::find_chromium_profiles),
            restricted_url_matchers: vec![],
            profile_args_fn: |profile_cli_arg_value| {
                vec![format!("--profile-directory={}", profile_cli_arg_value)]
            },
            incognito_args: vec!["-incognito".to_string()],
            url_transform_fn: chromium_url_transform_fn,
            url_as_first_arg: true,
        }
    }

    fn firefox_based_app(
        app_id: AppIdentifier,
        app_config_dir_absolute: PathBuf,
        snap_app_config_dir_absolute: PathBuf,
        macos_sandbox_app_config_dir_absolute: PathBuf,
    ) -> SupportedApp {
        let firefox_url_transform_fn: UrlTransformFn = |common_browser_profile, url| {
            let container_name_maybe: Option<&String> =
                common_browser_profile.profile_cli_container_name.as_ref();
            return if let Some(container_name) = container_name_maybe {
                let fake_url = "ext+container:name=".to_string() + container_name;
                let full_url = fake_url + "&url=" + url.clone();
                full_url.to_string()
            } else {
                url.to_string()
            };
        };

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: app_config_dir_absolute,
            snap_app_config_dir_absolute: snap_app_config_dir_absolute,
            macos_sandbox_app_config_dir_absolute: macos_sandbox_app_config_dir_absolute,
            find_profiles_fn: Some(firefox_profiles_parser::find_firefox_profiles),
            restricted_url_matchers: vec![],
            profile_args_fn: |profile_cli_arg_value| {
                vec!["-P".to_string(), profile_cli_arg_value.to_string()]
            },
            incognito_args: vec!["-private".to_string()],
            url_transform_fn: firefox_url_transform_fn,
            url_as_first_arg: true,
        }
    }

    fn generic_app(app_id: AppIdentifier, restricted_domain_patterns: Vec<String>) -> SupportedApp {
        Self::generic_app_with_url(app_id, restricted_domain_patterns, |_, url| url.to_string())
    }

    fn generic_app_with_url(
        app_id: AppIdentifier,
        restricted_domain_patterns: Vec<String>,
        url_transform_fn: UrlTransformFn,
    ) -> SupportedApp {
        let restricted_url_matchers =
            Self::generate_restricted_hostname_matchers(&restricted_domain_patterns);

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: PathBuf::new(),
            snap_app_config_dir_absolute: PathBuf::new(),
            macos_sandbox_app_config_dir_absolute: PathBuf::new(),
            find_profiles_fn: None,
            restricted_url_matchers: restricted_url_matchers,
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: url_transform_fn,
            url_as_first_arg: false,
        }
    }

    fn generate_restricted_hostname_matchers(
        restricted_domains: &Vec<String>,
    ) -> Vec<UrlGlobMatcher> {
        let restricted_hostname_matchers: Vec<UrlGlobMatcher> = restricted_domains
            .iter()
            .map(|url_pattern| {
                let url_matcher = url_rule::to_url_matcher(url_pattern.as_str());
                let glob_matcher = url_matcher.to_glob_matcher();
                glob_matcher
            })
            .collect();

        return restricted_hostname_matchers;
    }

    fn linear_app() -> SupportedApp {
        let app_id = AppIdentifier::new("com.linear", "NOLINUXAPPEXISTS.desktop", "TODOWINDOWS");

        Self::generic_app(app_id, vec!["linear.app".to_string()])
    }

    fn notion_app() -> SupportedApp {
        let app_id = AppIdentifier::new("notion.id", "NOLINUXAPPEXISTS.desktop", "TODOWINDOWS");

        Self::generic_app(
            app_id,
            vec!["notion.so".to_string(), "www.notion.so".to_string()],
        )
    }

    fn spotify_app() -> SupportedApp {
        let app_id = AppIdentifier::new("com.spotify.client", "spotify_spotify.desktop", "spotify");

        Self::generic_app_with_url(
            app_id,
            vec!["open.spotify.com".to_string()],
            convert_spotify_uri,
        )
    }

    fn telegram_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "ru.keepcoder.Telegram".to_string(),
            linux_desktop_id: "telegram-desktop_telegram-desktop.desktop".to_string(),
            windows_app_id: "WINDOWSTODO".to_string(),
        };

        Self::generic_app(app_id, vec!["t.me".to_string()])
    }

    fn slack_app(
        app_id: AppIdentifier,
        app_config_dir_absolute: PathBuf,
        snap_app_config_dir_absolute: PathBuf,
        macos_sandbox_app_config_dir_absolute: PathBuf,
    ) -> SupportedApp {
        // todo: filter only specific profiles? But per profile?
        let restricted_domain_patterns = vec![
            "*.slack.com".to_string(),
            "*.enterprise.slack.com".to_string(),
        ];
        let restricted_url_matchers =
            Self::generate_restricted_hostname_matchers(&restricted_domain_patterns);

        SupportedApp {
            app_id: app_id,
            app_config_dir_absolute: app_config_dir_absolute,
            snap_app_config_dir_absolute: snap_app_config_dir_absolute,
            macos_sandbox_app_config_dir_absolute: macos_sandbox_app_config_dir_absolute,
            find_profiles_fn: Some(slack_profiles_parser::find_slack_profiles),
            restricted_url_matchers: restricted_url_matchers,
            profile_args_fn: |_profile_cli_arg_value| vec![],
            incognito_args: vec![],
            url_transform_fn: convert_slack_uri,
            url_as_first_arg: false,
        }
    }

    fn workflowy_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "com.workflowy.desktop".to_string(),
            linux_desktop_id: "LINUXTODO".to_string(),
            windows_app_id: "URL:workflowy".to_string(),
        };

        Self::generic_app_with_url(app_id, vec!["workflowy.com".to_string()], convert_workflowy_uri)
    }

    fn zoom_app() -> SupportedApp {
        let app_id = AppIdentifier {
            mac_bundle_id: "us.zoom.xos".to_string(),
            linux_desktop_id: "Zoom.desktop".to_string(),
            windows_app_id: "WINDOWSTODO".to_string(),
        };

        Self::generic_app(
            app_id,
            vec![
                "zoom.us".to_string(),
                "eu01web.zoom.us".to_string(),
                "us02web.zoom.us".to_string(),
                "us03web.zoom.us".to_string(),
                "us04web.zoom.us".to_string(),
                "us05web.zoom.us".to_string(),
                "us06web.zoom.us".to_string(),
                "us07web.zoom.us".to_string(),
            ],
        )
    }
}

#[derive(Clone)]
pub struct SupportedApp {
    app_id: AppIdentifier,
    app_config_dir_absolute: PathBuf,
    snap_app_config_dir_absolute: PathBuf,
    macos_sandbox_app_config_dir_absolute: PathBuf,
    restricted_url_matchers: Vec<UrlGlobMatcher>,
    find_profiles_fn: Option<
        fn(
            app_config_dir_absolute: &Path,
            binary_path: &Path,
            app_id: &str,
        ) -> Vec<InstalledBrowserProfile>,
    >,
    profile_args_fn: fn(profile_cli_arg_value: &str) -> Vec<String>,
    incognito_args: Vec<String>,
    url_transform_fn: UrlTransformFn,
    url_as_first_arg: bool,
}

pub type UrlTransformFn = fn(&CommonBrowserProfile, url: &str) -> String;

//
// profile_cli_arg_value: workspace.id.to_string(),
// profile_cli_container_name: Some(workspace.domain.to_string()),
// I need mapping from domain to team id (workspace id)
// so in this case from cli_container_name to cli_arg_value
// or maybe just pass the whole profile info?
// or maybe its better to make it a closure instead,
// so we don't depend on profile selection by user, so user can select wrong profile as well

impl SupportedApp {
    pub fn get_app_id(&self) -> &str {
        return self.app_id.app_id();
    }

    pub fn get_app_config_dir_abs(&self, is_snap: bool, is_macos_sandbox: bool) -> &Path {
        return if is_snap {
            &self.snap_app_config_dir_absolute.as_path()
        } else if is_macos_sandbox {
            &self.macos_sandbox_app_config_dir_absolute.as_path()
        } else {
            &self.app_config_dir_absolute.as_path()
        };
    }

    pub fn get_app_config_dir_absolute(&self, is_snap: bool, is_macos_sandbox: bool) -> &str {
        return self
            .get_app_config_dir_abs(is_snap, is_macos_sandbox)
            .to_str()
            .unwrap();
    }

    pub fn get_restricted_hostname_matchers(&self) -> &Vec<UrlGlobMatcher> {
        return &self.restricted_url_matchers;
    }

    pub fn find_profiles(
        &self,
        binary_path: &Path,
        is_snap: bool,
        is_macos_sandbox: bool,
    ) -> Vec<InstalledBrowserProfile> {
        return if let Some(find_profiles_fn) = self.find_profiles_fn {
            let app_config_dir_abs = self.get_app_config_dir_abs(is_snap, is_macos_sandbox);
            let mut browser_profiles: Vec<InstalledBrowserProfile> =
                find_profiles_fn(app_config_dir_abs, binary_path, self.get_app_id());

            browser_profiles.sort_by_key(|p| p.profile_name.clone());
            browser_profiles
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
            profile_restricted_url_patterns: vec![],
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
        common_browser_profile: &CommonBrowserProfile,
        url: &str,
    ) -> String {
        return (self.url_transform_fn)(common_browser_profile, url);
    }

    pub fn is_url_as_first_arg(&self) -> bool {
        return self.url_as_first_arg;
    }
}

#[derive(Clone)]
pub struct AppIdentifier {
    mac_bundle_id: String,
    linux_desktop_id: String,
    windows_app_id: String,
}

impl AppIdentifier {
    fn new(mac_bundle_id: &str, linux_desktop_id: &str, windows_app_id: &str) -> Self {
        Self {
            mac_bundle_id: mac_bundle_id.to_string(),
            linux_desktop_id: linux_desktop_id.to_string(),
            windows_app_id: windows_app_id.to_string(),
        }
    }

    pub fn new_for_os(app_id: &str) -> Self {
        #[cfg(target_os = "macos")]
        return Self::new_mac(app_id);

        #[cfg(target_os = "linux")]
        return Self::new_linux(app_id);

        #[cfg(target_os = "windows")]
        return Self::new_windows(app_id);
    }

    pub fn new_mac(mac_bundle_id: &str) -> Self {
        Self {
            mac_bundle_id: mac_bundle_id.to_string(),
            linux_desktop_id: "".to_string(),
            windows_app_id: "".to_string(),
        }
    }

    fn new_linux(linux_desktop_id: &str) -> Self {
        Self {
            mac_bundle_id: "".to_string(),
            linux_desktop_id: linux_desktop_id.to_string(),
            windows_app_id: "".to_string(),
        }
    }

    fn new_windows(windows_app_id: &str) -> Self {
        Self {
            mac_bundle_id: "".to_string(),
            linux_desktop_id: "".to_string(),
            windows_app_id: windows_app_id.to_string(),
        }
    }

    fn app_id(&self) -> &str {
        #[cfg(target_os = "macos")]
        return self.mac_bundle_id.as_str();

        #[cfg(target_os = "linux")]
        return self.linux_desktop_id.as_str();

        #[cfg(target_os = "windows")]
        return self.windows_app_id.as_str();
    }
}

struct AppConfigDir {
    root_path: PathBuf,
    mac_config_dir_relative: PathBuf,
    linux_config_dir_relative: PathBuf,
    windows_config_dir_relative: PathBuf,
}

impl AppConfigDir {
    pub fn new_mac(root_path: PathBuf, mac_config_dir_relative: PathBuf) -> Self {
        Self {
            root_path: root_path,
            mac_config_dir_relative: mac_config_dir_relative,
            linux_config_dir_relative: PathBuf::from(""),
            windows_config_dir_relative: PathBuf::from(""),
        }
    }

    pub fn new_linux(root_path: PathBuf, linux_config_dir_relative: PathBuf) -> Self {
        Self {
            root_path: root_path,
            mac_config_dir_relative: PathBuf::from(""),
            linux_config_dir_relative: linux_config_dir_relative,
            windows_config_dir_relative: PathBuf::from(""),
        }
    }

    pub fn new_windows(root_path: PathBuf, windows_config_dir_relative: PathBuf) -> Self {
        Self {
            root_path: root_path,
            mac_config_dir_relative: PathBuf::from(""),
            linux_config_dir_relative: PathBuf::from(""),
            windows_config_dir_relative: windows_config_dir_relative,
        }
    }

    fn config_dir_relative(&self) -> &Path {
        #[cfg(target_os = "macos")]
        return self.mac_config_dir_relative.as_path();

        #[cfg(target_os = "linux")]
        return self.linux_config_dir_relative.as_path();

        #[cfg(target_os = "windows")]
        return self.windows_config_dir_relative.as_path();
    }

    fn config_dir_absolute(&self) -> PathBuf {
        return self.root_path.join(self.config_dir_relative());
    }
}

fn convert_slack_uri(common_browser_profile: &CommonBrowserProfile, url_str: &str) -> String {
    let profile_team_id: &str = &common_browser_profile.profile_cli_arg_value;
    let profile_team_domain_maybe: Option<&String> =
        common_browser_profile.profile_cli_container_name.as_ref();
    let profile_team_domain = profile_team_domain_maybe.unwrap();

    let unknown = format!("slack://channel?team={}", profile_team_id);

    let result = Url::parse(url_str);
    if result.is_err() {
        return unknown;
    }
    let url = result.unwrap();

    return slack_url_parser::convert_slack_uri(profile_team_id, profile_team_domain, &url);
}

fn convert_workflowy_uri(_: &CommonBrowserProfile, url_str: &str) -> String {
    let result = Url::parse(url_str);
    if result.is_err() {
        return "".to_string();
    }
    let mut url = result.unwrap();
    let _ = url.set_scheme("workflowy");

    return url.as_str().to_string();
}

fn convert_spotify_uri(_: &CommonBrowserProfile, url_str: &str) -> String {
    let unknown = "spotify:track:2QFvsZEjbketrpCgCNC9Zp".to_string();

    let result = Url::parse(url_str);
    if result.is_err() {
        return unknown;
    }

    let url = result.unwrap();
    //let host_maybe = url1.host_str();
    // verify it's "open.spotify.com" ?
    //let x = url1.path();

    let segments_maybe = url.path_segments().map(|c| c.collect::<Vec<_>>());
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
}
