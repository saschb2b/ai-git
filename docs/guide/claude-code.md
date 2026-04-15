---
description: "Set up Claude Code to use aig automatically — LLM-enhanced checkpoints, session management, and semantic commit messages without manual intervention."
---

# Claude Code Integration

aig works with any AI coding assistant out of the box — it captures conversations and tracks changes regardless of which tool you use. But with **Claude Code**, you can go a step further: make the AI agent *use aig directly*, so session management, checkpoints, and semantic messages happen automatically as part of your agentic workflow.

This page explains the two levels of integration and how to set them up.

## Level 1: Passive (works out of the box)

Without any setup, aig already captures Claude Code conversations:

```bash
aig session start "Add authentication"
# ... use Claude Code to write code ...
aig checkpoint
aig session end    # auto-captures Claude Code conversation
```

Claude Code doesn't know about aig. You run the commands manually. aig captures what Claude Code did by reading its conversation history on `session end`.

**This is documented in [Daily Workflow — Working With an AI Assistant](/guide/daily-workflow#working-with-an-ai-assistant).**

## Level 2: Active (Claude Code uses aig for you)

With a `CLAUDE.md` file and a `/aig` slash command, Claude Code becomes aig-aware. It will:

- Start sessions before beginning work
- Create checkpoints with rich, semantic messages (not just "Update file.ts")
- Record design decisions as conversation notes
- End sessions when work is complete

The difference is dramatic. Instead of generic auto-messages:

```
(322a3d4f) Update docs/index.md
(753a37d5) Update HeroTerminal.vue
(75f1c03f) Update custom.css
```

You get LLM-crafted descriptions:

```
(322a3d4f) Restructure landing page with GitHub-style dark canvas
(753a37d5) Add full aig workflow animation to hero terminal
(75f1c03f) Replace stacked gradient blocks with continuous dark surface
```

### Setup

Two files in your repo enable this. Both are included in the aig repository as examples — copy them to your project or adapt them.

#### 1. `CLAUDE.md` — Agent instructions

Create a `CLAUDE.md` file in your project root. This is read by Claude Code at the start of every conversation and tells it how to behave:

```markdown
# Version Control

This project uses **aig** for version control on top of git.

## Workflow

- Before making changes, check `aig status`. If no session is active, start one:
  `aig session start "description of the work"`
- After meaningful progress, checkpoint with a descriptive message:
  `aig checkpoint "added rate_limit middleware with per-IP throttling"`
- Use `aig checkpoint` instead of `git commit` when a session is active.
- Record design decisions: `aig conversation add "chose X over Y because..."`
- End sessions when done: `aig session end`
- Push metadata after git push: `aig push`
```

The key instruction is telling the agent to use `aig checkpoint` instead of `git commit` and to write semantic messages. Claude Code follows these instructions across all conversations in this project.

#### 2. `.claude/commands/aig.md` — The /aig slash command

Create `.claude/commands/aig.md` in your project. This registers a `/aig` slash command you can invoke manually for LLM-enhanced operations:

| Command | What it does |
|---|---|
| `/aig checkpoint` | Runs semantic diff, crafts a rich message, checkpoints |
| `/aig session start "desc"` | Starts a session (checks for existing one first) |
| `/aig session end` | Ends session with a summary of accomplishments |
| `/aig why file:line` | Explains why code exists with full LLM synthesis |
| `/aig status` | Enhanced status with pending semantic changes |
| `/aig log` | Shows intent history |

The full command file is available in the [aig repository](https://github.com/saschb2b/ai-git/blob/main/.claude/commands/aig.md).

### How `/aig checkpoint` works

When you run `/aig checkpoint`, Claude Code:

1. Runs `aig diff --semantic` to see structural changes (functions added/modified/removed)
2. Reads the raw diff for additional context
3. **Uses its language understanding** to write a checkpoint message that describes the *meaning* of the changes
4. Runs `aig checkpoint "the generated message"`

This is the LLM bridge — aig provides the structured semantic diff, Claude Code provides the natural language synthesis. No LLM API key needed in aig itself.

### How `CLAUDE.md` auto-behavior works

With the `CLAUDE.md` in place, you don't need to invoke `/aig` explicitly for basic operations. Claude Code will:

- Check `aig status` at the start of a task
- Run `aig session start` if no session is active
- Use `aig checkpoint` instead of `git commit` throughout the conversation
- Write meaningful checkpoint messages based on what it changed

The `/aig` command is there for when you want explicit control — e.g., `/aig checkpoint` to force a checkpoint with an LLM-crafted message at a specific point, or `/aig why` for deep explanations.

## What gets captured

With Level 2 integration, your aig history captures everything:

| Layer | Source | Example |
|---|---|---|
| **Intent** | `aig session start` | "Add API rate limiting" |
| **Semantic changes** | `aig checkpoint` | + added `rate_limit` (middleware.rs) |
| **Checkpoint message** | LLM-generated | "Add rate limiting middleware with per-IP throttling and Redis backend" |
| **Design decisions** | `aig conversation add` | "Chose token bucket over sliding window — simpler, good enough for our scale" |
| **AI conversation** | Auto-captured on `session end` | Full Claude Code transcript |
| **Trust metadata** | Automatic | Which changes were AI-assisted vs human-authored |

Six months later, anyone can run `aig why middleware.rs:42` and understand not just *what* the code does, but *why* it exists, *who* (human or AI) wrote it, and *what alternatives* were considered.

## Other AI tools

The `CLAUDE.md` + `/aig` command approach is specific to Claude Code. For other AI assistants:

- **Cursor / Copilot / other editors** — Use Level 1 (passive). Run aig commands manually. Capture conversations with `aig capture --file conversation.jsonl`.
- **Any tool with a CLI/API** — You could build similar integration. The pattern is: read `aig diff --semantic`, use your LLM to write a message, call `aig checkpoint "message"`.

The aig CLI is the stable interface. How you invoke it — manually, via Claude Code, or via your own scripts — is up to you.
