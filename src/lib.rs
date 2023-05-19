use std::borrow::Borrow;
use std::fmt::Debug;
use std::process::Command;
use std::str::FromStr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{env, thread};

use druid::{ExtEventSink, Target, UrlOpenInfo};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use url::Url;

use ui::UI;

use crate::browser_repository::{SupportedApp, SupportedAppRepository};
use crate::ui::MoveTo;
use crate::utils::{OSAppFinder, ProfileAndOptions};

mod ui;

pub mod paths;
pub mod utils;

mod browser_repository;

#[cfg(target_os = "macos")]
mod macos_utils;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux_utils;

#[cfg(target_os = "windows")]
mod windows_utils;

pub mod communicate;

mod chromium_profiles_parser;
mod firefox_profiles_parser;
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
            executable_path: installed_browser.executable_path.to_string(),
            display_name: installed_browser.display_name.to_string(),
            icon_path: installed_browser.icon_path.to_string(),
        };

        let arc = Arc::new(app.clone());
        let mut profiles: Vec<CommonBrowserProfile> = Vec::new();
        for installed_profile in &installed_browser.profiles {
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
    executable_path: String,
    display_name: String,
    icon_path: String,
    supported_app: SupportedApp,
}

impl BrowserCommon {
    fn supports_profiles(&self) -> bool {
        return self.supported_app.supports_profiles();
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
        profile_cli_arg_value: &str,
        profile_cli_container_name: Option<&String>,
        url: &str,
        incognito_mode: bool,
    ) -> Command {
        let profile_args = self.supported_app.get_profile_args(profile_cli_arg_value);
        let app_url = self
            .supported_app
            .get_transformed_url(profile_cli_container_name, url);

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
            if !profile_args.is_empty() {
                arguments.args(profile_args);
            }
            if self.supported_app.is_url_as_first_arg() {
                arguments.arg(app_url.clone());
            }

            if incognito_mode && self.supported_app.supports_incognito() {
                let incognito_args = self.supported_app.get_incognito_args();
                arguments.args(incognito_args);
            }

            debug!("Launching: {:?}", cmd);
            return cmd;
        } else if cfg!(target_os = "linux") {
            let mut cmd = Command::new(self.executable_path.to_string());
            cmd.args(profile_args).arg(app_url);

            return cmd;
        } else if cfg!(target_os = "windows") {
            let mut cmd = Command::new(self.executable_path.to_string());
            cmd.args(profile_args).arg(app_url);
            return cmd;
        }

        unimplemented!("platform is not supported yet");
    }
}

#[derive(Clone)]
pub struct CommonBrowserProfile {
    profile_cli_arg_value: String,
    profile_cli_container_name: Option<String>,
    profile_name: String,
    profile_icon: Option<String>,
    app: Arc<BrowserCommon>,
}

impl CommonBrowserProfile {
    fn new(installed_browser_profile: &InstalledBrowserProfile, app: Arc<BrowserCommon>) -> Self {
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
            app: app,
        }
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
        let app_executable_path = (&self).get_browser_common().executable_path.to_string();
        return app_executable_path;
    }

    fn get_browser_common(&self) -> &BrowserCommon {
        return self.app.borrow();
    }

    pub fn has_priority_ordering(&self) -> bool {
        return !self.get_restricted_domains().is_empty();
    }

    fn get_restricted_domains(&self) -> &Vec<String> {
        return self
            .get_browser_common()
            .supported_app
            .get_restricted_domains();
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

    fn open_link(&self, url: &str, incognito_mode: bool) {
        let _ = &self.create_command(url, incognito_mode).spawn();
    }

    fn create_command(&self, url: &str, incognito_mode: bool) -> Command {
        return self.app.create_command(
            &self.profile_cli_arg_value,
            self.profile_cli_container_name.as_ref(),
            url,
            incognito_mode,
        );
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledBrowser {
    // unique path of the executable
    // specially useful if multiple versions/locations of bundles exist
    executable_path: String,

    display_name: String,

    // macOS only
    bundle: String,

    user_dir: String,

    icon_path: String,

    profiles: Vec<InstalledBrowserProfile>,

    #[serde(default)]
    restricted_domains: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledBrowserProfile {
    profile_cli_arg_value: String,
    profile_cli_container_name: Option<String>,
    profile_name: String,
    profile_icon: Option<String>,
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
    profile: String,
}

fn generate_all_browser_profiles(
    app_finder: &OSAppFinder,
    force_reload: bool,
) -> (
    Vec<OpeningRule>,
    Option<ProfileAndOptions>,
    Vec<CommonBrowserProfile>,
    Vec<CommonBrowserProfile>,
) {
    let installed_browsers = app_finder.get_installed_browsers_cached(force_reload);
    let config = app_finder.get_installed_browsers_config();
    let hidden_apps = config.get_hidden_apps();
    let hidden_profiles = config.get_hidden_profiles();

    let config_rules = config.get_rules();
    let default_profile = config.get_default_profile();
    let opening_rules = config_rules
        .iter()
        .map(|r| OpeningRule {
            source_app: r.source_app.clone(),
            url_pattern: r.url_pattern.clone(),
            profile: r.profile.clone(),
        })
        .collect();

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
    let unordered_index = profile_order.len();

    visible_browser_profiles.sort_by_key(|p| {
        let profile_unique_id = p.get_unique_id();
        let order_maybe = profile_order.iter().position(|x| x == &profile_unique_id);
        // return the explicit order, or else max order (preserves natural ordering)
        return order_maybe.unwrap_or(unordered_index);
    });

    // always show special apps first
    visible_browser_profiles.sort_by_key(|b| !b.has_priority_ordering());

    return (
        opening_rules,
        default_profile.clone(),
        visible_browser_profiles,
        hidden_browser_profiles,
    );
}

fn get_rule_for_source_app_and_url(
    opening_rules: &Vec<OpeningRule>,
    default_profile_maybe: Option<ProfileAndOptions>,
    url: &str,
    source_app_maybe: Option<String>,
) -> Option<ProfileAndOptions> {
    let url_result = Url::from_str(url);
    if url_result.is_err() {
        return None;
    }
    let given_url = url_result.unwrap();

    for r in opening_rules {
        let mut source_app_match = false;
        let url_match = if let Some(ref url_pattern) = r.url_pattern {
            let url_matches = url_rule::to_url_matcher(url_pattern.as_str())
                .to_glob_matcher()
                .url_matches(&given_url);

            url_matches
        } else {
            true
        };

        if let Some(ref source_app) = r.source_app {
            let source_app_rule = source_app.clone();
            if let Some(ref source_app) = source_app_maybe {
                let source_app = source_app.clone();
                source_app_match = source_app_rule == source_app;
            }
        } else {
            source_app_match = true;
        }

        if url_match && source_app_match {
            let profile_and_options = ProfileAndOptions {
                profile: r.profile.clone(),
                incognito: false,
            };
            return Some(profile_and_options);
        }
    }

    if default_profile_maybe.is_some() {
        return default_profile_maybe;
    }

    return None;
}

fn get_browser_profile_by_id<'a>(
    visible_profiles: &'a [CommonBrowserProfile],
    hidden_profiles: &'a [CommonBrowserProfile],
    unique_id: &str,
) -> Option<&'a CommonBrowserProfile> {
    let visible_profile_maybe = visible_profiles
        .iter()
        .find(|p| p.get_unique_id() == unique_id);
    if visible_profile_maybe.is_some() {
        return visible_profile_maybe;
    }

    let hidden_profile_maybe = hidden_profiles
        .iter()
        .find(|p| p.get_unique_id() == unique_id);
    if hidden_profile_maybe.is_some() {
        return hidden_profile_maybe;
    }

    return None;
}

pub fn basically_main(
    url: &str,
    show_gui: bool,
    force_reload: bool,
    main_sender: Sender<MessageToMain>,
    main_receiver: Receiver<MessageToMain>,
) {
    let app_finder = OSAppFinder::new();

    let is_default = utils::set_as_default_web_browser();
    let show_set_as_default = !is_default;

    let (opening_rules, default_profile, mut visible_browser_profiles, mut hidden_browser_profiles) =
        generate_all_browser_profiles(&app_finder, force_reload);

    // TODO: url should not be considered here in case of macos
    //       and only the one in LinkOpenedFromBundle should be considered
    let opening_profile_maybe =
        get_rule_for_source_app_and_url(&opening_rules, default_profile.clone(), url, None);
    if let Some(opening_profile_id) = opening_profile_maybe {
        let profile_and_options = opening_profile_id.clone();
        let profile_id = profile_and_options.profile;
        let incognito = profile_and_options.incognito;

        let profile_maybe = get_browser_profile_by_id(
            visible_browser_profiles.as_slice(),
            hidden_browser_profiles.as_slice(),
            profile_id.as_str(),
        );
        if let Some(profile) = profile_maybe {
            profile.open_link(url, incognito);
            return;
        }
    }

    let localizations_basedir = paths::get_localizations_basedir();

    let ui2 = UI::new(
        localizations_basedir,
        main_sender.clone(),
        url,
        UI::real_to_ui_browsers(visible_browser_profiles.as_slice()),
        UI::real_to_ui_browsers(hidden_browser_profiles.as_slice()),
        show_set_as_default,
    );
    let initial_ui_state = ui2.create_initial_ui_state();
    let launcher = ui2.create_app_launcher();
    let ui_event_sink = launcher.get_external_handle();

    thread::spawn(move || {
        for message in main_receiver.iter() {
            match message {
                MessageToMain::Refresh => {
                    info!("refresh called");
                    let (_, _, visible_browser_profiles, _) =
                        generate_all_browser_profiles(&app_finder, true);

                    let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
                    ui_event_sink
                        .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
                        .ok();
                }
                MessageToMain::OpenLink(profile_index, incognito_mode, url) => {
                    let option = &visible_browser_profiles.get(profile_index);
                    let profile = option.unwrap();
                    profile.open_link(url.as_str(), incognito_mode);
                    ui_event_sink
                        .submit_command(
                            ui::OPEN_LINK_IN_BROWSER_COMPLETED,
                            "meh2".to_string(),
                            Target::Global,
                        )
                        .ok();
                }
                MessageToMain::UrlOpenRequest(from_bundle_id, url) => {
                    let url_open_info = UrlOpenInfo {
                        url: url,
                        source_bundle_id: from_bundle_id,
                    };
                    ui_event_sink
                        .submit_command(ui::URL_OPENED, url_open_info, Target::Global)
                        .ok();
                }
                MessageToMain::LinkOpenedFromBundle(from_bundle_id, url) => {
                    // TODO: do something once we have rules to
                    //       prioritize/default browsers based on source app and/or url
                    debug!("source_bundle_id: {}", from_bundle_id.clone());
                    debug!("url: {}", url);
                    let opening_profile_id_maybe = get_rule_for_source_app_and_url(
                        &opening_rules,
                        default_profile.clone(),
                        url.as_str(),
                        Some(from_bundle_id.clone()),
                    );
                    if let Some(opening_profile_id) = opening_profile_id_maybe {
                        let profile_and_options = opening_profile_id.clone();
                        let profile_id = profile_and_options.profile;
                        let incognito = profile_and_options.incognito;

                        let profile_maybe = get_browser_profile_by_id(
                            visible_browser_profiles.as_slice(),
                            hidden_browser_profiles.as_slice(),
                            profile_id.as_str(),
                        );
                        if let Some(profile) = profile_maybe {
                            profile.open_link(url.as_str(), incognito);
                            ui_event_sink
                                .submit_command(
                                    ui::OPEN_LINK_IN_BROWSER_COMPLETED,
                                    "meh2".to_string(),
                                    Target::Global,
                                )
                                .ok();
                        }
                    }
                }
                MessageToMain::SetBrowsersAsDefaultBrowser => {
                    utils::set_as_default_web_browser();
                }
                MessageToMain::HideAllProfiles(app_id) => {
                    info!("Hiding all profiles of app {}", app_id);

                    let to_hide: Vec<String> = visible_browser_profiles
                        .iter()
                        .filter(|p| p.get_unique_app_id() == app_id)
                        .map(|p| p.get_unique_id())
                        .collect();

                    let mut config = app_finder.get_installed_browsers_config();
                    config.hide_all_profiles(&to_hide);
                    app_finder.save_installed_browsers_config(&config);

                    visible_browser_profiles.retain(|visible_profile| {
                        let delete = visible_profile.get_unique_app_id() == app_id;
                        if delete {
                            hidden_browser_profiles.push(visible_profile.clone());
                        }
                        !delete
                    });

                    let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
                    ui_event_sink
                        .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
                        .ok();

                    let ui_hidden_browsers = UI::real_to_ui_browsers(&hidden_browser_profiles);
                    ui_event_sink
                        .submit_command(
                            ui::NEW_HIDDEN_BROWSERS_RECEIVED,
                            ui_hidden_browsers,
                            Target::Global,
                        )
                        .ok();
                }
                MessageToMain::HideAppProfile(unique_id) => {
                    info!("Hiding profile {}", unique_id);

                    let mut config = app_finder.get_installed_browsers_config();
                    config.hide_profile(unique_id.as_str());
                    app_finder.save_installed_browsers_config(&config);

                    let visible_profile_index_maybe = visible_browser_profiles
                        .iter()
                        .position(|p| p.get_unique_id() == unique_id);
                    if let Some(visible_profile_index) = visible_profile_index_maybe {
                        let visible_profile =
                            visible_browser_profiles.remove(visible_profile_index);
                        hidden_browser_profiles.push(visible_profile);

                        let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
                        ui_event_sink
                            .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
                            .ok();

                        let ui_hidden_browsers = UI::real_to_ui_browsers(&hidden_browser_profiles);
                        ui_event_sink
                            .submit_command(
                                ui::NEW_HIDDEN_BROWSERS_RECEIVED,
                                ui_hidden_browsers,
                                Target::Global,
                            )
                            .ok();
                    }
                }
                MessageToMain::RestoreAppProfile(unique_id) => {
                    info!("Restoring profile {}", unique_id);
                    // will add to the end of visible profiles

                    let mut config = app_finder.get_installed_browsers_config();
                    config.restore_profile(unique_id.as_str());
                    app_finder.save_installed_browsers_config(&config);

                    let hidden_profile_index_maybe = hidden_browser_profiles
                        .iter()
                        .position(|p| p.get_unique_id() == unique_id);
                    if let Some(hidden_profile_index) = hidden_profile_index_maybe {
                        let hidden_profile = hidden_browser_profiles.remove(hidden_profile_index);
                        visible_browser_profiles.push(hidden_profile);

                        // always show special apps first
                        visible_browser_profiles.sort_by_key(|b| !b.has_priority_ordering());

                        let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
                        ui_event_sink
                            .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
                            .ok();

                        let ui_hidden_browsers = UI::real_to_ui_browsers(&hidden_browser_profiles);
                        ui_event_sink
                            .submit_command(
                                ui::NEW_HIDDEN_BROWSERS_RECEIVED,
                                ui_hidden_browsers,
                                Target::Global,
                            )
                            .ok();
                    }
                }
                MessageToMain::MoveAppProfile(unique_id, move_to) => move_app_profile(
                    &app_finder,
                    &mut visible_browser_profiles,
                    unique_id,
                    move_to,
                    &ui_event_sink,
                ),
            }
        }
        info!("Exiting waiting thread");
    });

    if show_gui {
        launcher.launch(initial_ui_state).expect("error");
    }
}

fn move_app_profile(
    app_finder: &OSAppFinder,
    visible_browser_profiles: &mut Vec<CommonBrowserProfile>,
    unique_id: String,
    move_to: MoveTo,
    ui_event_sink: &ExtEventSink,
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

    match move_to {
        MoveTo::UP | MoveTo::TOP => {
            if visible_profile_index <= first_orderable_item_index {
                info!("Not moving profile {} higher as it's already first", unique_id);
                return;
            }
            info!("Moving profile {} higher", unique_id);
        }
        MoveTo::DOWN | MoveTo::BOTTOM => {
            if visible_profile_index == visible_browser_profiles.len() - 1 {
                info!("Not moving profile {} lower as it's already last", unique_id);
                return;
            }
            info!("Moving profile {} lower", unique_id);
        }
    }

    // 1. update visible_browser_profiles
    match move_to {
        MoveTo::UP => {
            visible_browser_profiles[visible_profile_index - 1..visible_profile_index + 1]
                .rotate_left(1);
        }
        MoveTo::DOWN => {
            visible_browser_profiles[visible_profile_index..visible_profile_index + 2]
                .rotate_right(1);
        }
        MoveTo::TOP => {
            visible_browser_profiles[first_orderable_item_index..visible_profile_index + 1]
                .rotate_right(1);
        }
        MoveTo::BOTTOM => {
            visible_browser_profiles[visible_profile_index..].rotate_left(1);
        }
    }

    // 2. send visible_browser_profiles to ui
    let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
    ui_event_sink
        .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
        .ok();

    // 3. update config file
    let profile_ids_sorted: Vec<String> = visible_browser_profiles
        .iter()
        .filter(|b| b.get_restricted_domains().is_empty())
        .map(|p| p.get_unique_id())
        .collect();

    let mut config = app_finder.get_installed_browsers_config();
    config.set_profile_order(&profile_ids_sorted);
    app_finder.save_installed_browsers_config(&config);
}

#[derive(Debug)]
pub enum MessageToMain {
    Refresh,
    OpenLink(usize, bool, String),
    UrlOpenRequest(String, String), // almost as LinkOpenedFromBundle, but triggers ui, not from ui
    LinkOpenedFromBundle(String, String),
    SetBrowsersAsDefaultBrowser,
    HideAppProfile(String),
    HideAllProfiles(String),
    RestoreAppProfile(String),
    MoveAppProfile(String, MoveTo),
}
