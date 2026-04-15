---
outline: deep
description: "aig roadmap: from working tool to git-equivalent ecosystem. Semantic merge, trust scoring, TUI review, web UI, multi-agent coordination, and beyond."
---

# Roadmap

aig has proven that intent-based version control works. The core is solid — intent tracking, semantic diffing, conversation capture, remote sync, and review are all shipped. To become a real alternative to git-centered workflows, aig needs to grow from a developer tool into a full ecosystem. This page maps out the path from where we are to where we need to be.

## What Works Today (v0.1)

The foundation is in place. These features are shipped and working:

| Capability | Commands | Status |
|---|---|---|
| Intent tracking | `aig session start/end`, `aig checkpoint` | Working |
| Semantic diff | `aig diff --semantic` (11 languages) | Working |
| Intent history | `aig log` | Working |
| Line-to-intent tracing | `aig why file:line` | Working |
| Git import (incremental) | `aig import` | Working |
| AI conversation capture | `aig capture` (any AI tool) | Working |
| File watching | `aig watch --auto-checkpoint` | Working |
| Conversation notes | `aig conversation add` | Working |
| Intent review | `aig review` | Working |
| Remote sync | `aig push`, `aig pull` (via git notes) | Working |
| Metadata repair | `aig repair` (after rebase/cherry-pick) | Working |
| Cross-platform CI | Linux, macOS, Windows | Working |
| Bundle export/import | `aig export`, `aig import-bundle` | Working |
| Trust & provenance | `aig trust`, `aig reviewed` | Working |
| LLM explanations | `aig why --explain` | Working |
| Interactive TUI | `aig review --tui` | Working |
| Git hooks | `aig hooks install/remove` | Working |
| One-command onboarding | `aig init --import` | Working |
| Installation | `cargo install --git` or GitHub Releases | Working |

## Phase 1: Make It Shareable <Badge type="tip" text="complete" />

*All Phase 1 goals are shipped.*

### ~~Remote sync via git notes~~ — Shipped
Intent and conversation metadata is serialized into [git notes](https://git-scm.com/docs/git-notes) attached to commits. `aig push` and `aig pull` sync metadata to any git remote. Any aig-aware client reconstructs the local database from notes; any git-only client ignores them.

### ~~Incremental import~~ — Shipped
`aig import` detects commits added since the last import and processes only those — safe to re-run as new commits arrive. Essential for mixed teams where some members use aig and others don't.

### ~~`.aig/` portability~~ — Shipped
`aig export` creates a portable `.tar.gz` bundle containing the full database and object store. `aig import-bundle` restores it — useful for backups, migration, or sharing metadata outside of git notes.

## Phase 2: Make It Smart

*The semantic layer needs to go beyond diffing into merging and understanding.*

### Semantic merge engine
When two developers (or two AI agents) modify the same file, git produces text-level conflicts. A semantic merge engine understands that adding a method to a class and renaming that class are compatible operations — and composes them automatically. This requires AST-level merge logic built on the same tree-sitter infrastructure we already use for diffing.

### ~~Trust scoring and provenance~~ — Shipped
`aig trust [file]` shows which code regions are human-written vs AI-assisted, and whether they've been reviewed. `aig reviewed <file|intent>` marks regions as human-reviewed. Provenance is recorded automatically during each checkpoint based on whether the session has AI conversation captures. Useful for security audits, onboarding, and compliance.

### ~~LLM-powered `aig why`~~ — Shipped
`aig why file:line --explain` synthesizes a natural-language explanation from the intent, checkpoint, conversation notes, and semantic changes. Without `--explain`, the raw metadata is shown as before. Requires the `@aig/llm` package.

### ~~Broader language support~~ — Shipped
Semantic diff now supports 11 languages: TypeScript/JavaScript, Python, Rust, Go, Java, C#, C++, Ruby, PHP, Kotlin, and Swift. The infrastructure supports any language with a tree-sitter grammar.

## Phase 3: Make It Visual

*CLI is for power users. Teams need visual tools.*

### ~~TUI review interface~~ — Shipped
`aig review --tui` opens an interactive terminal UI (React/Ink) with a navigable intent list on the left and details on the right — checkpoints, semantic changes, trust/provenance, and conversation notes. Navigate with j/k or arrow keys, q to quit.

### Web UI
A locally-served web interface for team-level features:
- Intent graph visualization (D3) — see how features, refactors, and fixes relate
- Cross-session search — "find every conversation where we discussed caching"
- Team dashboard — who's working on what intent, how many checkpoints this week
- Timeline view — visualize the rate and shape of change over time

### IDE integration
VS Code / JetBrains extensions that show `aig why` inline (hover over a line to see its intent and conversation), highlight AI-generated vs human-written code, and let you start/end sessions from the editor.

## Phase 4: Make It Collaborative

*From single-developer tool to team infrastructure.*

### Multi-agent coordination
When multiple AI agents work on the same codebase simultaneously, the VCS needs to:
- Track which agent made which changes
- Detect intent-level conflicts before code is written ("two agents are both restructuring auth")
- Merge agent outputs using semantic merge
- Maintain a unified conversation history across agents

### Ownership maps
Define zones of authority: "the billing module requires human approval for any change," "the utility library is AI-autonomous within test constraints." The VCS enforces these boundaries — an agent can't modify a human-only zone without explicit approval.

### Drift detection
Monitor the codebase for architectural drift: when accumulated AI changes gradually shift the codebase away from documented design goals (e.g., dependency depth creeping up, module boundaries eroding, patterns shifting from event-driven to request-response). Raises alerts when cumulative changes cross thresholds.

### Real-time collaboration (CRDTs)
For teams that want Google-Docs-style concurrent editing of the intent graph, layer CRDT support (Automerge/Yjs) on top of the storage model. This is a long-term goal — the data model is designed to support it, but the initial versions target async workflows.

## Phase 5: Replace the Git Layer

*The endgame: aig becomes the primary interface, git becomes an implementation detail.*

### Native aig remotes
Instead of relying on git notes for metadata transport, implement native aig remotes that sync the full intent graph, conversation history, and semantic snapshots efficiently. Git remains the content-addressable store underneath, but the protocol layer is aig-native.

### Native `aig push` / `aig pull`
Today's `aig push` and `aig pull` work via git notes — functional but limited. Native remotes would transfer intents, conversations, and semantic metadata as first-class objects with their own wire protocol, not git commits with notes bolted on.

### Continuous versioning
Remove the need for explicit checkpoints entirely. The system captures state continuously (leveraging the existing `aig watch` infrastructure) and uses AI to automatically identify meaningful boundaries — "it looks like you just finished the auth feature" — and crystallize checkpoints from the stream.

### Intent-level PR workflows
Today's `aig review` shows intent summary, semantic changes, and conversation context. The endgame is a complete replacement for the PR review workflow: approve at the intent level, drill into specifics only where needed, with integration into GitHub/GitLab review flows.

## The Gap Between aig and Git's Ecosystem

To be honest about where aig stands relative to git:

| Git has | aig status |
|---|---|
| 20 years of battle-testing | Months old, actively developed |
| GitHub/GitLab/Bitbucket hosting | Syncs via git notes (`aig push/pull`) |
| Pull request workflows | `aig review` (CLI); PR integration planned (phase 5) |
| Branch/merge/rebase | Delegates to git (phase 2: semantic merge) |
| Thousands of integrations | CLI only (phase 3: IDE extensions) |
| `.gitignore`, hooks, submodules | Delegates to git |
| Millions of users | You, right now |

aig doesn't need to replace all of this. Most of it (hosting, CI, hooks) works because aig repos *are* git repos. The gap is in collaboration and visualization — the parts where aig's richer data model can offer something genuinely better than what git provides.

The strategy is not "replace git" but "make git's data model a storage layer underneath a richer interface." The same way git didn't replace the filesystem — it built a better abstraction on top of it.

---

*Want to contribute? The codebase is at [github.com/saschb2b/ai-git](https://github.com/saschb2b/ai-git). The [RESEARCH.md](https://github.com/saschb2b/ai-git/blob/main/RESEARCH.md) has the full vision. Pick a phase, open an issue, or just start hacking.*
