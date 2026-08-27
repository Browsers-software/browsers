use std::sync::Arc;

use url::Url;

use crate::CommonBrowserProfile;
use crate::url_rule::UrlGlobMatcher;
use crate::utils::{BehavioralConfig, Config, ConfiguredTheme, ProfileAndOptions, UIConfig};

// plain, framework-agnostic mirror of a browser (+ profile) entry - gets converted into the
// generated slint::BrowserItem for display, see gui::main_window::to_browser_item
#[derive(Clone, Debug)]
pub struct UIBrowser {
    // index in the not-explicitly-hidden browsers list, used to send OpenLink
    // to the main event loop. Not impacted by the current url filter.
    pub browser_profile_index: usize,
    pub is_first: bool,
    pub is_last: bool,
    pub restricted_url_matchers: Arc<Vec<UrlGlobMatcher>>,
    pub browser_name: String,
    pub profile_name: String,
    pub supports_profiles: bool,
    pub supports_incognito: bool,

    pub icon_path: String,
    pub profile_icon_path: String,
    pub unique_id: String,
    pub unique_app_id: String,

    // index among currently visible (filtered-by-url) browsers
    pub filtered_index: usize,
}

impl UIBrowser {
    pub fn has_priority_ordering(&self) -> bool {
        !self.restricted_url_matchers.is_empty()
    }

    /// Returns app name + optionally profile name if app supports multiple profiles
    pub fn get_full_name(&self) -> String {
        if self.supports_profiles {
            format!("{} ({})", self.browser_name, self.profile_name)
        } else {
            self.browser_name.clone()
        }
    }
}

#[derive(Clone)]
pub struct UISettings {
    pub default_opener: Option<UIProfileAndIncognito>,
    pub rules: Vec<UISettingsRule>,
    pub visual_settings: UIVisualSettings,
    pub behavioral_settings: UIBehavioralSettings,
}

#[derive(Clone, Debug)]
pub struct UIVisualSettings {
    pub show_hotkeys: bool,
    pub quit_on_lost_focus: bool,
    pub theme: ConfiguredTheme,
}

#[derive(Clone, Debug)]
pub struct UIBehavioralSettings {
    pub unwrap_urls: bool,
}

#[derive(Clone, Debug)]
pub struct UIProfileAndIncognito {
    pub profile: String,
    pub incognito: bool,
}

#[derive(Clone, Debug)]
pub struct UISettingsRule {
    pub index: usize,
    pub source_app: String,
    pub url_pattern: String,
    pub opener: Option<UIProfileAndIncognito>,
}

impl UISettingsRule {
    // converts empty string to None
    pub fn get_source_app(&self) -> Option<String> {
        if self.source_app.is_empty() {
            None
        } else {
            Some(self.source_app.clone())
        }
    }

    // converts empty string to None
    pub fn get_url_pattern(&self) -> Option<String> {
        if self.url_pattern.is_empty() {
            None
        } else {
            Some(self.url_pattern.clone())
        }
    }
}

fn map_as_ui_profile(
    profile_and_options: &Option<ProfileAndOptions>,
) -> Option<UIProfileAndIncognito> {
    profile_and_options.as_ref().map(|p| UIProfileAndIncognito {
        profile: p.profile.clone(),
        incognito: p.incognito,
    })
}

fn map_as_visual_settings(ui_config: &UIConfig) -> UIVisualSettings {
    UIVisualSettings {
        show_hotkeys: ui_config.show_hotkeys,
        quit_on_lost_focus: ui_config.quit_on_lost_focus,
        theme: ui_config.theme,
    }
}

fn map_as_ui_behavioural_settings(behavior: &BehavioralConfig) -> UIBehavioralSettings {
    UIBehavioralSettings {
        unwrap_urls: behavior.unwrap_urls,
    }
}

pub fn config_to_ui_settings(config: &Config) -> UISettings {
    let rules = config
        .get_rules()
        .iter()
        .enumerate()
        .map(|(i, rule)| UISettingsRule {
            index: i,
            source_app: rule.source_app.clone().unwrap_or_default(),
            url_pattern: rule.url_pattern.clone().unwrap_or_default(),
            opener: map_as_ui_profile(&rule.get_opener()),
        })
        .collect();

    UISettings {
        default_opener: map_as_ui_profile(config.get_default_profile()),
        rules,
        visual_settings: map_as_visual_settings(config.get_ui_config()),
        behavioral_settings: map_as_ui_behavioural_settings(config.get_behavior()),
    }
}

pub fn real_to_ui_browsers(all_browser_profiles: &[CommonBrowserProfile]) -> Vec<UIBrowser> {
    if all_browser_profiles.is_empty() {
        return vec![];
    }

    // TODO: this is a bit ugly; we keep profiles with has_priority_ordering() always on top
    //       and everything else comes after; it might make sense to keep them in two separate
    //       vectors (or slices)
    let first_orderable_item_index = all_browser_profiles
        .iter()
        .position(|b| !b.has_priority_ordering())
        .unwrap_or(0);

    let profiles_count = all_browser_profiles.len();

    all_browser_profiles
        .iter()
        .enumerate()
        .map(|(i, p)| UIBrowser {
            browser_profile_index: i,
            is_first: i == first_orderable_item_index,
            is_last: i == profiles_count - 1,
            restricted_url_matchers: Arc::new(p.get_restricted_url_matchers().clone()),
            browser_name: p.get_browser_name().to_string(),
            profile_name: p.get_profile_name().to_string(),
            supports_profiles: p.get_browser_common().has_real_profiles(),
            supports_incognito: p.get_browser_common().supports_incognito(),
            icon_path: p.get_browser_icon_path().to_string(),
            profile_icon_path: p
                .get_profile_icon_path()
                .map_or("".to_string(), |a| a.to_string()),
            unique_id: p.get_unique_id(),
            unique_app_id: p.get_unique_app_id(),
            filtered_index: i,
        })
        .collect()
}

pub fn get_filtered_browsers(url: &str, ui_browsers: &[UIBrowser]) -> Vec<UIBrowser> {
    let url_maybe = Url::parse(url).ok();

    let mut filtered: Vec<UIBrowser> = ui_browsers
        .iter()
        .cloned()
        .filter(|b| {
            if b.restricted_url_matchers.is_empty() {
                true
            } else {
                url_maybe
                    .as_ref()
                    .map(|url| {
                        b.restricted_url_matchers
                            .iter()
                            .any(|matcher| matcher.url_matches(url))
                    })
                    .unwrap_or(false)
            }
        })
        .enumerate()
        .map(|(index, mut browser)| {
            browser.filtered_index = index;
            browser
        })
        .collect();

    // always show special apps first
    filtered.sort_by_key(|b| !b.has_priority_ordering());

    filtered
}
