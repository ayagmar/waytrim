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
```

### Auto mode

```bash
printf 'Value one  \n\n\nValue two   \n' | cargo run -- auto
```

### Preview output

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run -- prose --preview
```

### Planned clipboard adapter shape

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

## Test and format

```bash
cargo fmt --check
cargo test
```

## Repository layout

```text
src/
  lib.rs        core repair logic
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
