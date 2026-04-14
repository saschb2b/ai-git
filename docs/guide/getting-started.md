# Getting Started

This guide walks you through installing **aig**, initializing it in a project, and running through a complete workflow so you can see how intent-aware version control works in practice.

## Installation

### From source (recommended)

```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
```

### Build locally

```bash
git clone https://github.com/saschb2b/ai-git.git
cd ai-git
cargo build --release
# Binary at target/release/aig (.exe on Windows)
```

To make the `aig` command available everywhere, add the binary to your `PATH`:

```bash
# Linux / macOS
export PATH="$PWD/target/release:$PATH"

# Windows (PowerShell)
$env:PATH = "$PWD\target\release;$env:PATH"
```

### Prerequisites

- **Rust toolchain (1.75+)** --- install from [rustup.rs](https://rustup.rs)
- **An existing git repository** --- aig layers on top of git; it does not replace it
- **Node.js 20+** (optional) --- needed for LLM-powered import

## Quick Start

### Try it on your existing repo

If you already have a git repo and want to see aig in action immediately:

```bash
cd your-existing-repo
aig import          # imports git history, builds intent graph
aig log             # browse intents instead of flat commits
aig why src/app.py:42  # trace any line to its intent
aig review          # review the most recent intent
```

That's it. Four commands, no setup beyond installing aig. Read on for the full workflow.

The following walkthrough uses a small Python project as an example. Every command is runnable --- follow along in your own repository.

### 1. Initialize aig

Navigate to an existing git repository and run:

```bash
cd your-project
aig init
```

```
Initialized aig in .aig/
  database: .aig/aig.db
  objects:  .aig/objects/

Start a session with: aig session start "your intent"
```

This creates a `.aig/` directory alongside `.git/`. It contains a SQLite database for tracking intents, sessions, and conversations, plus a content-addressable blob store for AST snapshots. Your `.git/` directory is not modified.

### 2. Start a session with an intent

Before you write any code, declare what you are about to work on:

```bash
aig session start "Add user registration endpoint"
```

```
Session started
  intent:  Add user registration endpoint
  session: e786f65dcf8b

Make your changes, then run: aig checkpoint "what you accomplished"
```

The intent is recorded before any changes are made. This is the key difference from conventional commits --- the *why* is captured up front, not retrofitted into a commit message after the fact.

### 3. Add conversation context

Attach your reasoning, constraints, or decisions to the session. This step is optional but makes the history far more useful for anyone reading it later:

```bash
aig conversation add "Using bcrypt for password hashing"
```

```
Conversation note added to session
  intent: Add user registration endpoint
  note:   Using bcrypt for password hashing
```

You can add as many notes as you like throughout the session. Think of these as the design rationale that normally lives only in Slack threads or your head.

### 4. Make your changes

Edit files, run your AI coding assistant, iterate. aig does not interfere with your editor, your build tools, or git itself. Work the way you normally do.

### 5. Check what changed

When you are ready to review, aig gives you two views:

```bash
aig status
```

```
Active session
  intent:        Add user registration endpoint
  session:       e786f65dcf8b
  started:       2026-04-14T00:37:15+00:00
  checkpoints:   0
  conversations: 1

  modified files:
    src/auth.py
    src/routes.py
```

For a deeper look, use the semantic diff:

```bash
aig diff --semantic
```

```
--- src/auth.py (semantic)
  + added `hash_password` — added function `hash_password`
  + added `validate_email` — added function `validate_email`
--- src/routes.py (semantic)
  + added `register_user` — added function `register_user`
  ~ modified `setup_routes` — modified function `setup_routes`
```

The semantic diff operates at the AST level --- it shows which functions, classes, and structures were added, removed, or modified, rather than raw line changes. Use `aig diff` (without `--semantic`) for a traditional line-based diff when you need it.

### 6. Create a checkpoint

A checkpoint bundles a git commit with aig metadata. It links the commit to the current intent, conversation notes, and semantic changes. **The message is optional** — if you omit it, aig generates one from the semantic diff:

```bash
aig checkpoint
```

```
  auto-message: added hash_password, added validate_email, added register_user, modified setup_routes
  semantic:
    + added hash_password (src/auth.py)
    + added validate_email (src/auth.py)
    + added register_user (src/routes.py)
    ~ modified setup_routes (src/routes.py)
Checkpoint created
  message:    added hash_password, added validate_email, added register_user, modified setup_routes
  intent:     Add user registration endpoint
  git commit: 4d95745f
  checkpoint: 3ee53e687f0c
```

Under the hood, this runs `git commit` and then writes metadata into `.aig/aig.db`. The git commit is a normal commit --- tools like `git log`, GitHub, and CI all work exactly as before.

### 7. Capture Claude Code conversation

If you are working with Claude Code, you can pull its conversation history into your session at any time:

```bash
aig capture
```

```
Captured 42 conversation entries from Claude Code
  intent:  Add user registration endpoint
  session: e786f65dcf8b
```

This is also done automatically when you end a session, so you do not need to run it manually unless you want to capture mid-session.

### 8. Continue working or end the session

You can create as many checkpoints as needed within a session. Each one is linked to the same intent. When the work is done, end the session:

```bash
aig session end
```

```
Auto-captured 42 conversation entries from Claude Code
Session ended
  intent:      Add user registration endpoint
  checkpoints: 2
  duration:    2026-04-14T00:37:15+00:00 -> 2026-04-14T01:24:30+00:00
```

Notice that `session end` automatically captures any Claude Code conversation entries, so nothing is lost.

### 9. Browse history by intent

Instead of a flat list of commits, `aig log` groups history by intent:

```bash
aig log
```

```
[ad26f1ba] Add user registration endpoint (done)
         2 checkpoint(s) | 2026-04-14T00:37:15+00:00
           (4d95745f) Registration endpoint with email validation
                     + added `hash_password` (src/auth.py)
                     + added `validate_email` (src/auth.py)
           (7f8c9d2a) Added rate limiting to registration
```

This makes it easy to find when and why a feature was introduced, even months later.

### 10. Understand any line

Point `aig why` at a specific file and line to trace it back through the full chain of intent, checkpoint, and reasoning:

```bash
aig why src/auth.py:42
```

```
src/auth.py:42

  Intent:     [ad26f1ba] Add user registration endpoint
  Checkpoint: Registration endpoint with email validation
  Commit:     4d95745f
  Time:       2026-04-14T00:37:16+00:00

  Semantic changes:
    + added `hash_password` — added function `hash_password`
    + added `validate_email` — added function `validate_email`

  Conversation notes:
    - Using bcrypt for password hashing
```

No more guessing why a line exists. The full context --- from high-level intent down to design rationale --- is one command away.

## Commands Reference

| Command | Description |
|---|---|
| `aig init` | Initialize aig in current git repo |
| `aig session start "intent"` | Start a tracked session |
| `aig session end` | End current session (auto-captures conversation) |
| `aig checkpoint [message]` | Create checkpoint — auto-generates message from semantic diff if omitted |
| `aig status` | Show active session and working tree state |
| `aig log` | Show intent-level history with semantic changes |
| `aig diff` | Show line-based diff |
| `aig diff --semantic` | Show semantic (AST-level) diff |
| `aig why file:line` | Trace a line to its intent, semantics, and reasoning |
| `aig review [intent-id]` | Review an intent — summary, changes, conversations |
| `aig import` | Import existing git history (idempotent) |
| `aig push [remote]` | Push aig metadata to remote via git notes |
| `aig pull [remote]` | Pull aig metadata from remote via git notes |
| `aig capture` | Import Claude Code conversation into active session |
| `aig watch [--auto-checkpoint]` | Watch files for changes, optionally auto-checkpoint |
| `aig conversation add "note"` | Add manual reasoning note to session |

## What's Created

When you run `aig init`, a single directory is created alongside your existing `.git/`:

```
.aig/
├── aig.db      # SQLite database (intents, sessions, checkpoints, conversations)
└── objects/    # Content-addressable blob store (AST snapshots)
```

Everything aig tracks lives here. Your `.git/` directory is completely untouched --- standard git commands (`git log`, `git push`, `git diff`, etc.) continue to work exactly as before.

If you do not want to commit aig metadata to version control, add `.aig/` to your `.gitignore`:

```
# .gitignore
.aig/
```

aig metadata can be shared across clones via git notes using `aig push` and `aig pull`. See the [Commands Reference](#commands-reference) for details.

## Supported Languages (Semantic Diff)

The semantic diff (`aig diff --semantic`) parses source files into ASTs to detect structural changes. The following languages are fully supported:

- **TypeScript** (`.ts`, `.tsx`)
- **Python** (`.py`)
- **Rust** (`.rs`)
- **Go** (`.go`)
- **Java** (`.java`)
- **C#** (`.cs`)
- **C++** (`.cpp`, `.cc`, `.cxx`, `.hpp`, `.h`)
- **Ruby** (`.rb`)

For all other file types, aig falls back to a standard line-based diff automatically. No configuration is needed — language detection is based on file extension.

## Next Steps

Now that you've seen the basics, read the [Daily Workflow](/guide/daily-workflow) guide to learn how aig fits into your actual routine — how it works alongside git, what to do with branches and PRs, how it integrates with AI assistants, and how to work in a team where not everyone uses aig.
