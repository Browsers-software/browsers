use druid::widget::{Container, Flex, Label};
use druid::{DelegateCtx, Monitor, WindowDesc};
use tracing::info;

use crate::gui::ui::UIState;

pub fn show_settings_dialog(ctx: &mut DelegateCtx) {
    info!("show_settings_dialog");

    let app_name_row: Label<UIState> = Label::new("Settings Hey");

    let col = Flex::column().with_child(app_name_row);

    let new_win = WindowDesc::new(col)
        // OpenBox on linux changes title to "Unnamed Window" if it's empty string,
        // so using space instead
        .title("Settings")
        .show_titlebar(true);

    ctx.new_window(new_win);
}
