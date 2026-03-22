# waytrim

waytrim is a Linux-native text repair tool for copy/display damage from terminal, TUI, AI-console, and similar plaintext-oriented interfaces.

It is a repair tool, not a rewrite tool.

## Current CLI

```bash
waytrim prose
waytrim command
waytrim auto
waytrim prose --preview
waytrim prose --clipboard
waytrim prose --clipboard --print
waytrim prose --clipboard --preview
```

The canonical interface is mode-centered. Clipboard actions use `waytrim <mode> --clipboard`, not `waytrim clipboard <mode>`.

The CLI reads from stdin and writes cleaned text to stdout by default. In clipboard mode it reads the current clipboard text, repairs it through the same core logic, and writes the repaired text back.

## Modes

### `prose`
- primary mode
- repairs wrapped paragraphs and copy-induced spacing noise
- preserves visible structure such as bullets, headings, blockquotes, fenced code blocks, indented sections, alignment-sensitive / table-ish text, and obvious standalone command blocks inside mixed prose snippets
- repairs obvious wrapped blockquotes while leaving fenced code, aligned columns, and standalone command blocks untouched

### `command`
- bounded secondary mode
- strips obvious prompts and repairs command presentation damage
- handles bare prompts and common host-style shell prompts conservatively
- leaves already-clean shell commands unchanged
- leaves mixed command/output snippets unchanged unless the command shape is obvious

### `auto`
- conservative convenience mode
- chooses command cleanup when the input is clearly command-like
- declines to merge short label-plus-command snippets such as `Install command:` followed by a command
- stays conservative on mixed prose-plus-command snippets and falls back to minimal prose-safe cleanup
- otherwise prefers prose repair or minimal prose-safe cleanup

## Preview

Use `--preview` to print a simple before/after diff-like view instead of cleaned text.

In clipboard mode, `--preview` is explicitly non-mutating. It shows what would change without writing back to the clipboard.

## Clipboard mode

Clipboard support is a manual adapter over the same repair core.

Behavior:
- `waytrim prose --clipboard` repairs current clipboard text and writes it back when it changes
- `waytrim prose --clipboard --print` prints repaired text to stdout and also writes it back when it changes
- `waytrim prose --clipboard --preview` previews changes without mutating the clipboard
- `clipboard unchanged` is a first-class success outcome when no effective change is needed
- empty clipboard input returns a clear success message instead of crashing
- `--clipboard --preview --print` is rejected as ambiguous

Runtime dependency:
- Wayland clipboard support uses `wl-paste` and `wl-copy`

## Development docs

- `docs/architecture.md`
- `docs/development.md`

## Testing

```bash
cargo test
cargo fmt --check
```

## Fixtures

Fixtures live under `tests/fixtures/` and are organized by mode first, then source/type. Metadata files (`*.meta.txt`) capture notes plus lightweight `preserve` and `avoid` rules so heuristics stay aligned with the product boundary.

Current corpus coverage includes:
- AI-terminal wrapped prose
- TUI status-update bullets
- PI/TUI wrapped prose paragraphs
- PI/TUI wrapped bullet and numbered-list continuations
- wrapped doc and PI blockquotes
- mixed doc and PI prose with preserved standalone command blocks
- alignment-sensitive / table-ish text that prose should preserve
- already-clean prose that should remain unchanged
- fenced-code preservation cases, including PI output
- bare and host-style shell prompts
- already-clean shell commands that should remain unchanged
- multiline PI command cleanup
- mixed command/output transcripts that should stay unchanged
- ambiguous label-plus-command, transcript-like, mixed prose-command, and aligned-columns snippets that `auto` should leave alone
- unchanged preview and clipboard no-op cases for safe inputs, including command-mode clipboard/transcript paths
