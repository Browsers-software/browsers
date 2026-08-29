// a point in logical desktop pixels
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// a rectangle in logical desktop pixels - a monitor's work area, excluding the taskbar/menu
// bar/dock, wherever the platform can actually tell us that
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

// global cursor position + monitor work area.
// winit only knows about its own windows, so each platform has to implement this natively
pub fn mouse_position_and_work_area() -> (Point, Rect) {
    #[cfg(target_os = "macos")]
    {
        crate::macos::macos_native::mouse_position_and_work_area()
    }

    #[cfg(target_os = "windows")]
    {
        crate::windows::windows_utils::mouse_position_and_work_area()
    }

    #[cfg(target_os = "linux")]
    {
        linux_mouse_position_and_work_area()
    }

    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::wasm_utils::mouse_position_and_work_area()
    }
}

#[cfg(target_os = "linux")]
fn linux_mouse_position_and_work_area() -> (Point, Rect) {
    use mouse_position::mouse_position::Mouse;

    let point = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => Point {
            x: x as f32,
            y: y as f32,
        },
        Mouse::Error => Point { x: 0.0, y: 0.0 },
    };

    // can't reliably query the work area across Linux desktops without heavier deps, so we just
    // skip clamping - popup won't get nudged back on-screen if it's near an edge, oh well
    let work_area = Rect {
        x0: f32::MIN / 2.0,
        y0: f32::MIN / 2.0,
        x1: f32::MAX / 2.0,
        y1: f32::MAX / 2.0,
    };

    (point, work_area)
}
