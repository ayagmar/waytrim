# Integrations

waytrim keeps the repair engine separate from delivery layers.

Current state:
- canonical human-facing interface: `waytrim`
- optional local service: `waytrimd`
- optional JSON IPC client: `waytrimctl`
- automatic clipboard watcher: `waytrim-watch`
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

## Automatic clipboard watcher

Run the watcher directly:

```bash
waytrim-watch auto
waytrim-watch prose
waytrim-watch command
```

One-shot and recovery commands:

```bash
waytrim-watch auto --clean-once
waytrim-watch --restore-original
waytrim-watch --status
waytrim-watch --status --json
```

Behavior:
- uses `wl-paste --watch` to react to clipboard changes
- repairs clipboard text through the same Rust core
- keeps one original clipboard buffer in watcher state for `--restore-original`
- records the last watcher mode, status, message, clipboard source, and restore availability in watcher state
- keeps all cleanup logic and manual override behavior out of QML

Restore semantics are intentionally narrow:
- only the most recent pre-clean clipboard text is stored
- `--restore-original` writes that one saved value back to the clipboard
- no clipboard history manager behavior is introduced

### Direct terminal run

Use this when you want to test behavior before wiring it into the session:

```bash
waytrim-watch auto
```

In another terminal, copy some wrapped text, then inspect the latest watcher snapshot:

```bash
waytrim-watch --status
```

If a cleanup was wrong, restore the saved pre-clean clipboard value:

```bash
waytrim-watch --restore-original
```

### systemd user service install

Example service files:
- `contrib/systemd/user/waytrim-watch.service`
- `contrib/systemd/user/waytrim-watch@.service`

Use the fixed service when auto mode is enough:

```bash
mkdir -p ~/.config/systemd/user
cp /path/to/waytrim/contrib/systemd/user/waytrim-watch.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now waytrim-watch.service
```

Use the templated service when Quickshell / Noctalia should be able to switch modes:

```bash
mkdir -p ~/.config/systemd/user
cp /path/to/waytrim/contrib/systemd/user/waytrim-watch@.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user start waytrim-watch@auto.service
```

The shipped unit files assume a user-local install at:

```bash
~/.local/bin/waytrim-watch
```

If you install `waytrim-watch` somewhere else, adjust `ExecStart=` to match your real binary path.

If your graphical session does not already export Wayland environment variables into user services, import them before enabling or starting the service:

```bash
systemctl --user import-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR
```

### Verify, stop, and recover

Check service status:

```bash
systemctl --user status waytrim-watch.service
systemctl --user status waytrim-watch@auto.service
waytrim-watch --status --json
```

Useful failure checks:

```bash
journalctl --user -u waytrim-watch.service -n 50 --no-pager
journalctl --user -u waytrim-watch@auto.service -n 50 --no-pager
```

Stop or disable the watcher:

```bash
systemctl --user stop waytrim-watch.service
systemctl --user disable waytrim-watch.service
systemctl --user stop waytrim-watch@auto.service
systemctl --user disable waytrim-watch@auto.service
```

If a bad clean already landed in the clipboard:

```bash
waytrim-watch --restore-original
```

### Arch Linux + Niri + Quickshell walkthrough

A practical always-on setup for this repo's target environment:

1. Build or install `waytrim` so `waytrim-watch` is on your `PATH`.
2. Install the templated user service:

```bash
mkdir -p ~/.config/systemd/user
cp /path/to/waytrim/contrib/systemd/user/waytrim-watch@.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable waytrim-watch@auto.service
```

3. Copy the Niri session helper somewhere stable if you do not want to call it from the checkout directly:

```bash
install -Dm755 /path/to/waytrim/contrib/niri/waytrim-watch-session ~/.local/bin/waytrim-watch-session
```

4. In your Niri config, run the helper once at session startup so the user manager learns the current Wayland environment and restarts the enabled watcher unit inside the session:

```kdl
spawn-at-startup "sh" "-lc" "~/.local/bin/waytrim-watch-session waytrim-watch@auto.service"
```

5. Copy the Quickshell example files into your shell config:

```bash
mkdir -p ~/.config/quickshell/waytrim
cp /path/to/waytrim/contrib/quickshell/waytrim/*.qml ~/.config/quickshell/waytrim/
```

6. In Quickshell / Noctalia, point `WaytrimWatchControl` at the same service template:

```qml
WaytrimWatchControl {
    serviceUnitTemplate: "waytrim-watch@.service"
}
```

7. After logging into Niri, verify that the watcher is active and that Quickshell sees the same state:

```bash
systemctl --user status waytrim-watch@auto.service
waytrim-watch --status
```

If clipboard access fails after login, the usual fix is to confirm the Niri startup helper ran and then check the user-service journal shown above.

## Manual Niri workflow

Helper entrypoints:
- `contrib/niri/waytrim-clipboard-prose`
- `contrib/niri/waytrim-watch-session`

They are intentionally thin and just call:

```bash
waytrim prose --clipboard
systemctl --user import-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR
```

with a conditional restart for an enabled watcher unit.

End-to-end Niri recipe:

1. Build or install `waytrim` so the binary is on your `PATH`.
2. The helpers are already shipped as executable scripts. If you copied them elsewhere and lost the mode bit, restore it with:

```bash
chmod +x /path/to/waytrim/contrib/niri/waytrim-clipboard-prose
chmod +x /path/to/waytrim/contrib/niri/waytrim-watch-session
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
- `contrib/quickshell/waytrim/WaytrimWatchControl.qml`
- `contrib/quickshell/waytrim/WaytrimNotifications.qml`

These examples stay thin:
- manual cleanup still talks to the Unix socket through Quickshell's `Socket` type
- watcher lifecycle talks to `waytrim-watch`, `systemctl --user`, and watcher state
- no repair heuristics are implemented in QML

### What the examples give you

- `WaytrimClient.qml`
  - sends `repair` requests to the local socket
  - parses the JSON response
  - exposes typed last-result fields such as `lastOutput`, `lastChanged`, `lastEffectiveMode`, and `lastDecision`
- `WaytrimClipboardAction.qml`
  - reads `Quickshell.clipboardText`
  - sends it through `WaytrimClient`
  - writes repaired text back only when the report says it changed
  - reports `updated`, `unchanged`, `empty`, or `error`
- `WaytrimWatchControl.qml`
  - starts and stops a systemd-managed watcher when you use `waytrim-watch@.service`
  - exposes `enabled`, `mode`, `busy`, `lastStatus`, `lastMessage`, `originalAvailable`, and `clipboardSource`
  - offers `toggle()`, `start(mode)`, `stop()`, `cleanOnce(mode)`, and `restoreOriginal()`
  - polls `waytrim-watch --status --json` instead of guessing watcher state in QML
- `WaytrimNotifications.qml`
  - listens for watcher event-id changes
  - defaults to conservative popups: errors and restore events on, unchanged off, updated off
  - can emit a shell-level signal or optionally call `notify-send`

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

### Minimal watcher control example

```qml
import QtQuick
import Quickshell
import qs.waytrim

QtObject {
    id: root

    WaytrimWatchControl {
        id: watchControl
        serviceUnitTemplate: "waytrim-watch@.service"
    }

    WaytrimNotifications {
        control: watchControl

        onNotificationRequested: (title, message, urgency) => {
            console.log(`waytrim: ${urgency}: ${title}: ${message}`)
        }
    }

    function toggleWatcher() {
        watchControl.toggle()
    }

    function restoreOriginalClipboard() {
        watchControl.restoreOriginal()
    }

    function cleanClipboardNow() {
        watchControl.cleanOnce("auto")
    }
}
```

### Socket path note

`WaytrimClient.qml` auto-fills `socketPath` only when `XDG_RUNTIME_DIR` is available.
If your service is using an explicit or fallback socket path, set `socketPath` yourself in QML.

### Noctalia direction

The intended Noctalia path is:
1. keep all repair logic in the Rust core
2. let Quickshell or Noctalia call the local socket contract for manual cleanup
3. let Quickshell or Noctalia call `waytrim-watch`, `systemctl --user`, and `waytrim-watch --status --json` for watcher UX
4. branch on machine-readable status instead of re-implementing heuristics
5. use `report.changed`, `report.output`, `report.effective_mode`, `report.decision`, and watcher status fields for UI behavior

That keeps Quickshell and Noctalia as thin UI adapters over stable local contracts instead of re-implementing repair logic.
