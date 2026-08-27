use slint::{ComponentHandle, ModelRc, VecModel};

use crate::gui::app::{self, SharedState};
use crate::gui::generated::{AboutWindow, DirEntry};
use crate::gui::icon_loader;
use crate::paths;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// (label, path) pairs
pub fn directories() -> Vec<(String, String)> {
    // .join("") adds a trailing "/", indicating for the user that it's a directory
    let config = paths::get_config_root_dir().join("");
    let cache = paths::get_cache_root_dir().join("");
    let logs = paths::get_logs_root_dir().join("");
    let resources = paths::get_resources_basedir().join("");

    vec![
        ("Config".to_string(), config.to_string_lossy().to_string()),
        ("Cache".to_string(), cache.to_string_lossy().to_string()),
        ("Logs".to_string(), logs.to_string_lossy().to_string()),
        ("Resources".to_string(), resources.to_string_lossy().to_string()),
    ]
}

// DirEntry used to be nominally distinct per window (each was its own separately-compiled
// document), so this stayed private and settings_window.rs duplicated the mapping. now that
// ui/app.slint is one compile unit they share the same DirEntry type, so this can just be reused
// directly.
pub fn directories_entries() -> Vec<DirEntry> {
    directories()
        .into_iter()
        .map(|(label, path)| DirEntry {
            label: label.into(),
            path: path.into(),
        })
        .collect()
}

pub fn open(state: &SharedState) {
    // focus instead of spawning a second instance if one's already open - state.about_window gets
    // cleared to None on actual close, so Some here only ever means "currently showing"
    if let Some(win) = state.borrow().about_window.as_ref().map(|w| w.clone_strong()) {
        win.window().show().ok();
        return;
    }

    let win = AboutWindow::new().expect("failed to create about window");

    let mut loader = icon_loader::IconLoader::new();
    win.set_app_icon(loader.load(&paths::get_app_icon_path().to_string_lossy()));
    win.set_version_text(format!("Version {}", VERSION).into());
    win.set_directories(ModelRc::new(VecModel::from(directories_entries())));

    state.borrow_mut().about_window = Some(win.clone_strong());
    app::apply_theme(state);
    state.borrow_mut().extra_windows_open += 1;
    win.window().show().ok();

    let state = state.clone();
    win.window().on_close_requested(move || {
        let new_count = state.borrow().extra_windows_open.saturating_sub(1);
        state.borrow_mut().extra_windows_open = new_count;
        // HideWindow is the only "proceed" variant Slint has - clearing state.about_window below
        // (the last strong ref) is what actually drops the window
        state.borrow_mut().about_window = None;
        slint::CloseRequestResponse::HideWindow
    });
}
