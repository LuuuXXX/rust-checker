# rust-checker

A unified Rust project quality checker CLI tool that runs multiple code analysis tools and generates consolidated reports.

## Features

- **Multiple presets**: `minimal`, `standard`, and `full` tool configurations
- **Parallel-friendly tool runner** with topological dependency ordering
- **Multiple report formats**: Markdown, HTML, and JSON
- **Run history** with diff support between runs
- **File watcher** for continuous checking during development
- **Plugin system** for extensible tool configurations
- **CI mode** with JSON output for machine-readable results

## Installation

```bash
cargo install --path .
```

## Usage

### Initialize a project

```bash
# Standard preset (build, test, clippy, fmt, doc, audit)
rust-checker init

# Minimal preset (build, test, clippy, fmt)
rust-checker init --preset minimal

# Full preset (all tools)
rust-checker init --preset full
```

### Run checks

```bash
# Run all configured tools
rust-checker run

# Output as HTML
rust-checker run --format html

# CI mode (JSON output, non-zero exit on failure)
rust-checker run --ci
```

### Diff history

```bash
# Diff last two runs
rust-checker diff

# Diff last N runs
rust-checker diff --last 3
```

### Upgrade config schema

```bash
rust-checker upgrade
```

### Manage plugins

```bash
rust-checker plugin list
rust-checker plugin add <name>
rust-checker plugin remove <name>
rust-checker plugin update
```

### Watch mode

```bash
# Watch src/ for changes and re-run checks
rust-checker watch

# Watch with specific tools only
rust-checker watch --tools build,test,clippy
```

## Configuration

Config is stored at `.localcheck/config.toml`. Example:

```toml
schema_version = 1

[rust]
version = "stable"

[tools.build]
desc = "Build the project"
active = "true"
input_command = "cargo build"
output_path = ".localcheck/reports/build"

[tools.test]
desc = "Run tests"
active = "true"
input_command = "cargo test"
output_path = ".localcheck/reports/test"
depends_on = ["build"]

[tools.clippy]
desc = "Run clippy linter"
active = "true"
input_command = "cargo clippy -- -D warnings"
output_path = ".localcheck/reports/clippy"
depends_on = ["build"]

[history]
max_entries = 50

[watch]
debounce_ms = 500
```

## Supported Tools

| Tool | Description | Preset |
|------|-------------|--------|
| `build` | `cargo build` | minimal+ |
| `test` | `cargo test` | minimal+ |
| `clippy` | `cargo clippy` | minimal+ |
| `fmt` | `cargo fmt --check` | minimal+ |
| `doc` | `cargo doc` | standard+ |
| `audit` | `cargo audit` | standard+ |
| `deny` | `cargo deny check` | full |
| `geiger` | `cargo geiger` | full |
| `deps` | `cargo tree` | full |
| `udeps` | `cargo +nightly udeps` | full |
| `bloat` | `cargo bloat` | full |
| `msrv` | `cargo msrv` | full |
| `semver-checks` | `cargo semver-checks` | full |

## Reports

Reports are saved to `.localcheck/reports/`. Each tool generates an individual report, plus a summary:

- `.localcheck/reports/summary.md` — Markdown summary table
- `.localcheck/reports/summary.json` — JSON summary (with `--format json`)
- `.localcheck/reports/<tool>.md` — Per-tool report

## Development

```bash
cargo build
cargo test
```
