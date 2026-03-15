# pr-tracker

A terminal-first PR dashboard for quickly viewing active pull requests across repositories, powered by GitHub CLI with GitHub API fallback.

## Overview

`pr-tracker` helps you monitor active PRs without leaving the terminal. It surfaces high-signal information — what is open, which repo it belongs to, who owns it, and what changed most recently — in a keyboard-driven TUI.

## Tech Stack

| Crate | Role |
|---|---|
| `tokio` | Async runtime for concurrent fetch and event handling |
| `ratatui` | Structured TUI widgets, layout, and rendering |
| `crossterm` | Terminal backend — raw mode, keyboard events, screen control |
| `clap` | CLI argument parsing (`--config`, `--repos`, `--no-fallback`) |
| `toml` + `serde` | Config file parsing and data (de)serialization |
| `reqwest` | HTTP client for GitHub API fallback (blocking, rustls) |
| `anyhow` / `thiserror` | Ergonomic error propagation and typed domain errors |
| `chrono` | PR timestamp parsing and display formatting |
| `open` | Cross-platform browser launch |
| `tracing` | Structured diagnostics (controlled via `RUST_LOG`) |

## Technical Decisions

### `Rust + tokio`

Rust was chosen for long-term reliability in a CLI tool: compile-time safety, predictable performance, and easy single-binary distribution. `tokio` keeps the event loop responsive and makes concurrent per-repo fetching and future background polling straightforward to add without redesigning the core architecture.

### `ratatui + crossterm`

`ratatui` provides structured widgets and layout that stay maintainable as the feature set grows. `crossterm` handles cross-terminal raw mode, keyboard events, and screen control and integrates directly with `ratatui`'s rendering model. Together they are the de facto standard for modern Rust TUI applications.

### `clap + toml + serde`

`clap` ensures well-documented CLI flags with validation and generated `--help` output. `toml` is human-readable and idiomatic in the Rust ecosystem. `serde` unifies all serialization — config files, `gh` JSON output, and API payloads — through a single derive-based interface, eliminating custom parsing code.

## Features

### v1 (Implemented)

- List active open PRs across multiple configured repositories
- Sort by most recently updated (descending)
- Keyboard row navigation
- Manual refresh
- Open selected PR in browser (`o`)
- `gh` CLI as primary fetch source
- GitHub REST API fallback per-repo when `gh` fails or is unavailable
- Partial-result behavior: show fetched repos and report per-source warnings

### Future (Out of v1 scope)

- Interactive filtering and search
- PR detail pane (files changed, review state, checks)
- Background auto-refresh with notification
- Linux and Windows packaging
- Optional secure token storage (macOS Keychain)

## Architecture

```
CLI args (clap)
    └── Config load (toml / serde)
            └── fetch::fetch_all()
                    ├── gh::fetch()   ← gh pr list --json ...
                    │       ↓ fails?
                    └── api::fetch()  ← GitHub REST API (reqwest)
                            ↓
                    sorted_prs()      ← flatten + sort by updated desc
                            ↓
                    ui::app::run()    ← ratatui table + keyboard loop
                            ↓
                    open::that(url)   ← system browser
```

## Project Layout

```
src/
├── main.rs          # Entry point, CLI args, refresh loop
├── lib.rs           # Public re-exports (for integration tests)
├── config.rs        # Config model, TOML loading, validation
├── model.rs         # PullRequest, FetchResult, FetchSource
├── fetch/
│   ├── mod.rs       # Orchestration: gh-first, API fallback, partial results
│   ├── gh.rs        # gh CLI execution and JSON normalisation
│   └── api.rs       # GitHub REST API fallback client
└── ui/
    ├── mod.rs
    └── app.rs       # ratatui rendering, table state, keyboard actions
tests/
├── gh_parser.rs     # gh JSON fixture tests
└── fallback.rs      # Fetch orchestration and sort order tests
```

## Roadmap

| Milestone | Status |
|---|---|
| M1: Bootstrap — Cargo project, dep setup, core models | Done |
| M2: Data layer — `gh` fetch, API fallback, orchestration | Done |
| M3: TUI MVP — table view, keybindings, open in browser | Done |
| M4: Quality — tests, CI, installable package | In progress |
| M5: Filters — interactive filter/search UI | Planned |
| M6: Detail pane — PR files, checks, review state | Planned |
| M7: Cross-platform packaging | Planned |

## Installation

### Prerequisites

- Rust toolchain (`rustup` stable)
- [GitHub CLI](https://cli.github.com/) (`gh`)
- Authenticated GitHub session: `gh auth login`

### Build and install locally

```bash
git clone https://github.com/yangj8/pr-tracker
cd pr-tracker
cargo install --path .
```

### Run from source

```bash
cargo run -- --config ~/.config/pr-tracker/config.toml
```

## Configuration

Default config path: `~/.config/pr-tracker/config.toml`

```toml
[github]
# Repositories to track in "owner/repo" format (required)
repos = ["owner/repo-a", "owner/repo-b"]

# Fall back to GitHub REST API when gh CLI fails (default: true)
use_api_fallback = true

# Environment variable name containing a GitHub token for API fallback
token_env = "GITHUB_TOKEN"

[ui]
# Auto-refresh interval in seconds; 0 = manual only (default: 60)
refresh_interval_seconds = 60
```

## Usage

```bash
# Use default config path
pr-tracker

# Specify config explicitly
pr-tracker --config path/to/config.toml

# Override repos from CLI
pr-tracker --repos owner/repo-a,owner/repo-b

# Disable API fallback
pr-tracker --no-fallback
```

### Keybindings

| Key | Action |
|---|---|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `o` / `Enter` | Open selected PR in browser |
| `r` | Refresh |
| `q` / `Esc` | Quit |

## Development

```bash
# Type-check
cargo check

# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Style check
cargo fmt --check

# Lint (treat warnings as errors)
cargo clippy -- -D warnings
```

Set `RUST_LOG=debug` to see fetch source decisions and browser-open events in stderr.

## Limitations (v1)

- No interactive filter UI (manual `--repos` override only)
- No PR detail pane
- macOS-first; Linux should work but is untested
- `r` refresh exits and re-fetches but does not yet loop in-process

## License

TBD
