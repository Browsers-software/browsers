use druid::widget::Label;
use druid::Widget;

use crate::gui::ui::UISettings;

pub(crate) fn general_content() -> impl Widget<UISettings> {
    return Label::new("TODO");
}
