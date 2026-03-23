# Development

## Requirements

- Rust stable toolchain
- `cargo`

If Rust is not installed yet:

```bash
rustup default stable
```

## Build

```bash
cargo build
```

## Run

### Prose mode

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run -- prose
```

### Command mode

```bash
printf '$ cargo test \\\n    --test cli_smoke\n' | cargo run -- command
printf 'ayagmar@archbox:~/projects/waytrim$ cargo test \\\n  --test cli_smoke\n' | cargo run -- command
```

### Auto mode

```bash
printf 'Value one  \n\n\nValue two   \n' | cargo run -- auto
printf 'Install command:\napt-get install ripgrep\n' | cargo run -- auto
```

### Preview output

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run -- prose --preview
printf 'This is a wrapped\nparagraph.\n' | cargo run -- --no-preview
```

### Explain output

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run -- prose --explain
printf '$ cargo test \\\n  --test clipboard_flow\n' | cargo run -- command --explain
printf 'This is a wrapped\nparagraph.\n' | cargo run -- --no-explain
```

### User config

Config path:
- `XDG_CONFIG_HOME/waytrim/config.toml`
- `~/.config/waytrim/config.toml`

Rules:
- missing config is silent
- invalid config warns and falls back to defaults
- CLI flags override config file values

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

Config smoke example:

```bash
config_home="$(mktemp -d)"
mkdir -p "$config_home/waytrim"
cat > "$config_home/waytrim/config.toml" <<'EOF'
[defaults]
mode = "auto"
preview = true
EOF
printf 'This is a wrapped\nparagraph.\n' | XDG_CONFIG_HOME="$config_home" cargo run --
```

### Clipboard adapter shape

```bash
cargo run -- prose --clipboard
cargo run -- prose --clipboard --print
cargo run -- prose --clipboard --preview
cargo run -- prose --clipboard --explain
```

### Local service and IPC

```bash
cargo run --bin waytrimd
printf 'This is a wrapped\nparagraph.\n' | cargo run --bin waytrimctl -- repair prose
printf 'This is a wrapped\nparagraph.\n' | cargo run --bin waytrimctl -- repair prose --text
cargo run --bin waytrimctl -- shutdown
```

Notes:
- clipboard mode stays mode-centered
- `--preview` is non-mutating in clipboard mode
- `--explain` is non-mutating in clipboard mode
- `--print` means print repaired text and also write it back
- `--clipboard --preview --print` is invalid
- `--clipboard --explain --print` is invalid
- `clipboard unchanged` should be reported clearly when nothing changes
- empty clipboard input should report a clear success message
- clipboard integration depends on `wl-paste` and `wl-copy`
- explicit overrides `--no-preview`, `--no-explain`, `--no-print`, and `--no-clipboard` can disable config-provided defaults
- the IPC socket defaults to `XDG_RUNTIME_DIR/waytrim/waytrim.sock`
- when `XDG_RUNTIME_DIR` is missing, the fallback is `${TMPDIR:-/tmp}/waytrim-<uid>/waytrim.sock`
- the service refuses to remove non-socket paths and refuses startup when another listener already owns the socket
- `waytrimctl` prints JSON by default and can print only repaired text with `--text`

## Test and format

```bash
cargo fmt --check
cargo test
```

## Repository layout

```text
src/
  core/         repair modes, detection, rendering, and policy/report types
  lib.rs        crate exports
  cli.rs        CLI config resolution and clipboard flow
  config.rs     XDG user config loader
  clipboard.rs  wl-paste / wl-copy adapter
  ipc.rs        Unix-socket request/response types and helpers
  service.rs    local service loop over the repair core
  main.rs       canonical CLI adapter
  bin/
    waytrimd.rs   daemon entrypoint
    waytrimctl.rs IPC client entrypoint

contrib/
  niri/
    waytrim-clipboard-prose  thin helper for mode-centered clipboard cleanup
  quickshell/
    waytrim/
      WaytrimClient.qml           Quickshell socket client example
      WaytrimClipboardAction.qml  Quickshell clipboard action example

tests/
  *.rs          integration tests
  support/      shared test helpers
  fixtures/     sample corpus inputs, expected outputs, metadata

docs/
  architecture.md
  development.md
  integrations.md
```

## Notes

- Keep new heuristics fixture-driven.
- Prefer conservative repairs over aggressive cleanup.
- Add negative fixtures when a change could overreach.
- Keep platform-specific integration work outside the core library.
- Keep clipboard behavior as a thin adapter over the same repair contracts.
- Current prose boundaries explicitly cover wrapped PI/TUI paragraphs, wrapped bullet and numbered-list continuations, wrapped blockquotes, AI-terminal spacing-noise wraps, wrapped inline-code followups, useless blank-line noise inside one paragraph, copy-induced heading padding, inline-code bullets, fenced-code preservation, alignment-sensitive / table-ish text, already-clean no-op prose, heading and indented-section no-op cases, and obvious standalone command blocks inside mixed prose snippets.
- Current command boundaries explicitly cover common host-style shell prompts, already-clean shell command no-op cases including clean pipelines, multiline PI command cleanup, transcript-with-status refusal, and host-prompt-plus-output refusal.
- Current auto boundaries explicitly avoid merging short label-plus-command snippets, transcript-like command/output captures, mixed prose-plus-command snippets, prose-then-command examples, install sections, indented command examples, alignment-sensitive column text, and prose-preferred wrapped snippets under the conservative default, while leaving already-clean command-like input, headings, and indented-section fixtures untouched.
- Current preview, explain, and clipboard boundaries explicitly cover unchanged no-op outcomes in addition to changed-text paths, including command-mode clipboard/transcript cases, clean command no-op fixtures, heading/indented prose fixtures, and policy-backed auto / command-block paths.
- Current config-backed policy surface intentionally stays small: `protect.aligned_columns`, `protect.command_blocks`, and `[auto].policy`.
