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
        crate::linux::linux_utils::mouse_position_and_work_area()
    }
}
