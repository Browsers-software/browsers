use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::{Map, Value};
use tracing::{debug, info};

use crate::{paths, InstalledBrowserProfile};

pub fn find_slack_profiles(
    slack_user_dir: &Path,
    _binary_path: &Path,
    app_id: &str,
) -> Vec<InstalledBrowserProfile> {
    let mut browser_profiles: Vec<InstalledBrowserProfile> = Vec::new();

    let root_state_file = slack_user_dir.join("storage/root-state.json");
    debug!("Slack Root State Path: {:?}", root_state_file);
    // Depending if installed from app store or not, path is:
    // ~/Library/Containers/com.tinyspeck.slackmacgap/Data/Library/Application Support/Slack/storage/root-state.json
    // ~/Library/Application Support/Slack/storage/root-state.json

    if !root_state_file.exists() {
        info!("Could not find {}", root_state_file.display());
        return browser_profiles;
    }

    let workspace_map = SlackWorkspacesMap::new_from_root_state(root_state_file.as_path());
    let workspaces = workspace_map.parse_profiles();
    for workspace in workspaces {
        let cache_root_dir = paths::get_cache_root_dir();
        let profiles_icons_root_dir = cache_root_dir.join("icons").join("profiles");
        fs::create_dir_all(profiles_icons_root_dir.as_path()).unwrap();
        let profiles_icons_root = profiles_icons_root_dir.join(app_id);
        fs::create_dir_all(profiles_icons_root.as_path()).unwrap();

        /*let image_file_path = workspace.icon_88;
        let profile_icon_path = if image_file_path != "" {
            let png_file = File::open(image_file_path.as_path()).unwrap();
            let mut png_file_reader = BufReader::new(png_file);
            let mut buffer = Vec::new();
            // Read file into vector
            let result = png_file_reader.read_to_end(&mut buffer);
            if result.is_err() {
                None
            } else {
                let to_filename = workspace.domain.to_string() + ".png";
                let png_file_path = profiles_icons_root.join(to_filename);
                utils::save_as_circular(buffer, png_file_path.as_path());
                Some(png_file_path.to_str().unwrap().to_string())
            }
        } else {
            None
        };*/
        let profile_icon_path = None;

        let workspace_url_pattern = format!("{}.slack.com", workspace.domain.as_str());
        let enterprise_workspace_url_pattern =
            format!("{}.enterprise.slack.com", workspace.domain.as_str());
        let slack_gov_workspace_url_pattern =
            format!("{}.slack-gov.com", workspace.domain.as_str());

        let app_url_pattern = format!("app.slack.com/**/{}/**", workspace.id.as_str());

        browser_profiles.push(InstalledBrowserProfile {
            profile_cli_arg_value: workspace.id.to_string(),
            profile_cli_container_name: Some(workspace.domain.to_string()),
            profile_name: workspace.name,
            profile_icon: profile_icon_path,
            profile_restricted_url_patterns: vec![
                workspace_url_pattern,
                //enterprise_workspace_url_pattern,
                //slack_gov_workspace_url_pattern,
                //app_url_pattern,
            ],
        })
    }
    return browser_profiles;
}

pub struct SlackWorkspacesMap {
    workspaces_map: Map<String, Value>,
}

impl SlackWorkspacesMap {
    pub fn new_from_root_state(local_state_file_path: &Path) -> Self {
        Self {
            workspaces_map: Self::workspaces_map(local_state_file_path),
        }
    }

    fn workspaces_map(local_state_file_path: &Path) -> Map<String, Value> {
        // Open the file in read-only mode with buffer.
        let file = File::open(local_state_file_path).unwrap();
        let reader = BufReader::new(file);
        let v: Value = serde_json::from_reader(reader).unwrap();
        let workspaces = &v["workspaces"];
        let workspaces_map = workspaces.as_object().unwrap();
        return workspaces_map.to_owned();
    }

    pub fn parse_profiles(self) -> Vec<SlackWorkspace> {
        //let chrome_attributes_finder = ChromeAttributesFinder::new();
        let workspaces_map = self.workspaces_map;
        let profiles_count = workspaces_map.len();

        let mut profiles_vec: Vec<SlackWorkspace> = Vec::with_capacity(profiles_count);

        for (domain_name, workspace) in workspaces_map {
            let domain = workspace["domain"].as_str().unwrap_or("").to_string();

            let id = workspace["id"].as_str().unwrap_or("").to_string();

            let name = workspace["name"].as_str().unwrap_or("").to_string();

            let icon = workspace["icon"].as_object().unwrap();
            let image_68 = icon["image_68"].as_str().unwrap_or("").to_string();
            let image_88 = icon["image_88"].as_str().unwrap_or("").to_string();

            profiles_vec.push(SlackWorkspace {
                domain: domain,
                id: id,
                name: name,
                icon_68: image_68,
                icon_88: image_88,
            });
        }
        // constant ordering (well based on name)
        profiles_vec.sort_by(|p1, p2| p1.name.cmp(&p2.name));
        return profiles_vec;
    }
}

pub struct SlackWorkspace {
    pub domain: String,
    pub id: String,
    pub name: String,
    pub icon_68: String,
    pub icon_88: String,
}

/*
 "workspaces": {
   "T09NY5SBT": {
     "domain": "kubernetes",
     "id": "T09NY5SBT",
     "icon": {
       "image_68": "https://avatars.slack-edge.com/2018-04-11/346300247111_7257b45e7a19f5230da9_68.png",
       "image_88": "https://avatars.slack-edge.com/2018-04-11/346300247111_7257b45e7a19f5230da9_88.png"
     },
     "name": "Kubernetes",
     "order": 0
   },
   "T05ALPT0XU6": {
     "domain": "browsersgroup",
     "id": "T05ALPT0XU6",
     "icon": {
       "image_68": "https://a.slack-edge.com/80588/img/avatars-teams/ava_0017-68.png",
       "image_88": "https://a.slack-edge.com/80588/img/avatars-teams/ava_0017-88.png"
     },
     "name": "Browsers",
     "order": 1
   }
 },
*/
