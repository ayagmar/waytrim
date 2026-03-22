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
```

The canonical interface is mode-centered. Clipboard actions use `waytrim <mode> --clipboard`, not `waytrim clipboard <mode>`.

The CLI reads from stdin and writes cleaned text to stdout by default. In clipboard mode it reads the current clipboard text, repairs it through the same core logic, and writes the repaired text back.

## Modes

### `prose`
- primary mode
- repairs wrapped paragraphs and copy-induced spacing noise
- preserves visible structure such as bullets, headings, blockquotes, and indented sections

### `command`
- bounded secondary mode
- strips obvious prompts and repairs command presentation damage
- leaves mixed command/output snippets unchanged unless the command shape is obvious

### `auto`
- conservative convenience mode
- chooses command cleanup when the input is clearly command-like
- otherwise prefers prose repair or minimal prose-safe cleanup

## Preview

Use `--preview` to print a simple before/after diff-like view instead of cleaned text.

In clipboard mode, `--preview` is explicitly non-mutating. It shows what would change without writing back to the clipboard.

## Clipboard mode

Clipboard support is a manual adapter over the same repair core.

Planned behavior:
- `waytrim prose --clipboard` repairs current clipboard text and writes it back
- `waytrim prose --clipboard --print` prints repaired text to stdout and also writes it back
- `waytrim prose --clipboard --preview` previews changes without mutating the clipboard
- `clipboard unchanged` is a first-class success outcome when no effective change is needed
- `--clipboard --preview --print` is rejected as ambiguous

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
