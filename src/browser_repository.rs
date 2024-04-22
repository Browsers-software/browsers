use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::info;
use url::form_urlencoded::byte_serialize;
use url::Url;

use crate::url_rule::UrlGlobMatcher;
use crate::{
    chromium_profiles_parser, firefox_profiles_parser, paths, slack_profiles_parser,
    slack_url_parser, url_rule, CommonBrowserProfile, InstalledAppProfiles,
    InstalledBrowserProfile,
};

// Holds list of custom SupportedApp configurations
// All other apps will be the "default" supported app implementation
pub struct SupportedAppRepository {
    snap_base: PathBuf,
    chromium_user_dir_base: PathBuf,
    firefox_user_dir_base: PathBuf,
    supported_apps: HashMap<String, SupportedApp>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum AppOS {
    LINUX,
    MAC,
    WINDOWS,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
enum AppKind {
    GENERIC,
    CHROMIUM,
    FIREFOX,
    SLACK,
    LINEAR,
    MIMESTREAM,
    NOTION,
    SPOTIFY,
    TELEGRAM,
    WORKFLOWY,
    ZOOM,
}

#[derive(Deserialize, Debug)]
struct AppConfigRepository {
    apps: Vec<AppConfig>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
struct AppConfig {
    // linux, etc
    os: AppOS,
    // chromium, etc
    kind: AppKind,
    id: String,
    config_dir_relative: String,
    snap_id: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            os: AppOS::LINUX,
            kind: AppKind::GENERIC,
            id: "".to_string(),
            config_dir_relative: "".to_string(),
            snap_id: None,
        }
    }
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

    fn load_repository(&self) -> Vec<AppConfig> {
        let repository_toml_path = paths::get_repository_toml_path();

        info!("Repository: {}", repository_toml_path.display());

        if !repository_toml_path.exists() {
            return vec![];
        }

        // Open the file in read-only mode with buffer.
        let file = File::open(repository_toml_path.as_path()).unwrap();
        let mut reader = BufReader::new(file);

        let mut data: String = String::new();
        reader.read_to_string(&mut data).unwrap();
        let result = toml::from_str(data.as_ref());
        let repository: AppConfigRepository = result.unwrap();
        return repository.apps;
    }

    fn add_apps_from_repository_file(&mut self) {
        let app_configs = self.load_repository();
        for app_config in app_configs {
            let app = self.create_app_from_app_config(app_config);
            self.add(app);
        }
    }

    fn generate_app_id_to_supported_app(&mut self) {
        self.add_apps_from_repository_file();
    }

    fn create_firefox_based_windows(
        &mut self,
        bundle_id: &str,
        config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_windows(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(config_dir_relative),
        );

        let app_id = AppIdentifier::new_windows(bundle_id);
        let app = Self::firefox_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            PathBuf::from(""),
        );
        return app;
    }

    fn create_firefox_based_mac(
        &mut self,
        mac_bundle_id: &str,
        mac_config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_mac(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        let app_id = AppIdentifier::new_mac(mac_bundle_id);
        let app = Self::firefox_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            PathBuf::from(""),
        );
        return app;
    }

    fn create_firefox_based_linux(
        &mut self,
        linux_desktop_id: &str,
        linux_snap_id: &str,
        linux_config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_linux(
            self.firefox_user_dir_base.clone(),
            PathBuf::from(linux_config_dir_relative),
        );

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        let app_id = AppIdentifier::new_linux(linux_desktop_id);
        let app = Self::firefox_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            snap_app_config_dir_absolute.clone(),
            PathBuf::from(""),
        );
        return app;
    }

    fn start(&mut self) -> &mut SupportedAppRepository {
        return self;
    }

    fn create_app_from_app_config(&mut self, app_config: AppConfig) -> SupportedApp {
        let app_id = app_config.id.as_str();

        let config_dir_relative = app_config.config_dir_relative.as_str();

        let snap_id_owned = app_config.snap_id.unwrap_or_default();
        let linux_snap_id = snap_id_owned.as_str();

        let app = match app_config.kind {
            AppKind::GENERIC => {
                let restricted_domain_patterns = vec![];
                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
            AppKind::CHROMIUM => match app_config.os {
                AppOS::LINUX => {
                    self.create_chromium_based_linux(app_id, linux_snap_id, config_dir_relative)
                }
                AppOS::MAC => self.create_chromium_based_mac(app_id, config_dir_relative),
                AppOS::WINDOWS => self.create_chromium_based_windows(app_id, config_dir_relative),
            },
            AppKind::FIREFOX => match app_config.os {
                AppOS::LINUX => {
                    self.create_firefox_based_linux(app_id, linux_snap_id, config_dir_relative)
                }
                AppOS::MAC => self.create_firefox_based_mac(app_id, config_dir_relative),
                AppOS::WINDOWS => self.create_firefox_based_windows(app_id, config_dir_relative),
            },
            AppKind::LINEAR => {
                let restricted_domain_patterns = vec!["linear.app".to_string()];
                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
            AppKind::MIMESTREAM => {
                let restricted_domain_patterns = vec!["links.mimestream.com".to_string()];
                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
            AppKind::NOTION => {
                let restricted_domain_patterns =
                    vec!["notion.so".to_string(), "www.notion.so".to_string()];
                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
            AppKind::SLACK => match app_config.os {
                AppOS::LINUX => self.create_slack_linux(app_id, linux_snap_id, config_dir_relative),
                AppOS::MAC => self.create_slack_mac(app_id, config_dir_relative),
                AppOS::WINDOWS => self.create_slack_windows(app_id, config_dir_relative),
            },
            AppKind::SPOTIFY => {
                let restricted_domain_patterns = vec!["open.spotify.com".to_string()];
                Self::create_generic_app_with_url(
                    app_config.os,
                    app_id,
                    restricted_domain_patterns,
                    convert_spotify_uri,
                )
            }
            AppKind::TELEGRAM => {
                let restricted_domain_patterns = vec!["t.me".to_string()];
                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
            AppKind::WORKFLOWY => {
                let restricted_domain_patterns = vec!["workflowy.com".to_string()];
                Self::create_generic_app_with_url(
                    app_config.os,
                    app_id,
                    restricted_domain_patterns,
                    convert_workflowy_uri,
                )
            }
            AppKind::ZOOM => {
                let restricted_domain_patterns = vec![
                    "zoom.us".to_string(),
                    "eu01web.zoom.us".to_string(),
                    "us02web.zoom.us".to_string(),
                    "us03web.zoom.us".to_string(),
                    "us04web.zoom.us".to_string(),
                    "us05web.zoom.us".to_string(),
                    "us06web.zoom.us".to_string(),
                    "us07web.zoom.us".to_string(),
                ];

                Self::create_generic_app(app_config.os, app_id, restricted_domain_patterns)
            }
        };

        return app;
    }

    fn create_slack_linux(
        &mut self,
        linux_desktop_id: &str,
        linux_snap_id: &str,
        linux_config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_linux(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(linux_config_dir_relative),
        );

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        let app_id = AppIdentifier::new_linux(linux_desktop_id);
        let app = Self::slack_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            snap_app_config_dir_absolute.clone(),
            PathBuf::from(""),
        );

        return app;
    }

    fn create_slack_mac(
        &mut self,
        mac_bundle_id: &str,
        mac_config_dir_relative: &str,
    ) -> SupportedApp {
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
        return app;
    }

    fn create_slack_windows(&mut self, bundle_id: &str, config_dir_relative: &str) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_windows(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(config_dir_relative),
        );

        let app_id = AppIdentifier::new_windows(bundle_id);
        let app = Self::slack_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            PathBuf::from(""),
        );
        return app;
    }

    fn create_chromium_based_mac(
        &mut self,
        mac_bundle_id: &str,
        mac_config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_mac(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(mac_config_dir_relative),
        );

        let app_id = AppIdentifier::new_mac(mac_bundle_id);
        let app = Self::chromium_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            PathBuf::from(""),
        );
        return app;
    }

    fn create_chromium_based_windows(
        &mut self,
        bundle_id: &str,
        config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_windows(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(config_dir_relative),
        );

        let app_id = AppIdentifier::new_windows(bundle_id);
        let app = Self::chromium_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            PathBuf::from(""),
            PathBuf::from(""),
        );
        return app;
    }

    fn create_chromium_based_linux(
        &mut self,
        linux_desktop_id: &str,
        linux_snap_id: &str,
        linux_config_dir_relative: &str,
    ) -> SupportedApp {
        let app_config_dir = AppConfigDir::new_linux(
            self.chromium_user_dir_base.clone(),
            PathBuf::from(linux_config_dir_relative),
        );

        let snap_app_config_dir_absolute =
            self.snap_config_dir_absolute_path(linux_snap_id, linux_config_dir_relative);

        let app_id = AppIdentifier::new_linux(linux_desktop_id);
        let app = Self::chromium_based_app(
            app_id,
            app_config_dir.config_dir_absolute(),
            snap_app_config_dir_absolute.clone(),
            PathBuf::from(""),
        );
        return app;
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
            incognito_args: vec!["--incognito".to_string()],
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

                // encode target url so it can be passed as a parameter
                let url_encoded: String = byte_serialize(url.as_bytes()).collect();

                let full_url = fake_url + "&url=" + url_encoded.as_ref();
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
            incognito_args: vec!["--private-window".to_string()],
            url_transform_fn: firefox_url_transform_fn,
            url_as_first_arg: true,
        }
    }

    fn create_generic_app(
        os: AppOS,
        app_id: &str,
        restricted_domain_patterns: Vec<String>,
    ) -> SupportedApp {
        let url_transform_fn: UrlTransformFn = |_, url| url.to_string();
        return Self::create_generic_app_with_url(
            os,
            app_id,
            restricted_domain_patterns,
            url_transform_fn,
        );
    }

    fn create_generic_app_with_url(
        os: AppOS,
        app_id: &str,
        restricted_domain_patterns: Vec<String>,
        url_transform_fn: UrlTransformFn,
    ) -> SupportedApp {
        let app_identifier = Self::create_app_identifier(os, app_id);
        let app = Self::generic_app_with_url(
            app_identifier,
            restricted_domain_patterns,
            url_transform_fn,
        );
        return app;
    }

    fn create_app_identifier(os: AppOS, app_id: &str) -> AppIdentifier {
        match os {
            AppOS::LINUX => AppIdentifier::new_linux(app_id),
            AppOS::MAC => AppIdentifier::new_mac(app_id),
            AppOS::WINDOWS => AppIdentifier::new_windows(app_id),
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

    pub fn get_restricted_hostname_matchers(&self) -> &Vec<UrlGlobMatcher> {
        return &self.restricted_url_matchers;
    }

    pub fn find_profiles(
        &self,
        binary_path: &Path,
        app_config_dir_abs: &Path,
    ) -> InstalledAppProfiles {
        return if let Some(find_profiles_fn) = self.find_profiles_fn {
            let mut browser_profiles: Vec<InstalledBrowserProfile> =
                find_profiles_fn(app_config_dir_abs, binary_path, self.get_app_id());

            browser_profiles.sort_by_key(|p| p.profile_name.clone());
            if browser_profiles.is_empty() {
                InstalledAppProfiles::new_placeholder()
            } else {
                InstalledAppProfiles::new_real(browser_profiles)
            }
        } else {
            InstalledAppProfiles::new_placeholder()
        };
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

// TODO: support
// "https://links.mimestream.com/g/madis@qminderapp.com/t/18f06bace4319301"
// "mimestream:///open/g/madis@qminderapp.com/t/18f06bace4319301"
// "https://links.mimestream.com/LINK" to "mimestream:///open/LINK"
fn convert_mimestream_uri(_: &CommonBrowserProfile, url_str: &str) -> String {
    let result = Url::parse(url_str);
    if result.is_err() {
        return "".to_string();
    }
    let mut url = result.unwrap();
    let _ = url.set_scheme("mimestream");

    return url.as_str().to_string();
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
