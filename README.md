# waytrim

waytrim is a Linux-native text repair tool for copy/display damage from terminal, TUI, AI-console, and similar plaintext-oriented interfaces.

It is a repair tool, not a rewrite tool.

## Getting started

Build and install the binaries to `~/.local/bin`:

```bash
cargo build --release --bin waytrim --bin waytrim-watch --bin waytrimctl --bin waytrimd
install -Dm755 target/release/waytrim ~/.local/bin/waytrim
install -Dm755 target/release/waytrim-watch ~/.local/bin/waytrim-watch
install -Dm755 target/release/waytrimctl ~/.local/bin/waytrimctl
install -Dm755 target/release/waytrimd ~/.local/bin/waytrimd
```

For local rebuild + reinstall during development:

```bash
./scripts/reinstall-local
```

Repair wrapped prose from stdin:

```bash
printf 'This is a wrapped\nparagraph.\n' | waytrim prose
```

Repair the current clipboard in place:

```bash
waytrim prose --clipboard
```

Run the conservative Wayland watcher:

```bash
waytrim-watch auto
```

Further guides:
- `docs/getting-started.md`
- `docs/integrations.md`
- `docs/troubleshooting.md`

## Current CLI

```bash
waytrim prose
waytrim command
waytrim auto
waytrim prose --preview
waytrim prose --explain
waytrim prose --clipboard
waytrim prose --clipboard --print
waytrim prose --clipboard --preview
waytrim prose --clipboard --explain
waytrim --no-preview
waytrim --no-explain
waytrim --no-print
waytrim --no-clipboard
```

The canonical interface is mode-centered. Clipboard actions use `waytrim <mode> --clipboard`, not `waytrim clipboard <mode>`.

The CLI reads from stdin and writes cleaned text to stdout by default. `--preview` prints a diff-like before/after view, and `--explain` prints a human-readable report of what changed and why. In clipboard mode it reads the current clipboard text, repairs it through the same core logic, and writes the repaired text back unless the selected output mode is explicitly non-mutating. User config can provide default mode, output behavior, and a small policy surface; explicit CLI flags override config values.

## Modes

### `prose`
- primary mode
- repairs wrapped paragraphs, copy-induced spacing noise, obvious blank-line noise inside one paragraph, and copy-induced heading padding
- preserves visible structure such as bullets, headings, blockquotes, fenced code blocks, indented sections, alignment-sensitive / table-ish text, and obvious standalone command blocks inside mixed prose snippets
- repairs obvious wrapped blockquotes while leaving fenced code, aligned columns, and standalone command blocks untouched

### `command`
- bounded secondary mode
- strips obvious prompts and repairs command presentation damage
- handles bare prompts and common host-style shell prompts conservatively
- leaves already-clean shell commands unchanged
- leaves transcript-shaped or mixed command/output snippets unchanged unless the command shape is obvious

### `auto`
- conservative convenience mode
- chooses command cleanup when the input is clearly command-like
- declines to merge short label-plus-command snippets such as `Install command:` followed by a command
- declines prose-framed command examples and install sections such as `Run this:` or `Install command:` plus a command block
- stays conservative on mixed prose-plus-command snippets and falls back to minimal prose-safe cleanup
- otherwise prefers prose repair or minimal prose-safe cleanup

## User config

Config path:
- `XDG_CONFIG_HOME/waytrim/config.toml`
- or `~/.config/waytrim/config.toml`

Rules:
- missing config is silent
- invalid config prints a warning and falls back to built-in defaults
- explicit CLI flags override config values

Initial config-backed policy surface:
- `protect.aligned_columns`
- `protect.command_blocks`
- `[auto].policy = "conservative" | "prose_preferred"`

Example:

```toml
[defaults]
mode = "prose"
preview = false
explain = false
clipboard = false
print = false

[protect]
aligned_columns = true
command_blocks = true

[auto]
policy = "conservative"
```

## Preview

Use `--preview` to print a simple before/after diff-like view instead of cleaned text.

In clipboard mode, `--preview` is explicitly non-mutating. It shows what would change without writing back to the clipboard.

## Clipboard mode

Clipboard support is a manual adapter over the same repair core.

Behavior:
- `waytrim prose --clipboard` repairs current clipboard text and writes it back when it changes
- `waytrim prose --clipboard --print` prints repaired text to stdout and also writes it back when it changes
- `waytrim prose --clipboard --preview` previews changes without mutating the clipboard
- `waytrim prose --clipboard --explain` explains changes without mutating the clipboard
- `clipboard unchanged` is a first-class success outcome when no effective change is needed
- empty clipboard input returns a clear success message instead of crashing
- non-text clipboard offers such as images are skipped before payload read, so image copies do not trigger slow text decoding paths
- `--clipboard --preview --print` is rejected as ambiguous
- `--clipboard --explain --print` is rejected as ambiguous

Runtime dependency:
- Wayland clipboard support uses `wl-paste` and `wl-copy`

## Optional local automation layer

The canonical user interface is still `waytrim`.

For local automation and future desktop integrations, waytrim also ships:
- `waytrimd` — Unix-socket service exposing the same repair core
- `waytrimctl` — thin JSON IPC client for the service
- `waytrim-watch` — automatic clipboard watcher for Wayland

Default socket path:
- `XDG_RUNTIME_DIR/waytrim/waytrim.sock`
- fallback: `${TMPDIR:-/tmp}/waytrim-<uid>/waytrim.sock`

The IPC response carries a stable machine-readable report with:
- `requested_mode`
- `effective_mode`
- `decision`
- `changed`
- `output`
- `explain`

See `docs/integrations.md` for the JSON contract, service usage, and desktop workflow examples.

## Documentation

- `docs/getting-started.md`
- `docs/integrations.md`
- `docs/troubleshooting.md`
- `docs/architecture.md`
- `docs/development.md`

## Testing

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## Automatic clipboard workflow

To keep the clipboard cleaned in the background, run:

```bash
waytrim-watch auto
waytrim-watch prose
```

Other watcher commands:

```bash
waytrim-watch --clean-once auto
waytrim-watch --restore-original
waytrim-watch --status
waytrim-watch --status --json
```

Behavior:
- watches clipboard changes through `wl-paste --watch`
- probes offered clipboard MIME types first and skips non-text content such as images without reading the payload
- repairs new clipboard text through the same core logic
- defaults to conservative `auto` mode unless CLI mode overrides it
- uses the existing repair policy surface from config
- stores one original clipboard buffer for `waytrim-watch --restore-original`
- records the last watcher status, mode, and restore availability in watcher state for desktop adapters
- keeps manual override behavior in Rust, including `--clean-once` ignoring the self-update skip guard

## Manual desktop workflow

For thin Wayland/Niri entrypoints, use:
- `contrib/niri/waytrim-clipboard-prose`
- `contrib/niri/waytrim-watch-session`

The helpers are shipped as executable scripts.

They forward to the existing CLI and systemd user-service flow rather than adding UI-side logic.

For Quickshell / Noctalia-oriented integration examples, see:
- `contrib/quickshell/waytrim/WaytrimClient.qml`
- `contrib/quickshell/waytrim/WaytrimClipboardAction.qml`
- `contrib/quickshell/waytrim/WaytrimWatchControl.qml`
- `contrib/quickshell/waytrim/WaytrimNotifications.qml`

See `docs/integrations.md` for Niri setup, watcher lifecycle examples, and the Quickshell / Noctalia integration examples.

## Troubleshooting

If the watcher does not see the Wayland clipboard, the most common fix is:

```bash
systemctl --user import-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR
```

If you also use clipboard history, seeing both the original and cleaned entries in history is normal.

Then verify state with:

```bash
waytrim-watch --status
journalctl --user -u waytrim-watch@auto.service -n 50 --no-pager
```

See `docs/troubleshooting.md` for the full checklist.

## Fixtures

Fixtures live under `tests/fixtures/` and are organized by mode first, then source/type. Metadata files (`*.meta.txt`) capture notes plus lightweight `preserve` and `avoid` rules so heuristics stay aligned with the product boundary.

Current corpus coverage includes:
- AI-terminal wrapped prose, spacing-noise wraps, wrapped inline-code followups, spacing-noise paragraphs, blank-line noise, and heading-padding cleanup
- TUI status-update bullets and real TUI-copied watcher bullets with edge-padding noise
- PI/TUI wrapped prose paragraphs
- PI/TUI wrapped bullet and numbered-list continuations
- wrapped doc and PI blockquotes
- real TUI-copied watcher prose wraps and prose-framed command examples
- mixed doc and PI prose with preserved standalone command blocks
- alignment-sensitive / table-ish text, including docs option tables, that prose should preserve by default
- already-clean prose that should remain unchanged, including real section-break cases
- heading and indented-section no-op cases that prose should preserve
- fenced-code preservation cases, including PI output
- bare and host-style shell prompts
- already-clean shell commands that should remain unchanged, including clean pipeline commands
- multiline PI command cleanup
- mixed command/output transcripts that should stay unchanged, including transcript-with-status and host-prompt-plus-output captures
- ambiguous label-plus-command, transcript-like, mixed prose-command, prose-then-command-example, install-section, indented-command-example, aligned-columns, prose-preferred, and prose-framed-wrap auto snippets that `auto` should leave alone under the conservative default unless policy opts into prose-preferred repair
- unchanged preview and clipboard no-op cases for safe inputs, including command-mode clipboard/transcript paths, clean command no-op fixtures, install-section no-op cases, and heading/indented prose fixtures
