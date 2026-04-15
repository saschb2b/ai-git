---
outline: deep
description: "aig tech stack: Rust core engine, tree-sitter semantic diff, SQLite storage, TypeScript LLM integration. Architecture, project structure, and build instructions."
---

# Tech Stack

aig is built as a three-layer architecture: a Rust core engine for performance, a TypeScript orchestration layer for LLM integration, and React-based interfaces.

::: info
Full technical decision document: [TECH_STACK.md on GitHub](https://github.com/saschb2b/ai-git/blob/main/TECH_STACK.md)
:::

## Architecture

```
┌─────────────────────────────────────────┐
│              User (CLI/TUI)             │
├─────────────────────────────────────────┤
│         Rust Core Engine (aig)          │
│  ┌──────────┐ ┌──────────┐ ┌────────┐  │
│  │  CLI     │ │ Storage  │ │  Git   │  │
│  │ (clap)   │ │ (SQLite) │ │(git2)  │  │
│  └──────────┘ └──────────┘ └────────┘  │
│  ┌──────────────────────────────────┐   │
│  │  Tree-sitter (semantic diff)    │   │
│  └──────────────────────────────────┘   │
├───────────── NDJSON IPC ────────────────┤
│      TypeScript Orchestration           │
│  ┌──────────┐ ┌──────────────────────┐  │
│  │ LLM SDK  │ │ Conversation Mgmt   │  │
│  │(Anthropic)│ │ (intent inference)  │  │
│  └──────────┘ └──────────────────────┘  │
└─────────────────────────────────────────┘
```

## Core Technologies

| Component | Technology | Why |
|---|---|---|
| **CLI & core engine** | Rust | Fast, single binary, no runtime deps, cross-platform |
| **CLI parsing** | clap (derive) | Mature, self-documenting |
| **AST parsing** | tree-sitter | 100+ languages, incremental parsing, battle-tested |
| **Semantic diff** | Custom (GumTree-style) | AST-level change detection across language grammars |
| **Storage** | SQLite (rusqlite) | Embedded, zero-config, complex queries for intent graph |
| **Blob store** | Content-addressable (SHA-256) | Deduplication, git-compatible approach |
| **Git interop** | libgit2 (git2-rs) | Programmatic git access, no shell-out |
| **LLM integration** | TypeScript + Anthropic SDK | Best SDK ecosystem, streaming support |
| **IPC** | NDJSON over stdin/stdout | Simple, debuggable, low overhead |
| **Serialization** | MessagePack | Compact, schema-less, fast |
| **File watching** | notify (v8) | Cross-platform fs events (inotify/FSEvents/ReadDirectoryChanges) |
| **Async runtime** | Tokio | File watching, IPC, future network ops |

## Project Structure

```
ai-git/
├── crates/
│   ├── aig-core/           # Rust: CLI binary, storage, git interop
│   │   └── src/
│   │       ├── main.rs      # CLI entry point (clap) — 12 commands
│   │       ├── db.rs        # SQLite database layer
│   │       ├── storage.rs   # Content-addressable blob store
│   │       ├── session.rs   # Session management
│   │       ├── checkpoint.rs# Checkpoint creation + semantic change recording
│   │       ├── intent.rs    # Intent CRUD
│   │       ├── git_interop.rs # git2 integration
│   │       ├── diff.rs      # Line-based diff + git working tree
│   │       ├── import.rs    # Git history import + clustering + IPC
│   │       ├── capture.rs   # Claude Code conversation capture
│   │       └── watch.rs     # File system watching + auto-checkpoint
│   └── aig-treesitter/      # Rust: multi-language semantic diff
│       └── src/lib.rs       # Tree-sitter parsing + AST comparison
├── packages/
│   ├── aig-llm/             # TypeScript: LLM integration
│   │   └── src/
│   │       ├── providers/   # Anthropic SDK, provider abstraction
│   │       ├── ipc.ts       # NDJSON IPC server
│   │       └── import.ts    # Commit clustering + LLM inference
│   └── aig-tui/             # TypeScript: Terminal UI (phase 2)
├── docs/                    # VitePress documentation site
├── Cargo.toml               # Rust workspace
├── package.json             # pnpm root
└── pnpm-workspace.yaml      # Workspace config
```

## Build & Development

```bash
# Rust
cargo build              # Build the aig binary
cargo test               # Run all tests
cargo run -- --help      # Run the CLI

# TypeScript
pnpm install             # Install dependencies
pnpm build               # Build all packages

# Docs
cd docs && pnpm dev      # Local dev server
```

## Supported Platforms

Built and tested on Linux, macOS, and Windows. Cross-compilation via Cargo + GitHub Actions matrix builds.

## Semantic Diff: Supported Languages

| Language | Tree-sitter Grammar | Definition Types Tracked |
|---|---|---|
| TypeScript/JS | `tree-sitter-typescript` | functions, classes, interfaces, type aliases, methods |
| Python | `tree-sitter-python` | functions, classes |
| Rust | `tree-sitter-rust` | functions, structs, enums, impls, traits, types |
| Go | `tree-sitter-go` | functions, methods, types |
| Java | `tree-sitter-java` | classes, interfaces, methods, constructors, enums |
| C# | `tree-sitter-c-sharp` | classes, interfaces, methods, structs, enums, constructors |
| C++ | `tree-sitter-cpp` | functions, classes, structs, enums, templates |
| Ruby | `tree-sitter-ruby` | methods, classes, modules |
| PHP | `tree-sitter-php` | functions, methods, classes, interfaces, traits, enums |
| Kotlin | `tree-sitter-kotlin-ng` | functions, classes, objects |
| Swift | `tree-sitter-swift` | functions, classes, protocols, type aliases |

Other languages fall back gracefully to line-based diffing.
