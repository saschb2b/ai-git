## Technical Decision Document: aig Tech Stack

This document recommends a concrete technology stack for building aig, the intent-based version control system described in RESEARCH.md. The goal is to identify what will actually ship --- not the theoretically ideal architecture, but the one a small team can build, test, and iterate on within months rather than years.

---

### 1. Architecture Overview

aig runs as three cooperating layers:

- **Core engine (Rust)** --- a single compiled binary handling file watching, AST parsing, content-addressable storage, semantic diffing, git interop, and the CLI surface. This is the performance boundary. Everything that touches the filesystem, parses source code, or computes diffs lives here.
- **Orchestration layer (TypeScript/Node)** --- a higher-level process (or library consumed by the Rust CLI via subprocess/IPC) that handles LLM integration, conversation management, intent inference, and summary generation. TypeScript gives access to the best LLM client libraries and allows rapid iteration on the parts of the system where correctness matters more than microseconds.
- **UI layer (React/Ink for TUI; React for web)** --- a terminal UI for day-to-day use (`aig review`, `aig why`) and an optional web interface for team-level intent browsing and review workflows.

The Rust binary is the entry point for all user commands. When it needs LLM capabilities or conversation processing, it spawns or communicates with the TypeScript process over stdin/stdout using a line-delimited JSON protocol. This keeps the CLI fast for local operations (sub-100ms for `aig status`, `aig checkpoint`) while deferring to Node only when network calls or complex orchestration are required.

---

### 2. Core Engine: Rust

**Language: Rust.** The core engine must be fast, cross-platform, and produce a single binary with no runtime dependencies. Rust is the only practical choice that satisfies all three constraints. Go was considered but lacks the zero-cost abstractions needed for efficient AST manipulation and tree diffing. C/C++ was rejected because the development velocity cost is too high for a project that needs to iterate quickly on a novel data model.

**Key libraries:**

- **clap** for CLI argument parsing --- mature, well-documented, derives help text from struct definitions.
- **notify** (v6+) for cross-platform filesystem watching. This powers the continuous state stream: notify watches the working directory and feeds change events into the state capture pipeline. On Linux this uses inotify, on macOS FSEvents, on Windows ReadDirectoryChanges.
- **libgit2 via git2-rs** for git interop (reading/writing git objects, creating commits, managing refs). This avoids shelling out to `git` and gives programmatic access to the object database.
- **tokio** for async I/O --- needed for the file watcher event loop, IPC with the TypeScript layer, and future network operations (remote sync).
- **anyhow + thiserror** for error handling: anyhow at the application boundary, thiserror for library-internal error types.

Rust compiles to native binaries for Linux, macOS, and Windows from a single codebase. Cross-compilation is handled by `cross` or GitHub Actions matrix builds.

---

### 3. AST and Semantic Layer: Tree-sitter

**Library: tree-sitter** (via the `tree-sitter` Rust crate). Tree-sitter is the clear winner for multi-language AST parsing. It supports 100+ languages via community-maintained grammars, produces concrete syntax trees (CSTs) that preserve all source information (including whitespace and comments), parses incrementally (critical for the continuous state stream), and is fast enough to re-parse on every keystroke.

**Semantic diffing** is built on top of tree-sitter output. The approach:

1. Parse both versions of a file into tree-sitter CSTs.
2. Run a tree-diff algorithm (GumTree-style: top-down then bottom-up matching) to produce an edit script of AST-level operations: node insertions, deletions, moves, and updates.
3. Classify each operation semantically: "function added," "parameter type changed," "class renamed," etc. This classification layer is aig-specific and maps tree-sitter node types (which vary by grammar) to a normalized semantic vocabulary.

The tree-diff implementation will be custom Rust code. GumTree (Java) is a reference but is not directly usable. The algorithm is well-documented in academic literature and the Rust implementation benefits from cache-friendly memory layout and zero-copy tree traversal.

Tree-sitter grammars are loaded dynamically at runtime from `.so`/`.dylib`/`.dll` files, or compiled into the binary for the most common languages (TypeScript, Python, Rust, Go, Java, C/C++, Ruby, C#). Less common languages fall back to line-based diffing gracefully.

---

### 4. Storage Layer: SQLite + Content-Addressable Blobs

**Structured metadata: SQLite** via the `rusqlite` crate. SQLite is the right choice for storing intents, conversations, checkpoints, provenance metadata, trust scores, and the intent DAG. It is embedded (no server), battle-tested, supports complex queries, and handles concurrent reads well. The `.aig/` directory contains a single `aig.db` file alongside the content-addressable blob store.

Alternatives considered: **redb** (pure Rust embedded KV store) is appealing for simplicity but lacks the query flexibility needed for the intent graph and conversation search. **sled** is unmaintained. A full database (Postgres) is wrong for a local-first tool.

**Content-addressable blobs:** AST snapshots, serialized conversation data, and large binary artifacts are stored as content-addressed files in `.aig/objects/`, using the same SHA-256 hashing scheme as git. SQLite rows reference these blobs by hash. This keeps the database small (metadata only) while the blob store handles bulk data with deduplication.

**Schema design:** The SQLite schema models the intent DAG directly: an `intents` table with parent references, a `conversations` table with foreign keys to intents, a `checkpoints` table linking intents to git commit SHAs, and a `provenance` table tracking per-hunk authorship (human vs. AI, confidence scores, review status).

---

### 5. LLM Integration Layer: TypeScript/Node

**Language: TypeScript on Node.js.** The LLM integration layer handles: calling LLM APIs to generate summaries of changes, inferring intents from existing git history (`aig import`), powering `aig why` with natural-language explanations, and managing conversation storage.

TypeScript is chosen over Python for three reasons: (1) the Anthropic, OpenAI, and other LLM SDKs have first-class TypeScript support with strong typing; (2) TypeScript's async model maps naturally to streaming LLM responses; (3) sharing a language with the UI layer (React/Ink) reduces context switching and enables code reuse for conversation rendering.

The TypeScript layer runs as a long-lived subprocess managed by the Rust CLI. Communication uses newline-delimited JSON over stdin/stdout --- simple, debuggable, and avoids the complexity of gRPC or HTTP for local IPC. The Rust side defines the protocol; the TypeScript side implements handlers.

**LLM provider abstraction:** A thin adapter interface wraps provider-specific SDKs (Anthropic, OpenAI, local models via Ollama). The system defaults to Claude for summary generation and intent inference but is not locked to any provider. API keys are stored in the user's OS keychain via `keytar` or equivalent, never in the repository.

---

### 6. Git Interop: libgit2

**Library: git2-rs** (Rust bindings for libgit2). Every aig repository is simultaneously a valid git repository. `aig checkpoint` creates a git commit under the hood. `aig push` delegates to git's transport layer. The `.aig/` directory sits alongside `.git/` and stores the supplementary metadata (intent graph, conversations, AST snapshots).

For operations that libgit2 does not support well (interactive rebase, complex merge strategies), aig shells out to the `git` CLI as a fallback. This is a pragmatic concession: libgit2 covers 90% of needs, and the remaining 10% is not worth reimplementing.

**Import from existing git repos:** `aig import` walks the git log, groups commits by heuristics (time proximity, file overlap, branch structure), and uses the LLM layer to infer intents and generate retroactive summaries. The result is a populated `.aig/` directory that enriches the existing history.

---

### 7. User Interfaces

**CLI:** Built directly into the Rust binary using clap. Fast, scriptable, no runtime dependencies. Covers all core operations: `aig session`, `aig checkpoint`, `aig status`, `aig why`, `aig review`, `aig merge`, `aig history`.

**TUI (terminal UI):** Built with **React/Ink** (TypeScript). Ink renders React components to the terminal, enabling a rich interactive review experience (`aig review --tui`) with keyboard navigation, expandable semantic diff trees, and inline conversation display. Ink is chosen over Rust TUI libraries (ratatui) because the UI is not performance-critical and React's component model is far more productive for building complex interactive layouts.

**Web UI (future):** A React web app served locally by the TypeScript layer. This provides team-oriented features: intent graph visualization (using D3 or a similar library), cross-repository search, and collaborative review workflows. This is a phase-2 deliverable; the CLI and TUI ship first.

---

### 8. Serialization and Wire Format

**AST snapshots and metadata: MessagePack.** AST snapshots (serialized tree-sitter CSTs) and checkpoint metadata are serialized with MessagePack via the `rmp-serde` crate in Rust and `@msgpack/msgpack` in TypeScript. MessagePack is chosen over Protocol Buffers because it is schema-less (the AST structure varies by language and evolves rapidly) and over JSON because it is 30-50% smaller and faster to parse.

**IPC between Rust and TypeScript: newline-delimited JSON (NDJSON).** For the local IPC channel, human-readability during development outweighs the performance benefit of a binary format. The volume is low (LLM requests/responses), so serialization cost is negligible.

**Git notes for portable metadata:** When pushing to remotes, aig stores a subset of its metadata as git notes attached to commits. This allows aig-aware clients to read intent and conversation data from any git remote without requiring a separate sync mechanism for the `.aig/` directory.

---

### 9. Build and Development Tooling

**Monorepo structure** using a single git repository with the following layout:

```
/crates/aig-core      # Rust: CLI, storage, AST, git interop
/crates/aig-treesitter # Rust: tree-sitter integration and semantic diff
/packages/aig-llm     # TypeScript: LLM integration, conversation management
/packages/aig-tui     # TypeScript: Ink-based terminal UI
/packages/aig-web     # TypeScript: React web UI (phase 2)
```

**Rust tooling:** Cargo workspaces for the Rust crates. `cargo-nextest` for fast parallel test execution. `cargo-release` for versioning. Clippy and rustfmt enforced in CI.

**TypeScript tooling:** pnpm workspaces for the TypeScript packages. `tsup` for bundling. Vitest for testing. ESLint and Prettier enforced in CI.

**Cross-language build:** A top-level `Makefile` (or `just` justfile) orchestrates both Cargo and pnpm builds. CI runs on GitHub Actions with a matrix of OS targets (Linux, macOS, Windows). Release binaries bundle the TypeScript layer as a compiled snapshot (using `pkg` or `bun compile`) embedded alongside the Rust binary, so the end user installs a single artifact.

**Testing strategy:** Unit tests for the tree-diff algorithm and storage layer (Rust). Integration tests that exercise the full CLI against sample repositories. Snapshot tests for semantic diff output. End-to-end tests using recorded LLM responses (no live API calls in CI).

---

### 10. Risks and Alternatives Considered

**Risk: Tree-sitter grammar coverage.** Some languages have incomplete or buggy tree-sitter grammars. Mitigation: fall back to line-based diffing for unsupported languages. The system degrades gracefully rather than failing.

**Risk: Rust-TypeScript IPC complexity.** Maintaining a subprocess protocol adds operational complexity. Alternative considered: writing the LLM layer in Rust using `reqwest` for HTTP calls. Rejected because LLM SDK ecosystems (streaming, function calling, structured output parsing) are significantly more mature in TypeScript, and the integration layer changes faster than the core engine.

**Risk: SQLite write concurrency.** SQLite's single-writer model could bottleneck the continuous state stream. Mitigation: use WAL mode (write-ahead logging), batch writes, and keep the hot path (file watching) writing to an append-only log that is periodically flushed to SQLite.

**Risk: AST diff performance on large files.** GumTree-style tree diffing is O(n^2) in the worst case. Mitigation: set a node-count threshold (e.g., 50,000 nodes) above which the system falls back to line-based diffing, and optimize the common case with top-down greedy matching before running the full algorithm.

**Alternative considered: Python for orchestration.** Python has strong LLM library support but weaker typing, slower startup, and a messier dependency story for end-user distribution. TypeScript is easier to bundle into a self-contained artifact and shares the React ecosystem with the UI layer.

**Alternative considered: Tauri for the desktop UI.** Tauri would provide a native desktop wrapper around the web UI. Deferred to phase 3; the TUI and web UI cover the initial use cases without the complexity of a desktop app distribution pipeline.

**Alternative considered: CRDT-based storage for real-time collaboration.** CRDTs (e.g., Automerge, Yjs) would enable Google-Docs-style concurrent editing of the intent graph. Deferred: the initial version targets single-developer and async-team workflows. CRDT support can be layered in later without changing the core data model.
