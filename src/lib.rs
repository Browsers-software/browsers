use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::Debug;
use std::process::{Command, exit};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use tracing::{debug, info, instrument, warn};
use url::Url;
use url::form_urlencoded::Parse;

use crate::browser_repository::{SupportedApp, SupportedAppRepository};
use crate::gui::app::UiHandle;
use crate::gui::app_state::{UIBehavioralSettings, real_to_ui_browsers};
use crate::gui::app_state::{UIBrowser, UIProfileAndIncognito, UISettingsRule, UIVisualSettings};
use crate::url_rule::UrlGlobMatcher;
use crate::utils::{
    BehavioralConfig, Config, ConfigRule, OSAppFinder, ProfileAndOptions, UIConfig,
};

// a URL the OS handed us (CLI args, or a platform "open URL" event), plus which app asked us to
// open it - used for source-app-based opening rules
#[derive(Clone, Debug)]
pub struct UrlOpenInfo {
    pub url: String,
    pub source_bundle_id: String,
}

// used after handing a URL off to the chosen browser, or when quitting without ever having
// opened one (Cmd+Q, focus lost, Safari's hard-launch workaround) - distinct from a default
// `exit(0)` so it's identifiable if it ever shows up in a crash/exit-code report
pub const QUIT_EXIT_CODE: i32 = 0x0100;

pub mod gui;

pub mod paths;
pub mod utils;

mod browser_repository;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

mod chromium_profiles_parser;
mod firefox_profiles_parser;
mod slack_profiles_parser;
mod slack_url_parser;
mod url_rule;

// a browser (with profiles), or Spotify, Zoom, etc
pub struct GenericApp {
    app: BrowserCommon,
    profiles: Vec<CommonBrowserProfile>,
}

impl GenericApp {
    fn new(installed_browser: &InstalledBrowser, app_repository: &SupportedAppRepository) -> Self {
        let supported_app = app_repository.get_or_generate(
            installed_browser.bundle.as_str(),
            &installed_browser.restricted_domains,
        );
        let app = BrowserCommon {
            supported_app: supported_app,
            command: installed_browser.command.clone(),
            executable_path: installed_browser.executable_path.to_string(),
            display_name: installed_browser.display_name.to_string(),
            icon_path: installed_browser.icon_path.to_string(),
            profiles_type: installed_browser.profiles.profiles_type.clone(),
        };

        let arc = Arc::new(app.clone());
        let mut profiles: Vec<CommonBrowserProfile> = Vec::new();
        for installed_profile in &installed_browser.profiles.profiles {
            profiles.push(CommonBrowserProfile::new(&installed_profile, arc.clone()));
        }

        return Self {
            app: app,
            profiles: profiles,
        };
    }

    fn get_profiles(&self) -> &[CommonBrowserProfile] {
        return &self.profiles;
    }
}

#[derive(Clone)]
pub struct BrowserCommon {
    command: Vec<String>,
    executable_path: String,
    display_name: String,
    icon_path: String,
    supported_app: SupportedApp,
    profiles_type: InstalledAppProfilesType,
}

impl BrowserCommon {
    // used in configuration file to uniquely identify this app
    fn get_unique_app_id(&self) -> String {
        return self.executable_path.to_string();
    }

    fn has_real_profiles(&self) -> bool {
        self.profiles_type == InstalledAppProfilesType::RealProfiles
    }

    fn supports_incognito(&self) -> bool {
        return self.supported_app.supports_incognito();
    }

    fn get_browser_icon_path(&self) -> &str {
        return self.icon_path.as_str();
    }

    fn get_display_name(&self) -> &str {
        return self.display_name.as_str();
    }

    fn create_command(
        &self,
        common_browser_profile: &CommonBrowserProfile,
        url: &str,
        incognito_mode: bool,
    ) -> Command {
        let profile_cli_arg_value: &str = &common_browser_profile.profile_cli_arg_value;
        let profile_args = self.supported_app.get_profile_args(profile_cli_arg_value);
        let app_url = self
            .supported_app
            .get_transformed_url(common_browser_profile, url);

        let (main_command, command_arguments) = self.command.split_at(1);
        let main_command = main_command.first().unwrap(); // guaranteed to not be empty

        // TODO: support BSD - https://doc.rust-lang.org/reference/conditional-compilation.html
        if cfg!(target_os = "macos") {
            let mut cmd = Command::new("open");

            let arguments = cmd.arg("-b").arg(&self.supported_app.get_app_id());

            if !self.supported_app.is_url_as_first_arg() {
                // e.g Safari requires url to be as the apple event
                arguments.arg(app_url.clone());
            } else {
                // no direct link between !is_url_as_first_arg,
                // but mostly for Safari so it wont open new window
                // and all other not special apps
                arguments.arg("-n");
            }

            arguments.arg("--args");
            arguments.args(profile_args);

            if incognito_mode && self.supported_app.supports_incognito() {
                let incognito_args = self.supported_app.get_incognito_args();
                arguments.args(incognito_args);
            }

            if self.supported_app.is_url_as_first_arg() {
                arguments.arg(app_url.clone());
            }

            debug!("Launching: {:?}", cmd);
            return cmd;
        } else if cfg!(target_os = "linux") {
            let has_url_placeholder = command_arguments
                .iter()
                .any(|arg| arg.eq_ignore_ascii_case("%u"));

            let arguments = if has_url_placeholder {
                replace_url_placeholder(command_arguments, app_url.as_str())
            } else {
                command_arguments.to_vec()
            };

            let mut cmd = Command::new(main_command.to_string());

            // this might mess up the command,
            // if `main_command` is not yet the actual program that takes the incognito argument;
            // that's because the actual program might be in `arguments` (depends what's in the .desktop file)
            if incognito_mode && self.supported_app.supports_incognito() {
                let incognito_args = self.supported_app.get_incognito_args();
                cmd.args(incognito_args);
            }

            cmd.args(arguments);
            cmd.args(profile_args);

            // Non-browser apps don't have the placeholder
            if !has_url_placeholder {
                cmd.arg(app_url);
            }

            return cmd;
        } else if cfg!(target_os = "windows") {
            let mut cmd = Command::new(main_command.to_string());
            cmd.args(profile_args);

            if incognito_mode && self.supported_app.supports_incognito() {
                let incognito_args = self.supported_app.get_incognito_args();
                cmd.args(incognito_args);
            }

            cmd.arg(app_url);

            return cmd;
        }

        unimplemented!("platform is not supported yet");
    }
}

fn replace_url_placeholder(command_arguments: &[String], app_url: &str) -> Vec<String> {
    return command_arguments
        .iter()
        .map(|arg| {
            if arg.eq_ignore_ascii_case("%u") {
                app_url.to_string()
            } else {
                arg.to_string()
            }
        })
        .collect();
}

#[derive(Clone)]
pub struct CommonBrowserProfile {
    profile_cli_arg_value: String,
    profile_cli_container_name: Option<String>,
    profile_name: String,
    profile_icon: Option<String>,
    profile_restricted_url_matchers: Vec<UrlGlobMatcher>,
    app: Arc<BrowserCommon>,
}

impl CommonBrowserProfile {
    fn new(installed_browser_profile: &InstalledBrowserProfile, app: Arc<BrowserCommon>) -> Self {
        let profile_restricted_url_matchers = Self::generate_restricted_hostname_matchers(
            &installed_browser_profile.profile_restricted_url_patterns,
        );

        CommonBrowserProfile {
            profile_cli_arg_value: installed_browser_profile.profile_cli_arg_value.to_string(),
            profile_cli_container_name: installed_browser_profile
                .profile_cli_container_name
                .clone(),
            profile_name: installed_browser_profile.profile_name.to_string(),
            profile_icon: installed_browser_profile
                .profile_icon
                .as_ref()
                .map(|path| path.clone()),
            profile_restricted_url_matchers: profile_restricted_url_matchers,
            app: app,
        }
    }

    fn generate_restricted_hostname_matchers(
        restricted_url_patterns: &Vec<String>,
    ) -> Vec<UrlGlobMatcher> {
        let restricted_hostname_matchers: Vec<UrlGlobMatcher> = restricted_url_patterns
            .iter()
            .map(|url_pattern| {
                let url_matcher = url_rule::to_url_matcher(url_pattern.as_str());
                let glob_matcher = url_matcher.to_glob_matcher();
                glob_matcher
            })
            .collect();

        return restricted_hostname_matchers;
    }

    // used in configuration file to uniquely identify this app+profile+container
    fn get_unique_id(&self) -> String {
        let app_id = self.get_unique_app_id();
        let app_and_profile = app_id + "#" + self.profile_cli_arg_value.as_str();

        if let Some(ref profile_cli_container_name) = self.profile_cli_container_name {
            return app_and_profile + "#" + profile_cli_container_name.as_str();
        }

        return app_and_profile;
    }

    // used in configuration file to uniquely identify this app
    fn get_unique_app_id(&self) -> String {
        let app_executable_path = self.get_browser_common().get_unique_app_id();
        return app_executable_path;
    }

    fn get_browser_common(&self) -> &BrowserCommon {
        return self.app.borrow();
    }

    pub fn has_priority_ordering(&self) -> bool {
        return !self.get_restricted_url_matchers().is_empty();
    }

    fn get_restricted_url_matchers(&self) -> &Vec<UrlGlobMatcher> {
        return if !&self.profile_restricted_url_matchers.is_empty() {
            &self.profile_restricted_url_matchers
        } else {
            self.get_browser_common()
                .supported_app
                .get_restricted_hostname_matchers()
        };
    }

    fn get_browser_name(&self) -> &str {
        return self.get_browser_common().get_display_name();
    }

    fn get_browser_icon_path(&self) -> &str {
        return self.get_browser_common().get_browser_icon_path();
    }

    fn get_profile_icon_path(&self) -> Option<&String> {
        return self.profile_icon.as_ref();
    }

    fn get_profile_name(&self) -> &str {
        return self.profile_name.as_str();
    }

    pub fn open_link(&self, url: &str, incognito_mode: bool) {
        let _ = &self.create_command(url, incognito_mode).spawn();
    }

    fn create_command(&self, url: &str, incognito_mode: bool) -> Command {
        return self.app.create_command(self, url, incognito_mode);
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledBrowser {
    // In Linux:
    //  "env",
    //  "MOZ_ENABLE_WAYLAND=1",
    //  "BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/firefox_firefox.desktop",
    //  "/snap/bin/firefox",
    //  "%u"
    //
    //  "qutebrowser",
    //  "--untrusted-args",
    //  "%u"

    // In Windows and mac:
    // single item with full path of the executable
    command: Vec<String>,

    // unique path of the executable
    // specially useful if multiple versions/locations of bundles exist
    executable_path: String,

    display_name: String,

    // macOS only
    bundle: String,

    user_dir: String,

    icon_path: String,

    profiles: InstalledAppProfiles,

    #[serde(default)]
    restricted_domains: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InstalledAppProfiles {
    pub(crate) profiles_type: InstalledAppProfilesType,
    pub(crate) profiles: Vec<InstalledBrowserProfile>,
}

impl InstalledAppProfiles {
    pub fn new_real(profiles: Vec<InstalledBrowserProfile>) -> InstalledAppProfiles {
        Self {
            profiles_type: InstalledAppProfilesType::RealProfiles,
            profiles,
        }
    }

    pub fn new_placeholder() -> InstalledAppProfiles {
        Self {
            profiles_type: InstalledAppProfilesType::PlaceholderProfiles,
            profiles: Self::find_placeholder_profiles(),
        }
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
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum InstalledAppProfilesType {
    RealProfiles,
    PlaceholderProfiles,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledBrowserProfile {
    profile_cli_arg_value: String,
    profile_cli_container_name: Option<String>,
    profile_name: String,
    profile_icon: Option<String>,
    profile_restricted_url_patterns: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum ProfileIcon {
    NoIcon,
    Remote { url: String },
    Local { path: String },
    Name { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpeningRule {
    source_app: Option<String>,
    url_pattern: Option<String>,
    opener: Option<ProfileAndOptions>,
}

pub struct OpeningRulesAndDefaultProfile {
    opening_rules: Vec<OpeningRule>,
    default_profile: Option<ProfileAndOptions>,
}

impl OpeningRulesAndDefaultProfile {
    #[instrument(skip_all)]
    fn get_rule_for_source_app_and_url(
        &self,
        url_open_context: &UrlOpenContext,
    ) -> Option<ProfileAndOptions> {
        let url_result = Url::from_str(url_open_context.cleaned_url.as_str());
        if url_result.is_err() {
            return None;
        }
        let given_url = url_result.unwrap();

        for r in &self.opening_rules {
            let url_match = Self::url_matches(r, &given_url);
            let source_app_match =
                Self::source_app_matches(r, url_open_context.source_app_maybe.as_ref());

            if url_match && source_app_match {
                return r.opener.clone();
            }
        }

        if self.default_profile.is_some() {
            return self.default_profile.clone();
        }

        return None;
    }

    fn url_matches(r: &OpeningRule, given_url: &Url) -> bool {
        let url_match = if let Some(ref url_pattern) = r.url_pattern {
            let url_matches = url_rule::to_url_matcher(url_pattern.as_str())
                .to_glob_matcher()
                .url_matches(&given_url);

            url_matches
        } else {
            true
        };

        return url_match;
    }

    fn source_app_matches(r: &OpeningRule, actual_source_app: Option<&String>) -> bool {
        let mut source_app_match = false;
        if let Some(ref source_app) = r.source_app {
            let source_app_rule = source_app.clone();
            if let Some(source_app) = actual_source_app {
                let source_app = source_app.clone();
                source_app_match = source_app_rule == source_app;
            }
        } else {
            source_app_match = true;
        }

        return source_app_match;
    }
}

pub struct VisibleAndHiddenProfiles {
    visible_browser_profiles: Vec<CommonBrowserProfile>,
    hidden_browser_profiles: Vec<CommonBrowserProfile>,
}

impl VisibleAndHiddenProfiles {
    pub(crate) fn get_browser_profile_by_id(
        &self,
        unique_id: &str,
    ) -> Option<&CommonBrowserProfile> {
        let visible_profile_maybe = self
            .visible_browser_profiles
            .iter()
            .find(|p| p.get_unique_id() == unique_id);
        if visible_profile_maybe.is_some() {
            return visible_profile_maybe;
        }

        let hidden_profile_maybe = self
            .hidden_browser_profiles
            .iter()
            .find(|p| p.get_unique_id() == unique_id);
        if hidden_profile_maybe.is_some() {
            return hidden_profile_maybe;
        }

        return None;
    }
}

pub fn get_opening_rules(config: &Config) -> OpeningRulesAndDefaultProfile {
    let config_rules = config.get_rules();
    let default_profile = config.get_default_profile();
    let opening_rules = to_opening_rules(config_rules);

    return OpeningRulesAndDefaultProfile {
        opening_rules: opening_rules,
        default_profile: default_profile.clone(),
    };
}

fn to_opening_rules(config_rules: &Vec<ConfigRule>) -> Vec<OpeningRule> {
    return config_rules
        .iter()
        .map(|r| OpeningRule {
            source_app: r.get_source_app(),
            url_pattern: r.get_url_pattern().clone(),
            opener: r.get_opener().clone(),
        })
        .collect();
}

#[instrument(skip_all)]
pub fn generate_all_browser_profiles(
    config: &Config,
    app_finder: &OSAppFinder,
    force_reload: bool,
) -> VisibleAndHiddenProfiles {
    let installed_browsers = app_finder.get_installed_browsers_cached(force_reload);
    let hidden_apps = config.get_hidden_apps();
    let hidden_profiles = config.get_hidden_profiles();

    let mut visible_browser_profiles: Vec<CommonBrowserProfile> = Vec::new();
    let mut hidden_browser_profiles: Vec<CommonBrowserProfile> = Vec::new();
    //let support_dir = macos_get_application_support_dir();
    debug!("Apps");
    for installed_browser in installed_browsers {
        debug!("App: {:?}", installed_browser.bundle);
        debug!("  Path: {:?}", installed_browser.executable_path);
        let app = GenericApp::new(&installed_browser, app_finder.get_app_repository());

        for p in app.get_profiles() {
            let app_id = p.get_unique_app_id();
            if hidden_apps.contains(&app_id) {
                debug!(
                    "Skipping Profile: {:?} because whole app is hidden",
                    p.get_profile_name()
                );
                hidden_browser_profiles.push(p.clone());
                continue;
            }

            let profile_unique_id = p.get_unique_id();

            if hidden_profiles.contains(&profile_unique_id) {
                debug!(
                    "Skipping Profile: {:?} because the specific profile is hidden",
                    p.get_profile_name()
                );
                hidden_browser_profiles.push(p.clone());
                continue;
            }
            debug!("Profile: {:?}", profile_unique_id.as_str());
            visible_browser_profiles.push(p.clone());
        }
    }

    let profile_order = config.get_profile_order();
    sort_browser_profiles(&mut visible_browser_profiles, profile_order);

    return VisibleAndHiddenProfiles {
        visible_browser_profiles: visible_browser_profiles,
        hidden_browser_profiles: hidden_browser_profiles,
    };
}

fn sort_browser_profiles(
    visible_browser_profiles: &mut Vec<CommonBrowserProfile>,
    profile_order: &Vec<String>,
) {
    let unordered_index = profile_order.len();

    visible_browser_profiles.sort_by_key(|p| {
        let profile_unique_id = p.get_unique_id();
        let order_maybe = profile_order.iter().position(|x| x == &profile_unique_id);
        // return the explicit order, or else max order (preserves natural ordering)
        return order_maybe.unwrap_or(unordered_index);
    });

    // always show special apps first
    visible_browser_profiles.sort_by_key(|b| !b.has_priority_ordering());
}

pub fn unwrap_url(url_str: &str, behavioral_settings: &BehavioralConfig) -> String {
    if !behavioral_settings.unwrap_urls {
        return url_str.to_string();
    }

    let url_maybe = Url::from_str(url_str).ok();
    if url_maybe.is_none() {
        return url_str.to_string();
    }
    let url = url_maybe.unwrap();

    let transformed_url = url.domain().and_then(|domain| {
        let domain_lowercase = domain.to_lowercase();

        return if domain_lowercase.ends_with("safelinks.protection.outlook.com") {
            let query_pairs: Parse = url.query_pairs();

            let target_url_maybe: Option<String> = query_pairs
                .into_iter()
                .find(|(key, _)| key == "url")
                .map(|(_, value)| value.to_string());

            target_url_maybe
        } else if domain_lowercase.ends_with("l.messenger.com") {
            let query_pairs: Parse = url.query_pairs();

            let target_url_maybe: Option<String> = query_pairs
                .into_iter()
                .find(|(key, _)| key == "u")
                .map(|(_, value)| value.to_string());

            target_url_maybe
        } else {
            None
        };
    });

    return transformed_url.unwrap_or(url_str.to_string());
}

pub fn handle_messages_to_main(
    main_receiver: Receiver<MessageToMain>,
    ui_event_sink: UiHandle,
    opening_rules_and_default_profile: &mut OpeningRulesAndDefaultProfile,
    visible_and_hidden_profiles: &mut VisibleAndHiddenProfiles,
    app_finder: &OSAppFinder,
    mut behavioral_config: BehavioralConfig,
) {
    for message in main_receiver.iter() {
        match message {
            MessageToMain::Refresh => {
                info!("refresh called");

                let config = app_finder.load_config();

                let visible_and_hidden_profiles =
                    generate_all_browser_profiles(&config, &app_finder, true);

                let ui_browsers =
                    real_to_ui_browsers(&visible_and_hidden_profiles.visible_browser_profiles);
                ui_event_sink.new_browsers_received(ui_browsers);
            }
            MessageToMain::OpenLink(profile_index, incognito_mode, url) => {
                let option = &visible_and_hidden_profiles
                    .visible_browser_profiles
                    .get(profile_index);
                let profile = option.unwrap();
                profile.open_link(url.as_str(), incognito_mode);
                ui_event_sink.open_link_completed();
            }
            MessageToMain::UrlPassedToMain(from_bundle_id, url) => {
                let routing = route_url(
                    Some(from_bundle_id.as_str()),
                    url.as_str(),
                    &behavioral_config,
                    opening_rules_and_default_profile,
                    visible_and_hidden_profiles,
                );
                match routing {
                    UrlRouting::Quit => exit(QUIT_EXIT_CODE),
                    UrlRouting::OpenInProfile {
                        profile,
                        incognito,
                        url,
                    } => {
                        profile.open_link(url.as_str(), incognito);
                        ui_event_sink.open_link_completed();
                    }
                    UrlRouting::ShowPicker { url } => {
                        let url_open_info = UrlOpenInfo {
                            url,
                            source_bundle_id: from_bundle_id,
                        };
                        ui_event_sink.cleaned_url_opened(url_open_info);
                    }
                }
            }
            MessageToMain::SetBrowsersAsDefaultBrowser => {
                utils::set_as_default_web_browser();
            }
            MessageToMain::HideAllProfiles(app_id) => {
                info!("Hiding all profiles of app {}", app_id);

                let to_hide: Vec<String> = visible_and_hidden_profiles
                    .visible_browser_profiles
                    .iter()
                    .filter(|p| p.get_unique_app_id() == app_id)
                    .map(|p| p.get_unique_id())
                    .collect();

                let mut config = app_finder.load_config();
                config.hide_all_profiles(&to_hide);
                app_finder.save_config(&config);

                visible_and_hidden_profiles
                    .visible_browser_profiles
                    .retain(|visible_profile| {
                        let delete = visible_profile.get_unique_app_id() == app_id;
                        if delete {
                            visible_and_hidden_profiles
                                .hidden_browser_profiles
                                .push(visible_profile.clone());
                        }
                        !delete
                    });

                let ui_browsers =
                    real_to_ui_browsers(&visible_and_hidden_profiles.visible_browser_profiles);
                ui_event_sink.new_browsers_received(ui_browsers);

                let ui_hidden_browsers =
                    real_to_ui_browsers(&visible_and_hidden_profiles.hidden_browser_profiles);
                ui_event_sink.new_hidden_browsers_received(ui_hidden_browsers);
            }
            MessageToMain::HideAppProfile(unique_id) => {
                info!("Hiding profile {}", unique_id);

                let mut config = app_finder.load_config();
                config.hide_profile(unique_id.as_str());
                app_finder.save_config(&config);

                let visible_profile_index_maybe = visible_and_hidden_profiles
                    .visible_browser_profiles
                    .iter()
                    .position(|p| p.get_unique_id() == unique_id);
                if let Some(visible_profile_index) = visible_profile_index_maybe {
                    let visible_profile = visible_and_hidden_profiles
                        .visible_browser_profiles
                        .remove(visible_profile_index);
                    visible_and_hidden_profiles
                        .hidden_browser_profiles
                        .push(visible_profile);

                    let ui_browsers =
                        real_to_ui_browsers(&visible_and_hidden_profiles.visible_browser_profiles);
                    ui_event_sink.new_browsers_received(ui_browsers);

                    let ui_hidden_browsers =
                        real_to_ui_browsers(&visible_and_hidden_profiles.hidden_browser_profiles);
                    ui_event_sink.new_hidden_browsers_received(ui_hidden_browsers);
                }
            }
            MessageToMain::RestoreAppProfile(unique_id) => {
                info!("Restoring profile {}", unique_id);
                // will add to the end of visible profiles

                let mut config = app_finder.load_config();
                config.restore_profile(unique_id.as_str());
                app_finder.save_config(&config);

                let profile_order = config.get_profile_order();

                let hidden_profile_index_maybe = visible_and_hidden_profiles
                    .hidden_browser_profiles
                    .iter()
                    .position(|p| p.get_unique_id() == unique_id);
                if let Some(hidden_profile_index) = hidden_profile_index_maybe {
                    let hidden_profile = visible_and_hidden_profiles
                        .hidden_browser_profiles
                        .remove(hidden_profile_index);
                    visible_and_hidden_profiles
                        .visible_browser_profiles
                        .push(hidden_profile);

                    sort_browser_profiles(
                        &mut visible_and_hidden_profiles.visible_browser_profiles,
                        profile_order,
                    );

                    let ui_browsers =
                        real_to_ui_browsers(&visible_and_hidden_profiles.visible_browser_profiles);
                    ui_event_sink.new_browsers_received(ui_browsers);

                    let ui_hidden_browsers =
                        real_to_ui_browsers(&visible_and_hidden_profiles.hidden_browser_profiles);
                    ui_event_sink.new_hidden_browsers_received(ui_hidden_browsers);
                }
            }
            MessageToMain::MoveAppProfile(unique_id, move_to) => move_app_profile(
                &app_finder,
                &mut visible_and_hidden_profiles.visible_browser_profiles,
                unique_id,
                move_to,
                &ui_event_sink,
            ),
            MessageToMain::SaveConfigRules(ui_rules) => {
                info!("Saving rules");

                let mut config = app_finder.load_config();
                let new_rules: Vec<ConfigRule> = ui_rules
                    .iter()
                    .map(|ui_rule| ConfigRule {
                        source_app: ui_rule.get_source_app(),
                        url_pattern: ui_rule.get_url_pattern(),
                        opener: map_as_profile_and_options(&ui_rule.opener),
                    })
                    .collect();

                config.set_rules(&new_rules);
                app_finder.save_config(&config);

                // refresh opening rules immediately
                // so that if same Browsers instance stays open,
                // it will already work with the new rule without restarting Browsers
                opening_rules_and_default_profile.opening_rules = to_opening_rules(&new_rules);
            }
            MessageToMain::SaveConfigDefaultOpener(default_opener) => {
                info!("Saving default opener");
                let new_default_profile = default_opener.map(|p| ProfileAndOptions {
                    profile: p.profile,
                    incognito: p.incognito,
                });

                let mut config = app_finder.load_config();
                config.set_default_profile(&new_default_profile);
                app_finder.save_config(&config);

                // refresh default opener immediately
                // so that if same Browsers instance stays open,
                // it will already work with the new rule without restarting Browsers
                opening_rules_and_default_profile.default_profile = new_default_profile.clone();
            }
            MessageToMain::SaveConfigUISettings(settings) => {
                info!("Saving UI settings");
                let ui_config = UIConfig {
                    show_hotkeys: settings.show_hotkeys,
                    quit_on_lost_focus: settings.quit_on_lost_focus,
                    theme: settings.theme,
                };

                let mut config = app_finder.load_config();
                config.set_ui_config(ui_config);
                app_finder.save_config(&config);
            }
            MessageToMain::SaveConfigUIBehavioralSettings(settings) => {
                info!("Saving Behavioral settings");
                behavioral_config = BehavioralConfig {
                    unwrap_urls: settings.unwrap_urls,
                };

                let mut config = app_finder.load_config();
                config.set_behavior(behavioral_config.clone());
                app_finder.save_config(&config);
            }
        }
    }

    info!("Exiting waiting thread");
}

pub struct PreparedUi {
    pub url: String,
    pub browsers: Vec<UIBrowser>,
    pub hidden_browsers: Vec<UIBrowser>,
    pub show_set_as_default: bool,
    pub ui_settings: crate::gui::app_state::UISettings,
}

#[instrument(skip_all)]
pub fn prepare_ui(
    url_open_context: &UrlOpenContext,
    visible_and_hidden_profiles: &VisibleAndHiddenProfiles,
    config: &Config,
    show_set_as_default: bool,
) -> PreparedUi {
    PreparedUi {
        url: url_open_context.cleaned_url.clone(),
        browsers: real_to_ui_browsers(
            visible_and_hidden_profiles
                .visible_browser_profiles
                .as_slice(),
        ),
        hidden_browsers: real_to_ui_browsers(
            visible_and_hidden_profiles
                .hidden_browser_profiles
                .as_slice(),
        ),
        show_set_as_default,
        ui_settings: crate::gui::app_state::config_to_ui_settings(config),
    }
}

pub fn print_visible_options(prepared_ui: &PreparedUi) {
    println!("BROWSERS");
    println!();

    for ui_browser in &prepared_ui.browsers {
        println!("{}", ui_browser.get_full_name())
    }
}

// what to do with a URL that was handed to Browsers
pub enum UrlRouting<'a> {
    // Safari hard-launch workaround, there is nothing to open
    Quit,
    // an opening rule picked the profile, so the picker is not needed
    OpenInProfile {
        profile: &'a CommonBrowserProfile,
        incognito: bool,
        url: String,
    },
    ShowPicker {
        url: String,
    },
}

// The one place that decides what happens with an incoming URL (unwrapping, the Safari
// workaround and opening rules), so all entry points behave the same way.
// source_app is None when the URL didn't come from another app (e.g. command line argument).
pub fn route_url<'a>(
    source_app: Option<&str>,
    url: &str,
    behavioral_config: &BehavioralConfig,
    opening_rules_and_default_profile: &OpeningRulesAndDefaultProfile,
    visible_and_hidden_profiles: &'a VisibleAndHiddenProfiles,
) -> UrlRouting<'a> {
    debug!("source_bundle_id: {:?}, url: {}", source_app, url);

    if source_app == Some("com.apple.Safari") {
        // workaround for weird bug where Safari opens default browser on hard launch
        // see https://github.com/Browsers-software/browsers/issues/79
        // We might need to remove this workaround if we want to allow Safari
        // to open Browsers via some extension
        info!("Safari has a weird bug and launched Browsers. Exiting Browsers.");
        return UrlRouting::Quit;
    }

    let cleaned_url = unwrap_url(url, behavioral_config);
    let url_open_context = UrlOpenContext {
        cleaned_url: cleaned_url.clone(),
        source_app_maybe: source_app.map(|s| s.to_string()),
    };

    let profile_and_options_maybe =
        opening_rules_and_default_profile.get_rule_for_source_app_and_url(&url_open_context);
    if let Some(profile_and_options) = profile_and_options_maybe {
        let profile_maybe = visible_and_hidden_profiles
            .get_browser_profile_by_id(profile_and_options.profile.as_str());
        if let Some(profile) = profile_maybe {
            return UrlRouting::OpenInProfile {
                profile,
                incognito: profile_and_options.incognito,
                url: cleaned_url,
            };
        }
    }

    UrlRouting::ShowPicker { url: cleaned_url }
}

pub struct UrlOpenContext {
    pub cleaned_url: String,
    pub source_app_maybe: Option<String>,
}

fn map_as_profile_and_options(opener: &Option<UIProfileAndIncognito>) -> Option<ProfileAndOptions> {
    return opener.as_ref().map(|p| ProfileAndOptions {
        profile: p.profile.clone(),
        incognito: p.incognito,
    });
}

fn move_app_profile(
    app_finder: &OSAppFinder,
    visible_browser_profiles: &mut Vec<CommonBrowserProfile>,
    unique_id: String,
    move_to: MoveTo,
    ui_event_sink: &UiHandle,
) {
    let visible_profile_index_maybe = visible_browser_profiles
        .iter()
        .position(|p| p.get_unique_id() == unique_id);

    if visible_profile_index_maybe.is_none() {
        warn!("Could not find visible profile for id {}", unique_id);
        return;
    }
    let visible_profile_index = visible_profile_index_maybe.unwrap();

    // TODO: this is a bit ugly; we keep profiles with has_priority_ordering() always on top
    //       and everything else comes after; it might make sense to keep them in two separate
    //       vectors (or slices)
    let first_orderable_item_index_maybe = visible_browser_profiles
        .iter()
        .position(|b| !b.has_priority_ordering());

    let first_orderable_item_index = match first_orderable_item_index_maybe {
        Some(first_orderable_item_index) => first_orderable_item_index,
        None => {
            warn!("Could not find orderable profiles");
            return;
        }
    };

    // translate every direction (including a drag-and-drop's absolute target) into a single
    // bounded index, clamped to the orderable range - so a multi-position drag becomes one
    // rotation instead of N single-step moves, each round-tripping through the config file
    let last_index = visible_browser_profiles.len() - 1;
    let target_index = match move_to {
        MoveTo::TOP => first_orderable_item_index,
        MoveTo::UP => visible_profile_index
            .saturating_sub(1)
            .max(first_orderable_item_index),
        MoveTo::DOWN => (visible_profile_index + 1).min(last_index),
        MoveTo::BOTTOM => last_index,
        MoveTo::ToIndex(index) => index.clamp(first_orderable_item_index, last_index),
    };

    if target_index == visible_profile_index {
        info!(
            "Not moving profile {} - already at the target position",
            unique_id
        );
        return;
    }
    info!("Moving profile {} to index {}", unique_id, target_index);

    // 1. update visible_browser_profiles
    if target_index < visible_profile_index {
        visible_browser_profiles[target_index..visible_profile_index + 1].rotate_right(1);
    } else {
        visible_browser_profiles[visible_profile_index..target_index + 1].rotate_left(1);
    }

    // 2. send visible_browser_profiles to gui
    let ui_browsers = real_to_ui_browsers(&visible_browser_profiles);
    ui_event_sink.new_browsers_received(ui_browsers);

    // 3. update config file
    let profile_ids_sorted: Vec<String> = visible_browser_profiles
        .iter()
        .filter(|b| !b.has_priority_ordering())
        .map(|p| p.get_unique_id())
        .collect();

    let mut config = app_finder.load_config();
    config.set_profile_order(&profile_ids_sorted);
    app_finder.save_config(&config);
}

#[derive(Clone, Copy, Debug)]
pub enum MoveTo {
    UP,
    DOWN,
    TOP,
    BOTTOM,
    // absolute target position within the orderable range, used for a single drag-and-drop move
    ToIndex(usize),
}

#[derive(Debug)]
pub enum MessageToMain {
    Refresh,
    OpenLink(usize, bool, String),
    // (source bundle id, url) - the URL as received, unwrapping and routing is done by main
    UrlPassedToMain(String, String),
    SetBrowsersAsDefaultBrowser,
    HideAppProfile(String),
    HideAllProfiles(String),
    RestoreAppProfile(String),
    MoveAppProfile(String, MoveTo),
    SaveConfigRules(Vec<UISettingsRule>),
    SaveConfigDefaultOpener(Option<UIProfileAndIncognito>),
    SaveConfigUISettings(UIVisualSettings),
    SaveConfigUIBehavioralSettings(UIBehavioralSettings),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(executable_path: &str, cli_arg: &str) -> CommonBrowserProfile {
        let supported_app =
            SupportedAppRepository::new().get_or_generate("test.browser", &Vec::new());
        let app = BrowserCommon {
            command: vec![executable_path.to_string()],
            executable_path: executable_path.to_string(),
            display_name: "Test Browser".to_string(),
            icon_path: "".to_string(),
            supported_app,
            profiles_type: InstalledAppProfilesType::RealProfiles,
        };
        let installed_profile = InstalledBrowserProfile {
            profile_cli_arg_value: cli_arg.to_string(),
            profile_cli_container_name: None,
            profile_name: cli_arg.to_string(),
            profile_icon: None,
            profile_restricted_url_patterns: Vec::new(),
        };
        CommonBrowserProfile::new(&installed_profile, Arc::new(app))
    }

    fn profiles() -> VisibleAndHiddenProfiles {
        VisibleAndHiddenProfiles {
            visible_browser_profiles: vec![profile("/bin/test", "Default")],
            hidden_browser_profiles: Vec::new(),
        }
    }

    fn rule(source_app: Option<&str>, url_pattern: Option<&str>, profile: &str) -> OpeningRule {
        OpeningRule {
            source_app: source_app.map(|s| s.to_string()),
            url_pattern: url_pattern.map(|s| s.to_string()),
            opener: Some(ProfileAndOptions {
                profile: profile.to_string(),
                incognito: true,
            }),
        }
    }

    fn rules(opening_rules: Vec<OpeningRule>) -> OpeningRulesAndDefaultProfile {
        OpeningRulesAndDefaultProfile {
            opening_rules,
            default_profile: None,
        }
    }

    const PROFILE_ID: &str = "/bin/test#Default";

    fn no_unwrap() -> BehavioralConfig {
        BehavioralConfig { unwrap_urls: false }
    }

    #[test]
    fn matching_url_rule_opens_in_profile() {
        let rules = rules(vec![rule(None, Some("example.com"), PROFILE_ID)]);
        let profiles = profiles();

        let routing = route_url(None, "https://example.com/a", &no_unwrap(), &rules, &profiles);

        match routing {
            UrlRouting::OpenInProfile {
                profile,
                incognito,
                url,
            } => {
                assert_eq!(profile.get_unique_id(), PROFILE_ID);
                assert!(incognito);
                assert_eq!(url, "https://example.com/a");
            }
            _ => panic!("expected OpenInProfile"),
        }
    }

    #[test]
    fn source_app_rule_only_matches_that_app() {
        let rules = rules(vec![rule(Some("com.example.Mail"), None, PROFILE_ID)]);
        let profiles = profiles();
        let url = "https://example.com/";

        let from_mail = route_url(Some("com.example.Mail"), url, &no_unwrap(), &rules, &profiles);
        assert!(matches!(from_mail, UrlRouting::OpenInProfile { .. }));

        let from_other = route_url(Some("com.example.Other"), url, &no_unwrap(), &rules, &profiles);
        assert!(matches!(from_other, UrlRouting::ShowPicker { .. }));

        let no_source = route_url(None, url, &no_unwrap(), &rules, &profiles);
        assert!(matches!(no_source, UrlRouting::ShowPicker { .. }));
    }

    #[test]
    fn no_matching_rule_shows_picker() {
        let rules = rules(vec![rule(None, Some("example.com"), PROFILE_ID)]);
        let profiles = profiles();

        let routing = route_url(None, "https://other.org/", &no_unwrap(), &rules, &profiles);

        match routing {
            UrlRouting::ShowPicker { url } => assert_eq!(url, "https://other.org/"),
            _ => panic!("expected ShowPicker"),
        }
    }

    #[test]
    fn rule_for_missing_profile_shows_picker() {
        let rules = rules(vec![rule(None, Some("example.com"), "/bin/gone#Default")]);
        let profiles = profiles();

        let routing = route_url(None, "https://example.com/", &no_unwrap(), &rules, &profiles);

        assert!(matches!(routing, UrlRouting::ShowPicker { .. }));
    }

    #[test]
    fn safari_hard_launch_quits() {
        let rules = rules(vec![rule(None, None, PROFILE_ID)]);
        let profiles = profiles();

        let routing = route_url(
            Some("com.apple.Safari"),
            "https://example.com/",
            &no_unwrap(),
            &rules,
            &profiles,
        );

        assert!(matches!(routing, UrlRouting::Quit));
    }

    #[test]
    fn unwrapped_url_is_what_rules_match_and_picker_shows() {
        let wrapped =
            "https://eur01.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com%2Fa";
        let rules = rules(vec![rule(None, Some("example.com"), PROFILE_ID)]);
        let profiles = profiles();
        let unwrap = BehavioralConfig { unwrap_urls: true };

        let unwrapped = route_url(None, wrapped, &unwrap, &rules, &profiles);
        match unwrapped {
            UrlRouting::OpenInProfile { url, .. } => assert_eq!(url, "https://example.com/a"),
            _ => panic!("expected OpenInProfile"),
        }

        let not_unwrapped = route_url(None, wrapped, &no_unwrap(), &rules, &profiles);
        match not_unwrapped {
            UrlRouting::ShowPicker { url } => assert_eq!(url, wrapped),
            _ => panic!("expected ShowPicker"),
        }
    }
}
