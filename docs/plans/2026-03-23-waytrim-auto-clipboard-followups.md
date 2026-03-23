# waytrim auto-clipboard follow-ups

Date: 2026-03-23
Status: next-step implementation plan

## Purpose

This plan captures the remaining work after landing:

- core repair engine
- CLI
- preview / explain
- manual clipboard flow
- IPC + local service
- Quickshell socket examples
- automatic clipboard watcher via `waytrim-watch`

The goal is to reach a polished daily workflow for:

> copy messy text, then paste already-clean text

without widening the product into rewrite behavior or duplicating repair logic outside the Rust core.

## Current shipped baseline

The current repo now has:

- conservative repair core with `prose`, `command`, and `auto`
- manual clipboard repair
- local IPC/service contract
- Quickshell example client and clipboard action
- automatic clipboard watcher with restore-last-original support
- systemd user service example for always-on watcher use

## Remaining work summary

The main remaining gaps are:

1. watcher control UX in Quickshell / Noctalia
2. status and notification UX
3. original/clean clipboard workflow polish
4. exact desktop/session startup docs and install flow
5. optional app-aware exclusions, only if real workflow evidence justifies them

## Priority order

1. Quickshell / Noctalia watcher controls
2. watcher status and notifications
3. restore-original and manual override UX
4. environment-specific service docs and install polish
5. corpus growth from real auto-watcher samples
6. app-aware exclusions, only if the corpus shows a real need

---

## 1. Quickshell / Noctalia watcher controls

### Goal

Make the watcher usable as a real desktop feature instead of just a terminal command.

### Deliverables

- a Quickshell-facing control object for:
  - watcher enabled/disabled state
  - chosen mode (`auto`, `prose`, maybe `command`)
  - last status (`updated`, `unchanged`, `empty`, `error`)
- a minimal Noctalia-facing action surface for:
  - toggle auto-clean on/off
  - restore original clipboard text
  - trigger one-shot manual clean if desired

### Recommended implementation

Keep this thin.

Do **not** move clipboard logic into QML.

Instead:

- add a small wrapper command layer around `waytrim-watch`, for example:
  - `waytrim-watch auto`
  - `waytrim-watch prose`
  - `waytrim-watch --restore-original`
- in Quickshell, use `Process` or `Quickshell.execDetached()` for lifecycle actions
- use the existing socket examples only for synchronous request/response actions
- let systemd own the long-running watcher lifecycle when possible

### Suggested files

- `contrib/quickshell/waytrim/WaytrimWatchControl.qml`
- `contrib/quickshell/waytrim/WaytrimNotifications.qml`
- optional helper shell script if Noctalia wants a tiny stable entrypoint

### Suggested QML surface

A minimal control object should expose:

- `enabled: bool`
- `mode: string`
- `busy: bool`
- `lastStatus: string`
- `lastMessage: string`
- `toggle()`
- `start(mode)`
- `stop()`
- `restoreOriginal()`

### Acceptance criteria

- watcher can be enabled and disabled from Quickshell
- restore-original can be triggered from Quickshell / Noctalia
- no repair heuristics are implemented in QML
- QML only calls `waytrim-watch`, `waytrimctl`, or the existing socket contract

---

## 2. Watcher status and notifications

### Goal

Give users confidence about what happened without creating noisy UX.

### Deliverables

- a small status model for watcher events
- optional notifications for:
  - clipboard updated
  - clipboard unchanged
  - watcher error
  - original restored

### Recommended implementation

Keep notifications conservative.

Good default:

- no popup for `unchanged`
- optional popup or subtle indicator for `updated`
- clear popup/log for `error`
- popup for `restored original`

### Implementation detail

The current watcher writes state and prints stderr messages, but it does not yet expose a live event stream.

Two reasonable next steps:

#### Option A: Keep it simple first

- Quickshell launches watcher through `Process`
- capture stderr with `StdioCollector` or `SplitParser`
- map watcher output lines into UI status

This is the simplest next slice.

#### Option B: Add a dedicated status socket later

- watcher writes structured status events to a local socket or file
- Quickshell subscribes to those events

This is cleaner long-term but not necessary for the next slice.

### Acceptance criteria

- user can tell whether the clipboard was updated or skipped
- errors are visible
- notification layer stays optional and thin
- watcher remains usable without notifications

---

## 3. Original/clean clipboard workflow polish

### Goal

Make auto-clean trustworthy by giving users an obvious escape hatch.

### Current baseline

Already shipped:

- watcher stores `last_original_input`
- `waytrim-watch --restore-original`

### Remaining improvements

- desktop-visible restore action
- clearer “what will happen” docs
- optional “clean once now” and “restore original” commands in Quickshell / Noctalia
- optional status indicator showing whether current clipboard likely came from watcher output

### Recommended implementation

Keep only one original buffer for now.

Do **not** build clipboard history.

If a more advanced restore flow is needed later, revisit only with real usage evidence.

### Acceptance criteria

- restore-original is available from desktop UI, not only terminal
- docs explain exactly what gets restored
- no clipboard history manager behavior is introduced

---

## 4. Desktop/session startup and install polish

### Goal

Make always-on use reliable in the real session environment.

### Remaining problems to solve

- Wayland environment variables may not reach user services automatically
- users need exact install instructions for systemd + Quickshell + Niri setups
- service restart and failure modes should be documented for normal use

### Recommended implementation

#### A. Improve docs first

Add exact sections for:

- direct terminal run
- systemd user service install
- importing `WAYLAND_DISPLAY` / `XDG_RUNTIME_DIR` when needed
- verifying watcher operation
- disabling the service
- restoring original clipboard after a bad clean

#### B. Add one environment-specific example

Because this repo is Wayland/Niri/Noctalia-oriented, include one example workflow for:

- Arch Linux
- systemd user service
- Niri session startup
- Quickshell running in the user session

### Acceptance criteria

- a user can install and enable the watcher with only repo docs
- failure modes are documented clearly
- docs stay aligned with the shipped service file and commands

---

## 5. Corpus growth from real auto-watcher samples

### Goal

Use real copied clipboard samples to harden trust boundaries in always-on mode.

### Why it matters now

Auto-clean increases the cost of false positives.

That means the highest-value heuristic work is now:

- more real samples
- more negative/no-op fixtures
- more examples of text that should **not** be rewritten automatically

### Suggested sample classes

- AI terminal answers copied from narrow panes
- markdown-ish docs snippets copied from browser or terminal
- mixed prose + command sections copied from docs
- package-manager output and shell transcript fragments
- tables and alignment-sensitive output
- small labels and short snippets that auto should leave alone

### Acceptance criteria

- every new positive sample has a nearby negative/no-op sample when overreach is plausible
- fixture metadata explains preserve/avoid intent
- docs fixture coverage stays aligned

---

## 6. App-aware exclusions, only if justified

### Goal

Avoid damaging workflows where automatic cleanup is clearly the wrong default.

### Important constraint

This should **not** be done speculatively.

Do it only if real usage shows repeated problems that cannot be solved by corpus tuning alone.

### Possible directions

- terminal-app exclusion list
- app allowlist / denylist for auto-clean
- mode override by app category

### Recommended implementation if needed later

- keep the policy surface small
- prefer explicit app IDs or window-class matches
- keep app-awareness outside the repair core
- implement it in watcher/UI config, not in the text heuristics

### Acceptance criteria

- problem is backed by real workflow examples
- integration-specific app routing lives outside the repair core
- defaults remain conservative

---

## Suggested next implementation sequence

### Phase 1

- add `WaytrimWatchControl.qml`
- add restore-original action in Quickshell / Noctalia
- add simple status parsing from watcher output

### Phase 2

- document exact systemd + Niri + Quickshell install flow
- add one environment-specific end-to-end walkthrough

### Phase 3

- grow the real clipboard sample corpus
- tune auto behavior only from failing or risky samples

### Phase 4

- consider app-aware exclusions if the corpus and real usage justify it

## Explicit non-goals for the next phase

Do not add these unless new evidence appears:

- broad transcript parsing
- general formatter behavior
- markdown prettification
- clipboard history manager behavior
- heavy daemon state beyond one original clipboard backup and watcher state
- UI-side repair logic

## Success criteria

The next phase is successful if:

- the watcher can be controlled from Quickshell / Noctalia
- users can restore original clipboard text easily
- always-on usage becomes practical in a real Wayland session
- trust improves through corpus growth, not aggressive heuristics
- all repair decisions still come from the same Rust core
