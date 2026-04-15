# aig-core

The main crate for the `aig` CLI — an intent-based version control layer on top of git.

This crate is both a **library** (public modules re-exported from `lib.rs`) and a **binary** (`aig` CLI via `main.rs`).

## Modules

| Module | Purpose |
|--------|---------|
| `main.rs` | CLI entry point (clap). All 21 subcommands are defined and dispatched here. |
| `session.rs` | Start/end development sessions tied to intents |
| `checkpoint.rs` | Create checkpoints (git commit + aig metadata + provenance) |
| `intent.rs` | Intent CRUD — the "why" behind changes |
| `diff.rs` | Line-based and semantic diff orchestration |
| `db.rs` | SQLite schema and database connection |
| `storage.rs` | Content-addressable blob store (`.aig/objects/`) |
| `git_interop.rs` | Git operations via `git2` (repo access, commit creation) |
| `capture.rs` | Auto-capture AI conversations (Claude Code, generic JSONL) |
| `import.rs` | Import existing git history into aig + IPC client for LLM calls |
| `sync.rs` | Push/pull aig metadata via git notes |
| `watch.rs` | File-system watcher for auto-checkpointing |
| `repair.rs` | Re-attach orphaned metadata after rebase/cherry-pick |
| `bundle.rs` | Export/import `.aig` metadata as portable tar.gz bundles |

## Dependencies

- **aig-treesitter** — semantic diff engine (tree-sitter AST parsing for 11 languages)
- **git2** — git operations without shelling out
- **rusqlite** — SQLite for all structured metadata
- **clap** — CLI argument parsing
- **notify** — file-system watching

## How it relates to the TypeScript packages

The Rust binary is self-contained for most commands. Two features spawn Node.js child processes:

- **`aig import`** / **`aig why --explain`** — calls `@aig/llm` via NDJSON IPC for LLM-powered inference
- **`aig review --tui`** — launches `@aig/tui` (React/Ink interactive terminal UI)

Both are optional — the CLI works without Node.js, just without LLM and TUI features.
