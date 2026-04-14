# Getting Started

This guide walks you through installing **aig**, initializing it in a project, and running through a full workflow so you can see how intent-aware version control works in practice.

## Installation

### Prerequisites

- **Rust toolchain (1.75+)** --- install from [rustup.rs](https://rustup.rs)
- **Node.js 20+ and pnpm 9+** --- required for the LLM integration layer
- **An existing git repository** --- aig layers on top of git; it does not replace it

### Build from source

```bash
git clone https://github.com/saschb2b/ai-git.git
cd ai-git
cargo build --release
# Binary is at target/release/aig (or aig.exe on Windows)
```

To make the `aig` command available everywhere, add the binary to your `PATH`:

```bash
# Linux / macOS
export PATH="$PWD/target/release:$PATH"

# Windows (PowerShell)
$env:PATH = "$PWD\target\release;$env:PATH"
```

Verify the installation:

```bash
aig --version
# aig 0.1.0
```

## Quick Start

The following walkthrough uses a small Python project as an example. Every command is runnable --- follow along in your own repository.

### 1. Initialize aig

Navigate to an existing git repository and run:

```bash
cd your-project
aig init
```

```
Initialized aig in /home/you/your-project/.aig
  Created aig.db (SQLite database)
  Created objects/ (blob store)
```

This creates a `.aig/` directory alongside `.git/`. It contains a SQLite database for tracking intents, sessions, and conversations, plus a content-addressable blob store for AST snapshots. Your `.git/` directory is not modified.

### 2. Start a session with an intent

Before you write any code, declare what you are about to work on:

```bash
aig session start "Add user registration endpoint"
```

```
Session started: s-a1b2c3d
Intent: "Add user registration endpoint"
```

The intent is recorded before any changes are made. This is the key difference from conventional commits --- the *why* is captured up front, not retrofitted into a commit message after the fact.

### 3. Add conversation context

Attach your reasoning, constraints, or decisions to the session. This step is optional but makes the history far more useful for anyone reading it later:

```bash
aig conversation add "Using bcrypt for password hashing, email validation via regex"
```

```
Note added to session s-a1b2c3d
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
Session: s-a1b2c3d
Intent:  "Add user registration endpoint"

Changes:
  modified: src/auth.py
  modified: src/routes.py
  new file: tests/test_auth.py
```

For a deeper look, use the semantic diff:

```bash
aig diff --semantic
```

```
src/auth.py
  + function: hash_password(plain: str) -> str
  + function: validate_email(email: str) -> bool

src/routes.py
  + function: register_user(request: Request) -> Response
  ~ function: setup_routes()  (body changed)

tests/test_auth.py
  + function: test_hash_password()
  + function: test_validate_email_valid()
  + function: test_validate_email_invalid()
```

The semantic diff operates at the AST level --- it shows which functions, classes, and structures were added, removed, or modified, rather than raw line changes. Use `aig diff` (without `--semantic`) for a traditional line-based diff when you need it.

### 6. Create a checkpoint

A checkpoint bundles a git commit with aig metadata. It links the commit to the current intent, conversation notes, and semantic changes:

```bash
aig checkpoint "Registration endpoint with email validation"
```

```
Checkpoint created: c-e4f5a6b
  Git commit: 7f8c9d2
  Intent:     "Add user registration endpoint"
  Session:    s-a1b2c3d
  Changes:    3 files, 2 functions added, 1 function modified
```

Under the hood, this runs `git commit` and then writes metadata into `.aig/aig.db`. The git commit is a normal commit --- tools like `git log`, GitHub, and CI all work exactly as before.

### 7. Continue working or end the session

You can create as many checkpoints as needed within a session. Each one is linked to the same intent:

```bash
aig checkpoint "Added rate limiting to registration"
```

```
Checkpoint created: c-b7c8d9e
  Git commit: 3a4b5c6
  Intent:     "Add user registration endpoint"
  Session:    s-a1b2c3d
```

When the work is done, end the session:

```bash
aig session end
```

```
Session s-a1b2c3d ended
  Intent:      "Add user registration endpoint"
  Checkpoints: 2
  Duration:    47 minutes
```

### 8. Browse history by intent

Instead of a flat list of commits, `aig log` groups history by intent:

```bash
aig log
```

```
Intent: "Add user registration endpoint"
  Session s-a1b2c3d (47 min)
  ├── c-e4f5a6b  Registration endpoint with email validation
  └── c-b7c8d9e  Added rate limiting to registration

Intent: "Set up project structure"
  Session s-x9y8z7 (12 min)
  └── c-f1e2d3c  Initial project scaffolding with FastAPI
```

This makes it easy to find when and why a feature was introduced, even months later.

### 9. Understand any line

Point `aig why` at a specific file and line to trace it back through the full chain of intent, checkpoint, and reasoning:

```bash
aig why src/auth.py:42
```

```
src/auth.py:42
  Content:      rounds = 12  # OWASP recommendation
  Function:     hash_password()
  Checkpoint:   c-e4f5a6b "Registration endpoint with email validation"
  Intent:       "Add user registration endpoint"
  Session:      s-a1b2c3d
  Conversation: "Using bcrypt for password hashing, email validation via regex"
  Author:       you@example.com
  Date:         2026-04-14
```

No more guessing why a line exists. The full context --- from high-level intent down to design rationale --- is one command away.

## Commands Reference

| Command | Description |
|---|---|
| `aig init` | Initialize aig in the current git repository |
| `aig session start "intent"` | Start a tracked session with a declared intent |
| `aig session end` | End the current session |
| `aig checkpoint "message"` | Create a checkpoint (git commit + aig metadata) |
| `aig status` | Show current session and working tree state |
| `aig log` | Show intent-level history |
| `aig diff` | Show line-based diff |
| `aig diff --semantic` | Show semantic (AST-level) diff |
| `aig why file:line` | Trace a line back to its intent and reasoning |
| `aig import` | Import existing git history into aig |
| `aig conversation add "note"` | Add reasoning or context to the current session |

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

In a future release, aig metadata will be shareable across clones via [git notes](https://git-scm.com/docs/git-notes), so team members can access intent history without committing the `.aig/` directory.

## Supported Languages (Semantic Diff)

The semantic diff (`aig diff --semantic`) parses source files into ASTs to detect structural changes. The following languages are fully supported:

- **TypeScript / JavaScript** (`.ts`, `.tsx`, `.js`, `.jsx`)
- **Python** (`.py`)
- **Rust** (`.rs`)
- **Go** (`.go`)

For all other file types, aig falls back to a standard line-based diff automatically. No configuration is needed --- language detection is based on file extension.
