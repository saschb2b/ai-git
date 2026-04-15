---
description: "aig is an intent-based version control system that layers on top of git. It captures why code changes, not just what changed. See real output from aig running on its own repo."
---

# What is aig?

aig is an intent-based version control system that layers on top of git. Instead of treating version history as a sequence of diffs, aig captures the full context behind every change: the goal you set out to accomplish, the conversation that shaped the implementation, and the structural impact on your codebase. The result is a history that anyone — human or AI — can actually understand months later.

## The problem

AI-assisted development is changing how code gets written. A developer can generate hundreds of lines in a single session, refactoring entire modules through conversation. Git was not designed for this. A commit message like "refactor auth module" tells you nothing about why the refactor happened, what trade-offs were considered, or which parts of a conversation led to which changes. The context that produced the code vanishes the moment the chat window closes.

## Three layers of understanding

aig organizes every change into three layers:

- **Intent layer** — The human goal. "Add JWT authentication" or "Fix race condition in queue worker." This is what you declared before any code was written. Intents form a graph, so sub-tasks link back to the larger objective.

- **Semantic layer** — The structural changes. Functions added, parameters modified, classes removed. This layer understands your code's shape, not just its text. It answers "what changed in the architecture?" without forcing you to read raw diffs.

- **Diff layer** — The line-level changes. The same data git stores today. Always available, but no longer the only way to understand what happened.

## See it in action

This is real output from aig running on its own repository.

### `git log` vs `aig log`

`git log` gives you a flat list of hashes:

```
43ed456 Initial commit
fd990ed Add aig MVP: intent-based version control with docs site
d74a8cd Fix GitHub Pages CI: use root lockfile for pnpm cache
6b42cef Add auto-generated CLI reference and CI sync check
7564dca Complete the core loop: semantic changes, IPC, 18 tests
6bdca12 Add conversation capture, file watching, cargo install
...
```

`aig log` groups by intent and shows semantic changes:

```
[44d3ab98] Rewrite docs for first-time visitors (active)
         7 checkpoint(s) | 2026-04-14
           (120ab1ee) Rewrite docs for first-time visitors
           (3e8142a2) Add Daily Workflow guide
           (ed3a9c5c) Add aig repair for rebase/cherry-pick
           (635eb923) Add Related Tools page
           ...

[479d2692] Initial commit (active)
         12 checkpoint(s) | 2026-04-14
           (fd990ede) Add aig MVP: intent-based version control
           (7564dcaa) Complete the core loop: semantic changes, IPC, 18 tests
           (6bdca120) Add conversation capture, file watching, cargo install
           (7975e6d3) Add remote sync: aig push/pull via git notes
           ...
```

19 commits become 2 intents. You see the narrative, not the noise.

### `git blame` vs `aig why`

`git blame capture.rs` shows:

```
6bdca12 (Sascha Becker 2026-04-14) use anyhow::Result;
```

A name and a date. That's it.

`aig why crates/aig-core/src/capture.rs:1` shows:

```
crates/aig-core/src/capture.rs:1

  Intent:     [479d2692] Initial commit
  Checkpoint: Add conversation capture, file watching, and cargo install
  Commit:     6bdca120
  Time:       2026-04-14T01:06:24+00:00
```

The intent, the checkpoint that introduced the file, and when. If there are conversation notes, those show up too.

### `aig checkpoint` with auto-generated messages

No commit messages to write. Just `aig checkpoint`:

```
$ aig checkpoint

  auto-message: added generate_token, added validate_token, added AuthMiddleware
  semantic:
    + added generate_token (auth.py)
    + added validate_token (auth.py)
    + added AuthMiddleware (auth.py)
Checkpoint created
  message:    added generate_token, added validate_token, added AuthMiddleware
  intent:     Add authentication
  git commit: 8d5b5ff9
```

The semantic diff becomes the commit message automatically.

### `aig diff --semantic` instead of line diffs

Instead of 300 lines of red and green:

```
$ aig diff --semantic

--- auth.py (semantic)
  + added `generate_token` -- added function `generate_token`
  + added `validate_token` -- added function `validate_token`
  ~ modified `authenticate` -- modified function `authenticate`
--- middleware.py (semantic)
  + added `require_auth` -- added function `require_auth`
```

Four lines that tell you what actually changed in the code's structure.

## Auto-capture AI conversations

aig can automatically capture the conversation that produced your changes. Claude Code is auto-detected — run `aig capture` during a session, or let it happen automatically when you end a session with `aig session end`. For any other AI tool, use `aig capture --file conversation.jsonl` to import a conversation in the generic JSONL format (one JSON object per line with `role` and `content` fields). The entire human-AI dialogue becomes part of your version history — no manual notes needed.

## File watching

Run `aig watch --auto-checkpoint` to have aig monitor your working directory and automatically create checkpoints after periods of quiet. No more forgetting to commit.

## Current status

aig ships 21 commands, semantic diff for 11 languages, trust scoring and provenance tracking, LLM-powered explanations (`aig why --explain`), an interactive TUI review (`aig review --tui`), git hooks for automatic tracking, portable bundle export/import, AI conversation capture, file watching, remote sync, and git import with incremental updates. Built in Rust + TypeScript, runs on Linux, macOS, and Windows.

```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
```
