# Getting started

## Install

Build the binaries:

```bash
cargo build --release --bin waytrim --bin waytrim-watch --bin waytrimctl --bin waytrimd
```

For the shipped user service files, install the binaries to `~/.local/bin`:

```bash
install -Dm755 target/release/waytrim ~/.local/bin/waytrim
install -Dm755 target/release/waytrim-watch ~/.local/bin/waytrim-watch
install -Dm755 target/release/waytrimctl ~/.local/bin/waytrimctl
install -Dm755 target/release/waytrimd ~/.local/bin/waytrimd
```

Or run them directly with `cargo run --bin ...` while developing.

Wayland clipboard integration depends on:
- `wl-paste`
- `wl-copy`

## First use

Repair wrapped prose from stdin:

```bash
printf 'This is a wrapped\nparagraph.\n' | waytrim prose
```

Preview instead of rewriting output:

```bash
printf 'This is a wrapped\nparagraph.\n' | waytrim prose --preview
```

Explain what changed:

```bash
printf 'This is a wrapped\nparagraph.\n' | waytrim prose --explain
```

## Clipboard use

Repair the current clipboard in place:

```bash
waytrim prose --clipboard
```

Preview without mutating the clipboard:

```bash
waytrim prose --clipboard --preview
```

## Always-on clipboard cleanup

Start the watcher directly:

```bash
waytrim-watch auto
```

Check watcher state:

```bash
waytrim-watch --status
```

Restore the last saved pre-clean clipboard value:

```bash
waytrim-watch --restore-original
```

## Desktop integration

For Niri, Quickshell, Noctalia, and systemd user-service examples, see:
- `docs/integrations.md`
