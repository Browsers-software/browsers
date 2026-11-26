// Wayland doesn't let an app take focus on its own.
// The picker asks the compositor for an xdg-activation token and passes it to the browser it opens,
// which uses it to bring its window forward.

use std::time::Duration;

use slint::ComponentHandle;
use slint::winit_030::winit::event::WindowEvent;
use slint::winit_030::winit::event_loop::AsyncRequestSerial;
use slint::winit_030::winit::platform::startup_notify::WindowExtStartupNotify;
use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use slint::winit_030::{EventResult, WinitWindowAccessor};

use crate::MessageToMain;
use crate::gui::app::SharedState;
use crate::gui::generated::MainWindow;

const TOKEN_TIMEOUT: Duration = Duration::from_millis(500);

// a link waiting for its activation token before it can be opened
pub(super) struct PendingLink {
    token_request: AsyncRequestSerial,
    profile_index: usize,
    incognito_mode: bool,
    url: String,
}

// opens the pending link when the compositor replies with its token
pub(super) fn watch_for_tokens(state: &SharedState) {
    let win = state.borrow().main_window.clone_strong();
    let state = state.clone();
    win.window().on_winit_window_event(move |_, event| {
        if let WindowEvent::ActivationTokenDone { serial, token } = event {
            let token = token.clone().into_raw();
            open_pending_link(&state, *serial, (!token.is_empty()).then_some(token));
        }
        EventResult::Propagate
    });
}

// returns true if the link is taken care of here: it opens once its activation token arrives.
// False if no token is coming (turned off in Settings, or e.g. X11),
// so the caller should open it right away.
pub(super) fn defer_open_link(state: &SharedState, profile_index: usize) -> bool {
    let st = state.borrow();
    if !st.ui_settings.behavioral_settings.wayland_activation {
        return false;
    }
    // ignore further clicks while a link is waiting
    if st.pending_link.is_some() {
        return true;
    }
    let Some(token_request) = request_token(&st.main_window) else {
        return false;
    };
    let link = PendingLink {
        token_request,
        profile_index,
        incognito_mode: st.main_window.get_incognito_mode(),
        url: st.url.clone(),
    };
    drop(st);
    state.borrow_mut().pending_link = Some(link);

    // don't leave the link unopened if the compositor never replies
    let state = state.clone();
    slint::Timer::single_shot(TOKEN_TIMEOUT, move || {
        open_pending_link(&state, token_request, None);
    });
    true
}

// asks the compositor for a token. It arrives later, with the returned serial.
// None if no token is coming.
fn request_token(win: &MainWindow) -> Option<AsyncRequestSerial> {
    win.window()
        .with_winit_window(|window| {
            let handle = window.window_handle().map(|handle| handle.as_raw());
            if !matches!(handle, Ok(RawWindowHandle::Wayland(_))) {
                return None;
            }
            window
                .request_activation_token()
                .inspect_err(|err| tracing::debug!("could not request an activation token: {err}"))
                .ok()
        })
        .flatten()
}

// opens the link that waited for this token request, if it still does:
// both the compositor's reply and the timeout call this, and the first one wins
fn open_pending_link(
    state: &SharedState,
    token_request: AsyncRequestSerial,
    token: Option<String>,
) {
    let mut st = state.borrow_mut();
    let Some(link) = st
        .pending_link
        .take_if(|link| link.token_request == token_request)
    else {
        return;
    };
    if token.is_none() {
        tracing::debug!("no activation token received, opening the link without one");
    }
    let _ = st.main_sender.send(MessageToMain::OpenLink(
        link.profile_index,
        link.incognito_mode,
        link.url,
        token,
    ));
}
