# Integrations

waytrim keeps the repair engine separate from delivery layers.

Current state:
- canonical human-facing interface: `waytrim`
- optional local service: `waytrimd`
- optional JSON IPC client: `waytrimctl`
- manual Niri/Wayland helper: `contrib/niri/waytrim-clipboard-prose`

## Core service contract

Socket path:
- default: `$XDG_RUNTIME_DIR/waytrim/waytrim.sock`
- fallback: `${TMPDIR:-/tmp}/waytrim.sock`

IPC version:
- `1`

### Request

```json
{
  "type": "repair",
  "version": 1,
  "mode": "prose",
  "text": "This is a wrapped\nparagraph.\n"
}
```

Optional `policy` may be sent with the same fields as `RepairPolicy`:

```json
{
  "protect_aligned_columns": true,
  "protect_command_blocks": true,
  "auto_policy": "conservative"
}
```

Shutdown request:

```json
{
  "type": "shutdown",
  "version": 1
}
```

### Response

Successful repair response:

```json
{
  "status": "ok",
  "version": 1,
  "report": {
    "requested_mode": "auto",
    "effective_mode": "prose",
    "decision": "auto_prose",
    "output": "This is a wrapped paragraph.\n",
    "changed": true,
    "explain": [
      { "message": "classified input as prose-like" },
      { "message": "joined wrapped paragraph lines 1-2" }
    ]
  }
}
```

Error response:

```json
{
  "status": "error",
  "version": 1,
  "message": "unsupported ipc version: 99"
}
```

## Local service usage

Start the service:

```bash
cargo run --bin waytrimd
cargo run --bin waytrimd -- --socket /tmp/waytrim.sock
```

Send a repair request and print JSON:

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run --bin waytrimctl -- repair prose
```

Send a repair request and print only repaired text:

```bash
printf 'This is a wrapped\nparagraph.\n' | cargo run --bin waytrimctl -- repair prose --text
```

Stop the service:

```bash
cargo run --bin waytrimctl -- shutdown
```

## Manual Niri workflow

Helper entrypoint:
- `contrib/niri/waytrim-clipboard-prose`

It is intentionally thin and just calls:

```bash
waytrim prose --clipboard
```

Example Niri bind:

```kdl
binds {
    Mod+Shift+v { spawn "sh" "-lc" "/path/to/waytrim/contrib/niri/waytrim-clipboard-prose"; }
}
```

If you want repaired text echoed to stdout for debugging:

```kdl
binds {
    Mod+Shift+Ctrl+v { spawn "sh" "-lc" "/path/to/waytrim/contrib/niri/waytrim-clipboard-prose --print"; }
}
```

## Quickshell / Noctalia direction

The intended Quickshell path is:
1. keep all repair logic in the Rust core
2. call `waytrimctl` or the Unix socket directly
3. consume the JSON `report`
4. let Quickshell decide UI, notifications, and Noctalia workflow behavior

That keeps Quickshell and Noctalia as thin UI adapters over a stable local contract instead of re-implementing heuristics.
