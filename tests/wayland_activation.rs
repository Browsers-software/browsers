#![cfg(target_os = "linux")]

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::fd::AsFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use wayland_protocols::xdg::activation::v1::server::xdg_activation_token_v1::{
    self, XdgActivationTokenV1,
};
use wayland_protocols::xdg::activation::v1::server::xdg_activation_v1::{self, XdgActivationV1};
use wayland_protocols::xdg::shell::server::xdg_surface::{self, XdgSurface};
use wayland_protocols::xdg::shell::server::xdg_toplevel::{self, XdgToplevel};
use wayland_protocols::xdg::shell::server::xdg_wm_base::{self, XdgWmBase};
use wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use wayland_server::protocol::wl_buffer::{self, WlBuffer};
use wayland_server::protocol::wl_callback::WlCallback;
use wayland_server::protocol::wl_compositor::{self, WlCompositor};
use wayland_server::protocol::wl_keyboard::{self, KeyState, KeymapFormat, WlKeyboard};
use wayland_server::protocol::wl_output::{self, Subpixel, Transform, WlOutput};
use wayland_server::protocol::wl_pointer::{self, WlPointer};
use wayland_server::protocol::wl_region::{self, WlRegion};
use wayland_server::protocol::wl_seat::{self, Capability, WlSeat};
use wayland_server::protocol::wl_shm::{self, Format, WlShm};
use wayland_server::protocol::wl_shm_pool::{self, WlShmPool};
use wayland_server::protocol::wl_surface::{self, WlSurface};
use wayland_server::{Client, DataInit, Dispatch, Display, GlobalDispatch, New, Resource};

const TEST_URL: &str = "https://example.com/wayland-activation-test";
const EXPECTED_TOKEN: &str = "browsers-test-activation-token";
const KEY_ENTER_EVDEV: u32 = 28;
const INPUT_PRESS_SERIAL: u32 = 0x1234;
const INPUT_RELEASE_SERIAL: u32 = 0x1235;

// Minimal XKB keymap that maps evdev KEY_ENTER (28 -> XKB keycode 36) to Return.
const MINIMAL_XKB_KEYMAP: &[u8] = b"\
xkb_keymap {
  xkb_keycodes \"minimal\" {
    minimum = 8;
    maximum = 255;
    <RTRN> = 36;
  };
  xkb_types \"minimal\" {
    type \"ONE_LEVEL\" {
      modifiers = none;
      level_name[Level1] = \"Base\";
    };
  };
  xkb_compatibility \"minimal\" {
    interpret.repeat = False;
  };
  xkb_symbols \"minimal\" {
    key <RTRN> { [ Return ] };
  };
};\0";

#[derive(Default)]
struct TestClientData;

impl ClientData for TestClientData {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

#[derive(Default, Clone, Debug)]
struct ActivationObservation {
    token_requested: bool,
    serial: Option<u32>,
    seat_matched: bool,
    surface_matched: bool,
    committed: bool,
}

#[derive(Default)]
struct TokenDraft {
    serial: Option<u32>,
    seat_matched: bool,
    surface_matched: bool,
}

struct CompositorState {
    next_serial: u32,
    reply_with_token: bool,
    seat: Option<WlSeat>,
    keyboard: Option<WlKeyboard>,
    picker_surface: Option<WlSurface>,
    pending_callbacks: Vec<WlCallback>,
    configure_acked: bool,
    buffer_committed: bool,
    input_sent: bool,
    tokens: HashMap<wayland_server::backend::ObjectId, TokenDraft>,
    observation: Arc<Mutex<ActivationObservation>>,
}

impl CompositorState {
    fn new(observation: Arc<Mutex<ActivationObservation>>, reply_with_token: bool) -> Self {
        Self {
            next_serial: 10,
            reply_with_token,
            seat: None,
            keyboard: None,
            picker_surface: None,
            pending_callbacks: Vec::new(),
            configure_acked: false,
            buffer_committed: false,
            input_sent: false,
            tokens: HashMap::new(),
            observation,
        }
    }

    fn next_serial(&mut self) -> u32 {
        let s = self.next_serial;
        self.next_serial += 1;
        s
    }

    fn maybe_send_return_key(&mut self) {
        if self.input_sent || !self.configure_acked || !self.buffer_committed {
            return;
        }
        let (Some(keyboard), Some(surface)) = (self.keyboard.clone(), self.picker_surface.clone())
        else {
            return;
        };

        let enter_serial = self.next_serial();
        let mod_serial = self.next_serial();
        keyboard.enter(enter_serial, &surface, Vec::new());
        keyboard.modifiers(mod_serial, 0, 0, 0, 0);
        keyboard.key(
            INPUT_PRESS_SERIAL,
            100,
            KEY_ENTER_EVDEV,
            KeyState::Pressed,
        );
        keyboard.key(
            INPUT_RELEASE_SERIAL,
            110,
            KEY_ENTER_EVDEV,
            KeyState::Released,
        );
        self.input_sent = true;
    }
}

impl GlobalDispatch<WlCompositor, ()> for CompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<WlCompositor>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ());
    }
}

impl Dispatch<WlCompositor, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlCompositor,
        request: wl_compositor::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_compositor::Request::CreateSurface { id } => {
                data_init.init(id, ());
            }
            wl_compositor::Request::CreateRegion { id } => {
                data_init.init(id, ());
            }
            _ => {}
        }
    }
}

impl Dispatch<WlSurface, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &WlSurface,
        request: wl_surface::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_surface::Request::Attach { buffer: Some(_), .. } => {
                if state
                    .picker_surface
                    .as_ref()
                    .is_some_and(|s| s.id() == resource.id())
                {
                    state.buffer_committed = true;
                }
            }
            wl_surface::Request::Frame { callback } => {
                let cb: WlCallback = data_init.init(callback, ());
                state.pending_callbacks.push(cb);
            }
            wl_surface::Request::Commit => {
                if state
                    .picker_surface
                    .as_ref()
                    .is_some_and(|s| s.id() == resource.id())
                {
                    state.maybe_send_return_key();
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlCallback, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlCallback,
        _request: wayland_server::protocol::wl_callback::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlRegion, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlRegion,
        _request: wl_region::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<WlShm, ()> for CompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<WlShm>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        let shm = data_init.init(resource, ());
        shm.format(Format::Argb8888);
        shm.format(Format::Xrgb8888);
    }
}

impl Dispatch<WlShm, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlShm,
        request: wl_shm::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_shm::Request::CreatePool { id, .. } = request {
            data_init.init(id, ());
        }
    }
}

impl Dispatch<WlShmPool, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlShmPool,
        request: wl_shm_pool::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        if let wl_shm_pool::Request::CreateBuffer { id, .. } = request {
            data_init.init(id, ());
        }
    }
}

impl Dispatch<WlBuffer, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlBuffer,
        _request: wl_buffer::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<WlOutput, ()> for CompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<WlOutput>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        let output = data_init.init(resource, ());
        output.geometry(
            0,
            0,
            300,
            200,
            Subpixel::Unknown,
            "Test".into(),
            "Monitor".into(),
            Transform::Normal,
        );
        output.mode(wl_output::Mode::Current | wl_output::Mode::Preferred, 1920, 1080, 60_000);
        output.scale(1);
        output.done();
    }
}

impl Dispatch<WlOutput, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlOutput,
        _request: wl_output::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<WlSeat, ()> for CompositorState {
    fn bind(
        state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<WlSeat>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        let seat = data_init.init(resource, ());
        seat.capabilities(Capability::Keyboard | Capability::Pointer);
        seat.name("seat0".into());
        state.seat = Some(seat);
    }
}

impl Dispatch<WlSeat, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &WlSeat,
        request: wl_seat::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            wl_seat::Request::GetKeyboard { id } => {
                let keyboard = data_init.init(id, ());
                let mut keymap_file = tempfile::tempfile().expect("create keymap tempfile");
                keymap_file
                    .write_all(MINIMAL_XKB_KEYMAP)
                    .expect("write keymap");
                keyboard.keymap(
                    KeymapFormat::XkbV1,
                    keymap_file.as_fd(),
                    MINIMAL_XKB_KEYMAP.len() as u32,
                );
                keyboard.repeat_info(25, 600);
                state.keyboard = Some(keyboard);
                state.maybe_send_return_key();
            }
            wl_seat::Request::GetPointer { id } => {
                data_init.init(id, ());
            }
            _ => {}
        }
    }
}

impl Dispatch<WlKeyboard, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlKeyboard,
        _request: wl_keyboard::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl Dispatch<WlPointer, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &WlPointer,
        _request: wl_pointer::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<XdgWmBase, ()> for CompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<XdgWmBase>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ());
    }
}

impl Dispatch<XdgWmBase, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &XdgWmBase,
        request: xdg_wm_base::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        if let xdg_wm_base::Request::GetXdgSurface { id, surface } = request {
            state.picker_surface = Some(surface);
            data_init.init(id, ());
        }
    }
}

impl Dispatch<XdgSurface, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &XdgSurface,
        request: xdg_surface::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_surface::Request::GetToplevel { id } => {
                let toplevel = data_init.init(id, ());
                // Mark the picker window as activated so it accepts focus.
                let activated = (xdg_toplevel::State::Activated as u32).to_ne_bytes().to_vec();
                toplevel.configure(320, 240, activated);
                let serial = state.next_serial();
                resource.configure(serial);
            }
            xdg_surface::Request::AckConfigure { .. } => {
                state.configure_acked = true;
                state.maybe_send_return_key();
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgToplevel, ()> for CompositorState {
    fn request(
        _state: &mut Self,
        _client: &Client,
        _resource: &XdgToplevel,
        _request: xdg_toplevel::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<XdgActivationV1, ()> for CompositorState {
    fn bind(
        _state: &mut Self,
        _handle: &wayland_server::DisplayHandle,
        _client: &Client,
        resource: New<XdgActivationV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, Self>,
    ) {
        data_init.init(resource, ());
    }
}

impl Dispatch<XdgActivationV1, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        _resource: &XdgActivationV1,
        request: xdg_activation_v1::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        data_init: &mut DataInit<'_, Self>,
    ) {
        if let xdg_activation_v1::Request::GetActivationToken { id } = request {
            let token: XdgActivationTokenV1 = data_init.init(id, ());
            state.tokens.insert(token.id(), TokenDraft::default());
            state.observation.lock().unwrap().token_requested = true;
        }
    }
}

impl Dispatch<XdgActivationTokenV1, ()> for CompositorState {
    fn request(
        state: &mut Self,
        _client: &Client,
        resource: &XdgActivationTokenV1,
        request: xdg_activation_token_v1::Request,
        _data: &(),
        _dhandle: &wayland_server::DisplayHandle,
        _data_init: &mut DataInit<'_, Self>,
    ) {
        let Some(draft) = state.tokens.get_mut(&resource.id()) else {
            return;
        };
        match request {
            xdg_activation_token_v1::Request::SetSerial { serial, seat } => {
                draft.serial = Some(serial);
                draft.seat_matched = state
                    .seat
                    .as_ref()
                    .is_some_and(|s| s.id() == seat.id());
            }
            xdg_activation_token_v1::Request::SetSurface { surface } => {
                draft.surface_matched = state
                    .picker_surface
                    .as_ref()
                    .is_some_and(|s| s.id() == surface.id());
            }
            xdg_activation_token_v1::Request::Commit => {
                let mut obs = state.observation.lock().unwrap();
                obs.serial = draft.serial;
                obs.seat_matched = draft.seat_matched;
                obs.surface_matched = draft.surface_matched;
                obs.committed = true;
                drop(obs);

                if !state.reply_with_token {
                    return;
                }

                // Like Niri, only grant a focus token when the request carries the user input
                // serial and seat together with the requesting surface.
                let valid = matches!(
                    draft.serial,
                    Some(INPUT_PRESS_SERIAL | INPUT_RELEASE_SERIAL)
                ) && draft.seat_matched
                    && draft.surface_matched;
                let token_value = if valid {
                    EXPECTED_TOKEN.to_string()
                } else {
                    "urgency-only-missing-serial".to_string()
                };
                resource.done(token_value);
            }
            _ => {}
        }
    }
}

struct PickerRunResult {
    observation: ActivationObservation,
    fields: HashMap<String, String>,
    launched_raw: String,
    stderr: String,
}

fn run_picker_under_mock_compositor(reply_with_token: bool) -> PickerRunResult {
    let temp = tempfile::tempdir().expect("create tempdir");
    let root = temp.path();
    let config_dir = root.join("config");
    let cache_dir = root.join("cache");
    let data_dir = root.join("data");
    let state_dir = root.join("state");
    let runtime_dir = root.join("runtime");
    let apps_dir = data_dir.join("applications");

    for dir in [&config_dir, &cache_dir, &data_dir, &state_dir, &runtime_dir, &apps_dir] {
        fs::create_dir_all(dir).expect("create xdg dir");
    }

    let launch_output = root.join("launch.txt");
    let fake_browser = root.join("fake-browser.sh");
    let script = format!(
        "#!/bin/sh\nset -eu\nout=\"{}\"\ntmp=\"${{out}}.tmp\"\nstdin_target=$(readlink /proc/$$/fd/0 2>/dev/null || echo unknown)\nprintf 'TOKEN=%s\\nURL=%s\\nSTDIN=%s\\n' \"${{XDG_ACTIVATION_TOKEN:-}}\" \"${{1:-}}\" \"$stdin_target\" > \"$tmp\"\nmv \"$tmp\" \"$out\"\n",
        launch_output.display(),
    );
    fs::write(&fake_browser, script).expect("write fake browser script");
    fs::set_permissions(&fake_browser, fs::Permissions::from_mode(0o755))
        .expect("chmod fake browser");

    let desktop_entry = format!(
        "[Desktop Entry]\nType=Application\nName=Test Browser\nExec={} %u\nMimeType=x-scheme-handler/https;x-scheme-handler/http;\n",
        fake_browser.display(),
    );
    fs::write(apps_dir.join("test-browser.desktop"), desktop_entry)
        .expect("write desktop entry");

    // Start an in-process Wayland compositor on a private socket.
    let socket_name = "browsers-test.sock";
    let socket_path = runtime_dir.join(socket_name);
    let listener = UnixListener::bind(&socket_path).expect("bind wayland socket");
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");

    let mut display: Display<CompositorState> = Display::new().expect("create wayland display");
    let mut dh = display.handle();
    dh.create_global::<CompositorState, WlCompositor, ()>(4, ());
    dh.create_global::<CompositorState, WlShm, ()>(1, ());
    dh.create_global::<CompositorState, WlOutput, ()>(2, ());
    dh.create_global::<CompositorState, WlSeat, ()>(5, ());
    dh.create_global::<CompositorState, XdgWmBase, ()>(2, ());
    dh.create_global::<CompositorState, XdgActivationV1, ()>(1, ());

    let observation = Arc::new(Mutex::new(ActivationObservation::default()));
    let mut compositor_state = CompositorState::new(observation.clone(), reply_with_token);

    let mut child = Command::new(env!("CARGO_BIN_EXE_browsers"))
        .arg(TEST_URL)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("HOME", root)
        .env("XDG_CONFIG_HOME", &config_dir)
        .env("XDG_CACHE_HOME", &cache_dir)
        .env("XDG_DATA_HOME", &data_dir)
        .env("XDG_DATA_DIRS", &data_dir)
        .env("XDG_STATE_HOME", &state_dir)
        .env("XDG_RUNTIME_DIR", &runtime_dir)
        .env("WAYLAND_DISPLAY", socket_name)
        .env("SLINT_BACKEND", "winit-software")
        .env_remove("DISPLAY")
        .env_remove("XDG_ACTIVATION_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn browsers");

    let deadline = Instant::now() + Duration::from_secs(10);
    let mut exit_status = None;

    while Instant::now() < deadline {
        if let Ok((stream, _)) = listener.accept() {
            let _ = dh.insert_client(stream, Arc::new(TestClientData));
        }

        let _ = display.dispatch_clients(&mut compositor_state);
        for cb in compositor_state.pending_callbacks.drain(..) {
            cb.done(1);
        }
        let _ = display.flush_clients();

        if let Some(status) = child.try_wait().expect("poll child") {
            exit_status = Some(status);
            break;
        }
        if launch_output.exists() {
            // Give browsers a moment to exit cleanly after spawning the target browser.
            std::thread::sleep(Duration::from_millis(50));
            if let Some(status) = child.try_wait().expect("poll child") {
                exit_status = Some(status);
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }

    if exit_status.is_none() {
        let _ = child.kill();
    }
    let output = child.wait_with_output().expect("collect child output");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        launch_output.exists(),
        "fake browser was never launched (exit={:?})\nstdout:\n{stdout}\nstderr:\n{stderr}",
        exit_status
    );

    let launched_raw = fs::read_to_string(&launch_output).expect("read launch output");
    let mut fields = HashMap::new();
    for line in launched_raw.lines() {
        if let Some((k, v)) = line.split_once('=') {
            fields.insert(k.to_string(), v.to_string());
        }
    }

    PickerRunResult {
        observation: observation.lock().unwrap().clone(),
        fields,
        launched_raw,
        stderr,
    }
}

#[test]
fn wayland_picker_passes_serial_backed_activation_token_to_launched_browser() {
    let run = run_picker_under_mock_compositor(true);
    let obs = &run.observation;

    assert!(
        obs.token_requested && obs.committed,
        "picker must request and commit an xdg_activation_v1 token before launching (obs={obs:?})\nlaunched:\n{}\nstderr:\n{}",
        run.launched_raw,
        run.stderr
    );
    assert!(
        matches!(obs.serial, Some(INPUT_PRESS_SERIAL | INPUT_RELEASE_SERIAL)),
        "xdg_activation_token_v1.set_serial must carry the user input serial (obs={obs:?})"
    );
    assert!(
        obs.seat_matched,
        "xdg_activation_token_v1.set_serial must pass the input wl_seat (obs={obs:?})"
    );
    assert!(
        obs.surface_matched,
        "xdg_activation_token_v1.set_surface must pass the picker wl_surface (obs={obs:?})"
    );
    assert_eq!(
        run.fields.get("URL").map(String::as_str),
        Some(TEST_URL),
        "launched browser should receive the target URL"
    );
    assert_eq!(
        run.fields.get("TOKEN").map(String::as_str),
        Some(EXPECTED_TOKEN),
        "launched browser must receive XDG_ACTIVATION_TOKEN from the compositor (launched:\n{})",
        run.launched_raw
    );
    assert_eq!(
        run.fields.get("STDIN").map(String::as_str),
        Some("/dev/null"),
        "launched browser should have stdin detached to /dev/null (launched:\n{})",
        run.launched_raw
    );
}

#[test]
fn wayland_picker_falls_back_when_compositor_never_sends_token() {
    let run = run_picker_under_mock_compositor(false);
    let obs = &run.observation;

    assert!(
        obs.token_requested && obs.committed,
        "picker should still request a token before falling back (obs={obs:?})"
    );
    assert_eq!(
        run.fields.get("URL").map(String::as_str),
        Some(TEST_URL),
        "launched browser should still receive the target URL after timeout"
    );
    assert_eq!(
        run.fields.get("TOKEN").map(String::as_str),
        Some(""),
        "XDG_ACTIVATION_TOKEN should remain unset when the compositor never replies"
    );
}
