# Notes for Claude

## Code Style

### Comments
When a comment spans multiple lines, wrap it at natural sentence boundaries, not merely at the maximum line width.
Do not start a new sentence near the end of a comment line.

## Testing the UI headlessly

3 Slint windows: main (`ui/main_window.slint`), Settings, About. Test via
`SLINT_BACKEND=headless` + Slint's MCP server (`curl` to
`http://127.0.0.1:9315/mcp`) - this sandbox has no real display anyway.

```sh
SLINT_EMIT_DEBUG_INFO=1 cargo build --features slint/mcp
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9315 SLINT_BACKEND=headless \
  ./target/debug/browsers "https://example.com/some/long/path" &
```

**Verify the log says `Slint MCP server listening`** - if it says
`fallback to default` instead, `pkill -9 -f "target/debug/browsers"`
immediately, or it silently opens a real window on the user's desktop.

- `--settings`/`--about` flags open those windows directly.
- `click_element` on the main popup `Menu` is off by two rows (MCP bug) -
  use keyboard nav instead.
- `dispatch_key_event`'s `text` needs the actual Unicode char - build it
  via a Python script from a hex codepoint, not Bash.
- `slint-viewer` needs `--component <Name>`, or it renders blank.
  (AppPalette is computed in Slint now, so `--load-data` is no longer needed.)

### More MCP tools (Slint 1.18.1 `slint/mcp`)

Besides `take_screenshot`, prefer the semantic tools over pixel diffing:

- `get_element_properties` / `get_element_tree`: geometry, accessible props,
  computed opacity. Element ids need `SLINT_EMIT_DEBUG_INFO=1`.
- `invoke_accessibility_action`, `set_element_value`: instead of synthesized
  clicks/typing.
- `hover_element`, `move_pointer`, `scroll_element`, `dispatch_pointer_scroll`.
  The pointer tools take a `modifiers` object (shift/control/alt/meta).
- `start_event_recording` -> interact -> `stop_event_recording`: lists the
  pointer/key events the runtime actually received. Use it first when a
  key/hover does nothing (FocusScope, KeyBinding, Shift incognito).

Not available yet: reading declared properties such as `AppPalette.dark` or
`focused-index` (slint-ui/slint#13251, open) and element-scoped screenshots
(#13252, blocked on #13250). Assert via accessible properties or screenshots.
The tool list lives in
`~/.cargo/registry/src/*/i-slint-backend-testing-<ver>/mcp_server.rs`.

### What headless can't check

- A live OS light/dark switch with "Match System" - there is no way to flip the
  OS appearance headlessly, so this needs a manual check on macOS (this is the
  case that was broken when AppPalette was pushed from Rust).
- Explicit Light/Dark can be tested headlessly via the Settings theme radios.
- First paint at launch (light-then-dark flash) - needs a real macOS launch.

## Theming (how it works now)

- Colors live in `ui/theme.slint` (`AppPalette`) as bindings on `dark`, which is
  `scheme-override` (set from Rust: Light/Dark, Unknown = Match System) else
  `Palette.color-scheme`. Rust (`src/gui/theme.rs`) only passes the override.
  Don't go back to pushing colors from Rust - that is what went stale on OS
  appearance changes.
- Slint's winit backend sets `Palette.color-scheme` at window creation, before
  the first frame, so no native "startup dark mode" fallback is needed.
- Don't set `Palette.color-scheme` back to Unknown on a window that uses
  `AppPalette` - it then stops reflecting the OS. Only the Settings window does
  that (to force native widgets), and it doesn't use `AppPalette`.
- Don't override the native title bar theme per window. It follows the OS on
  purpose (Apple HIG discourages app-specific appearance; nothing else on macOS
  does it).
- Use `Palette.foreground` etc. (std-widgets) for text in shared components, not
  hard-coded colors - Settings has no `AppPalette`.

## Verifying framework behavior

Slint, winit etc. are vendored under `~/.cargo/registry/src/*/`. Read that
source (e.g. `i-slint-backend-winit-<ver>/winitwindowadapter.rs`) instead of
guessing. Current release is checked against the changelog at
https://raw.githubusercontent.com/slint-ui/slint/master/CHANGELOG.md
(1.18.1 is the latest at time of writing; no 1.19).

## Working preferences

- Keep diffs small and scoped to the request. Don't reformat, extract
  components, or "simplify" unrelated code unprompted (a whitespace-only
  reindent or tidy-up goes in its own commit, and only if asked).
- Commit messages: short - symptom/cause, fix, one-line rationale. No long
  lists of removed items.
- No `slint-lsp`/`slint-fmt` installed here, so `.slint` indentation is by hand.

## Slint gotchas

- `TouchArea` beats a sibling `ContextMenuArea` unless declared later.
- `VerticalLayout` doesn't stretch inside a `HorizontalLayout` - compute
  width explicitly.
- `preferred-width`/`height` don't reliably apply - use `width`/`height`.

