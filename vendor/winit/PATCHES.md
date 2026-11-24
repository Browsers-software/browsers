# Browsers patches to winit 0.30.13

Base: crates.io `winit` 0.30.13. Only the Wayland backend differs.

1. `seat/pointer/mod.rs`, `seat/keyboard/mod.rs`: record `(serial, seat)` on each pointer
   press/release and key event into the window's `activation_input`.
2. `window/state.rs`: new `activation_input: Option<(u32, WlSeat)>` field.
3. `window/mod.rs` `request_activation_token`: call `set_serial` with the recorded input before
   `set_surface` and `commit`, so Niri treats the token as user-initiated focus rather than an
   urgency hint.
4. `seat/mod.rs` `remove_seat`: clear `activation_input` when its seat is removed.

Upstream: fold these into winit's Wayland backend, then delete `vendor/winit` and the
`[patch.crates-io]` entry in `Cargo.toml`.
