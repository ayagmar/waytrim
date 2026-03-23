# Integrations

waytrim keeps the repair engine separate from delivery layers.

Current state:
- canonical human-facing interface: `waytrim`
- optional local service: `waytrimd`
- optional JSON IPC client: `waytrimctl`
- manual Niri/Wayland helper: `contrib/niri/waytrim-clipboard-prose`
- Quickshell example client: `contrib/quickshell/waytrim/`

## Core service contract

Socket path:
- default: `$XDG_RUNTIME_DIR/waytrim/waytrim.sock`
- fallback: `${TMPDIR:-/tmp}/waytrim-<uid>/waytrim.sock`

Service startup safety:
- refuses to remove a non-socket path
- refuses startup if another service is already listening on the socket
- removes only confirmed stale socket files

IPC version:
- `1`

Compatibility expectations:
- clients must send a `version`
- the service returns an error for unsupported versions and does not process the request
- incompatible request or response shape changes require a new `IPC_VERSION`
- Quickshell / Noctalia clients should branch on `status` and `version`, then read `report`

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

End-to-end Niri recipe:

1. Build or install `waytrim` so the binary is on your `PATH`.
2. The helper is already shipped as an executable script. If you copied it elsewhere and lost the mode bit, restore it with:

```bash
chmod +x /path/to/waytrim/contrib/niri/waytrim-clipboard-prose
```

3. Add binds like this to your Niri config:

```kdl
binds {
    Mod+Shift+v { spawn "sh" "-lc" "/path/to/waytrim/contrib/niri/waytrim-clipboard-prose"; }
    Mod+Shift+Ctrl+v { spawn "sh" "-lc" "/path/to/waytrim/contrib/niri/waytrim-clipboard-prose --print"; }
}
```

4. Reload Niri, copy wrapped text, then press `Mod+Shift+v` to repair the current clipboard in place.
5. Use `Mod+Shift+Ctrl+v` when you want the same flow plus stdout output for debugging.

## Quickshell / Noctalia

Example files:
- `contrib/quickshell/waytrim/WaytrimClient.qml`
- `contrib/quickshell/waytrim/WaytrimClipboardAction.qml`

These examples talk to the Unix socket directly through Quickshell's `Socket` type.
They do not re-implement repair logic.

### What the example gives you

- `WaytrimClient.qml`
  - sends `repair` requests to the local socket
  - parses the JSON response
  - exposes typed last-result fields such as `lastOutput`, `lastChanged`, `lastEffectiveMode`, and `lastDecision`
- `WaytrimClipboardAction.qml`
  - reads `Quickshell.clipboardText`
  - sends it through `WaytrimClient`
  - writes repaired text back only when the report says it changed
  - reports `updated`, `unchanged`, `empty`, or `error`

### Quickshell setup

Because Quickshell root-relative imports only work inside the shell directory, copy the example folder into your shell config, for example:

```bash
mkdir -p ~/.config/quickshell/waytrim
cp /path/to/waytrim/contrib/quickshell/waytrim/*.qml ~/.config/quickshell/waytrim/
```

Then import it from your shell config:

```qml
import QtQuick
import Quickshell
import qs.waytrim
```

### Minimal clipboard action example

```qml
import QtQuick
import Quickshell
import qs.waytrim

QtObject {
    id: root

    WaytrimClipboardAction {
        id: repairClipboard

        onFinished: (status, message) => {
            console.log(`waytrim: ${status}: ${message}`)
        }
    }

    function repairNow() {
        repairClipboard.trigger()
    }
}
```

### Socket path note

`WaytrimClient.qml` auto-fills `socketPath` only when `XDG_RUNTIME_DIR` is available.
If your service is using an explicit or fallback socket path, set `socketPath` yourself in QML.

### Noctalia direction

The intended Noctalia path is:
1. keep all repair logic in the Rust core
2. let Quickshell or Noctalia call the local socket contract
3. branch on the response `status`
4. use `report.changed`, `report.output`, `report.effective_mode`, and `report.decision` for UI and workflow behavior

That keeps Quickshell and Noctalia as thin UI adapters over a stable local contract instead of re-implementing heuristics.
