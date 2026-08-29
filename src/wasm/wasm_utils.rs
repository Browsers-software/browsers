use crate::browser_repository::SupportedAppRepository;
use crate::gui::screen::{Point, Rect};
use crate::{InstalledAppProfiles, InstalledBrowser, InstalledBrowserProfile};
use std::path::PathBuf;

pub(crate) fn get_this_app_cache_root_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_this_app_config_root_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_unsandboxed_local_config_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_unsandboxed_home_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_this_app_resources_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_this_app_runtime_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn get_this_app_logs_root_dir() -> PathBuf {
    return PathBuf::new();
}

pub(crate) fn mouse_position_and_work_area() -> (Point, Rect) {
    let point = Point { x: 0.0, y: 0.0 };

    let rect = Rect {
        x0: f32::MIN / 2.0,
        y0: f32::MIN / 2.0,
        x1: f32::MAX / 2.0,
        y1: f32::MAX / 2.0,
    };

    return (point, rect);
}

pub struct OsHelper {
    app_repository: SupportedAppRepository,
}

impl OsHelper {
    pub fn new() -> OsHelper {
        let app_repository = SupportedAppRepository::new();
        Self {
            app_repository: app_repository,
        }
    }

    pub fn get_app_repository(&self) -> &SupportedAppRepository {
        &self.app_repository
    }

    pub(crate) fn get_installed_browsers(
        &self,
        schemes: Vec<(String, Vec<String>)>,
    ) -> Vec<InstalledBrowser> {
        let mut browsers: Vec<InstalledBrowser> = Vec::new();

        browsers.push(Self::stub("Chrome", "Personal"));
        browsers.push(Self::stub("LadyBird", ""));
        browsers.push(Self::stub("Firefox", "Personal"));
        browsers.push(Self::stub("Safari", ""));
        browsers.push(Self::stub("Brave", ""));
        browsers.push(Self::stub("Edge", "Personal"));
        browsers.push(Self::stub("Vivaldi", ""));

        return browsers;
    }

    fn stub(display_name: &str, profile_name: &str) -> InstalledBrowser {
        let executable_path = "fake".to_string();
        let executable_path = PathBuf::from(executable_path);

        let command_parts: Vec<String> = vec![executable_path.to_str().unwrap().to_string()];

        let icon_path_str = "fake/icon".to_string();

        let profiles = if profile_name == "" {
            Self::no_profiles()
        } else {
            Self::stub_profiles(profile_name)
        };

        InstalledBrowser {
            command: command_parts.clone(),
            executable_path: executable_path.to_str().unwrap().to_string(),
            display_name: display_name.to_string(),
            bundle: "some_bundle".to_string(),
            user_dir: "some_dir".to_string(),
            icon_path: icon_path_str.clone(),
            profiles: profiles,
            restricted_domains: vec![],
        }
    }

    fn no_profiles() -> InstalledAppProfiles {
        return InstalledAppProfiles::new_placeholder();
    }

    fn stub_profiles(profile_name: &str) -> InstalledAppProfiles {
        let profile = Self::stub_profile(profile_name);
        let profiles = vec![profile];

        InstalledAppProfiles::new_real(profiles)
    }

    fn stub_profile(profile_name: &str) -> InstalledBrowserProfile {
        InstalledBrowserProfile {
            profile_cli_arg_value: "".to_string(),
            profile_cli_container_name: None,
            profile_name: profile_name.to_string(),
            profile_icon: None,
            profile_restricted_url_patterns: vec![],
        }
    }
}
