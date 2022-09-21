use std::borrow::Borrow;
use std::fmt::Debug;
use std::process::Command;
use std::sync::{mpsc, Arc};
use std::{env, thread};

use druid::Target;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use ui::UI;

use crate::browser_repository::{SupportedApp, SupportedAppRepository};
use crate::utils::OSAppFinder;

mod ui;

pub mod utils;

mod browser_repository;

#[cfg(target_os = "macos")]
mod macos_utils;

#[cfg(target_os = "linux")]
mod linux_utils;

mod chromium_profiles_parser;
mod firefox_profiles_parser;

// a browser (with profiles), or Spotify, Zoom, etc
pub struct GenericApp {
    app: BrowserCommon,
    profiles: Vec<CommonBrowserProfile>,
}

impl GenericApp {
    fn new(installed_browser: &InstalledBrowser, app_repository: &SupportedAppRepository) -> Self {
        let supported_app = app_repository.get_or_generate(installed_browser.bundle.as_str());
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
        url: &str,
        incognito_mode: bool,
    ) -> Command {
        let profile_args = self.supported_app.get_profile_args(profile_cli_arg_value);
        let app_url = self.supported_app.get_transformed_url(url);

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

            info!("Launching: {:?}", cmd);
            return cmd;
        } else if cfg!(target_os = "linux") {
            let mut cmd = Command::new(self.executable_path.to_string());
            cmd.args(profile_args).arg(app_url);

            return cmd;
        } else if cfg!(target_os = "windows") {
            unimplemented!("windows is not supported yet");
        }

        unimplemented!("platform is not supported yet");
    }
}

#[derive(Clone)]
pub struct CommonBrowserProfile {
    profile_cli_arg_value: String,
    profile_name: String,
    profile_icon: Option<String>,
    app: Arc<BrowserCommon>,
}

impl CommonBrowserProfile {
    fn new(installed_browser_profile: &InstalledBrowserProfile, app: Arc<BrowserCommon>) -> Self {
        CommonBrowserProfile {
            profile_cli_arg_value: installed_browser_profile.profile_cli_arg_value.to_string(),
            profile_name: installed_browser_profile.profile_name.to_string(),
            profile_icon: installed_browser_profile.profile_icon.clone(),
            app: app,
        }
    }

    // used in configuration file to uniquely identify this app+profile
    fn get_unique_id(&self) -> String {
        let app_id = self.get_unique_app_id();
        return app_id + "#" + self.profile_cli_arg_value.as_str();
    }

    // used in configuration file to uniquely identify this app
    fn get_unique_app_id(&self) -> String {
        let app_executable_path = (&self).get_browser_common().executable_path.to_string();
        return app_executable_path;
    }

    fn get_browser_common(&self) -> &BrowserCommon {
        return self.app.borrow();
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

    fn get_profile_name(&self) -> &str {
        return self.profile_name.as_str();
    }

    fn open_link(&self, url: &str, incognito_mode: bool) {
        let _ = &self.create_command(url, incognito_mode).spawn();
    }

    fn create_command(&self, url: &str, incognito_mode: bool) -> Command {
        return self
            .app
            .create_command(&self.profile_cli_arg_value, url, incognito_mode);
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstalledBrowserProfile {
    profile_cli_arg_value: String,
    profile_name: String,
    profile_icon: Option<String>,
}

fn generate_all_browser_profiles(
    app_finder: &OSAppFinder,
    force_reload: bool,
) -> (Vec<CommonBrowserProfile>, Vec<CommonBrowserProfile>) {
    let installed_browsers = app_finder.get_installed_browsers_cached(force_reload);
    let config = app_finder.get_installed_browsers_config();
    let hidden_apps = config.get_hidden_apps();
    let hidden_profiles = config.get_hidden_profiles();

    let mut visible_browser_profiles: Vec<CommonBrowserProfile> = Vec::new();
    let mut hidden_browser_profiles: Vec<CommonBrowserProfile> = Vec::new();
    //let support_dir = macos_get_application_support_dir();
    info!("Apps");
    for installed_browser in installed_browsers {
        info!("App: {:?}", installed_browser.bundle);
        info!("  Path: {:?}", installed_browser.executable_path);
        let app = GenericApp::new(&installed_browser, app_finder.get_app_repository());

        for p in app.get_profiles() {
            let app_id = p.get_unique_app_id();
            if hidden_apps.contains(&app_id) {
                info!(
                    "Skipping Profile: {:?} because whole app is hidden",
                    p.get_profile_name()
                );
                hidden_browser_profiles.push(p.clone());
                continue;
            }

            let profile_unique_id = p.get_unique_id();

            if hidden_profiles.contains(&profile_unique_id) {
                info!(
                    "Skipping Profile: {:?} because the specific profile is hidden",
                    p.get_profile_name()
                );
                hidden_browser_profiles.push(p.clone());
                continue;
            }
            info!("Profile: {:?}", profile_unique_id.as_str());
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

    return (visible_browser_profiles, hidden_browser_profiles);
}

pub fn basically_main() {
    let args: Vec<String> = env::args().collect();
    //info!("{:?}", args);

    let mut url = "https://github.com".to_string();
    let url_input_maybe = args.iter().find(|i| i.starts_with("http"));
    if url_input_maybe.is_some() {
        url = url_input_maybe.unwrap().to_string();
    }

    let show_gui = !args.contains(&"--no-gui".to_string());
    let force_reload = args.contains(&"--reload".to_string());

    let app_finder = utils::OSAppFinder::new();

    let is_default = utils::set_as_default_web_browser();
    let show_set_as_default = !is_default;

    let (mut visible_browser_profiles, mut hidden_browser_profiles) =
        generate_all_browser_profiles(&app_finder, force_reload);

    let (main_sender, main_receiver) = mpsc::channel::<MessageToMain>();

    let ui2 = UI::new(
        main_sender,
        url.as_str(),
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
                    let profiles = generate_all_browser_profiles(&app_finder, true);
                    visible_browser_profiles = profiles.0;
                    hidden_browser_profiles = profiles.1;

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
                MessageToMain::LinkOpenedFromBundle(from_bundle_id, url) => {
                    // TODO: do something once we have rules to
                    //       prioritize/default browsers based on source app and/or url
                    info!("source_bundle_id: {}", from_bundle_id.clone());
                    info!("url: {}", url);
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
                    if visible_profile_index_maybe.is_some() {
                        let visible_profile_index = visible_profile_index_maybe.unwrap();
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
                    if hidden_profile_index_maybe.is_some() {
                        let hidden_profile_index = hidden_profile_index_maybe.unwrap();
                        let hidden_profile = hidden_browser_profiles.remove(hidden_profile_index);
                        visible_browser_profiles.push(hidden_profile);

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
                MessageToMain::MoveAppProfile(unique_id, to_higher) => {
                    let visible_profile_index_maybe = visible_browser_profiles
                        .iter()
                        .position(|p| p.get_unique_id() == unique_id);

                    if visible_profile_index_maybe.is_none() {
                        warn!("Could not find visible profile for id {}", unique_id);
                        continue;
                    }
                    let visible_profile_index = visible_profile_index_maybe.unwrap();

                    if to_higher {
                        if visible_profile_index == 0 {
                            info!(
                                "Not moving profile {} higher as it's already first",
                                unique_id
                            );
                            continue;
                        }
                        info!("Moving profile {} higher", unique_id);
                    } else {
                        if visible_profile_index == visible_browser_profiles.len() - 1 {
                            info!(
                                "Not moving profile {} lower as it's already last",
                                unique_id
                            );
                            continue;
                        }
                        info!("Moving profile {} lower", unique_id);
                    }

                    // 1. update visible_browser_profiles
                    if to_higher {
                        visible_browser_profiles
                            [visible_profile_index - 1..visible_profile_index + 1]
                            .rotate_left(1);
                    } else {
                        visible_browser_profiles[visible_profile_index..visible_profile_index + 2]
                            .rotate_right(1);
                    }

                    // 2. send visible_browser_profiles to ui
                    let ui_browsers = UI::real_to_ui_browsers(&visible_browser_profiles);
                    ui_event_sink
                        .submit_command(ui::NEW_BROWSERS_RECEIVED, ui_browsers, Target::Global)
                        .ok();

                    // 3. update config file
                    let profile_ids_sorted: Vec<String> = visible_browser_profiles
                        .iter()
                        .map(|p| p.get_unique_id().clone())
                        .collect();

                    let mut config = app_finder.get_installed_browsers_config();
                    config.set_profile_order(&profile_ids_sorted);
                    app_finder.save_installed_browsers_config(&config);
                }
            }
        }
        info!("Exiting waiting thread");
    });

    if show_gui {
        launcher.launch(initial_ui_state).expect("error");
    }
}

#[derive(Debug)]
pub enum MessageToMain {
    Refresh,
    OpenLink(usize, bool, String),
    LinkOpenedFromBundle(String, String),
    SetBrowsersAsDefaultBrowser,
    HideAppProfile(String),
    HideAllProfiles(String),
    RestoreAppProfile(String),
    MoveAppProfile(String, bool),
}
