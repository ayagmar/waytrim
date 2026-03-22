# waytrim

waytrim is a Linux-native text repair tool for copy/display damage from terminal, TUI, AI-console, and similar plaintext-oriented interfaces.

It is a repair tool, not a rewrite tool.

## Current CLI

```bash
waytrim prose
waytrim command
waytrim auto
waytrim prose --preview
```

The CLI reads from stdin and writes cleaned text to stdout.

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
