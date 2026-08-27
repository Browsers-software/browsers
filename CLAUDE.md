# Notes for Claude

## Code Style

### Comments
When a comment spans multiple lines, wrap it at natural sentence boundaries, not merely at the maximum line width.
Do not start a new sentence near the end of a comment line.

## Screenshotting the UI / driving the app headlessly

This is a macOS accessory app (no Dock icon, `LSUIElement`-style) with three
Slint windows: the main popover (`ui/main_window.slint`), Settings
(`ui/settings_window.slint`, 4 tabs: General/Appearance/Rules/Advanced), and
About (`ui/about_window.slint`).

### Prefer the headless backend for testing, period

Use `SLINT_BACKEND=headless` + Slint's built-in MCP server for UI testing in
general, not just as a workaround for this sandbox lacking a display -
it's deterministic and scriptable, unlike driving a real on-screen window.

That said, this particular sandboxed Claude Code execution also happens to
have no WindowServer session at all - `screencapture` fails here with
`could not create image from display`, and a normally-launched (non-headless)
GUI binary shows nothing and, once the process's `NSApplication` briefly
gains and loses "active" status, just exits cleanly (exit code 0) - not a
crash, just nothing to interact with. So a normal `cargo run` +
`screencapture` loop isn't even a fallback option here.

### Use Slint's built-in MCP server

Build and run with the `mcp` feature and a headless backend:

```sh
SLINT_EMIT_DEBUG_INFO=1 cargo build --features slint/mcp
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9315 SLINT_BACKEND=headless \
  ./target/debug/browsers "https://example.com/some/long/path" &
```

A URL arg is required or the main window's browser list may not populate
meaningfully. Confirm it's up: `curl -s -X POST http://127.0.0.1:9315/mcp -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream" -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'`.

**Critical: verify the log says `Slint MCP server listening` before doing
anything else - if it instead says `Could not load rendering backend
headless, fallback to default`, kill the process immediately
(`pkill -9 -f "target/debug/browsers"`).** This sandbox can't *capture* the
screen (`screencapture` fails with "could not create image from display" -
see below), but it evidently *can* open real windows, so that fallback
silently pops up a real, visible window on the user's actual desktop
instead of failing loudly. Observed this happening repeatedly and
disruptively in one session; the earlier "no WindowServer session" framing
below is about screen *capture*, not about whether windows can be shown -
don't conflate the two, and don't treat a failed headless launch as
harmless. This failure is intermittent (works, then inexplicably doesn't,
on identical commands run moments apart in the same session) - don't
assume a prior success means the next launch is safe; check the log every
time.

**Headless screenshots look softer than the real on-screen app - this is
expected, not a bug.** `SLINT_BACKEND=headless` (no suffix) does use Skia
here (`renderer-skia` is enabled in `Cargo.toml`, and Slint's headless
renderer picker defaults `""`/`"default"` to Skia when that feature is
compiled in) - verified by pixel-diffing a screenshot against one taken
with `SLINT_BACKEND=headless-skia` explicitly (near-identical, mean channel
diff ~0.3/255) vs. `SLINT_BACKEND=headless-software` (visibly different,
~11/255, noticeably blurrier). But it's still not the *same* Skia path the
real app uses: main.rs's `BackendSelector` (which picks the GPU/Metal-backed
`skia` renderer for on-screen use, see the comment at the top of `main()`)
only runs when `SLINT_BACKEND` is *unset* - setting it to `headless`
bypasses that block entirely. Headless mode instead goes through
`SkiaRenderer::default_software()`, a real Skia renderer but backed by an
abstract offscreen CPU pixel buffer with no attached display, so it can't
request LCD subpixel antialiasing (Skia needs a real display's physical
subpixel layout for that) and falls back to plain grayscale AA - same
glyph outlines/font manager, just softer-looking small text than what
you'd see running the windowed app for real. Not fixable from the
headless side; not worth chasing for screenshot purposes.

The project's `.mcp.json` already points a `browsers-slint-ui` MCP server at
this URL, but Claude Code only resolves MCP servers at session start - if
the app wasn't already running then, that server shows as connection-failed
for the rest of the session and its tools never become available. Just talk
to `http://127.0.0.1:9315/mcp` directly with `curl` instead (see
`debugging-and-mcp.md` in the `slint` plugin skill for the tool list/shapes).

### Reaching each window

- **Main window**: shown automatically on launch (`list_windows` ->
  handle `{"index":"1","generation":"1"}`).
- **Settings** and **About**: pass `--settings` or `--about` on the command
  line (in addition to/instead of a URL) and `main.rs` opens that window
  itself right after building `state`, before the event loop starts - no
  menu navigation needed at all. Added specifically to skip the
  click-through-the-menu dance below for verification. Note the *window
  handle* MCP assigns depends on `.show()` order, not creation order: since
  `open_settings`/`open_about` show before `app::run()` shows the main
  window, that window ends up as handle `{"index":"1",...}` and the main
  window becomes `{"index":"2",...}` - don't assume index 1 is always the
  main window like it is with no flags.
- **To identify which handle is which window**, don't guess from size -
  call `get_window_properties` for its `rootElementHandle`, then
  `get_element_tree` on that handle with `maxElements: 1`; the single
  returned element's `typeNamesAndIds[0].id` reads `MainWindow::root` /
  `SettingsWindow::root` / `AboutWindow::root`, naming the window directly.
  (`get_window_properties`' own size is still a fine sanity check if you
  want one: Settings 500x500, About 340x320, main window 222x239.)
- Once Settings is open (however it was reached), its 4 sidebar tab labels
  (`General`/`Appearance`/`Rules`/`Advanced`, found via `get_element_tree`
  off the window's `rootElementHandle`) click correctly with plain
  `click_element` - the coordinate bug below is specific to the popup
  `Menu`/`MenuItem` widget, not general.

#### Fallback: reaching them through the actual menu (no `--settings`/`--about`)

Only needed for verifying the menu-driven UX itself, or if those flags ever
go away. Settings/About live behind the main window's "..." context menu
(element id `MainWindow::optionsTa`, a plain left-click TouchArea - not the
invisible right-click `MainWindow::om` ContextMenuArea, which behaves the
same after the first click anyway). Open it, then **navigate with the
keyboard instead of `click_element`** (see the coordinate bug below for
why): send Down-arrow presses to walk the highlight from Refresh -> Restore
-> Settings... -> About -> Quit, then Return to activate. `dispatch_key_event`'s
`text` needs the actual Unicode char, not a name - see "Sending special
keys" below for the values and for why they must reach curl as a hex
codepoint, not a shell literal. **Send one key, then screenshot to confirm
the highlight moved, before sending the next** - firing several in a tight
loop with no delay silently drops some of them (observed: 4 rapid Down
presses only registered 2 - screenshot after each one instead of trusting
the count).

### Known bug: `click_element` on the main window's popup `Menu`

is off by exactly two rows (-64px, i.e. -2x row height) in this headless
session - avoid it for this menu entirely, use keyboard navigation instead
(above). Clicking the element reported as item N instead fires item N-2's
real handler (verified 3x): clicking "Settings..." (item 3) fired "Refresh"
(item 1); clicking "About" (item 4) opened "Restore"'s submenu (item 2);
clicking "Quit" (item 5) opened **Settings** (item 3) - which is how
Settings got reached before switching to keyboard nav; that trick doesn't
reach About since it'd need a nonexistent item 6.
`invoke_accessibility_action` with `Default_` on the About label also did
nothing (its `accessibleRole` is plain `Text`, not `Button`), and starting
a `drag_element` from a menu item's own handle suffers the same -64px bug
on the press itself, immediately opening the wrong submenu before the drag
even moves. Right-clicking the ContextMenuArea directly, once a submenu
(`Restore`) gets triggered, leaves it stuck open across further calls;
sending Escape (see below) closes it. If this needs revisiting, consider
filing/checking upstream Slint MCP server issues before re-deriving this by
trial and error again.

### Sending special keys through `dispatch_key_event`

The `text` field needs the actual character, not a name like `"Down"` -
Slint's special-key constants are specific Unicode codepoints, defined in
`i-slint-common`'s `key_codes.rs` (`~/.cargo/registry/src/*/i-slint-common-<ver>/key_codes.rs`).
The ones used here: Escape = `U+001B`, Down = `U+F701`, Up = `U+F700`,
Return/Enter = `U+000A`.

Getting the raw character into the `curl` payload is the hard part -
**don't** put a literal escape/control char or a `$'\uXXXX'` shell literal
straight into a Bash command: Claude Code's own command-text validator
rejects literal control bytes outright ("contains control characters that
would be hidden in the approval dialog"), and Private-Use-Area chars typed
as `$''` in a Bash tool call silently arrived as an **empty string**
here (verified with `repr()` - zero length) even though no error was
raised, for reasons that weren't tracked down. What worked reliably: write
a small Python script (via the `Write` tool, not a Bash heredoc) that takes
the codepoint as a **hex string CLI arg** (pure ASCII, e.g. `F701`) and
does `chr(int(sys.argv[1], 16))` to build the JSON payload in Python before
POSTing - sidesteps shell quoting entirely. See
`scratchpad send_key2.py` pattern used this session (recreate as needed,
scratch files aren't kept between sessions).

### Window sizing: `preferred-width`/`preferred-height` don't reliably apply

`SettingsWindow` had `preferred-width: 714px;` but was actually showing at
500px wide (verified both via the live app's `get_window_properties` and
via `slint-viewer`) - the shown size instead tracked the *currently visible
tab's own computed content width* (General's, since `current-tab` defaults
there), ignoring the Window's `preferred-width` hint entirely. This meant
switching to the wider Rules tab later never grew the window, so its
content (a 300px `LineEdit` plus label/button) didn't fit. Fixed by using
`width`/`height` instead (a hard size Slint always honors) - but note these
can't be combined with `min-width`/`min-height` on the same component
(compile error), so decide which pair you need. If a window's initial size
looks wrong, don't trust `preferred-*` - verify the actual shown size
(`get_window_properties` live, or `slint-viewer --screenshot` and check the
output image's dimensions) rather than assuming the hint took effect.

### `TouchArea` always wins over a sibling `ContextMenuArea` unless it's declared later

A right-click landing on a `TouchArea` and a `ContextMenuArea` that fully
overlap (same size, same parent) always goes to whichever one is declared
**later** in the file - sibling hit-testing visits children front-to-back
in *reverse declaration order* (`i-slint-core`'s `TraversalOrder::FrontToBack`,
see `item_tree.rs`), so the last-declared sibling is effectively "on top"
and gets first look at every press.

This matters because `TouchArea::input_event` (`i-slint-core`'s
`items/input_items.rs`) grabs **any** `MouseEvent::Pressed`, regardless of
button, and returns `GrabMouse` - a terminal result that stops the search
outright. If `TouchArea` is declared after `ContextMenuArea`, it silently
swallows right-clicks too; giving the `ContextMenuArea` an explicit size
does not help, since the hit test never reaches it. `ContextMenuArea`'s own
`input_event` (`items.rs`) only handles a right-`Pressed`/Menu-key, and
ignores everything else, so declaring it *last* (after the `TouchArea`)
fixes right-click without breaking the `TouchArea`'s left-click/hover -
those presses hit `ContextMenuArea` first, get ignored, and fall through.
`ui/main_window.slint`'s `BrowserRow` needed both fixes: an explicit
`width`/`height` on `cm` (a `ContextMenuArea` with only a non-visual `Menu`
child otherwise defaults to 0x0) *and* moving `cm`'s declaration after
`ta`'s.

### `slint_build::compile()`'s bare form silently defaults to Fluent, everywhere

`build.rs` used to call `slint_build::compile("ui/app.slint")` - the bare
form, no config. That does *not* mean "use the native/platform-appropriate
style" the way the general Slint docs imply ("if no style is selected,
native is the default") - it means literally Fluent, on every OS, macOS
included. Fixed by switching to `slint_build::compile_with_config()` with
`CompilerConfiguration::new().with_style("native".into())` (see
`compile_slint_files()` in `build.rs`).

Root cause, traced through `i-slint-compiler-<ver>/typeloader.rs`'s
`TypeLoader::new`:
```rust
let mut style = compiler_config.style.clone().unwrap_or_else(|| "fluent".into());
if style == "native" {
    style = get_native_style(&mut diag.all_loaded_files);
}
```
The `get_native_style()` call - which resolves the real per-platform style
(`cupertino` on macOS, `fluent` on Windows, `material` on Android, `qt`/
`fluent` on Linux, via `i-slint-common`'s `get_native_style()`, matching
target-triple substrings like `"apple"`) - only runs when `style` is the
*literal string* `"native"`. Leaving it unset (`None`, exactly what the bare
`compile()` produces via `CompilerConfiguration::default()`) hits the
`unwrap_or_else` arm instead and never reaches the native-detection branch
at all - it's hardcoded to `"fluent"`, full stop, regardless of platform.

This bit twice in the same investigation, worth flagging both:
1. `slint-viewer` (a separate, already-compiled binary) has the *same*
   underlying default-to-fluent behavior when no `--style` flag is passed,
   for an unrelated reason (its native-style env var lookups need
   `TARGET`/`OUT_DIR`, which are build-time-only Cargo vars that don't exist
   when just *running* an already-built binary) - so pass `--style
   cupertino` explicitly on every `slint-viewer` screenshot for this
   project, or it'll render boxy Fluent-style widgets that don't match the
   real app.
2. The real app had the *same visible symptom* (Fluent, not Cupertino) but
   from the `build.rs` bug above, not from anything `slint-viewer`-related -
   don't assume fixing one explains the other. Caught only by grepping the
   actually-current (not stale-cached) `slint_build`-generated Rust output
   for style-specific type names, e.g. `InnerFluentPalette_*` vs
   `InnerCupertinoPalette_*`:
   `grep -oi "cupertino\|fluent" target/debug/build/browsers-*/out/app.rs`
   - and note that this project's `target/debug/build/browsers-*` dirs
   accumulate a lot of stale, never-cleaned entries from old fingerprint
   hashes across a long session; `cargo clean -p browsers` before checking
   avoids being misled by an old cached one, the way an earlier pass in
   this same investigation was.

### A `VerticalLayout`-based component doesn't stretch to fill width inside a `HorizontalLayout`

`SettingsWindow`'s content area (`ui/settings_window.slint`) is a
`HorizontalLayout` holding a fixed-width sidebar plus whichever `*Tab`
component (`GeneralTab`, `AppearanceTab`, etc. - each `inherits
VerticalLayout`) is currently selected. Every row inside those tabs that
relied on a `Rectangle { horizontal-stretch: 1; }` spacer to push a control
to the far right (e.g. a `Switch`) rendered squeezed into a narrow column
instead, as if the tab itself had only claimed its own minimal content
width rather than the ~540px actually available.

Confirmed via `slint-viewer` on a series of isolated repros (a throwaway
`export` on the component plus a minimal wrapping `Window`/`HorizontalLayout`
test file) that this reproduces in the simplest possible case: a bare
`HorizontalLayout` with a single `VerticalLayout`-based custom component as
its only child does not stretch that child to the layout's width, no
matter what's tried on either side - `horizontal-stretch: 1` on the child,
on a wrapping plain `VerticalLayout`, `width: 100%`, all had no effect. The
exact same component, placed directly in a `Window` or inside a plain
`VerticalLayout` (no enclosing `HorizontalLayout`), stretches correctly
every time. So this seems specific to a `VerticalLayout`-typed child
crossing into a perpendicular-axis `HorizontalLayout` parent - not a
general "custom components don't stretch" issue, and not something
`alignment`/`cross-axis-alignment` (which default to `stretch` already,
per the docs) affects either.

Not root-caused further than that; the practical fix was to stop relying
on automatic stretch across that boundary and compute the width explicitly
instead: give the sidebar an id (`sidebar := VerticalLayout { width: 190px; ... }`)
and the content-area `VerticalLayout` `width: root.width - sidebar.width;`
instead of `horizontal-stretch: 1`. If a similar "content looks squeezed
into a narrow column despite the window being wide" symptom shows up
elsewhere, check for this exact shape (`HorizontalLayout` > single
`VerticalLayout`-based child) before assuming it's an `alignment` problem
(that was the first, wrong guess here) or a stretch-factor problem.

### Rendering a component standalone via `slint-viewer`

Useful for a component the live app can't easily reach, checking exact
rendered pixel dimensions (see above), or a render without launching the
whole app. Also handy for testing one specific tab/state without having to
drive the live app there first - e.g. `--load-data` can set
`current-tab: "rules"` plus a `rules` array directly, instead of opening
Settings and clicking to the Rules tab.

`AboutWindow`'s (and everything else's) colors all come from the
`AppPalette` global (`ui/theme.slint`), which has **no default values** -
they're only set at runtime by `src/gui/theme.rs::apply_palette`. Render
with `slint-viewer` alone (no `--load-data`) and everything but the
built-in Slint attribution badge is invisible (transparent-on-transparent).
Fix: pass `--load-data` with the real dark-palette values from
`Palette::dark()` in `src/gui/theme.rs`, plus any of the component's own
`in property`s (e.g. About's `version-text`, matching `Cargo.toml`'s
version, and `directories`, matching what `paths::get_*_dir()` produces).
Also pass `--component <Name>` explicitly - without it the
default-component heuristic did not pick `AboutWindow`.

An `image`-typed property (`app-icon`) given a plain file-path string in
`--load-data` does **not** load the image - it silently renders the
generic "broken image" placeholder glyph. Note this isn't just a
`slint-viewer` quirk: the same placeholder glyph shows up in the **live**
app too when run headless here, so `icon_loader::IconLoader` likely depends
on something (real graphics context / WindowServer) that isn't available
in this sandbox either way - not worth chasing further for screenshot
purposes.

```sh
slint-viewer --screenshot out.png --component AboutWindow \
  --load-data about_data.json ui/about_window.slint
```
