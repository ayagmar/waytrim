# waytrim product design

Date: 2026-03-21
Status: brainstormed draft

## Overview

waytrim is a Linux-native text repair tool for cleaning text that became messy because of how it was displayed, wrapped, padded, or copied from terminal, TUI, AI-console, and similar plaintext-oriented interfaces.

Its purpose is to repair copy-induced noise while preserving meaning and visible structure. waytrim is not a writing improver, formatter, or general prettifier. It should remove display and copy artifacts, not reinterpret content.

The product should be defined as **repair, not rewrite**.

## Product framing

waytrim is best understood as:

- a **text repair engine** first
- a **CLI-first tool** as its canonical interface
- a tool for **terminal-origin or similarly noisy copied text**
- a product with **clipboard and desktop integrations as adapter layers**

waytrim is **not** primarily:

- a clipboard manager
- a general-purpose formatter
- a markdown prettifier
- a writing improver
- a shell helper, even though command cleanup is an important bounded mode

## Problem statement

Users often copy text from terminals, TUIs, AI terminal/chat-style interfaces, docs, and similar environments where the displayed form is not the intended final form.

Common damage includes:

- awkward line wrapping
- broken paragraphs
- large or inconsistent heading padding
- useless blank lines
- terminal-style indentation noise
- prompt characters attached to copied commands
- mixed plaintext artifacts from narrow panes or wrapped interfaces

waytrim should repair this damage conservatively. The core product failure is silent overreach: output that looks cleaner but changes meaning or structure unexpectedly.

## Design principles

- **Repair, not rewrite**
- **Preserve meaning first**
- **Preserve visible structure by default**
- **Be conservative when uncertain**
- **Use explicit modes instead of pretending all text is the same**
- **Keep the core engine separate from platform integrations**
- **Prefer simple, testable heuristics over ambitious interpretation**

## Target users

Primary users:

- Linux and Wayland users
- terminal- and TUI-heavy users
- users copying prose from AI terminal/chat-style tools
- users copying messy plaintext from docs, shell-adjacent tools, or constrained-width interfaces

Secondary users:

- users who want command cleanup for copied shell commands
- users who want clipboard-based cleanup workflows after the core behavior is trusted

## Canonical interface vs daily workflow

These should be treated separately.

### Canonical interface

The canonical interface should be a CLI-first text transformer built around the repair engine.

Examples:

- `waytrim prose`
- `waytrim command`
- `waytrim auto`
- stdin-based Unix workflows

This keeps the product architecture clean, testable, composable, and independent of desktop integration details.

### Likely daily workflow

The most common day-to-day workflow may become a manual clipboard clean action, such as a keybind or shell command that cleans the current clipboard contents.

That workflow is important, but it should be implemented as an adapter over the same repair core rather than defining the product identity.

## Cleanup modes

## `prose`

Primary mode.

Purpose:

- repair wrapped or noisy terminal-origin prose
- clean copy/display artifacts without reformatting or prettifying
- preserve visible structure unless highly confident a repair is safe

Safe operations include:

- repairing broken paragraph lines
- removing obviously useless padding
- trimming edge whitespace caused by display artifacts
- normalizing excessive blank lines
- repairing awkward wraps caused by narrow displays

`prose` should not behave like a document formatter or generic reflow tool.

## `command`

Bounded secondary mode.

Purpose:

- repair copied command presentation damage
- strip obvious prompt artifacts
- clean command blocks without attempting deep shell interpretation

Safe operations may include:

- removing obvious leading prompts such as `$` or `#`
- repairing accidental visual line breaks in command blocks
- removing copy-induced padding around commands

`command` should remain narrowly defined and should not become a general shell transcript parser early on.

## `auto`

Conservative convenience mode.

Purpose:

- choose between prose-like and command-like cleanup when the classification is reasonably clear

Rules:

- should be secondary, not the center of gravity
- should fail safe by doing less when uncertain
- should prefer minimal change over aggressive cleanup

## Preservation contract

In default prose behavior, the following should be treated as high-protection structures unless the tool is very confident they are being repaired rather than altered:

- code blocks
- bullets
- numbered lists
- headings
- blockquotes
- indented sections

This allows paragraph repair to remain useful while preserving the structures users are most likely to perceive as intentional.

## Heuristic direction

waytrim should rely on bounded, testable heuristics rather than broad semantic interpretation.

Signals that may suggest prose-like input:

- mid-sentence line breaks
- natural-language punctuation patterns
- repeated wrap widths
- paragraph-like groupings
- low shell-token density

Signals that may suggest command-like input:

- obvious prompt markers
- shell metacharacter density
- flag-heavy tokens
- continuation patterns common in copied shell commands

Before modifying text, the engine should detect and protect likely structured regions. When uncertain, it should prefer leaving content partially messy instead of making a risky transformation.

## Trust and explainability

Trust is a core product requirement.

The canonical output should be cleaned text, but waytrim should also grow an early preview/explain/diff story so users can understand what changed and why. This is especially important for a product whose success depends on conservative repairs and visible structure preservation.

User-facing strength or safety levels may be useful later, but they should not define the initial product surface before heuristics are validated on real samples.

## Feature buckets

### Core product

- repair engine
- explicit `prose`, `command`, and conservative `auto` modes
- CLI-first interface
- preservation-first behavior
- fixture-based validation with real sample inputs
- early preview/explain/diff support

### Early additions

- manual clipboard clean action
- improved AI/TUI-specific artifact handling
- optional configuration once defaults are stable

### Later integrations

- Wayland-native clipboard adapter improvements
- daemon or background service
- IPC interface
- Quickshell / Noctalia integration
- Niri-friendly keybind workflows

### Risky or uncertain ideas

- aggressive smart cleanup
- broad transcript parsing
- deep shell-aware transformations
- plugin architecture before real extension pressure exists
- rich semantic format-specific normalization

### Out of scope

- clipboard history management
- writing improvement
- markdown prettification
- general code formatting
- shell explanation or validation tools

## Architecture / integration boundaries

The product core should remain a text repair engine with stable input/output contracts.

Platform and integration concerns should be treated as layers around that core.

### Core product boundary

The core product is:

- a Linux-native text repair engine for copy/display damage
- exposed first through a CLI-first interface
- designed for maintainability, testability, and future extension

### Integration boundary

The following should be treated as adapter or delivery layers, not the product core:

- Wayland clipboard handling
- manual clipboard-clean actions
- future daemon behavior
- IPC surfaces
- Quickshell / Noctalia integration
- Niri-oriented user workflows

Constraints:

- Wayland and clipboard handling should remain adapter layers over the same repair contracts
- any future daemon should build on the same core engine behavior rather than introducing separate cleanup logic
- Quickshell / Noctalia integration should stay thin and depend on stable CLI or IPC boundaries rather than internal engine details
- Niri-friendly workflows matter as delivery UX, not as product identity

This preserves clean architecture without prematurely freezing crate layout, IPC shape, or UI design.

## Suggested technical direction

Implementation should remain exploratory but likely follows this direction:

- Rust is a strong fit for the core tool
- keep the repair engine isolated from entrypoint-specific concerns
- use thin adapters for CLI, clipboard integration, and any later service or UI layers
- prefer simple rule pipelines and strong fixture tests over heavy abstraction early
- let future IPC or integration boundaries emerge from proven core workflows

## Best first complete slice

A strong first complete slice would be:

- Rust-based CLI repair engine
- `prose` as the primary mode
- bounded `command` mode
- conservative `auto` mode
- preservation-first heuristics
- real-world sample corpus and fixture tests
- preview/explain/diff support soon after the core output path
- manual clipboard clean action as an early adapter

This is not a statement that the product ends there. It is the smallest complete slice that establishes the product identity and trust model correctly.

## Open questions

- What exact sample corpus best represents the real-world text waytrim should repair?
- Which paragraph-repair heuristics are useful without becoming reflow?
- How much prompt stripping is safe in `command` mode?
- How often should `auto` decline to classify and do less?
- When, if ever, should safety or strength levels become user-facing?
- What adapter shape is best for clipboard integration on Wayland in your environment?
