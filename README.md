# rs-toolkit

A Rust workspace for small, configurable file-processing tools. Shared configuration and logging helpers live in ``libs/common`; the workspace also includes a derive macro for parsing enum values from configuration.

## Tools

### ``excel-handler`

Reads one `.xlsx` workbook or a directory of workbooks, optionally skips selected sheets, cleans cell contents, and exports eligible sheets as CSV files. Cleaning can be configured with presets, literal strings, and regular expressions.

### ``sql-file-renamer`

Selects files in a directory using include/exclude patterns, sorts them by name or file timestamps, then prefixes sequential numbers with configurable spacing and padding. By default, it copies originals to a timestamped directory before renaming; this can be changed in its settings.

## Build and run

Requires a stable Rust toolchain with Cargo. Install [just](https://github.com/casey/just) to use the task shortcuts below.

```sh
# Build the workspace
cargo build --workspace

# Run one tool
cargo run -p excel-handler
cargo run -p sql-file-renamer

# Run formatting, lint, tests, and project checks (requires just)
just check
```

Each tool loads its settings from the shared TOML configuration system; its generated configuration includes the available options and defaults. Review input paths and file-operation settings before running a tool on important data.

## Add a tool

A starter crate can be generated with `just new-tool <name>`. See the workspace `justfile` and `templates/tool` for the shared build and configuration conventions.

## License

MIT. See [LICENSE](LICENSE).
