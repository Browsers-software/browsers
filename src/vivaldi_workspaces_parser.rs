use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde_json::Value;

pub fn preferences_json_map(preferences_json_file_path: &Path) -> Vec<VivaldiWorkspace> {
    // Open the file in read-only mode with buffer.
    let file = File::open(preferences_json_file_path).unwrap();
    let reader = BufReader::new(file);
    let v: Value = serde_json::from_reader(reader).unwrap();
    let vivaldi = &v["vivaldi"];
    let workspaces = &vivaldi["workspaces"];
    let workspaces_list = &workspaces["list"];
    let workspaces_list_maybe = workspaces_list.as_array();
    if workspaces_list_maybe.is_none() {
        return Vec::new();
    }

    let workspaces_list_arr = workspaces_list_maybe.unwrap();
    let mut containers: Vec<VivaldiWorkspace> = Vec::new();

    for workspace in workspaces_list_arr {
        // id is stored as 1.702675590916e+12
        // but converting it to string converts it to 1702675590916
        let id = workspace["id"].as_f64().unwrap();
        let name = workspace["name"].as_str().unwrap();
        // also has icon string of svg
        // can also have "emoji":"👀"
        let container = VivaldiWorkspace {
            id: id.to_string(),
            name: name.to_string(),
        };
        containers.push(container);
    }
    return containers;
}

pub struct VivaldiWorkspace {
    pub id: String,
    pub name: String,
}
