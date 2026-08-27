use std::cmp;

use crate::gui::app_state::UIBrowser;
use crate::gui::generated::{BrowserItem, RestorableProfile};
use crate::gui::icon_loader::IconLoader;
use crate::gui::screen::{Point, Rect};

const WINDOW_BORDER_WIDTH: f32 = 1.0;
const PADDING_X: f32 = 5.0;
const PADDING_Y: f32 = 10.0;
const ITEM_WIDTH: f32 = 210.0;
const ITEM_HEIGHT: f32 = 32.0;

// icon styles are conventionally different on platforms,
// e.g most macos icons are actually with a lot of padding
pub fn get_icon_size() -> f32 {
    if cfg!(target_os = "macos") {
        32.0
    } else {
        24.0
    }
}

fn calculate_visible_browser_count(browsers_total: usize) -> usize {
    // max 6 items without scrollbar
    let item_count = cmp::min(6, browsers_total);
    // but at least 1 item in case of errors (or window size is too small)
    cmp::max(1, item_count)
}

fn visible_scroll_area_height(browsers_count_f32: f32) -> f32 {
    browsers_count_f32 * ITEM_HEIGHT
}

pub fn recalculate_window_size(browser_count: usize) -> (f32, f32) {
    let item_count = calculate_visible_browser_count(browser_count);
    calculate_window_size(item_count)
}

fn calculate_window_size(item_count: usize) -> (f32, f32) {
    let browsers_count_f32 = item_count as f32;
    let window_width = ITEM_WIDTH + PADDING_X * 2.0 + WINDOW_BORDER_WIDTH * 2.0;
    let visible_scroll_area_height = visible_scroll_area_height(browsers_count_f32);
    let window_height = visible_scroll_area_height + 5.0 + 12.0 + PADDING_Y * 2.0 + 10.0;
    (window_width, window_height)
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

pub fn to_browser_item(b: &UIBrowser, icons: &mut IconLoader, show_hotkeys: bool) -> BrowserItem {
    let show_hotkey = show_hotkeys && b.filtered_index < 9;
    let hotkey = if show_hotkey {
        (b.filtered_index + 1).to_string()
    } else {
        String::new()
    };

    BrowserItem {
        image: icons.load(&b.icon_path),
        profile_image: icons.load(&b.profile_icon_path),
        browser_name: b.browser_name.clone().into(),
        profile_name: b.profile_name.clone().into(),
        full_name: b.get_full_name().into(),
        show_profile: b.supports_profiles,
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

pub fn to_restorable_profile(b: &UIBrowser) -> RestorableProfile {
    RestorableProfile {
        unique_id: b.unique_id.clone().into(),
        full_name: b.get_full_name().into(),
    }
}
