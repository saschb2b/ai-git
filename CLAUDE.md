# aig — Intent-Based Version Control

This project uses **aig** for version control on top of git. You should use aig commands instead of raw git for tracking changes.

## Workflow

### Starting work
Before making changes, check if there's an active session:
```bash
aig status
```
If no session is active, start one describing what you're about to do:
```bash
aig session start "description of the work"
```

### While working
After making meaningful progress, create a checkpoint. Always craft a descriptive message based on what actually changed — don't use generic file names:
```bash
aig checkpoint "added rate_limit middleware with per-IP throttling"
```

To write a good checkpoint message:
1. Run `aig diff --semantic` to see what changed structurally
2. Summarize the semantic changes in a human-readable message
3. Focus on **what** and **why**, not file paths

### Recording decisions
When you make a design decision or choose between approaches, record it:
```bash
aig conversation add "Chose connection pooling over retry logic — lower latency"
```

### Ending work
When the task is complete:
```bash
aig session end
```

### Pushing changes
```bash
git push          # push git commits
aig push          # push aig metadata (intents, conversations)
```

## Rules
- Use `aig checkpoint` instead of `git commit` when a session is active
- Never skip `aig session start` — the intent is the most valuable piece of metadata
- Run `aig repair` after any rebase or cherry-pick
- Use `aig push` after `git push` to sync metadata

## Available /aig command
This project includes a `/aig` slash command for LLM-enhanced aig operations. Use `/aig checkpoint` for auto-generated semantic messages, `/aig why file:line` for explanations, etc.
