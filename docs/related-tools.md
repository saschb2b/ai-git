---
outline: deep
description: "How aig compares to Graphite, GitButler, Linear, CodeRabbit, Difftastic, Cursor, and other tools in the git ecosystem. What each tool adds and where aig fits."
---

# Related Tools

aig isn't the only tool rethinking how developers work with git. Several projects add layers on top of git to solve specific pain points. Understanding what they do — and what they don't — helps clarify what aig brings to the table.

## Tools That Layer on Top of Git

### Graphite

**What it solves:** Stacked pull requests. When you have a chain of dependent PRs (feature A depends on B depends on C), GitHub makes this painful — you manually rebase, update PR descriptions, and lose track of the stack. Graphite adds metadata that tracks the stack, auto-rebases when one PR merges, and provides a streamlined review UI.

**How it works:** Code stays on GitHub. Stack metadata lives on Graphite's server. A CLI (`gt`) wraps git commands to manage the stack.

**Comparison to aig:** Graphite improves the *mechanics* of how PRs flow. aig improves the *understanding* of what changes mean. They solve different problems and could work together — you could use Graphite for stacking and aig for intent tracking on the same repo.

### GitButler

**What it solves:** Branch switching friction. Git forces you to stash or commit before switching branches. GitButler introduces "virtual branches" — you work on multiple things in the same working directory simultaneously, and GitButler routes changes to the right branch based on which files you touch.

**How it works:** A desktop app that manages virtual branches locally. Commits go to real git branches when you're ready.

**Comparison to aig:** GitButler rethinks branching. aig doesn't touch branching at all — it adds an intent and conversation layer. Orthogonal tools that could coexist.

### Sturdy (sunset)

**What it solved:** The entire branch/PR ceremony. Sturdy tried to replace branches and PRs with a continuous collaboration model — you just write code, and Sturdy handles versioning and review. It shut down in 2023, but proved that developers will adopt tools that simplify git workflows.

**Comparison to aig:** Sturdy's "continuous flow" philosophy is similar to aig's `aig watch --auto-checkpoint` — the idea that version control should capture state continuously rather than requiring manual commits. aig takes a more incremental approach by layering on git rather than replacing it.

## Project Management Alongside Git

### Linear

**What it solves:** Issue tracking and project planning. Fast, opinionated, keyboard-driven. Issues live on Linear, code lives on GitHub, they link via PR references and branch naming conventions.

**Why it matters for aig:** Linear demonstrates the architecture model aig is heading toward. Linear doesn't try to replace GitHub — it's a separate layer for a separate concern (planning vs. code). Similarly, aig's future server would be the **intent and conversation layer** that sits next to any git host, the way Linear is the planning layer.

The key insight from Linear: developers will adopt a separate tool when it does one thing exceptionally well and integrates cleanly with what they already use.

## AI Code Review Tools

### CodeRabbit

**What it solves:** Automated PR review. CodeRabbit analyzes pull requests using LLMs and generates summaries of what changed, potential issues, and suggestions. It produces structural summaries like "3 functions added, 1 API endpoint changed."

**Comparison to aig:** CodeRabbit generates summaries at review time and posts them as PR comments. aig generates similar semantic analysis at checkpoint time and stores it permanently as part of the version history. CodeRabbit's summaries are ephemeral; aig's are durable. They could complement each other — CodeRabbit for real-time review feedback, aig for permanent context.

### GitHub Copilot for Pull Requests

**What it solves:** Auto-generated PR descriptions and review summaries. Copilot analyzes the diff and writes a summary of what changed.

**Comparison to aig:** Same pattern as CodeRabbit — generated at review time, not stored as version history. aig captures intent *before* the code is written and stores semantic changes *at commit time*, so the context exists from the start rather than being reconstructed after the fact.

## Semantic Diff Tools

### Difftastic

**What it solves:** Structural diffs in the terminal. Instead of line-based red/green output, Difftastic parses code into ASTs and shows what changed at the structural level. Understands 30+ languages.

**Comparison to aig:** Difftastic is a viewer — it shows you a better diff in real time. aig's `aig diff --semantic` does similar analysis but also **stores** the semantic changes as part of the checkpoint metadata. With aig, the structural diff is part of the permanent history, queryable via `aig why` and `aig review`.

### GumTree

**What it solves:** Academic research tool for computing edit scripts between abstract syntax trees. Provides algorithms for matching AST nodes across two versions of a file and producing a minimal edit script (insert, delete, move, update).

**Comparison to aig:** aig's tree-sitter-based semantic diff is inspired by GumTree's approach. GumTree is a research tool and Java library; aig implements similar concepts in Rust using tree-sitter for parsing and a simplified matching algorithm optimized for the "what definitions changed?" use case.

## AI-Native Editors

### Cursor

**What it solves:** AI-first code editor. Built on VS Code, deeply integrates LLM assistance into the editing experience. Maintains conversation history linked to code changes within a session.

**Comparison to aig:** Cursor keeps conversation history within its own UI, but it doesn't persist into version control. When you close a Cursor session, the conversation is gone from the project's perspective. aig captures this exact gap — it stores conversations as part of the version history so they survive beyond the editor session.

### Windsurf (Codeium)

**What it solves:** Similar to Cursor — AI-native editor with conversation-driven development.

**Comparison to aig:** Same gap as Cursor. The conversation that produced the code lives in the editor, not in the repo. aig's `aig capture` (currently supporting Claude Code) demonstrates how editor conversations can flow into version control. Future integrations with Cursor and Windsurf would follow the same pattern.

## The Common Thread

Every tool on this page proves the same thing: **developers want richer abstractions than raw git provides.** Each one adds a layer for a specific concern:

| Tool | Layer it adds |
|---|---|
| Graphite | PR stacking and flow |
| GitButler | Branch management |
| Linear | Planning and tracking |
| CodeRabbit | Review intelligence |
| Difftastic | Structural visualization |
| Cursor / Windsurf | AI-assisted editing |
| **aig** | **Intent, reasoning, and understanding** |

aig occupies a unique position: it's the only tool focused on preserving *why* code changes, not just improving *how* you manage or review those changes. The intent and conversation layer is the one no other tool covers — and it's the one that matters most as AI generates an increasing share of the code we maintain.
