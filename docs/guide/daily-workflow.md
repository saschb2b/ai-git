---
description: "How aig fits into your daily development routine. Replaces git commit, works with AI assistants, handles teams and branches. Includes quick reference table."
---

# Daily Workflow

You've imported your repo and browsed the intent history. Now what? This page explains how aig fits into your actual daily routine — whether you work alone, with a team, or with an AI assistant.

## The Short Version

**`aig checkpoint` replaces `git commit`.** You don't use both. A checkpoint creates a real git commit under the hood, plus records the intent, semantic changes, and conversation context. Everything downstream (GitHub, CI, PRs) sees a normal git commit.

**You don't need to write a message.** Run `aig checkpoint` with no arguments and aig auto-generates the message from the semantic diff — e.g., `"added generate_token, added validate_token, added AuthMiddleware"`. You can still pass an explicit message if you want: `aig checkpoint "my message"`.

```bash
# Before: git workflow
git add .
git commit -m "add auth"
git push

# After: aig workflow
aig session start "Add authentication"
# ... work ...
aig checkpoint                   # auto-message from semantic diff
# ... more work ...
aig checkpoint                   # auto-message: "modified authenticate, added require_auth"
aig session end
git push                         # still push with git — aig doesn't replace transport
aig push                         # also push the aig metadata (intents, conversations)
```

## Solo Developer Workflow

### Starting your day

```bash
aig session start "Fix payment timeout bug"
```

That's it. You've declared what you're working on. Now work normally — edit files, run tests, use your AI assistant.

### While working

Checkpoint whenever you reach a meaningful state. No message needed — aig generates one from what changed:

```bash
aig checkpoint                    # -> "added test_payment_timeout"
aig checkpoint                    # -> "modified connect, added pool_limit"
aig checkpoint                    # -> "modified test_payment_timeout"
```

Or pass an explicit message when you want to add context beyond what the code shows:

```bash
aig checkpoint "Fix applied — root cause was missing connection pool limit"
```

Each checkpoint is a git commit. You can make as many as you want. They all link back to the same intent.

If you want to record a decision or reasoning:

```bash
aig conversation add "Chose connection pooling over retry logic — lower latency"
```

### Ending the session

```bash
aig session end
```

This auto-captures your Claude Code conversation (if you used it) and closes the intent. Then push:

```bash
git push            # pushes the git commits
aig push            # pushes the intent/conversation metadata
```

### Reviewing what you did

```bash
aig review          # intent summary with semantic changes and conversations
aig log             # full intent history
```

## Working With an AI Assistant

### Does my AI need to know about aig?

**No.** Your AI assistant (Claude Code, Cursor, Copilot, etc.) writes code as usual. aig works around it:

1. **You** start the session and declare the intent
2. **The AI** writes code — it doesn't need to know about aig
3. **You** checkpoint when ready — aig analyzes what the AI changed semantically
4. **aig** auto-captures the Claude Code conversation on session end

The AI doesn't run aig commands. You do. aig is the wrapper around your AI-assisted workflow, not something the AI integrates with.

### What gets captured automatically?

- **Semantic changes** — every checkpoint analyzes what functions/classes were added, removed, or modified
- **Claude Code conversations** — auto-captured on `aig session end` or manually with `aig capture`
- **Git metadata** — commit SHA, timestamp, author

### What you add manually (optional but valuable)

- **Intent** — `aig session start "what you're doing"` (required, this is the point)
- **Reasoning** — `aig conversation add "why we chose X over Y"` (optional)

## Team Workflow

### If everyone uses aig

```bash
# Developer A
aig session start "Add OAuth2 support"
aig checkpoint "OAuth flow implemented"
aig session end
git push && aig push

# Developer B (later)
git pull && aig pull
aig log              # sees the OAuth intent with full context
aig why src/auth.py:15  # understands why OAuth was implemented this way
```

### If only some people use aig

That's fine. aig is additive:

- Everyone shares the git repo as normal
- `aig checkpoint` creates standard git commits — non-aig users see them in `git log`
- The commit message includes the intent for context: `"OAuth flow implemented\n\naig intent: Add OAuth2 support"`
- Non-aig users miss the conversations and semantic analysis, but nothing breaks
- An aig user can periodically run `aig import` to pick up commits made with plain git:

```bash
aig import           # safe to run repeatedly — skips already-imported commits
```

## What About Branches and PRs?

**Use branches and PRs exactly as you do now.** aig doesn't change the branching model:

```bash
git checkout -b feature/oauth
aig session start "Add OAuth2 support"
# ... work, checkpoint, checkpoint ...
aig session end
git push -u origin feature/oauth
aig push                              # pushes notes for this branch's commits
# open PR on GitHub as usual
```

The PR diff on GitHub is a normal diff. But anyone with aig can run `aig review` locally to get the intent-level summary instead of reading raw diffs.

## Branches, Merging, Rebase, Cherry-Pick

### Branches

**Work exactly as before.** Create branches, switch between them, merge them. aig sessions are not tied to a specific branch — a session tracks an intent across whatever branch you're on.

```bash
git checkout -b feature/payments
aig session start "Add Stripe payment flow"
# work, checkpoint...
git checkout main
# session is still active — you can resume on any branch
git checkout feature/payments
aig checkpoint "Payment webhook handler done"
```

### Merging

**Works fine.** Merge commits are normal git commits. They don't have aig metadata, but that's expected — the intent context lives on the feature branch commits that are being merged in.

```bash
git checkout main
git merge feature/payments
aig push    # push the notes from the feature branch commits
```

### Rebase — Handle With Care

**Rebase rewrites commit SHAs.** Since aig metadata (git notes) is attached to specific SHAs, a rebase orphans those notes. The intent history still exists in the `.aig/` database, but the git notes point to commits that no longer exist.

**Fix it with `aig repair`:**

```bash
git rebase main                 # SHAs change
aig repair                      # re-attaches orphaned notes to the new commits
aig push --force                # push the repaired notes (or: git push origin refs/notes/aig --force)
```

`aig repair` matches orphaned notes to new commits by comparing commit messages. It updates both the database and the git notes.

**Recommendation:** If you use rebase, always run `aig repair` immediately after.

### Cherry-Pick

**Same issue as rebase** — cherry-pick creates a new commit with a new SHA. The aig metadata from the original commit doesn't follow automatically.

```bash
git cherry-pick abc1234         # creates a new commit with new SHA
aig repair                      # finds the orphaned note and re-attaches it
```

### Interactive Rebase (squash, fixup, reorder)

When you squash commits during interactive rebase, multiple aig checkpoints merge into one git commit. After the squash:

- Run `aig repair` to re-attach what it can match
- Notes from squashed-away commits may become orphaned (the commit messages changed)
- The `.aig/` database still has the full checkpoint history — it's just the git notes that need repair

**Bottom line:** Branches and merging work seamlessly. Rebase and cherry-pick require `aig repair` afterward. This is a fundamental limitation of layering on git notes — SHA-rewriting operations break the link. The roadmap includes a more robust solution via content-based matching rather than SHA-based.

## What About `git commit`?

You can still use `git commit` directly. Those commits won't have aig metadata (no intent, no semantic analysis, no conversation capture), but they won't break anything. They're just "untracked" from aig's perspective.

If you mix `git commit` and `aig checkpoint` in the same session, the `git commit` changes will show up in the git history but won't be linked to the aig intent. The next time you run `aig import`, they'll be picked up and assigned to an inferred intent.

**Recommendation:** Use `aig checkpoint` instead of `git commit` whenever you have an active session. It does everything `git commit` does, plus more.

## File Watching (Hands-Free Mode)

If you don't want to remember to checkpoint:

```bash
aig session start "Refactor database layer"
aig watch --auto-checkpoint
# aig monitors your files and auto-checkpoints after 30 seconds of quiet
# just work — checkpoints happen automatically
```

Press Ctrl+C to stop watching, then `aig session end` when done.

## Quick Reference: Daily Commands

| When | Command |
|---|---|
| Start working on something | `aig session start "what you're doing"` |
| Reached a meaningful state | `aig checkpoint` (auto-message) or `aig checkpoint "message"` |
| Made a design decision | `aig conversation add "why you chose X"` |
| Done with this task | `aig session end` |
| Share with team | `git push && aig push` |
| Get team's context | `git pull && aig pull` |
| Understand a line | `aig why file:line` |
| Review recent work | `aig review` |
| Import non-aig commits | `aig import` |
| After rebase/cherry-pick | `aig repair` |
| Hands-free mode | `aig watch --auto-checkpoint` |
