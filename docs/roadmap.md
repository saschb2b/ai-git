---
outline: deep
---

# Roadmap

aig today is an MVP that proves intent-based version control works. But to be a real alternative to git-centered workflows, it needs to grow from a local tool into a full ecosystem. This page maps out the path from where we are to where we need to be.

## What Works Today (v0.1)

The foundation is in place. These features are shipped and working:

| Capability | Commands | Status |
|---|---|---|
| Intent tracking | `aig session start/end`, `aig checkpoint` | Working |
| Semantic diff | `aig diff --semantic` (TS, Python, Rust, Go) | Working |
| Intent history | `aig log` | Working |
| Line-to-intent tracing | `aig why file:line` | Working |
| Git import | `aig import` | Working |
| Claude Code capture | `aig capture`, auto on session end | Working |
| File watching | `aig watch --auto-checkpoint` | Working |
| Conversation notes | `aig conversation add` | Working |
| Cross-platform CI | Linux, macOS, Windows | Working |
| Installation | `cargo install --git` | Working |

## Phase 1: Make It Shareable

*The biggest gap: aig metadata is local-only. You can't collaborate.*

### Remote sync via git notes
The `.aig/` directory doesn't travel with `git push`. The plan is to serialize intent and conversation metadata into [git notes](https://git-scm.com/docs/git-notes) attached to commits. This way, `git push` and `git pull` carry the aig context automatically. Any aig-aware client reconstructs the local database from notes; any git-only client ignores them.

### Incremental import
Currently `aig import` can only run once (re-running creates duplicates). Incremental import would detect commits added since the last import and process only those — essential for mixed teams where some members use aig and others don't.

### `.aig/` portability
Define a stable export/import format so aig metadata can be backed up, migrated, or shared outside of git notes (e.g., for archival or cross-repo analysis).

## Phase 2: Make It Smart

*The semantic layer needs to go beyond diffing into merging, reviewing, and understanding.*

### Semantic merge engine
When two developers (or two AI agents) modify the same file, git produces text-level conflicts. A semantic merge engine understands that adding a method to a class and renaming that class are compatible operations — and composes them automatically. This requires AST-level merge logic built on the same tree-sitter infrastructure we already use for diffing.

### Trust scoring and provenance
Track which lines were human-written vs AI-generated, at what confidence level, and whether a human explicitly reviewed them. This creates a trust gradient per line/function, useful for:
- Security audits ("show me all AI-generated code in the auth module")
- Onboarding ("which parts of this codebase have the most human oversight?")
- Compliance ("flag unreviewed AI-generated code in regulated modules")

### LLM-powered `aig why`
The current `aig why` returns stored metadata. With LLM integration, it could synthesize a natural-language explanation: "This line exists because the team decided to use JWT with HS256 for simplicity. The alternative (RS256) was considered but rejected due to complexity in the single-service deployment. See the conversation from April 14."

### Broader language support
Add tree-sitter grammars for: Java, C#, C/C++, Ruby, PHP, Kotlin, Swift. The infrastructure supports any language with a tree-sitter grammar — it's a matter of adding grammar crates and extending the definition-kind mappings.

## Phase 3: Make It Visual

*CLI is for power users. Teams need visual tools.*

### TUI review interface
A terminal UI (React/Ink) for `aig review` that lets you navigate the three layers interactively: intent → semantic → diff. Expand/collapse semantic changes, view conversations inline, approve or flag changes. Think of it as a PR review tool that starts with meaning, not text.

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

### `aig push` / `aig pull`
Full-featured push and pull that transfer intents, conversations, and semantic metadata as first-class objects — not just git commits with notes bolted on.

### Continuous versioning
Remove the need for explicit checkpoints entirely. The system captures state continuously (leveraging the existing `aig watch` infrastructure) and uses AI to automatically identify meaningful boundaries — "it looks like you just finished the auth feature" — and crystallize checkpoints from the stream.

### `aig review`
A complete replacement for the PR review workflow. Instead of reviewing a branch diff, review an intent: see what the goal was, what conversation produced it, what semantic changes it introduced, and what tests it affected. Approve at the intent level, drill into specifics only where needed.

## The Gap Between aig and Git's Ecosystem

To be honest about where aig stands relative to git:

| Git has | aig status |
|---|---|
| 20 years of battle-testing | MVP, months old |
| GitHub/GitLab/Bitbucket hosting | Local only (phase 1: git notes) |
| Pull request workflows | Not yet (phase 3-5) |
| Branch/merge/rebase | Delegates to git (phase 2: semantic merge) |
| Thousands of integrations | CLI only (phase 3: IDE extensions) |
| `.gitignore`, hooks, submodules | Delegates to git |
| Millions of users | You, right now |

aig doesn't need to replace all of this. Most of it (hosting, CI, hooks) works because aig repos *are* git repos. The gap is in collaboration and visualization — the parts where aig's richer data model can offer something genuinely better than what git provides.

The strategy is not "replace git" but "make git's data model a storage layer underneath a richer interface." The same way git didn't replace the filesystem — it built a better abstraction on top of it.

---

*Want to contribute? The codebase is at [github.com/saschb2b/ai-git](https://github.com/saschb2b/ai-git). The [RESEARCH.md](https://github.com/saschb2b/ai-git/blob/main/RESEARCH.md) has the full vision. Pick a phase, open an issue, or just start hacking.*
