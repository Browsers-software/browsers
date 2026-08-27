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
- `slint-viewer` needs `--load-data` (real `AppPalette` values) and
  `--component <Name>`, or it renders blank.

## Slint gotchas

- `TouchArea` beats a sibling `ContextMenuArea` unless declared later.
- `VerticalLayout` doesn't stretch inside a `HorizontalLayout` - compute
  width explicitly.
- `preferred-width`/`height` don't reliably apply - use `width`/`height`.

