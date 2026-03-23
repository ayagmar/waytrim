# waytrim

waytrim is a Linux-native text repair tool for copy/display damage from terminal, TUI, AI-console, and similar plaintext-oriented interfaces.

It is a repair tool, not a rewrite tool.

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
- `--clipboard --preview --print` is rejected as ambiguous
- `--clipboard --explain --print` is rejected as ambiguous

Runtime dependency:
- Wayland clipboard support uses `wl-paste` and `wl-copy`

## Optional local automation layer

The canonical user interface is still `waytrim`.

For local automation and future desktop integrations, waytrim also ships:
- `waytrimd` — Unix-socket service exposing the same repair core
- `waytrimctl` — thin JSON IPC client for the service

Default socket path:
- `XDG_RUNTIME_DIR/waytrim/waytrim.sock`
- fallback: `${TMPDIR:-/tmp}/waytrim.sock`

The IPC response carries a stable machine-readable report with:
- `requested_mode`
- `effective_mode`
- `decision`
- `changed`
- `output`
- `explain`

See `docs/integrations.md` for the JSON contract, service usage, and desktop workflow examples.

## Development docs

- `docs/architecture.md`
- `docs/development.md`
- `docs/integrations.md`

## Testing

```bash
cargo test
cargo fmt --check
```

## Manual desktop workflow

For a thin Wayland/Niri entrypoint, use:
- `contrib/niri/waytrim-clipboard-prose`

It just forwards to:

```bash
waytrim prose --clipboard
```

See `docs/integrations.md` for example Niri binds and the future Quickshell / Noctalia direction.

## Fixtures

Fixtures live under `tests/fixtures/` and are organized by mode first, then source/type. Metadata files (`*.meta.txt`) capture notes plus lightweight `preserve` and `avoid` rules so heuristics stay aligned with the product boundary.

Current corpus coverage includes:
- AI-terminal wrapped prose, spacing-noise wraps, wrapped inline-code followups, spacing-noise paragraphs, blank-line noise, and heading-padding cleanup
- TUI status-update bullets
- PI/TUI wrapped prose paragraphs
- PI/TUI wrapped bullet and numbered-list continuations
- wrapped doc and PI blockquotes
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
