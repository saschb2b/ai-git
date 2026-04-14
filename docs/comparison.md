# Git vs aig: What Changes

## The Problem

Git was built for a world where humans write code line by line, think about it, and commit it with a message. That model held up remarkably well for decades. But AI-assisted development has changed the dynamics. A developer can now generate large, sweeping changes across dozens of files in seconds — an entire module refactored through a single conversation. Git's line-based diffs turn these changes into walls of red and green that obscure what actually happened. You end up scrolling through hundreds of changed lines with no structural overview of what matters.

Commit messages were already an afterthought when humans wrote every line. With AI-generated code, they're even further removed from reality. The developer types "refactor auth module" and moves on. The actual reasoning — why JWT was chosen over session tokens, why HS256 was picked over RS256, what alternatives were rejected — lives in a chat window that gets closed and forgotten. The richest context about your codebase exists in the human-AI conversation, and git has no concept of it.

This shows up most painfully when someone tries to understand existing code. `git blame` tells you that a developer changed a line six months ago, but that developer was prompting an AI. The person who wrote the commit message may barely remember the conversation that produced the code. The reasoning is gone. The trade-offs are gone. The "why" is gone.

The tools we use to track code history haven't kept up with how code is actually being written. That's the gap aig is designed to fill.

## Side-by-Side Comparison

| Aspect | Git | aig |
|---|---|---|
| Unit of work | Commit (text diff) | Intent (goal + conversation + semantic changes + diff) |
| History view | Linear commit log | Intent graph with nested checkpoints |
| Change tracking | Line-based diffs | Semantic AST-level changes |
| Commit message | Written after the fact | Intent declared before work begins |
| Context preserved | Commit message only | Full conversation and reasoning |
| Understanding code | `git blame` (who/when) | `aig why` (intent/reasoning/alternatives) |
| Review experience | Raw diffs, all or nothing | Multi-layer: intent, semantic, diff |
| Authorship | Single author field | Trust gradient (human/AI/confidence) |
| Migration | — | `aig import` (non-destructive) |
| Compatibility | — | Every aig repo is a valid git repo |

## Concrete Examples

### Example 1: Understanding a Change

Six months from now, a new developer is staring at a line of code and wants to know why it's there.

**With git:**

```bash
git log --oneline
# a3f1b2c refactor auth module
git show a3f1b2c
# ... 400 lines of diff, no context on WHY
git blame src/auth.py
# a3f1b2c (developer  2024-03-15) token = jwt.encode(...)
# Who can you ask? The developer who prompted an AI 6 months ago.
```

You get a name, a date, and a terse commit message. The developer vaguely remembers the conversation. The reasoning is lost.

**With aig:**

```bash
aig log
# [a3f1] Add JWT authentication (done)
#   (b7c2) Token generation and validation implemented
#   (d4e8) Auth middleware integrated
aig why src/auth.py:42
# Intent: "Add JWT authentication"
# Note: "Using HS256 over RS256 for simplicity in single-service deployment"
# Checkpoint: "Token generation and validation implemented"
```

The intent, the reasoning behind the decision, and the checkpoint that produced the line are all retrievable. No Slack archaeology required.

### Example 2: Reviewing AI-Generated Code

An AI assistant just refactored a module. You need to review what it did.

**With git:**

```
$ git diff
# 847 lines changed across 23 files
# Good luck figuring out what actually matters.
```

You're left scanning hundreds of lines trying to separate meaningful structural changes from mechanical ones.

**With aig:**

```
$ aig diff --semantic
# --- src/auth.py (semantic)
#   + added `generate_token`
#   + added `validate_token`
#   ~ modified `authenticate` — added JWT parameter
# --- src/middleware.py (semantic)
#   + added `require_auth`
# --- src/routes.py (semantic)
#   ~ modified 4 route handlers — wrapped with require_auth
```

The semantic diff shows you what changed in the code's structure. New functions, modified signatures, added wrappers. You can drill into the line-level diff when you need it, but you start with the overview.

### Example 3: Onboarding a New Developer

A new team member joins and needs to understand how the codebase got to its current state.

**With git:** Read outdated docs, grep through commit messages hoping for clues, ask around on Slack, piece together a mental model from fragments.

**With aig:** Browse the intent graph to understand the narrative of how the codebase evolved. Each feature, refactor, and bug fix is an intent with its rationale attached. Sub-tasks nest under larger objectives, so the structure of the work is visible. `aig why` any confusing line and get the full context — the goal, the conversation, the alternatives that were considered.

## What aig Does NOT Replace

To be clear about what aig is and isn't:

- **aig does not replace git.** It layers on top of it. Under the hood, aig uses git for all storage and transport.
- **All standard git commands still work.** You can `git push`, `git pull`, `git branch`, `git merge` — everything works exactly as before.
- **Your repo stays a valid git repo.** aig adds metadata that enriches the history. It does not modify git's data model.
- **If you stop using aig, nothing breaks.** Your git repo is perfectly intact. You lose the extra context (intents, conversations, semantic diffs), but the code and its git history remain untouched.
- **Collaborators don't need aig.** Team members who don't use aig can work with the repo normally through git. They just won't see the enriched context.

aig is additive. It captures information that currently gets lost, without changing or risking anything that already works.

## Recently Shipped

- **Claude Code conversation capture** — `aig capture` auto-captures the AI conversation into version history
- **Auto-capture on session end** — Conversations are preserved automatically
- **File watching with auto-checkpoint** — `aig watch --auto-checkpoint` continuously captures file state changes
- **`cargo install` support** — Install aig directly via Cargo

## What's Coming

Next up: remote sync via git notes, semantic merge engine, trust scoring, TUI review interface, and more. See the full [Roadmap](/roadmap) for the path from MVP to git-equivalent ecosystem.
