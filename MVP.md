# aig — Minimum Viable Product

## MVP: Proving the Thesis

### What the MVP Proves

The MVP demonstrates that a version control layer can capture intent, conversations, and semantic changes on top of git, making AI-assisted development transparent and reviewable. By layering structured metadata — intents, sessions, conversation notes, and AST-level diffs — onto standard git commits, aig proves that developers and teams can understand not just *what* changed but *why* it changed, who (or what) requested it, and how the change fits into a larger goal. The MVP is end-to-end demoable: initialize, track a session with intent, checkpoint changes, and later trace any line back to its originating intent.

### MVP Scope — What's In

1. **`aig init`** — Initialize a `.aig/` directory alongside an existing `.git/` repo. Creates the SQLite database (`aig.db`), blob store (`objects/`), and default config. Fails with a clear message if no `.git/` is present.

2. **`aig session start "<intent>"`** — Declare an intent and begin a tracked session. Records the intent description in the database, starts the session clock, and sets the session as active. Only one session can be active at a time.

3. **`aig session end`** — End the current session. Seals the intent with a summary (auto-generated from checkpoint messages within the session). Marks the session as closed and records the end timestamp.

4. **`aig checkpoint "<message>"`** — Crystallize current state. Creates a git commit under the hood and stores aig metadata: links the checkpoint to the active session and intent, records file-level changes, and captures semantic change data (functions added/removed/modified) for supported languages.

5. **`aig status`** — Show current session info, active intent, and changes since the last checkpoint. Richer than `git status`: displays the intent description, session duration, number of checkpoints so far, and a categorized list of modified files.

6. **`aig log`** — Show intent-level history. Each entry displays the intent description, number of checkpoints within that intent, files touched, and timestamp range. This is not a raw git log — it groups commits by intent and shows the higher-level narrative.

7. **`aig diff [--semantic]`** — Show changes since the last checkpoint. Default mode: an enhanced line diff with file-level context. With `--semantic`: an AST-level semantic diff for supported languages, reporting structural changes (e.g., "function `handleAuth` added", "parameter `timeout` added to `connect`", "class `UserService` removed").

8. **`aig why <file>:<line>`** — Trace a line back to its intent and checkpoint. Outputs: the intent that introduced or last modified the line, the checkpoint message, the session context (including any attached conversation notes), and the timestamp.

9. **`aig import`** — Migrate an existing git repository into aig. Walks the git log, groups commits into intent clusters using heuristics, calls an LLM to infer intents and generate summaries, and populates `.aig/` with a retroactive intent graph. This is the git-to-aig migration path. See dedicated subsection below.

10. **`aig conversation add "<message>"`** — Attach a conversation note to the current session. For the MVP, this is manual — the user types context such as "discussed approach with Claude, decided to split the handler into two functions." Future versions will auto-capture from AI tools.

### MVP Scope — What's Out (Deferred)

- Continuous file watching / state streaming (phase 2)
- Semantic merge engine (phase 2)
- Trust scoring / provenance tracking (phase 2)
- TUI review interface (phase 2)
- Remote sync of `.aig/` metadata (phase 2)
- Web UI (phase 3)
- Multi-agent coordination (phase 3)
- Ownership maps (phase 3)
- Drift detection (phase 3)
- CRDT-based collaboration (phase 3+)

### Supported Languages for Semantic Diff (MVP)

MVP ships with tree-sitter grammars compiled in for: **TypeScript/JavaScript**, **Python**, **Rust**, and **Go**. These cover the most common languages in AI-assisted development workflows. All other languages fall back to line-based diff gracefully — no errors, no degraded UX, just less structural detail.

### The Migration Story: `aig import`

Migration is the critical onboarding path. Most teams already have git repositories with months or years of history. `aig import` makes that history useful by retrofitting intent metadata onto it. The user runs `aig import` in any existing git repo (after `aig init`), and the system does the rest.

The import process walks the git log from oldest to newest, reading every commit. It groups commits into "intent clusters" using heuristics: commits within a short time window (default: 2 hours) by the same author touching overlapping files are likely part of a single intent. Merge commits, branch points, and large time gaps act as natural cluster boundaries. The heuristics are tunable but ship with sensible defaults.

For each cluster, the system sends the commit messages and diff stats to an LLM, which infers three things: (a) a high-level intent description (e.g., "Add user authentication with OAuth2"), (b) a summary of what was accomplished, and (c) suggested grouping adjustments (the LLM may recommend splitting or merging clusters based on semantic coherence). The system applies these adjustments and writes the final intent graph to the `.aig/` database.

The result: an existing git repo now has aig metadata. `aig log` shows intent-level history instead of a flat commit list. `aig why` works, tracing lines back to inferred intents. The team can start using aig workflows — sessions, checkpoints, conversation notes — going forward, with the full history already indexed. The import is non-destructive. It only creates `.aig/` — the git history is completely untouched, and standard git tools continue to work normally.

### Success Criteria

- A user can clone any git repo, run `aig init` and `aig import`, and browse intent-level history with `aig log`
- A user can start a session with a declared intent, make changes with an AI assistant, add conversation notes, checkpoint progress, and later run `aig why` to understand the reasoning behind any line
- `aig diff --semantic` shows meaningful structural changes (e.g., "function added", "parameter changed", "import removed") for TypeScript/JavaScript, Python, Rust, and Go
- The system degrades gracefully for unsupported languages — line-based diff, no semantic analysis, no errors
- All aig metadata lives in `.aig/` — the underlying git repo is fully functional with standard git tools, and `.aig/` can be deleted without any side effects on the git history
- `aig import` can process a repo with 1000+ commits in under 5 minutes

### Architecture for MVP

- **Rust binary** (clap CLI) handles: `init`, `session`, `checkpoint`, `status`, `log`, `diff`, `why`. This is the primary user-facing binary.
- **TypeScript/Node subprocess** handles: LLM calls for `aig import` (intent inference, summary generation) and session-end summary generation. Invoked by the Rust binary as a child process.
- **SQLite** in `.aig/aig.db` for all structured metadata — intents, sessions, checkpoints, conversation notes, file-change records, line-to-intent mappings.
- **Content-addressable blobs** in `.aig/objects/` for AST snapshots and semantic diff artifacts.
- **Git interop** via `git2-rs` — checkpoints create real git commits, import reads git history, all without shelling out to the git CLI.
- **pnpm workspaces** for TypeScript packages, **Cargo workspaces** for Rust crates.
- **Monorepo managed with pnpm** at the root, with the Rust build integrated into the workspace scripts.
