
use slint::Image;

use crate::gui::app_state::UIBrowser;
use crate::gui::generated::{BrowserItem, RestorableProfile};
use crate::gui::icon_loader::IconLoader;
use crate::gui::screen::{Point, Rect};

// icon styles are conventionally different on platforms,
// e.g most macos icons are actually with a lot of padding
pub fn get_icon_size() -> f32 {
    if cfg!(target_os = "macos") {
        32.0
    } else {
        24.0
    }
}

pub fn calculate_window_position(
    mouse: Point,
    screen_rect: Rect,
    window_size: (f32, f32),
) -> Point {
    let (window_width, window_height) = window_size;
    let mut x = mouse.x;
    let mut y = mouse.y;

    // if x is less than starting point, start from min starting rect
    if x < screen_rect.x0 {
        x = screen_rect.x0;
    }
    // if it doesn't fit, put it as far as it does fit
    if x + window_width > screen_rect.x1 {
        x = screen_rect.x1 - window_width;
    }
    // if y is less than starting point, start from min starting rect
    if y < screen_rect.y0 {
        y = screen_rect.y0;
    }
    if y + window_height > screen_rect.y1 {
        y = screen_rect.y1 - window_height;
    }

    Point { x, y }
}

// shared by the main window's full item (real icons + hotkey) and the Settings window's
// lightweight one (blank icons, no hotkey) - see settings_window::to_lightweight_browser_item
pub fn browser_item(
    b: &UIBrowser,
    image: Image,
    profile_image: Image,
    hotkey: String,
    show_hotkey: bool,
) -> BrowserItem {
    BrowserItem {
        image,
        profile_image,
        browser_name: b.browser_name.clone().into(),
        profile_name: b.profile_name.clone().into(),
        full_name: b.get_full_name().into(),
        show_incognito: b.supports_incognito,
        hotkey: hotkey.into(),
        show_hotkey,
        unique_id: b.unique_id.clone().into(),
        unique_app_id: b.unique_app_id.clone().into(),
        browser_profile_index: b.browser_profile_index as i32,
        filtered_index: b.filtered_index as i32,
        is_first: b.is_first,
        is_last: b.is_last,
        has_priority_ordering: b.has_priority_ordering(),
        supports_profiles: b.supports_profiles,
    }
}

pub fn to_browser_item(b: &UIBrowser, icons: &mut IconLoader, show_hotkeys: bool) -> BrowserItem {
    let show_hotkey = show_hotkeys && b.filtered_index < 9;
    let hotkey = if show_hotkey {
        (b.filtered_index + 1).to_string()
    } else {
        String::new()
    };

    browser_item(
        b,
        icons.load(&b.icon_path),
        icons.load(&b.profile_icon_path),
        hotkey,
        show_hotkey,
    )
}

pub fn to_restorable_profile(b: &UIBrowser) -> RestorableProfile {
    RestorableProfile {
        unique_id: b.unique_id.clone().into(),
        full_name: b.get_full_name().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Rect = Rect { x0: 0.0, y0: 25.0, x1: 1000.0, y1: 800.0 };

    #[test]
    fn a_window_that_grew_past_the_bottom_moves_up() {
        let p = calculate_window_position(Point { x: 100.0, y: 700.0 }, SCREEN, (300.0, 400.0));
        assert_eq!((p.x, p.y), (100.0, 400.0));
    }

    #[test]
    fn a_window_that_fits_stays_where_it_is() {
        let p = calculate_window_position(Point { x: 100.0, y: 200.0 }, SCREEN, (300.0, 400.0));
        assert_eq!((p.x, p.y), (100.0, 200.0));
    }
}
