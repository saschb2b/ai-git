---
outline: deep
description: "Why git breaks down with AI-assisted development and what intent-based version control looks like. Five problems, a vision for semantic change tracking, and the case for conversation as first-class history."
---

# Research & Vision

::: info
This is a condensed version of the full [RESEARCH.md](https://github.com/saschb2b/ai-git/blob/main/RESEARCH.md) document. Read the full ~5,000-word version on GitHub for the complete analysis.
:::

## The Core Thesis

Git was designed in 2005 for a world where humans typed every line of code. That world is ending. AI-assisted development changes the fundamental dynamics of how code is written, reviewed, and understood — and git's foundational abstractions are increasingly mismatched with this reality.

**aig** proposes a new model: **intent-based version control** that captures not just *what* changed but *why*, preserving the full chain of reasoning from human intent through AI collaboration to final code.

## Five Problems with Git in the AI Age

### 1. Commit Granularity Mismatch

When an AI restructures 30 files in eight seconds, do you create one massive commit (unreviewable) or artificially split it into sequential steps that never actually happened (manufacturing false history)? Neither option is good. Git assumes the pace of change matches the granularity of commits. AI violates that assumption completely.

### 2. The Diff Problem

Line-based diffs become walls of red and green when an AI rewrites a function or file. A reviewer staring at a 400-line diff cannot tell which changes are load-bearing and which are stylistic. When you ask an AI to convert a React class component to hooks, the diff shows the entire class deleted and a new function added — even though 80% of the logic is identical.

### 3. Authorship and Attribution

Git's author field holds one name. When a human writes a prompt, an AI generates 200 lines, and the human tweaks 3 — who is the author? `git blame` now attributes AI-generated code to people who may barely understand it. The premise of blame ("this person can explain this line") collapses.

### 4. Branching Model Friction

Feature branches, PRs, rebasing — the entire workflow assumes parallel work over days or weeks. AI collapses that timeline to minutes. The ceremony of branch management becomes pure friction, disproportionate to the actual work.

### 5. Knowledge Loss

The richest context in AI-assisted development — the conversation, the alternatives considered, the constraints discussed — is completely lost. Git stores the output of a decision process but none of the process itself. We are generating more code than ever while retaining less understanding of why it exists.

## The Vision: Intent-Based Version Control

### Intent as the Primary Unit

Instead of commits (diffs with afterthought messages), the unit of work is a **declared intent**: "Refactor the payment module to support multiple providers." The conversation that fulfills the intent, the alternatives explored, and the resulting code changes all live together as a single coherent unit.

### Semantic Change Tracking

Instead of line-based diffs, track changes at the structural level: "function `authenticate()` was added," "the return type of `getUser()` changed from `User | null` to `Result<User>`." This makes review meaningful regardless of how many lines changed.

### Three Layers of Understanding

Every change can be viewed at three levels:

| Layer | Shows | Audience |
|---|---|---|
| **Intent** | The goal: "Support multiple currency providers" | PMs, future devs skimming history |
| **Semantic** | Structural impact: "Added interface, extracted class, modified constructor" | Code reviewers |
| **Diff** | Raw line changes | Auditing, debugging |

### Conversation as First-Class History

The human-AI conversation is stored as part of the version history. `aig why` surfaces the original reasoning — not a terse commit message, but the actual discussion that led to the code.

### Continuous Versioning

Instead of manual commits, the system captures state continuously. Developers "crystallize" checkpoints when meaningful milestones are reached. No more forgotten commits, no more giant squash merges.

## Human Oversight

- **Impact-first review** — show what behaviors changed before showing diffs
- **Trust scoring** — track which changes are human-written vs AI-generated
- **Ownership maps** — define zones of human authority vs AI autonomy
- **Explainable history** — `aig why` replaces `git blame` with full reasoning chains
- **Drift detection** — catch architectural drift across accumulated AI changes

## Collaboration

- **Semantic merge** — merge at the AST level, not text level
- **Intent-level conflict detection** — catch conflicting goals before code is written
- **Knowledge propagation** — share discoveries across AI sessions
- **Role-based views** — developers see semantic diffs, PMs see intent progress, auditors see provenance

## Open Questions

### Session ceremony vs. quick fixes

Sessions (`aig session start` → `checkpoint` → `session end`) work well for multi-step features — they group checkpoints under a named intent and capture the full narrative. But what about small bug fixes, typo corrections, or one-liner changes? They still have intent, but a full session feels like overhead.

The tension:

- **If `aig checkpoint` works without a session** and auto-creates a one-shot intent, how does it differ from `git commit`? The semantic diff, conversation capture, and trust metadata still add value — but the workflow feels nearly identical.
- **If we keep sessions mandatory**, quick fixes require three commands (`start`, `checkpoint`, `end`) for what should be a single action.
- **A middle ground** — something like `aig quick "Fix off-by-one"` that creates a session, checkpoints, and closes in one step — keeps the data model clean (every change has an intent) while making the UX lightweight. This could also be handled by the `/aig` Claude Code skill without adding a native command.

Unresolved: is the right answer a native command, a skill-level shortcut, or a rethink of what "session" means? Does every change *need* to be wrapped in a session, or should intents be a broader concept that sessions are just one way to create?

---

*Read the full research document: [RESEARCH.md on GitHub](https://github.com/saschb2b/ai-git/blob/main/RESEARCH.md)*
