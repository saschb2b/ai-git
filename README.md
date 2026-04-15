# aig — Version Control for the AI Age

[![CI](https://github.com/saschb2b/ai-git/actions/workflows/ci.yml/badge.svg)](https://github.com/saschb2b/ai-git/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/saschb2b/ai-git)](https://github.com/saschb2b/ai-git/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey)]()

**Git tracks *what* changed. aig tracks *why*.**

aig is an intent-based version control layer that sits on top of git. It captures the goal behind every change, the AI conversation that shaped it, and the structural impact on your codebase — so your history tells a story, not just a diff.

> Every aig repo is a valid git repo. Your existing tools, CI, and workflows keep working unchanged.

## See the difference

**`git log`** gives you this:

```
43ed456 Initial commit
fd990ed Add auth module
7564dca Fix tests
6bdca12 Refactor middleware
```

**`aig log`** gives you this:

```
[a3f1b2c4] Add JWT authentication (done)
         3 checkpoint(s) | 2 h 15 min
           (b7c2d3e4) Token generation and validation
                     + added generate_token (auth.py)
                     + added validate_token (auth.py)
           (d4e8f9a0) Auth middleware integration
                     ~ modified setup_routes (routes.py)
           (f1a2b3c4) Test coverage
                     + added test_auth_flow (test_auth.py)
```

Four commits become one intent. You see the narrative, not the noise.

**`git blame`** gives you a name and a date. **`aig why`** gives you the full story:

```
$ aig why src/auth.py:42

  Intent:     [a3f1b2c4] Add JWT authentication
  Checkpoint: Token generation and validation
  Commit:     b7c2d3e4
  Time:       2026-04-14 01:06

  Semantic changes:
    + added `generate_token`
    + added `validate_token`

  Conversation notes:
    - Chose HS256 over RS256 for simplicity in single-service deployment
```

## Quick start

```bash
# Install (Linux / macOS)
curl -fsSL https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.sh | sh
```

```powershell
# Install (Windows)
irm https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.ps1 | iex
```

```bash
# Try it on any existing git repo
cd your-project
aig init --import        # initialize + import git history in one step
aig log                  # browse intents instead of flat commits
aig why src/app.py:42    # trace any line to its intent
```

## How it works

```bash
# 1. Declare what you're working on
aig session start "Add user authentication"

# 2. Work normally — edit files, use AI, iterate
# 3. Checkpoint when ready (message auto-generated from code changes)
aig checkpoint

# 4. Add reasoning (optional but valuable)
aig conversation add "Chose bcrypt over argon2 for compatibility"

# 5. End the session
aig session end

# 6. Share with your team
git push && aig push
```

## Features

| Feature | Command | What it does |
|---------|---------|-------------|
| Intent tracking | `aig session start/end` | Declare goals before writing code |
| Smart checkpoints | `aig checkpoint` | Git commit + semantic analysis, auto-generated messages |
| Semantic diff | `aig diff --semantic` | AST-level changes across 11 languages |
| Line provenance | `aig why file:line` | Trace any line to its intent and conversation |
| LLM explanations | `aig why --explain` | Natural-language synthesis of why code exists |
| Trust scoring | `aig trust [file]` | Track human vs AI-authored code regions |
| Interactive review | `aig review --tui` | Terminal UI for navigating intents |
| Git history import | `aig import` | Retrofit intents onto existing repos (incremental) |
| AI conversation capture | `aig capture` | Auto-capture from Claude Code or any AI tool |
| Remote sync | `aig push / pull` | Share intent metadata via git notes |
| Git hooks | `aig hooks install` | Auto-checkpoint, auto-session, auto-sync |
| File watching | `aig watch` | Auto-checkpoint after quiet periods |
| Releases | `aig release <tag>` | Tag a release grouping intents since the last one |
| Changelog | `aig changelog` | Auto-generate release notes from intents |
| Portable backup | `aig export` | Bundle `.aig` metadata for backup or migration |

## Installation

### One-line install (recommended)

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.ps1 | iex
```

Auto-detects your platform, downloads the latest release, and adds `aig` to your PATH.

### From source

```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
```

### Manual download

Browse [all releases](https://github.com/saschb2b/ai-git/releases) for pre-built binaries (Linux x86_64, macOS aarch64, Windows x86_64).

### Build locally

```bash
git clone https://github.com/saschb2b/ai-git.git
cd ai-git
cargo build --release
# Binary at target/release/aig (.exe on Windows)
```

**Requirements:** Rust 1.75+ and an existing git repository. Node.js 20+ is optional (needed for LLM-powered import and TUI review).

## Semantic diff: 11 languages

TypeScript/JS, Python, Rust, Go, Java, C#, C++, Ruby, PHP, Kotlin, Swift. All other languages fall back to line-based diffing automatically.

## Documentation

- **[Getting Started](https://saschb2b.github.io/ai-git/guide/getting-started)** — install and try it in 60 seconds
- **[Daily Workflow](https://saschb2b.github.io/ai-git/guide/daily-workflow)** — how aig fits into your routine
- **[CLI Reference](https://saschb2b.github.io/ai-git/guide/cli-reference)** — all 21 commands with flags and examples
- **[Migration Guide](https://saschb2b.github.io/ai-git/guide/migration)** — import existing git repos
- **[Roadmap](https://saschb2b.github.io/ai-git/roadmap)** — what's shipped, what's next
- **[RESEARCH.md](RESEARCH.md)** — the full vision (~5,000 words)
- **[TECH_STACK.md](TECH_STACK.md)** — technology choices and rationale

## Repository structure

```
crates/aig-core/          Rust CLI + library (21 commands)
crates/aig-treesitter/    Semantic diff engine (11 languages, tree-sitter)
packages/aig-llm/         LLM integration (TypeScript, Anthropic Claude)
packages/aig-tui/         Interactive TUI (TypeScript, React/Ink)
docs/                     VitePress documentation site
```

## License

[MIT](LICENSE)
