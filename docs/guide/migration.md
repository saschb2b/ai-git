---
description: "Migrate any existing git repository to aig with a single command. Non-destructive import that builds an intent graph from your commit history."
---

# Migrating from Git

## Overview

aig is designed as a non-destructive layer on top of git. Migration is a single command that reads your existing git history and builds an intent graph from it. Your git repo is untouched --- you gain context, you lose nothing.

## Quick Migration

```bash
cd your-existing-repo
aig import
```

That's it. Here is what happens under the hood:

1. **Initialization** --- creates a `.aig/` directory with a SQLite database if one doesn't already exist.
2. **History scan** --- reads up to 10,000 commits from the git log.
3. **Commit clustering** --- groups related commits into intent clusters using heuristics:
   - Same author within a 2-hour window is treated as likely one intent.
   - Adjacent commits with related messages are grouped together.
4. **Intent inference** --- generates intent descriptions from commit messages:
   - Single-commit cluster: uses the commit message directly.
   - Multi-commit cluster: uses the first commit message and summarizes the rest.
5. **Database population** --- creates intent and checkpoint records linking back to the original git commits.

## What You Get After Import

```bash
# Before: just git
git log --oneline
# a1b2c3d Fix login redirect
# d4e5f6a Add OAuth integration
# 7g8h9i0 Update dependencies
# ...

# After: intent-level history
aig log
# [a1b2c3d0] Fix login redirect (done)
#          1 checkpoint(s)
#            (a1b2c3d0) Fix login redirect
#
# [d4e5f6a0] Add OAuth integration (done)
#          3 checkpoint(s)
#            (d4e5f6a0) Add OAuth integration
#            (b2c3d4e5) Add OAuth callback handler
#            (c3d4e5f6) Add OAuth token refresh
```

You can now:

- Browse intent-level history with `aig log`.
- Trace any file back to its originating intent with `aig why file:line`.
- Start using aig sessions going forward for new work.

## Migrating a Large Repository

- Repos with 1,000+ commits may take a few seconds for the clustering step.
- The import processes up to 10,000 commits. Repositories with longer histories are imported up to that limit.
- Each cluster becomes one intent --- a repo with 500 commits might produce roughly 100--200 intents.
- The clustering heuristic is conservative: it prefers smaller clusters over incorrect groupings. You may see some commits that could logically be grouped together end up as separate intents. This is by design --- false separation is less harmful than false grouping.

## Working With Both Git and aig

aig does not take over your repository. The two coexist:

- All standard git commands continue to work normally.
- `aig checkpoint` creates a real git commit under the hood.
- You can still use `git commit` directly --- those commits won't have aig metadata, but that's fine.
- `git push`, `git pull`, `git branch` all work as expected.
- The `.aig/` directory is local-only. Add it to your `.gitignore`:

```
# .gitignore
.aig/
```

### Mixed Team Workflow

If some team members use aig and others don't:

- Everyone shares the git repo as normal.
- aig users get enriched history with intents and conversations.
- Non-aig users see normal git commits. The checkpoint commit messages include the intent for context, so they are still readable.
- Running `aig import` at any point catches up on new commits made outside of aig.

## What the Import Does NOT Do

- **It cannot recover original conversations.** The reasoning behind past commits is lost. Going forward, conversations are captured automatically from supported AI tools (Claude Code is auto-detected, or any tool via `aig capture --file`), or manually via `aig conversation add`.
- **Intent inference from commit messages is best-effort.** Terse commit messages produce terse intents. A commit message like `"fix"` will result in an intent called `"fix"` --- there is no magic.
- **Commits that lack context in their messages will have less meaningful intent descriptions.** The import can only work with what the git log provides.
- **The import does not modify any git history.** It only creates `.aig/` metadata. No commits are rewritten, rebased, or amended.

## After Migration: Using aig Going Forward

Once the import is complete, start using aig sessions for new work:

```bash
# Start working with intents
aig session start "Add payment processing"
aig conversation add "Using Stripe API v3, webhook-based confirmation"

# Work, checkpoint, iterate
aig checkpoint "Stripe webhook handler implemented"
aig checkpoint "Payment confirmation flow working"

# End the session — auto-captures AI conversation
aig session end

# Or run file watching for hands-free checkpointing
aig watch --auto-checkpoint

# Your intent graph grows richer over time
aig log
```

The imported history and your new aig sessions live side by side in the same database. Over time, the proportion of richly annotated intents (with conversations, semantic diffs, and explicit intent declarations) grows as you use aig for day-to-day work.

## Re-importing

If you want to re-import --- for example, after more commits were made by non-aig users:

- Currently, re-running `aig import` will add duplicate intents. There is no deduplication in the MVP.
- **Recommendation:** import once, then use aig sessions going forward.
- Future versions will support incremental import, processing only commits that appeared since the last import.
