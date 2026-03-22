# Architecture

waytrim is structured around a small repair core with thin delivery layers.

## Design goals

- repair, not rewrite
- preserve meaning and visible structure
- keep platform integrations out of the core
- make heuristics easy to test with fixtures
- keep future clipboard, daemon, and UI integrations thin

## Layers

### Core library

`src/lib.rs`

The core library owns:

- repair modes (`prose`, `command`, `auto`)
- conservative cleanup heuristics
- preview rendering
- stable text-in / text-out behavior

The core should remain independent from:

- Wayland clipboard APIs
- daemon state
- IPC transport details
- Quickshell / Noctalia UI concerns
- Niri-specific workflow glue

### CLI adapter

`src/main.rs`

The CLI is the current canonical interface. It is intentionally thin:

- parse args
- read stdin or clipboard text through an adapter
- call the core library
- print repaired text or preview output
- write repaired clipboard text back when clipboard mode is active

The preferred clipboard UX is mode-centered: `waytrim prose --clipboard`, not `waytrim clipboard prose`.

## Mode boundaries

### Prose

Primary mode for repairing wrapped terminal-origin prose and copy-induced spacing noise while preserving structure.

### Command

Bounded mode for copied command presentation cleanup. It strips obvious prompts and repairs line continuations without trying to become a shell interpreter.

### Auto

Conservative convenience mode. It chooses a clear mode when confidence is high and otherwise falls back to minimal prose-safe cleanup.

## Testing strategy

Fixtures under `tests/fixtures/` are the main behavioral contract.

The test layout mirrors the product boundary:

- mode-specific integration tests
- positive repair cases
- negative preservation cases
- metadata describing preserve/avoid intent

## Future integration direction

These are expected to stay outside the core:

- manual clipboard-clean action
- Wayland clipboard adapter
- background daemon
- IPC layer
- Quickshell / Noctalia integration
- Niri-oriented keybind workflows

Those layers should call the same core repair contracts rather than introducing separate cleanup logic.

For the manual clipboard slice, `--preview` must remain non-mutating, `--print` must have explicit semantics, and `clipboard unchanged` should be treated as a first-class successful outcome.
