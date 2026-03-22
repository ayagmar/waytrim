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
```

### Clipboard adapter shape

```bash
cargo run -- prose --clipboard
cargo run -- prose --clipboard --print
cargo run -- prose --clipboard --preview
```

Notes:
- clipboard mode stays mode-centered
- `--preview` is non-mutating in clipboard mode
- `--print` means print repaired text and also write it back
- `--clipboard --preview --print` is invalid
- `clipboard unchanged` should be reported clearly when nothing changes
- empty clipboard input should report a clear success message
- clipboard integration depends on `wl-paste` and `wl-copy`

## Test and format

```bash
cargo fmt --check
cargo test
```

## Repository layout

```text
src/
  lib.rs        core repair logic
  cli.rs        CLI config and clipboard flow
  clipboard.rs  wl-paste / wl-copy adapter
  main.rs       CLI adapter

tests/
  *.rs          integration tests
  support/      shared test helpers
  fixtures/     sample corpus inputs, expected outputs, metadata

docs/
  architecture.md
  development.md
```

## Notes

- Keep new heuristics fixture-driven.
- Prefer conservative repairs over aggressive cleanup.
- Add negative fixtures when a change could overreach.
- Keep platform-specific integration work outside the core library.
- Keep clipboard behavior as a thin adapter over the same repair contracts.
- Current prose boundaries explicitly cover wrapped blockquotes and fenced-code preservation.
- Current command boundaries explicitly cover common host-style shell prompts.
- Current auto boundaries explicitly avoid merging short label-plus-command snippets.
