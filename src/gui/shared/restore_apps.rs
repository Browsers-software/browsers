use std::sync::Arc;

use druid::{LocalizedString, Menu, MenuItem};

use crate::gui::main_window::RESTORE_HIDDEN_PROFILE;
use crate::gui::ui::{UIBrowser, UIState};

pub(crate) fn make_hidden_apps_menu(hidden_profiles: Arc<Vec<UIBrowser>>) -> Menu<UIState> {
    let mut submenu_hidden_apps = Menu::new(LocalizedString::new("Restore"));

    if !hidden_profiles.is_empty() {
        for hidden_profile in hidden_profiles.iter() {
            let item_name = hidden_profile.get_full_name();
            let profile_unique_id = hidden_profile.unique_id.clone();

            submenu_hidden_apps = submenu_hidden_apps.entry(MenuItem::new(item_name).on_activate(
                move |ctx, _data: &mut UIState, _env| {
                    let command = RESTORE_HIDDEN_PROFILE.with(profile_unique_id.clone());
                    ctx.submit_command(command);
                },
            ));
        }
    } else {
        submenu_hidden_apps =
            submenu_hidden_apps.entry(MenuItem::new("No hidden apps or profiles").enabled(false));
    }

    return submenu_hidden_apps;
}
