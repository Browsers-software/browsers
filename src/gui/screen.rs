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

// winit only knows about its own windows, so each platform has to implement these natively

// the global cursor position
fn mouse_position() -> Point {
    #[cfg(target_os = "macos")]
    {
        crate::macos::macos_native::mouse_position()
    }

    #[cfg(target_os = "windows")]
    {
        crate::windows::windows_utils::mouse_position()
    }

    #[cfg(target_os = "linux")]
    {
        crate::linux::linux_utils::mouse_position()
    }
}

// the work area of the monitor that contains `point` - for a window that is already on screen,
// whose own monitor is not necessarily the cursor's
pub fn work_area_at(point: Point) -> Rect {
    #[cfg(target_os = "macos")]
    {
        crate::macos::macos_native::work_area_at(point)
    }

    #[cfg(target_os = "windows")]
    {
        crate::windows::windows_utils::work_area_at(point)
    }

    #[cfg(target_os = "linux")]
    {
        crate::linux::linux_utils::work_area_at(point)
    }
}

// global cursor position + the work area of the monitor it is on
pub fn mouse_position_and_work_area() -> (Point, Rect) {
    let mouse = mouse_position();
    (mouse, work_area_at(mouse))
}
