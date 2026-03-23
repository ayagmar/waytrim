# Troubleshooting

## `waytrim prose --clipboard` fails

Make sure these commands exist on `PATH`:
- `wl-paste`
- `wl-copy`

## The watcher service starts but does not see the clipboard

The usual problem is missing session environment in the systemd user manager.

Import the current Wayland session variables:

```bash
systemctl --user import-environment WAYLAND_DISPLAY XDG_RUNTIME_DIR
```

If you use Niri, the shipped helper can do that during session startup:

```bash
/path/to/waytrim/contrib/niri/waytrim-watch-session waytrim-watch@auto.service
```

## Check watcher state

```bash
waytrim-watch --status
waytrim-watch --status --json
```

## Check service logs

```bash
journalctl --user -u waytrim-watch.service -n 50 --no-pager
journalctl --user -u waytrim-watch@auto.service -n 50 --no-pager
```

## Stop or disable the watcher

```bash
systemctl --user stop waytrim-watch.service
systemctl --user disable waytrim-watch.service
systemctl --user stop waytrim-watch@auto.service
systemctl --user disable waytrim-watch@auto.service
```

## Undo a bad automatic clean

```bash
waytrim-watch --restore-original
```

This restores only one saved pre-clean clipboard value. It is not a clipboard history feature.

If you also run a clipboard history manager such as cliphist, it is normal to see both the original and cleaned entries in history. History is still useful as a broader safety net; `--restore-original` is just the fast one-step undo for the last waytrim replacement.

## The watcher left text unchanged

That is often the correct result.

`auto` is intentionally conservative and should prefer a no-op over a risky rewrite.

## A real copied sample still looks wrong

Save the raw copied text and turn it into a fixture near the affected mode. The repo is intentionally fixture-driven so trust-boundary changes come from real samples.
