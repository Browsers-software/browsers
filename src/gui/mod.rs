// ui/app.slint is the single compile root that re-exports all three windows (MainWindow,
// SettingsWindow, AboutWindow), so one include_modules!() here is enough
pub mod generated {
    #![allow(clippy::all)]
    slint::include_modules!();
}

pub mod about_window;
pub mod app;
pub mod app_state;
pub mod icon_loader;
pub mod main_window;
pub mod screen;
pub mod settings_window;
pub mod theme;
pub mod ui_util;
