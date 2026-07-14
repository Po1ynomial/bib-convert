# bib-convert

Convert a `.bib` file into YAML, TOML, or JSON using Rust and `biblatex`.

The current goal is simple, readable export rather than strict schema modeling. Each record is emitted as:

- `type`
- `key`
- `fields`

where `fields` contains whatever fields were present in the bibliography entry.

By default, well-known person-list fields such as `author`, `editor`,
`translator`, and `bookauthor` are projected into arrays of readable names.
Use `--raw-fields` to keep the older raw string-style field output.

## Status

This project currently favors a loose, practical output shape:

- no serious record-level validation yet
- no opinionated per-entry schema enforcement
- normalized field values come from `biblatex`
- some well-known fields, especially person-list fields, may become
  structured values
- output is intended to be easy to inspect and reuse downstream

## Build

```bash
cargo build
```

## Run

Show CLI help:

```bash
cargo run -- --help
```

Current CLI:

<!-- markdownlint-disable MD013 -->
```text
Convert .bib files into structured YAML, TOML, or JSON

Usage: bib-convert [OPTIONS] <INPUT>

Arguments:
  <INPUT>

Options:
  -f, --format <FORMAT>                  [default: yaml] [possible values: yaml, toml, json]
  -o <OUTPUT>
      --debug-biblatex <DEBUG_BIBLATEX>
      --debug-entries <DEBUG_ENTRIES>
      --raw-fields
  -h, --help                             Print help
```
<!-- markdownlint-enable MD013 -->

### Basic conversion

```bash
cargo run -- path/to/input.bib
```

By default, the output path replaces the input extension:

- `refs.bib` → `refs.yaml`
- `refs.bib` with `-f json` → `refs.json`

### Choose a format

```bash
cargo run -- path/to/input.bib -f yaml
cargo run -- path/to/input.bib -f toml
cargo run -- path/to/input.bib -f json
```

### Choose an explicit output file

```bash
cargo run -- path/to/input.bib -f json -o out/records.json
```

### Keep raw field strings

```bash
cargo run -- path/to/input.bib --raw-fields
```

This disables special handling for well-known fields like `author` and
keeps the raw normalized string representation for every field.

### Write debug artifacts

```bash
cargo run -- path/to/input.bib \
  --debug-biblatex debug/biblatex.yaml \
  --debug-entries debug/entries.yaml
```

Notes:

- main converted output uses the selected format
- debug outputs are currently written as YAML

## Example output

Given a BibTeX entry like:

```bibtex
@article{edge1,
  author = {Gompf, Robert E. and Stipsicz, Andr\'as I.},
  title = {State sum invariants of $3$-manifolds and quantum $6j$-symbols},
  month = aug,
  note = {A \& B},
  year = {2024},
}
```

The default YAML output looks like:

```yaml
- type: article
  key: edge1
  fields:
    author:
      - Robert E. Gompf
      - András I. Stipsicz
    month: August
    note: A & B
    title: State sum invariants of $3$-manifolds and quantum
      $6j$-symbols
    year: '2024'
```

## Output shape

Each exported record has this general shape:

```yaml
- type: <entry type>
  key: <citation key>
  fields:
    <field-name>: <string value | list of strings>
```

Properties of the current projection:

- unknown or uncommon BibTeX/BibLaTeX fields are allowed through unchanged
- well-known person-list fields like `author` may be exported as lists
  of readable names
- use `--raw-fields` if you want every field rendered as a raw normalized string
- math delimiters like `$...$` are preserved in projected values
- some values are normalized by `biblatex`, for example:
  - month abbreviations may become full names
  - TeX accents may become Unicode
  - escaped `\&` may become `&`

This is intentionally permissive for now.

## Project structure

```text
.
├── Cargo.toml
├── src/
│   ├── lib.rs      # conversion and serialization logic
│   └── main.rs     # CLI wrapper
└── tests/
    └── integration.rs
```

## Development

Run tests:

```bash
cargo test
```

Current coverage includes:

- normalization behavior on representative entries
- YAML/TOML/JSON serialization
- CLI output path behavior
- debug artifact generation
