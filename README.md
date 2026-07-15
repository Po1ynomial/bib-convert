# bib-convert

`bib-convert` converts BibTeX/BibLaTeX `.bib` files into readable YAML, TOML, or JSON using Rust and the [`biblatex`](https://crates.io/crates/biblatex) crate.

The project currently prioritizes practical, inspectable exports over a strict bibliography schema. Each bibliography entry is emitted as a record with:

- `type` — the BibTeX/BibLaTeX entry type, such as `article` or `book`
- `key` — the citation key
- `fields` — the fields present on the entry after projection/normalization

By default, common person-list fields such as `author`, `editor`, `translator`, and `bookauthor` are exported as arrays of readable names. Use `--raw-fields` if you want the older raw string-style field output instead.

## Quick start

Build the project:

```bash
cargo build
```

Convert a bibliography file to YAML:

```bash
cargo run -- path/to/refs.bib
```

Choose a different output format:

```bash
cargo run -- path/to/refs.bib -f json
cargo run -- path/to/refs.bib -f toml
```

By default, the output path is derived from the input path:

- `refs.bib` → `refs.yaml`
- `refs.bib -f json` → `refs.json`

To choose the output path explicitly:

```bash
cargo run -- path/to/refs.bib -f json -o out/records.json
```

## CLI

Show the full help text:

```bash
cargo run -- --help
```

Current CLI:

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

## Output model

Each exported record has this general shape:

```yaml
- type: <entry type>
  key: <citation key>
  fields:
    <field-name>: <string value | list of strings>
```

The default projection is intentionally permissive:

- unknown or uncommon BibTeX/BibLaTeX fields are passed through
- well-known person-list fields may be exported as lists of readable names
- date-like fields may be normalized from explicit `date` fields or synthesized from `year` / `month` / `day`
- when a smart `date` field is present, the component date fields used to produce it are removed for consistency
- math delimiters such as `$...$` are preserved in projected values
- some values are normalized by `biblatex`; for example:
  - month abbreviations may become full names
  - TeX accents may become Unicode
  - escaped `\&` may become `&`

Use `--raw-fields` when you want every field rendered as a raw normalized string and do not want synthesized or normalized fields added.

## Example

Given this BibTeX entry:

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
    date: 2024-08
    note: A & B
    title: State sum invariants of $3$-manifolds and quantum
      $6j$-symbols
```

## Raw field mode

Run with `--raw-fields` to disable special handling for well-known fields:

```bash
cargo run -- path/to/refs.bib --raw-fields
```

In raw field mode:

- person-list fields such as `author` are not projected into readable-name arrays
- synthesized or normalized fields such as smart `date` values are not added
- fields are rendered as raw normalized strings

## Debug outputs

You can write intermediate debug artifacts while converting:

```bash
cargo run -- path/to/refs.bib \
  --debug-biblatex debug/biblatex.yaml \
  --debug-entries debug/entries.yaml
```

Notes:

- the main converted output uses the selected output format
- debug outputs are currently written as YAML

## Project status

`bib-convert` currently favors a loose, practical output shape:

- no serious record-level validation yet
- no opinionated per-entry schema enforcement
- normalized field values come from `biblatex`
- some well-known fields, especially person-list fields, may become structured values
- date-like fields may be normalized or synthesized in smart mode
- output is intended to be easy to inspect and reuse downstream

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

Run the test suite:

```bash
cargo test
```

Current coverage includes:

- normalization behavior on representative entries
- YAML/TOML/JSON serialization
- CLI output path behavior
- debug artifact generation
