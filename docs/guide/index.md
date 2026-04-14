# What is aig?

aig is an intent-based version control system that layers on top of git. Instead of treating version history as a sequence of diffs, aig captures the full context behind every change: the goal you set out to accomplish, the conversation that shaped the implementation, and the structural impact on your codebase. The result is a history that anyone — human or AI — can actually understand months later.

## The problem

AI-assisted development is changing how code gets written. A developer can generate hundreds of lines in a single session, refactoring entire modules through conversation. Git was not designed for this. A commit message like "refactor auth module" tells you nothing about why the refactor happened, what trade-offs were considered, or which parts of a conversation led to which changes. The context that produced the code vanishes the moment the chat window closes.

## Three layers of understanding

aig organizes every change into three layers:

- **Intent layer** — The human goal. "Add JWT authentication" or "Fix race condition in queue worker." This is what you declared before any code was written. Intents form a graph, so sub-tasks link back to the larger objective.

- **Semantic layer** — The structural changes. Functions added, parameters modified, classes removed. This layer understands your code's shape, not just its text. It answers "what changed in the architecture?" without forcing you to read raw diffs.

- **Diff layer** — The line-level changes. The same data git stores today. Always available, but no longer the only way to understand what happened.

## What the workflow looks like

```bash
aig session start "Add JWT authentication"
# ... make changes with your AI assistant ...
aig checkpoint
# -> auto-message: "added generate_token, added validate_token, added AuthMiddleware"

# Or add your own message when context matters:
aig checkpoint "Using HS256 over RS256 for single-service simplicity"

# Later, anyone can ask:
aig why src/auth.py:42
# -> Intent: "Add JWT authentication"
# -> Semantic: + added `generate_token`
# -> Conversation: (auto-captured from Claude Code)
```

You start a session with an intent, optionally annotate decisions along the way, and checkpoint your progress. The semantic diff shows structural changes at a glance. And `aig why` traces any line back through the intent and conversation that produced it.

## Auto-capture from Claude Code

If you use Claude Code, aig can automatically import the conversation that produced your changes. Run `aig capture` during a session, or let it happen automatically when you end a session with `aig session end`. The entire human-AI dialogue becomes part of your version history — no manual notes needed.

## File watching

Run `aig watch --auto-checkpoint` to have aig monitor your working directory and automatically create checkpoints after periods of quiet. No more forgetting to commit.

## Current status

aig ships 16 commands, semantic diff for 8 languages, Claude Code integration, file watching, remote sync via git notes, and git import with incremental updates. Built in Rust + TypeScript, runs on Linux, macOS, and Windows.

```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
```
