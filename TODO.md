# Druid → Slint migration status

This branch replaces the `druid` GUI with [Slint](https://github.com/slint-ui/slint).
`cargo build`/`cargo test` are green on macOS, and - as of this session -
`cargo check --target x86_64-pc-windows-gnu` and `cross check --target
x86_64-unknown-linux-gnu` (via the project's own locally-built Docker
images) both come back clean too, so all three platforms now genuinely
compile, not just "reviewed by eye". The app has been confirmed to launch
and render on macOS; specific interactive behaviors (always-on-top, context
menu click positions, Settings open/close, clipboard) are still
runtime-unconfirmed on any platform - see the numbered list below.

## Font rendering (reported by user, fix confirmed working)

User reported the Settings dialog's text looked "wobbly" - individual
letters within one word (e.g. "General") sitting at visibly different
baselines, not just a layout/alignment issue. A static screenshot confirmed
clean overall layout otherwise, ruling out the vertical-alignment/centering
theories considered first. This matched a documented upstream
characteristic of femtovg (Slint's default GPU renderer): no font hinting,
and glyphs blitted from an atlas with a plain bilinear filter - both known
sources of exactly this kind of small-text artifact (see slint-ui/slint
issues #5177, #6298, #6365, #10752 and discussion #10390 for the general
pattern, though none match this exact symptom precisely).
**Fix confirmed by the user**: added the `renderer-skia` Cargo feature
(macOS-only - see the comment on the `slint` dependency in `Cargo.toml`:
Skia needs a full native build or a matching prebuilt-binary release per
target, and it failed outright trying to build from source when
cross-checking for `x86_64-pc-windows-gnu`, no MSVC toolchain available for
that cross combination), and `main.rs` sets `SLINT_BACKEND=skia` by default
on macOS (overridable via the env var) so Slint delegates glyph rendering to
CoreText instead of its own atlas. If the same artifact ever shows up on
Windows/Linux, the fix there would need a *different* approach (Skia's
prebuilt binaries/build story differ per platform - see the Cargo.toml
comment), not just removing the macOS gate.

## Theme "Match System" not tracking the actual macOS setting (reported by
   user, fix unconfirmed)

User reported that picking "Match System" for the theme didn't reflect
their actual macOS Light/Dark setting. Root cause candidate: `Auto` theme
resolution used the separate `dark-light` crate (`dark_light::detect()`),
which reads `AppleInterfaceStyle` from `NSUserDefaults`'s "Apple Global
Domain" directly - a different mechanism than whatever Slint's own
std-widgets use to track the OS theme for `Switch`/`LineEdit` (which *do*
already correctly follow "Auto" per the earlier `ColorScheme` fix), so any
discrepancy between the two detection paths would show up exactly like
this: native widgets tracking the OS correctly, `AppPalette`-driven UI not.
**Fix**: dropped the `dark-light` dependency entirely and based
`resolve_palette`'s "Auto" branch on Slint's own `Palette.color-scheme`
instead (`theme::detect_system_is_dark`, reading
`main_window.global::<Palette>().get_color_scheme()` - left un-forced,
that property already tracks the OS live, which is exactly what makes
`Switch`/`LineEdit` follow it). This guarantees our custom `AppPalette` and
Slint's own native widget colors always agree, since they now come from the
exact same detection call. Needed `main_window.slint` to also
`import`+`export { Palette } from "std-widgets.slint";` (mirroring
`settings_window.slint`) even though the main window doesn't use any
std-widgets controls itself - purely to reach the global from Rust.
Compiles clean on macOS/Windows/Linux; **not yet visually confirmed** to
actually track macOS's Light/Dark switch correctly, since I still can't run
the GUI myself.

## Licensing

Slint is triple-licensed (GPL-3.0-only / Royalty-free / Commercial - your
choice, see https://slint.dev/legal/licensing). **Decision: this project
uses the Royalty-free license.** That keeps `browsers` itself under
MIT/Apache-2.0 as before (the Slint license only governs Slint, not code
that merely uses it), but requires attribution: either the `AboutSlint`
widget in an About dialog, or a badge on the public download page. Went
with the widget - it's now in `ui/about_window.slint` (`import { AboutSlint
} from "std-widgets.slint";` + `AboutSlint {}` at the bottom of the
window). **Do not remove it** without switching to GPL-3.0 or a commercial
license instead. Not yet visually confirmed to render correctly (window
height bumped 260px -> 320px to fit it, a guess at the widget's size).

## Done

- `Cargo.toml`: `druid` removed entirely; added `slint`, `slint-build`, `image`
  (direct dep, was previously pulled in via `druid::image`), `mouse_position`
  (Linux cursor position), `copypasta` (clipboard).
- `build.rs`: compiles `ui/main_window.slint`, `ui/settings_window.slint`,
  `ui/about_window.slint` via `slint_build::compile()`. Dropped the Fluent
  `.ftl` resource copy (localization removed, see below).
- New `.slint` UI, under `ui/`:
  - `theme.slint` — `AppPalette` global (named to avoid colliding with
    Slint's own built-in `Palette` global — that collision silently
    no-ops your bindings with **no compile error**, cost real time to
    find; see git history if this trips you up again).
  - `common.slint` — `Card`, `SectionLabel`, `LabeledTextField`, and a
    hand-rolled `ThemeRadioGroup`/`ThemeChoice` (Slint's std-widgets has no
    built-in `RadioButton`).
  - `main_window.slint` — the borderless popup: browser list, hover/focus,
    per-row context menu (`ContextMenuArea`/`Menu`), the "⋮" options menu,
    keyboard handling (Escape/Enter/Space/digits/Shift/Cmd,Cmd,).
  - `settings_window.slint` — sidebar-tabbed window (General/Appearance/
    Rules/Advanced), all four tabs ported.
  - `about_window.slint`.
  - **Important gotcha**: each `slint_build::compile()` call is a fully
    separate compilation unit. A struct/global merely `import`ed (not
    re-exported) into a file is invisible to Rust for that file - see the
    `export { AppPalette } from "theme.slint";` lines and the
    `generated::{main_window,settings_window,about_window}` submodule split
    in `src/gui/mod.rs`. `BrowserItem`/`RestorableProfile`/`DirEntry` exist as
    3 independent nominal Rust types (one per window that uses them) even
    though they look identical - conversions between them are manual (see
    `settings_window.rs`'s `to_lightweight_browser_item`, the `directories()`
    helper in `about_window.rs`).
- New Rust glue, under `src/gui/`:
  - `app_state.rs` - plain-data port of the old `UIState`/`UISettings`/
    `UIBrowser`/etc (no more druid `Data`/`Lens`).
  - `theme.rs` - palette resolution logic ported from `ui_theme.rs`
    (dark/light/auto/custom-from-hex), now pushes into the Slint
    `AppPalette` global per-window instead of a druid `Env`.
  - `icon_loader.rs` - replaces the druid `Controller`-based lazy image
    loader; decodes eagerly into `slint::Image` with a path-keyed cache.
  - `screen.rs` + platform natives (`macos_native.rs`,
    `windows_utils.rs::mouse_position_and_work_area`) - global mouse
    position + monitor work area, which neither Slint nor winit expose
    (previously a druid-shell private-fork feature). Linux has **no** work
    area lookup (see Known gaps).
  - `app.rs` - `AppState`/`SharedState` (`Rc<RefCell<..>>`, single-threaded,
    replaces the old `UIDelegate` command-bus) + `UiHandle` (replaces
    `druid::ExtEventSink` for the background "main" thread to push updates:
    new browsers list, cleaned-url-opened, etc - via
    `Weak::upgrade_in_event_loop` + a thread-local, see the comment on
    `APP_STATE` in `app.rs` for why it's done that way instead of just
    capturing `Rc` in the cross-thread closure).
  - `main_window.rs` - pure geometry math ported verbatim
    (`calculate_window_position`/`recalculate_window_size`/etc) +
    `UIBrowser -> BrowserItem` conversion.
  - `settings_window.rs`, `about_window.rs` - build+wire those windows on
    demand, singleton-per-app (opening twice just re-shows/refreshes).
- `src/lib.rs`: `handle_messages_to_main` now takes `UiHandle` instead of
  `ExtEventSink`; added `pub struct UrlOpenInfo` (was `druid::UrlOpenInfo`);
  `prepare_ui` now returns a plain `PreparedUi` struct instead of the old
  druid-flavored `UI` type.
- `src/main.rs`: builds `AppState`/`UiHandle` instead of an `AppLauncher`;
  on macOS, installs `macos_native::install_event_bridge` (see below).
- `src/macos/macos_native.rs`: added
  - `mouse_position_and_work_area()` (`NSEvent`/`NSScreen`).
  - `make_window_floating()` (NSWindow level + collection behavior), now
    called once from `gui::app::run()` right after the main window's first
    `.show()`, via `raw-window-handle` 0.6 (`slint::Window::window_handle()`
    -> `RawWindowHandle::AppKit(..).ns_view`). Compiles; not yet visually
    confirmed to actually float above other apps/across Spaces.
  - `EventBridge` (`objc2::define_class!`) + `install_event_bridge()`: a
    *separate* NSObject (not the app delegate, to avoid fighting winit for
    that slot) that (a) observes `NSApplicationDidResignActiveNotification`
    for `quit_on_lost_focus`, and (b) registers an `NSAppleEventManager`
    handler for the `GURL` Apple Event, i.e. the "open this URL" event macOS
    sends a registered default-browser app - this is what makes
    `quit_on_lost_focus` and being-the-default-browser-that-receives-URLs
    work at all on macOS. **Untested** - never actually exercised at
    runtime this session.
- `src/windows/windows_utils.rs`: added `mouse_position_and_work_area()`
  (`GetCursorPos`/`MonitorFromPoint`/`GetMonitorInfoW`) and
  `hide_from_taskbar()` (`WS_EX_TOOLWINDOW`) - **neither is called from
  anywhere yet** (see Known gaps), and none of it has been compiled on
  Windows, only reviewed by eye.
- Localization dropped entirely (was Fluent/`.ftl`, only `en-US` existed
  anyway): the ~8 strings (context menu items, tab titles) are now plain
  Rust string literals. `resources/i18n/` and the now-dead
  `paths::get_localizations_basedir()` have been deleted.
- `linux_utils.rs`/`windows_utils.rs`: `use druid::image::...` → `use
  image::...` (druid used to re-export the `image` crate).
- `utils.rs`: dropped `Data`/`Lens` derives.
- **Custom palette feature removed** (per explicit request, not part of the
  original port): `ConfiguredTheme::Custom` and the whole `CustomPalette`
  struct/config field are gone from `utils.rs`; the Appearance tab is back
  to just Auto/Light/Dark. Removed the matching Slint side too -
  `CustomPaletteFields`, `LabeledTextField`, `SectionLabel` (all now dead)
  from `common.slint`/`settings_window.slint`, and the `custom` variant
  from `ThemeChoice`/`ThemeRadioGroup`. Existing `config.json` files with
  `"theme": "Custom"` will fail to deserialize that enum value; `Config`
  loading falls back to defaults in that case (not specifically verified
  this session, but pre-existing behavior for any config parse failure).

## Fixed: main popup window was draggable-resizable despite being borderless

User reported dragging the *main popup's* left edge caused rendering
artifacts on the right edge. Checked Slint's issue tracker before touching
anything: this matches a known, still-**open** upstream bug -
[slint-ui/slint#3990](https://github.com/slint-ui/slint/issues/3990),
"Artifacts when resizing window", open since Slint 1.3.0, reproduced on
**both femtovg and Skia** by the original reporter - not something fixable
from application code. Rather than chase it, removed the ability to
trigger it: initially assumed `no-frame: true` meant the popup was already
non-resizable, but confirmed otherwise by reading winit 0.30.13's actual
window-creation source
(`platform_impl/macos/window_delegate.rs`) - for a decorations-disabled
window, winit's default style mask is
`NSWindowStyleMask::Borderless | Resizable | Miniaturizable` *unless* the
window was explicitly created with `resizable: false`. Slint's `.slint`
language has no `resizable` window property to request that (checked the
full `Window` property list), so every Slint window - borderless or not -
gets `NSWindowStyleMask::Resizable` by default. `resize-border-width: 0px`
(added earlier this session for a different complaint - a resize cursor
showing near the "⋮" button) only hides Slint's own resize-cursor *hint
region*, a separate, higher-level mechanism from the OS-level style mask
that actually lets the user grab an edge - so it never touched the real
cause. Fixed with `winit_window.set_resizable(false)` (confirmed against
winit's source: this clears the exact same `NSWindowStyleMask::Resizable`
bit), reached via the same `WinitWindowAccessor`/`winit_window()` async
pattern already used for always-on-top and the theme re-apply below -
folded into the same `spawn_local` task rather than adding a second one -
in `gui::app::run()`. Compiles clean on macOS/Windows/Linux, runs without
warnings; **not yet confirmed live** that dragging the edge no longer does
anything (no real display available this session to test the actual
mouse-drag interaction).

## Fixed: brief light-then-dark flash on launch

User confirmed the dark-mode-on-launch fix above works, but reported a
"for subsecond period I can see it opening with light theme and then
quickly switching to dark theme" - the visible version of that fix's own
trade-off (the eager `apply_theme()` call still paints a guess before the
async re-apply corrects it). Initially deferred fixing this outright,
reasoning that a second synchronous native theme-detection codepath would
risk reintroducing the exact two-mechanisms-disagreeing bug that got the
`dark-light` crate removed - user pushed back and asked for actual
investigation instead of a judgment call to leave it. Re-examined: the
`dark-light` bug was about *ongoing* tracking diverging from Slint's own
mechanism, not about a one-time startup guess, and confirmed
`NSApplication.sharedApplication` is already used successfully
synchronously before any window exists (`hide_dock_icon`, called first
thing in `main()`) - so a native check for *just* the first paint carries
none of the original risk, since Slint's own tracking still takes over
immediately once it's ready and remains the sole source of truth for
everything after that. Added `macos_native::is_dark_mode()`
(`NSApplication.effectiveAppearance()`, the same property AppKit itself
uses to theme native controls - checking whether its name contains "Dark"
covers both the standard and accessibility/high-contrast dark variants).
`theme::detect_system_is_dark` now only falls back to it when Slint's own
`Palette.color-scheme` reports `Unknown` (the pre-window-exists state) -
once Slint reports a real `Dark`/`Light` answer, that's used as-is, so
there's exactly one live source of truth once the app is actually running.
Compiles clean on macOS/Windows/Linux (non-macOS keeps the old
"default to light while unknown" behavior, since there's no native
fallback to call there), runs without warnings; **not yet confirmed live**
that the flash is actually gone (needs a real launch to watch for it).

## Second polish batch: overlap bug + root cause, dark-mode-on-launch, more alignment fixes

- **Real bug, found and root-caused: "Restore App..." button's bottom
  border overlapped the hint text below it.** Root cause: the "Position
  context menus at the actual click point" commit (`c1a7095`) nested each
  button's `TouchArea` *inside* its `ContextMenuArea` (needed so
  `mouse-x`/`mouse-y` share a coordinate space with `.show()`), which
  moved the actual VerticalLayout child from the size-carrying `TouchArea`
  to the `ContextMenuArea` wrapping it. `ContextMenuArea` doesn't forward
  its visual content's size to whatever layout contains it, so the
  enclosing `VerticalLayout` started reserving ~0 height for that row,
  while the button itself kept rendering at its own explicit 26px height -
  overflowing into the next element. Fixed by adding explicit
  `width`/`height` directly on the `ContextMenuArea` (`ram`) matching its
  child. Checked the other two `ContextMenuArea` usages (main window's "⋮"
  options button, the Rules-tab profile-picker dropdown) via screenshot -
  neither is actually affected in practice (different layout contexts),
  left them alone rather than applying a speculative fix to something not
  demonstrated broken.
- **Real bug: app launches in light mode even with "Match System" selected
  and macOS in dark mode, self-correcting only once Settings is opened.**
  Same root cause as the always-on-top/skip-taskbar timing issue from
  earlier in this file: the first `apply_theme()` call (in `app::new()`,
  before `.show()`) reads `Palette.color-scheme` before the winit
  backend's OS dark/light watcher is wired up, so "Auto" mode can read a
  stale default. `settings_window.rs::open()` happens to call
  `apply_theme()` again later, by which point the backend has caught up -
  which is why opening Settings "fixed" the main window's colors too.
  Fixed the same way as the window-handle race: `gui::app::run()` now also
  `spawn_local`s an async task awaiting `winit_window()` (via
  `slint::winit_030::WinitWindowAccessor`) and re-applies the theme once
  the native window actually exists, on top of the existing eager call
  (kept so there's at least an approximate theme immediately rather than
  no styling at all before this resolves). Compiles clean; **not yet
  confirmed live** (needs a real launch with macOS in dark mode and
  "Match System" selected - not observable via the headless MCP backend).
- **"☰ List of Apps" showed a tofu box instead of the ☰ glyph** with the
  active font. Removed the glyph entirely (now plain "List of Apps"),
  matching the plain-text style of every other menu item in the app -
  simpler than chasing font coverage for one character.
- **Rules-tab profile picker shifted sideways whenever the incognito
  checkbox appeared/disappeared** (depends on whether the newly-picked
  browser supports incognito). `ProfilePicker`'s `HorizontalLayout` used
  the default `stretch` alignment, which redistributes space between the
  dropdown and the checkbox whenever the checkbox's presence changes -
  same stretch-redistribution mechanism as the hotkey-alignment bug, just
  causing movement instead of full collapse. Fixed with the user's
  suggested simpler option: `alignment: start;` so both pack to their
  natural size regardless of the checkbox's presence.
- **Settings sidebar/theme radio labels were centered, not left-aligned.**
  `ThemeRadioRow` (`common.slint`)'s label `TouchArea` had no explicit
  width, so it stretched to fill the row by default and the `Text` inside
  centered itself within that stretched box - same class of bug as
  above. Added `alignment: start;` to `ThemeRadioRow`.
- **About dialog text made selectable, and its website link made
  clickable**, per request. Replaced the title/version/copyright `Text`
  elements with `TextInput { read-only: true; }` (Slint's documented
  idiom for selectable-but-not-editable text - plain `Text` has no
  built-in selection). Split "Visit us at https://browsers.software."
  into its own line as a distinct clickable link (`TouchArea` + `Text`,
  `mouse-cursor: pointer`, accent-color on hover) rather than also making
  it a `TextInput`, since combining click-to-open with drag-to-select in
  one element is awkward - clicking a short link to open it is the more
  useful behavior than copying its text character-by-character. Added
  `about_window::open_website()` (`std::process::Command`, `open`/
  `xdg-open`/`cmd start` per platform) - deliberately not routed through
  this app's own browser-picker logic, which would pop up another
  instance of the picker on top of the About window.
- **Settings → Advanced: only the directory *path* should be selectable,
  not the label** ("Config"/"Cache"/etc.), per explicit clarification.
  Applied to both Settings → Advanced and the About window's matching
  directories list for consistency (the About window's labels had
  initially been made selectable too, in the same pass as the title/
  version/copyright text above - reverted just the label column back to
  plain `Text` once the narrower scope was clarified).

All verified: `slint-viewer --check` clean on every file, screenshot
comparisons for each visual fix, `cargo build`/`test`/`fmt`/`clippy`
clean (no new warnings), `cargo check --target x86_64-pc-windows-gnu` and
`cross check --target x86_64-unknown-linux-gnu` both clean.

## UI polish batch: settings sidebar/buttons, Rules-tab height, About centering, resizing, resize-cursor

A round of concrete visual requests, verified with `slint-viewer`/MCP rather
than guessed:

- **Settings sidebar tab labels** ("General"/"Appearance"/"Rules"/
  "Advanced") were implicitly horizontally centered (no explicit `x`, and
  Slint centers un-positioned elements within their parent by default).
  Added `x: 12px;` to each - confirmed via screenshot they're now
  left-aligned, vertical centering (untouched) still intact.
- **"Restore App..." button text** had the same implicit-centering issue;
  same fix (`x: 8px;` instead of `horizontal-alignment: center`).
- **Real bug, same root cause as the hotkey-alignment fix above: the Rules
  tab's content didn't fill the available dialog height.** `RulesTab`
  (a `VerticalLayout`) had `alignment: start;`, which - per Slint's stretch
  algorithm - pins every child, including
  `ScrollView { vertical-stretch: 1; }`, to its minimum/preferred size
  instead of letting stretch distribute leftover space. This is the same
  bug class as the hotkey badge, just in a different component -
  I'd previously (incorrectly) written off the visual gap below the rule
  list as just alpha-blending against the headless viewer's white canvas;
  that explanation was only actually true for the *outer* SettingsWindow
  screenshot's background shade, and I'd wrongly generalized it to also
  explain the *isolated* RulesTab probe screenshot, which was showing a
  real "packed to the top" layout bug the whole time. Removed
  `alignment: start` from `RulesTab`; confirmed via an isolated
  `slint-viewer --screenshot --component RulesTab` render with an explicit
  460px height that the rule list's `ScrollView` now actually expands to
  fill it, pushing "Add Rule" to the bottom instead of sitting right below
  a short list with a dead gap underneath.
- **"Set Browsers as a Default Browser" / "Refresh Installed Applications"**
  were plain clickable text (`TouchArea` + `Text`, no chrome). Restyled as
  bordered, content-sized buttons matching the existing "Add Rule"/"Restore
  App..." button pattern (border + border-radius + hover background),
  confirmed via screenshot.
- **Settings window made resizable.** It used literal `width`/`height`
  bindings, which Slint treats as a fixed size (window manager can't
  resize it). Switched to `preferred-width`/`preferred-height` (sets the
  initial size only) plus `min-width: 500px; min-height: 350px;` so it
  can't be shrunk into an unusable layout.
- **About window content re-centered.** The icon/title/version/copyright
  block (icon through "Visit us at...") was left-aligned because the
  outer `VerticalLayout`'s cross-axis-alignment defaulted to `stretch`,
  which pins fixed-size children (the icon) to the start of the cross
  axis instead of centering them. Added `cross-axis-alignment: center;`
  plus explicit `horizontal-alignment: center;` on the multi-line text
  blocks; confirmed via screenshot that the directories table and the
  `AboutSlint` attribution widget below (explicitly called out as
  "already fine") are unaffected/still look correct.
- **Main popup's bottom-right corner showed a resize cursor over the "⋮"
  button.** Borderless (`no-frame: true`) windows get an invisible
  resize-hit-region near the edges by default on the winit backend, even
  though this popup is never actually resizable (Rust sizes it
  programmatically). Set `resize-border-width: 0px;` on `MainWindow` to
  disable it.
- **Dock icon fix carried over from the previous batch** (`hide_dock_icon`
  via `NSApplication::setActivationPolicy`) - see below; grouped here only
  because it was raised and fixed in the same conversation, not because it
  shares a root cause with the layout items above.

**Not done - genuinely ambiguous, and the user asked not to be asked
further questions this session:** "add the slint themes in the theme
selection" could mean several different things (expose Slint's
compile-time widget *styles* - fluent/material/cupertino/native - as a
runtime option, which doesn't fit how Slint styles actually work since
they're selected at compile time, not swappable per-session; or something
about the existing Auto/Light/Dark `ThemeChoice`, which already forces
Slint's own `Palette.color-scheme`). Left as-is pending clarification next
session rather than guess and risk building the wrong thing.

All verified: `slint-viewer --check` clean on every `.slint` file, `cargo
build`/`test`/`fmt` clean, `cargo check --target x86_64-pc-windows-gnu` and
`cross check --target x86_64-unknown-linux-gnu` both clean (only
pre-existing, unrelated warnings).

## Main window row polish: hotkey alignment, vertical centering, profile icon, Dock icon

User asked to right-align hotkey badges into a single vertical column, then
reported "the badges still don't line up" after an initial attempted fix.
Investigated with real evidence instead of re-guessing - used the Slint MCP
server's `get_element_tree` against the actual running app (not just a
`slint-viewer` screenshot) to read every hotkey badge's exact absolute `x`
coordinate. Found the real root cause: `BrowserRow`'s `HorizontalLayout` had
`alignment: start;`, which (per Slint's own layout docs) pins *every* child
to its minimum size instead of distributing leftover space by stretch
factor - so the `Rectangle { horizontal-stretch: 1; }` spacer meant to push
the badge to the right edge was silently rendered at 0 width, and the
badge's position tracked each row's browser/profile name length instead of
lining up. (A separate, real but insufficient-alone fix was already needed
and applied first: `BrowserRow` had no explicit `width`, so inside
`ListView`'s `Flickable` viewport - not itself a layout - each row sized to
its own content instead of the list's full width; added `width: 100%;`.)
Removed `alignment: start` entirely (default is `stretch`). Verified via
the MCP element tree against real installed-browser data: every badge now
sits at the exact same `x` across all 7 rows.

Same MCP inspection also confirmed two more fixes, both requested in the
same pass:
- Hotkey badges were pinned to the *top* of each 32px-tall row (badge `y`
  equalled row `y`) instead of vertically centered, since the row's
  `HorizontalLayout` cross-axis-alignment defaulted to `stretch` and a
  fixed-height (18px) child can't stretch, so it just sat at the start.
  Added `cross-axis-alignment: center;` - confirmed each badge's `y` now
  equals `row_y + (32 - 18) / 2` exactly, for every row.
- The profile-icon overlay (16x16, on top of the 32x32 browser icon) moved
  from top-left (`x: 1px; y: 1px;`) to bottom-right
  (`x: 32px - self.width - 1px; y: 32px - self.height - 1px;`) - confirmed
  via the same element tree (e.g. icon at `(9, 5)` size 32 -> overlay at
  exactly `(24, 20)`, matching the formula).

Also (unrelated to layout, raised by the user testing the dev binary):
running the raw binary directly (`cargo run`/`./target/debug/browsers`)
showed a generic Dock icon, even though `extra/macos/Info.plist` sets
`LSUIElement`. Root cause: `LSUIElement` only takes effect once packaged
into a real `.app` bundle (via `build-mac.sh`) - an unbundled process has
no `Info.plist` for macOS to read. Confirmed the old druid-based `main`
branch avoids this a different way: its private `druid-shell` fork
(`browsers-software/druid`, `backend/mac/application.rs`) calls
`NSApp().setActivationPolicy_(NSApplicationActivationPolicyAccessory)`
directly from `applicationDidFinishLaunching:`, which hides the Dock icon
regardless of bundling. Added the equivalent
(`macos_native::hide_dock_icon`, called from the top of `main()`) using
`objc2_app_kit::NSApplication::setActivationPolicy` - a real public API,
no workaround needed. Both mechanisms are now in place (Info.plist for the
packaged app, activation policy for the raw binary too), so the icon
should be hidden either way.

All four fixes verified: `slint-viewer --check` clean, `cargo
build`/`test`/`fmt` clean, `cargo check --target x86_64-pc-windows-gnu` and
`cross check --target x86_64-unknown-linux-gnu` both clean (no new
warnings), and the hotkey/centering/profile-icon fixes independently
confirmed via exact coordinates from the Slint MCP server against the real
running app with real installed-browser data - not just a visual
screenshot guess. `hide_dock_icon` itself is not yet visually confirmed
(needs a real, non-headless macOS run - the headless MCP backend used for
verification doesn't create a real NSApplication/Dock presence to check
against).

## Always-on-top/skip-taskbar: dropped, then reinstated on a cleaner path

Follow-up to the audit below. User pushed back hard on the raw-window-handle
retry loop ("why the fuck do we need that... i dont think we need it").
Investigated for real instead of just defending it: instrumented the code,
ran it on a real display, and confirmed `make_main_window_floating` (called
synchronously right after `.show()`, before `run_event_loop()` even starts)
**always** fails on attempt 0 and succeeds on attempt 1, 16ms later, once
the event loop is actually running - not a flaky timing issue, a structural
one. Gave the user three options (keep as-is, simplify to one retry, drop
entirely); **user chose to drop it entirely**, relying solely on
`main_window.slint`'s declarative `always-on-top: true`. Removed
`make_main_window_floating`/`hide_main_window_from_taskbar` and their retry
machinery from `app.rs`, `macos_native::make_window_floating`,
`windows_utils::hide_from_taskbar`, and the `raw-window-handle` dependency
entirely.

User then found (via Slint's own GitHub discussions #8036 and
#4284#discussioncomment-8077128) that Slint's own maintainers point people
at `i_slint_backend_winit::WinitWindowAccessor` (reachable as
`slint::winit_030::{WinitWindowAccessor, winit}`, feature
`unstable-winit-030`) for exactly this kind of native-handle need - still
explicitly called "private API"/"no stable API yet" by the maintainers
themselves, but meaningfully cleaner than what we had:
- Exposes `winit_window()` as an **async future** that resolves once the
  native window actually exists, replacing the hand-rolled
  `Timer`-based polling/magic-constants retry with `slint::spawn_local`
  awaiting that future - a real "wait until ready" primitive instead of a
  guess at interval/attempt-count.
- On Windows, winit itself ships `WindowExtWindows::set_skip_taskbar(bool)`
  - a real, documented winit API - so `hide_main_window_from_taskbar` no
  longer needs any raw `HWND`/`WS_EX_TOOLWINDOW` code at all.
- On macOS, no improvement was available: winit's `platform::macos`
  extension trait has no window-level/collection-behavior hook (confirmed
  by reading winit 0.30.13's actual source - a user-suggested
  `WindowExtMacOS::ns_window()` from an old docs mirror doesn't exist in
  this version; it was removed once winit standardized on
  `raw-window-handle`). `make_window_floating` still does the same
  `NSWindow.setLevel`/`setCollectionBehavior(FullScreenAuxiliary)` dance via
  `objc2`, just reached through winit's own `HasWindowHandle` impl instead
  of Slint's.
User said "use it" - reinstated both functions on this path. Verified: (a)
compiles clean on macOS/Windows/Linux (`cargo build`, `cargo check --target
x86_64-pc-windows-gnu`, `cross check --target x86_64-unknown-linux-gnu`),
(b) ran on a real macOS display and confirmed no "could not obtain the
native window" warning is logged (the async path resolves silently on the
first attempt, as expected), (c) `cargo test`/`fmt`/`clippy` clean, no new
warnings vs. before this change. Windows' `set_skip_taskbar` path itself is
still unverified at runtime (no Windows machine available this session).

## Audit of remaining "hacks" (per user request) - two removed, rest justified

Went through every workaround/unstable-API/native-platform-code spot added
during this migration and checked whether Slint now offers (or always
offered) a cleaner alternative. Two were real, fixed; the rest are
confirmed to have no better option:

- **Fixed: `theme::ColorScheme` now uses the stable
  `slint::language::ColorScheme`** instead of
  `slint::private_unstable_api::re_exports::ColorScheme`. Checked the
  `slint` crate's own source: `slint::language` re-exports every builtin
  `.slint` enum (including `ColorScheme`) from `i-slint-core` via a stable,
  public path - no reason to go through the unstable one.
- **Fixed: `main.rs` no longer does `unsafe { env::set_var("SLINT_BACKEND",
  "skia") }`** to force the Skia renderer on macOS. Slint's own
  `BackendSelector` (`i_slint_backend_selector`, re-exported as
  `slint::BackendSelector`) is the documented "programmatic substitute for
  the `SLINT_BACKEND` environment variable" - `BackendSelector::new()
  .renderer_name("skia".to_string()).select()`. Still gated behind
  `env::var_os("SLINT_BACKEND").is_none()` so `SLINT_BACKEND=headless` (used
  for the MCP server) and any manual override keep working exactly as
  before - reran the MCP server with it set and confirmed `list_windows`
  still responds.
  (**Superseded** - see the section above this one: the
  `Timer::single_shot` retry loop this bullet defended was subsequently
  found to be unnecessary complexity, dropped, and rebuilt on
  `slint::winit_030::WinitWindowAccessor`'s async `winit_window()` instead
  of a hand-rolled retry.)
- **Kept, confirmed no better option exists:**
  - Native macOS (`NSWindow.setLevel`/`setCollectionBehavior`) code for
    always-on-top refinement - re-confirmed against Slint's complete `Window` property
    list (`always-on-top`, `full-screen`, `minimized`, `maximized`,
    `background`, `default-font-*`, `icon`, `no-frame`,
    `resize-border-width`, `title`, `safe-area-insets`,
    `virtual-keyboard-*`): none of it covers per-Space visibility or
    full-screen-auxiliary behavior, and winit itself has no macOS extension
    for either (checked `platform::macos::WindowExtMacOS`'s actual source -
    fullscreen/shadow/tabbing/edit-state only). Native `objc2` code is
    required either way. (Windows taskbar-hiding turned out to have a real
    winit API after all - `WindowExtWindows::set_skip_taskbar` - see the
    section above.)
  - `export { Palette } from "std-widgets.slint";` (now centralized in
    `ui/app.slint`, see below) purely to read the OS color scheme from
    Rust, despite no std-widgets controls actually being used - confirmed
    via Slint's own docs ("Use Palette's `color-scheme` to determine the
    currently used scheme") that this is the documented, intended
    mechanism, not a workaround.
- **Fixed, this was actually solvable:** cross-file struct duplication
  (`BrowserItem`/`RestorableProfile`/`DirEntry` used to be 3 separate
  nominal Rust types, one per independently-compiled `.slint` file).
  Assumed unfixable without losing file separation - wrong. Checked
  `slint-ui/slint`'s own examples (`examples/system-tray`) and found the
  actual pattern: keep the 3 files separate, add one thin root
  (`ui/app.slint`) that does `export { MainWindow, BrowserItem,
  RestorableProfile } from "main_window.slint";` (and similarly for the
  other two), compile only that root in `build.rs`, and use a single
  `slint::include_modules!()` in `gui/mod.rs` instead of the hand-rolled
  triple `include!()`. One compile unit, `BrowserItem`/`DirEntry`/etc.
  are single shared types now, `settings_window.rs` and `about_window.rs`
  no longer duplicate the `DirEntry` mapping - and each window still gets
  its own independent `Palette`/`AppPalette` globals (confirmed via the
  same system-tray example: separate top-level exported components don't
  share global instances even compiled together), so `apply_palette!`
  still needs to be called per window exactly as before.

## macOS: popup no longer follows you to every Space

`macos_native::make_window_floating` originally set
`NSWindowCollectionBehavior::CanJoinAllSpaces` (in addition to
`FullScreenAuxiliary`) so the always-on-top popup could show up even while
a full-screen app occupied the current Space. Per user feedback, that also
had the side effect of making the popup a member of *every* Space rather
than just the one it was triggered from. Removed `CanJoinAllSpaces`,
keeping only `FullScreenAuxiliary` - the popup now only lives on the
current Space (matching normal window behavior) while still being able to
appear over a full-screen app on that same Space. Compiles; **not yet
confirmed on a real desktop** (would need triggering the popup, switching
Spaces, and confirming it's no longer visible on the new one - not
observable via the headless MCP backend since that doesn't create a real
`NSWindow`).

## Idiomatic-Slint review pass (headless screenshot tooling now available)

Installed `slint-viewer` (`cargo install slint-viewer`) and used
`--check`/`--screenshot`/`--load-data` to actually render the `.slint` files
for the first time this session, instead of reasoning about layout blind.
All 4 real component files (`common.slint`, `main_window.slint`,
`settings_window.slint`, `about_window.slint`) compile with zero warnings.
Reviewed against the official `slint` Claude Code plugin's guidance
(installed by the user this session) and fixed two real anti-patterns:

- **String concatenation → interpolation.** `main_window.slint` (context
  menu item titles: "Move "+name+" to Top" etc, "Hide "+name) and
  `settings_window.slint` (profile-picker label) used `+` string
  concatenation throughout instead of `"\{expr}"` interpolation. Cosmetic
  only (both compile to the same thing) but non-idiomatic; fixed all
  instances.
- **Real bug, now fixed: full list-model replacement caused Rules-tab
  `LineEdit`s to lose cursor position on every keystroke.** `app.rs` and
  `settings_window.rs` rebuilt every list-valued property from scratch on
  every refresh (`win.set_rules(ModelRc::new(VecModel::from(items)))`, same
  for `browsers`/`restorable_app_profiles`/`directories`). Since
  `settings_window.rs::wire`'s `on_rule_url_changed` (bound to the Rules
  tab's URL-pattern `LineEdit.edited`, which fires on *every character
  typed*) calls `save_and_refresh` → `refresh()` on each keystroke, this
  meant the entire `rules` `ModelRc` - and therefore every `RuleCard`
  component instance in the `for` loop, including the very `LineEdit` the
  user was actively typing into - was torn down and recreated after each
  character, resetting its cursor/selection state. This is exactly the
  anti-pattern the `slint` plugin's own guidance calls out: "keep an
  `Rc<VecModel<T>>` and mutate it in place... instead of replacing the
  whole model." Fixed by seeding each list property with a real
  `VecModel::default()` once at window creation, and adding
  `gui::ui_util::sync_vec_model()` (downcasts the existing `ModelRc` back to
  `VecModel`, then does `set_row_data`/`push`/`remove` to match the new
  `Vec` instead of replacing the model) - used for `rules`, `browsers`,
  `restorable_app_profiles`, `directories` in `settings_window.rs` and
  `browsers`/`restorable_app_profiles` in `app.rs`. (`about_window.rs`'s
  `directories` is set once at window creation and never refreshed, so it
  was left as a one-shot `ModelRc::new(VecModel::from(...))` - there's no
  live-update case to fix there.) Verified via `cargo build`/`test`/`fmt`,
  `cargo check --target x86_64-pc-windows-gnu`, and `cross check --target
  x86_64-unknown-linux-gnu`, all clean. **Not yet confirmed against the
  real running app** (would need the Slint MCP server or the user typing
  into a Rules-tab URL field) that cursor position is in fact now stable -
  the fix addresses a mechanism confirmed present in the code, but the
  originally-reported "off" feeling in the Rules tab was never precisely
  pinned down by the user, so treat this as a strong, well-evidenced
  candidate fix rather than a user-confirmed one.
- Also screenshotted `settings_window.slint`'s Rules tab with fake data to
  sanity-check layout after the change: initially looked like the rule list
  wasn't filling available vertical space (`ScrollView { vertical-stretch:
  1; }` inside `RulesTab`), but isolating the component confirmed this is
  just `AppPalette.background-color`'s alpha channel (`#262626e6`, ~90%
  opaque) blending against the headless viewer's plain white canvas instead
  of a real desktop/window backdrop - not a layout bug. Real layout is
  fine.
- Not acted on (judged lower-value/riskier for the remaining time): no
  hover-state transition animations (`polish.md` suggests 150-200ms on
  `has-hover`-driven backgrounds - currently instant), `px` used everywhere
  for font sizes instead of `rem`, and the `export global` interop pattern
  for cross-file shared state (not really applicable here since structs
  already need per-file re-export/conversion given the multi-window
  architecture - a global wouldn't avoid that).

## Second review pass (no runtime testing available, found by reading code)

- **Real bug: row hover never worked via plain mouse movement.** The
  `TouchArea` in `BrowserRow` (`main_window.slint`) used the `moved`
  callback to call `set-focused` (drives hover highlight + which row
  Enter/Space activates). `moved` only fires while the mouse button is
  *held down* (dragging) - confirmed against Slint's own source
  (`i-slint-core`'s `MouseEvent::Moved` handler only calls `moved()` when
  `grabbed`). So hovering a row with the mouse up would never have
  highlighted it or moved keyboard focus there - only click-dragging
  would have. Fixed by switching to `pointer-event(event)` and checking
  `event.kind == PointerEventKind.move`, which Slint fires unconditionally
  on any pointer movement over the area.
- **Real bug: `extra_windows_open` could over-count.** In both
  `settings_window.rs` and `about_window.rs`, `open()`'s "already exists,
  just re-show" fast path incremented `extra_windows_open` unconditionally
  - so triggering "Settings..." (or About) again while that window was
  *already visible* (e.g. clicking the menu item twice) inflated the
  counter without a matching close to bring it back down, permanently
  breaking the `quit_on_lost_focus_applies()` guard for the rest of the
  process. Fixed by only incrementing when `!win.window().is_visible()`.
- **Robustness: `focused-index` could go out of range.** If the filtered
  browser list shrinks (hide/restore/move, or the url changing) while
  `focused-index` pointed past the new end, it stayed stuck there instead
  of clamping back into range. `refresh_main_window_model` now clamps it
  to `[0, browser_count - 1]` (or resets to `0` when the list is empty)
  whenever the model is rebuilt.
- **Minor, not fixed: digit hotkeys are layout-dependent.** The old druid
  code matched `Code::Digit1`/`Code::Numpad1` (physical key position,
  layout-independent). Slint's `.slint`-language `key-pressed` only
  exposes `event.text` (the actual character produced), which the
  language itself has no physical-keycode escape hatch for - so on a
  non-QWERTY layout where the physical "1" key doesn't produce the "1"
  character, that hotkey wouldn't fire. Would need lower-level native
  keyboard handling to fix properly; not done given how narrow the impact
  is.
- **Fixed: std-widgets now follow the Auto/Light/Dark choice.** The
  Settings window's `Switch`/`LineEdit`/`ScrollView` style themselves from
  Slint's own built-in `Palette` global (`std-widgets.slint`), which by
  default just tracks the OS light/dark setting - so picking "Light" or
  "Dark" explicitly wouldn't have been reflected in those controls.
  `settings_window.rs::refresh()` now also sets that global's
  `color-scheme` (`Unknown` for Auto, `Light`/`Dark` otherwise) via
  `win.global::<Palette>().set_color_scheme(...)`. One caveat: the actual
  `ColorScheme` type isn't re-exported from the stable `slint` crate root
  - only reachable via `slint::private_unstable_api::re_exports`
    (aliased in `settings_window.rs`), which is what Slint's own generated
  code uses internally but isn't a documented-stable path, so it could
  break on a future Slint upgrade. Compiles; not visually confirmed.

## Known gaps / not done

These are the things most likely to bite whoever picks this up next,
roughly in priority order:

1. **Never run.** Nothing in this session has actually launched the app -
   only `cargo build`/`cargo test`. Next step should be `cargo run --
   https://example.com` and working through the main window (hover,
   keyboard nav, hotkeys, incognito, context menus, the ⋮ menu, Settings,
   About) before trusting any of the above.
2. **macOS always-on-top: real bug found by actually running it (by a
   parallel session, not this one) and fixed.** A parallel Claude session
   working the same repo's `xilem` branch built+ran this `slint` branch
   and reported `could not obtain a raw window handle for the main
   window` firing every time, right when `gui::app::run()` called
   `make_main_window_floating()`/`hide_main_window_from_taskbar()`
   synchronously right after the first `.show()`. Root cause: `.show()`
   only *requests* the window be shown - the underlying native
   window/NSView isn't necessarily created or mapped by the time it
   returns, so `window_handle()` legitimately returns `Err` at that exact
   point (this isn't specific to a headless/no-display environment - it
   reproduced for them on a real display too). Fixed by retrying via
   `slint::Timer::single_shot` (a stable, public API - deliberately not
   the internal `window_shown_hook` in `i-slint-core`, which is
   explicitly testing-only) every 16ms for up to ~0.5s until the handle
   is available, applied to both the macOS and Windows variants. Still
   not applied to the About/Settings windows if that turns out to matter
   (they use Slint's default window level) - and still needs a real run
   to confirm the retry loop actually converges rather than just quieting
   the warning.
   The same parallel session also saw the main window's Y position jitter
   between two values (632/696) across several rapid `Moved` events in
   the winit debug log, on repeated runs. Not investigated or reproduced
   here. Plausible, boring explanation: the window can legitimately be
   repositioned twice in quick succession by design - once at initial
   `app::new()`/`reposition_main_window`, and again if `cleaned_url_opened`
   fires shortly after (the real URL arriving via macOS's `GetURL` event
   after the window's already shown) - so two distinct Y values isn't
   necessarily wrong, just unconfirmed. Worth a real look if it turns out
   to visibly flicker rather than settle immediately.
3. **Windows and Linux now both actually compile-check clean - found and
   fixed two real, pre-existing Cargo.toml bugs to get there.**
   `rustup target add x86_64-pc-windows-gnu` + `cargo check --target
   x86_64-pc-windows-gnu` works from this machine (no native sysroot
   needed for anything in this dependency tree). Linux needed more:
   `cargo check --target x86_64-unknown-linux-gnu` fails on
   `yeslogic-fontconfig-sys` (a transitive Slint font dependency) needing
   a real sysroot for `pkg-config` - but this machine already had the
   project's own `cross` Docker images built locally
   (`browsers.software/x86_64-unknown-linux-gnu-gtk:local`, referenced in
   `Cargo.toml`'s `[package.metadata.cross...]`), and `docker ps` showed
   the daemon running, so `cross check --target x86_64-unknown-linux-gnu`
   worked too. **Both now finish clean** (`cargo build`/`cross check`
   both report `Finished`, only pre-existing unrelated warnings) - this
   is real compiler verification, not just "reviewed by eye" as this
   entry originally said.
   Getting there surfaced two genuine bugs, both fixed:
   - `winapi`'s Cargo.toml entry only declared the `ntdef` feature, but
     `windows_utils.rs` uses `windef`/`winuser`/`minwindef`/`shellapi`/
     `wingdi` too - these only ever compiled "by accident" via feature
     unification with druid's own much broader winapi feature set.
     **Removing druid would have silently broken the Windows build**,
     with no way to catch it from macOS alone without this cross-check.
   - Bigger one: `xdg-mime`, `freedesktop-desktop-entry`,
     `freedesktop-icons`, `shell-words`, and (newly, this migration)
     `mouse_position` were all declared right after a *comment* reading
     `#[target.'cfg(target_os = "linux")'.dependencies]` - note the
     leading `#`, making it a comment, not a real table header. With no
     real header to open a new table, TOML kept parsing all of them as
     part of the *previous* real table, `[target.'cfg(target_os =
     "macos")'.dependencies]`. So every one of those Linux-only crates
     was **silently macOS-only** - a pre-existing bug that predates this
     migration entirely (druid never depended on any of them, so it
     never masked this one the way it masked the winapi issue). It's
     exactly why the Windows check *also* failed to resolve `copypasta`
     (added in this migration, into the same mis-scoped block) even
     though `copypasta` built fine standalone and resolved cleanly via
     `cargo tree` - I initially wrote that off as an unexplained
     sandbox/cargo quirk before realizing, once the same pattern showed
     up for several *unrelated, long-standing* crates on the Linux cross
     check too, that it had to be structural. Fixed by giving Linux a
     real `[target.'cfg(target_os = "linux")'.dependencies]` table and
     moving `copypasta` (used on every platform) up into the general
     `[dependencies]` section.
   Also wired up the always-on-top/taskbar-hiding pair fully now on
   Windows: `gui::app::run()` calls a new
   `hide_main_window_from_taskbar()` (mirroring
   `make_main_window_floating()`) that gets the `HWND` via
   `raw-window-handle` and calls `windows_utils::hide_from_taskbar`.
   Once `cross` was working, also checked the other two Linux
   architectures this project ships (per `Cargo.toml`'s
   `[package.metadata.cross...]`): `cross check --target
   aarch64-unknown-linux-gnu` and `--target armv7-unknown-linux-gnueabihf`
   both finish clean too, using the project's own pre-built local images
   for each.
   What's *still* unconfirmed on every platform is purely runtime
   behavior (does the window land in the right place, does the taskbar
   icon actually disappear, etc.) - `cross`/`cargo check` prove it
   compiles and type-checks, not that it behaves correctly, and none of
   Windows/Linux/aarch64/armv7 was actually run, only macOS. `cross
   clippy --target x86_64-unknown-linux-gnu` also came back clean (no new
   findings beyond pre-existing style nits already noted throughout this
   file, e.g. `redundant_field_names`, `needless_return`, unrelated to
   this migration). `mouse_position`'s Linux backend uses `x11-dl` (real
   X11 protocol calls) - won't work under pure Wayland without XWayland.
4. **Source-app-based opening rules now resolve the sender on macOS's
   `GURL` path too, and the handler registers before it can race a cold
   launch.** `EventBridge::handle_get_url_event` (in `macos_native.rs`)
   now also reads the event's `keyAddressAttr` attribute and coerces it
   to `typeApplicationBundleID` to get the sending app's bundle id, so the
   `on_open_urls` callback is now `Fn(Option<String>, String)` (sender,
   url) instead of just the url. Separately, found and fixed a real
   ordering bug while reviewing this: `install_event_bridge` used to be
   called from deep inside `main()`, after config loading, browser
   discovery, and the default-browser check - all of which take real
   time. On a cold launch via a registered URL scheme, macOS can deliver
   the `GetURL` Apple Event essentially immediately, before that point,
   and an event that arrives before the handler is registered is silently
   lost - meaning "Browsers" could occasionally fail to receive the very
   URL that launched it. Split the function into
   `register_event_bridge()` (no callbacks, called as the very first line
   of `main()`) and `EventBridge::set_resign_active_handler`/
   `set_open_urls_handler` (called later once `state` exists to build the
   real callbacks from). `main.rs` passes the resolved sender through to
   `MessageToMain::UrlPassedToMain` unchanged otherwise. Compiles; not
   runtime-verified (needs an actual app - e.g. Mail.app or Slack - to
   open an `https://` link while Browsers is set as the default browser,
   ideally via a fresh cold launch, and a rule keyed on that app's bundle
   id to confirm it matches).
5. **Settings/About window close/reopen bug fixed.** Found and fixed a real
   bug while reviewing this: `CloseRequestResponse::HideWindow` only stops
   the *native* window from closing - Slint's component handle is still an
   `Rc`-like ref count, so the old code (which set
   `AppState.settings_window`/`about_window` back to `None` in
   `on_close_requested`) was dropping the only strong reference and
   actually destroying the window anyway, contradicting the `HideWindow`
   return value; separately, the "already open, just re-show" fast path in
   `open()` never incremented `extra_windows_open`, so closing and
   reopening once left the always-quit-on-focus-loss guard
   (`quit_on_lost_focus_applies`) under-counting real open windows. Both
   fixed in `settings_window.rs`/`about_window.rs`: the window is now kept
   alive (just hidden) across a close, `open()`'s reuse path increments
   `extra_windows_open` and (for Settings) calls `refresh()`. Compiles;
   not yet run through a manual open/close/reopen cycle to confirm no
   leftover visual/state staleness.
6. **`quit_on_lost_focus` on Linux/Windows: investigated, turns out to be a
   non-issue.** Looked into implementing this and found Slint's *public*
   API has no window-focus-changed callback at all (`i-slint-core` has one
   - `set_window_event_hook` - but it's explicitly documented as "Internal
   function ... used by the system testing module", not meant for app
   code; the language-level `changed` property trigger that could observe
   a `FocusScope.has-focus` is still gated behind
   `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`). But it turns out this doesn't
   matter here: `settings_window.slint`'s General tab only shows the "Quit
   when focus is lost" toggle `if show-quit-on-lost-focus` (`Rust: show_quit_on_lost_focus:
   cfg!(target_os = "macos")`) - **identical to the old druid
   `general_view.rs`**, which gated the same row behind `is_mac`. So on
   Linux/Windows a user could never turn this setting on in either version
   of the app; the config default is `false`. Not a regression - leaving
   as macOS-only is consistent with original behavior, not a gap to fill.
7. **Context menu positioning improved; still not visually verified.**
   `TouchArea.clicked()` has no position argument, but `TouchArea` does
   expose `mouse-x`/`mouse-y` (last pointer position within it), so the
   "⋮" options button (`main_window.slint`) and the profile-picker/
   "Restore App..." buttons (`settings_window.slint`) were restructured so
   the `TouchArea` is nested *inside* the `ContextMenuArea` (as its visual
   content, matching Slint's own example pattern) rather than a sibling -
   they need to share one coordinate space for `.show({x: mouse-x, y:
   mouse-y})` to land in the right place. Compiles; still needs an actual
   click to confirm the menu now opens at the cursor instead of a fixed
   offset. Copy-to-clipboard (`copypasta`) and the general
   `Window::set_position`/`.show()` calls are likewise compiled-only,
   unverified.
8. **`cargo fmt` + `cargo clippy` done for all code touched by this
   migration.** `cargo fmt` reformatted the new/changed `src/gui/*`,
   `src/lib.rs`, `src/macos/macos_native.rs`,
   `src/windows/windows_utils.rs`. `cargo clippy --lib` came back
   completely clean for every file this migration added or touched
   (`src/gui/*`, `src/main.rs`, the new bits of `macos_native.rs`/
   `windows_utils.rs`) except one unnecessary `unsafe` block in
   `mouse_position_and_work_area` (fixed - none of the calls in it
   actually need it in this `objc2-app-kit` version). All remaining
   clippy/warning output is pre-existing, unrelated to this migration
   (`slack_profiles_parser.rs`, `browser_repository.rs::AppIdentifier`,
   `build.rs`, older functions in `macos_native.rs` like
   `has_sandbox_entitlement2`/`get_bundle_url`) - deliberately left alone,
   out of scope for a druid->Slint port.

## Files removed

`src/gui/ui.rs`, `focus_widget.rs`, `image_controller.rs`, `ui_theme.rs`,
`about_dialog.rs`, `settings_window/` (the whole druid directory), `shared/`
(`directories_info.rs`, `restore_apps.rs` - folded into the new files).
