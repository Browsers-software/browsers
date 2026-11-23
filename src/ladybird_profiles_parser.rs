use std::path::Path;

use tracing::info;

use crate::InstalledBrowserProfile;

// Ladybird keeps each profile in `<data dir>/Ladybird/Profiles/<name>`,
// see `Profile::create` in Libraries/LibWebView/Profile.cpp.
// Builds from before profiles were introduced have no `Profiles` directory.
pub fn find_ladybird_profiles(
    ladybird_data_dir: &Path,
    _binary_path: &Path,
    _app_id: &str,
) -> Vec<InstalledBrowserProfile> {
    let profiles_dir = ladybird_data_dir.join("Profiles");

    let entries = match std::fs::read_dir(&profiles_dir) {
        Ok(entries) => entries,
        Err(_) => {
            info!(
                "Skipping Ladybird profiles, because '{}' cannot be read",
                profiles_dir.display()
            );
            return vec![];
        }
    };

    let mut browser_profiles: Vec<InstalledBrowserProfile> = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        // Ladybird refuses to start with any other name
        if !entry.path().is_dir() || !is_valid_profile_name(&name) {
            continue;
        }

        browser_profiles.push(InstalledBrowserProfile {
            profile_cli_arg_value: name.clone(),
            profile_cli_container_name: None,
            profile_name: name,
            profile_icon: None,
            profile_restricted_url_patterns: vec![],
        });
    }
    return browser_profiles;
}

// Same as `Profile::is_valid_name` in Ladybird
fn is_valid_profile_name(name: &str) -> bool {
    return !name.is_empty()
        && name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-');
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_names(ladybird_dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = find_ladybird_profiles(ladybird_dir, Path::new(""), "")
            .into_iter()
            .map(|p| p.profile_cli_arg_value)
            .collect();
        names.sort();
        return names;
    }

    #[test]
    fn lists_profile_directories() {
        let dir = tempfile::tempdir().unwrap();
        let profiles = dir.path().join("Profiles");
        std::fs::create_dir_all(profiles.join("default")).unwrap();
        std::fs::create_dir_all(profiles.join("work-2_x")).unwrap();

        assert_eq!(find_names(dir.path()), vec!["default", "work-2_x"]);
    }

    #[test]
    fn skips_files_and_invalid_names() {
        let dir = tempfile::tempdir().unwrap();
        let profiles = dir.path().join("Profiles");
        std::fs::create_dir_all(profiles.join("default")).unwrap();
        std::fs::create_dir_all(profiles.join("Work")).unwrap();
        std::fs::create_dir_all(profiles.join(".hidden")).unwrap();
        std::fs::write(profiles.join("notes"), "").unwrap();

        assert_eq!(find_names(dir.path()), vec!["default"]);
    }

    #[test]
    fn no_profiles_directory() {
        let dir = tempfile::tempdir().unwrap();

        assert!(find_names(dir.path()).is_empty());
    }
}
