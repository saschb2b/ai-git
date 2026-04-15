# /aig — LLM-enhanced aig workflow

You are acting as an intelligent wrapper around the `aig` CLI. Parse the user's argument to determine which subcommand to run, then execute it with LLM enhancement.

**User argument:** $ARGUMENTS

## Subcommands

### `checkpoint` (or no argument)
Create an LLM-enhanced checkpoint:

1. Run `aig status` to confirm there's an active session.
2. Run `aig diff --semantic` to get the semantic diff of current changes.
3. Run `git diff --staged` and `git diff` to see the raw changes for additional context.
4. Analyze the semantic diff and raw changes. Write a concise, meaningful checkpoint message that describes **what** changed and **why** it matters — not just file names. Use the format: "verb + what changed" (e.g., "Add rate limiting middleware with per-IP throttling", "Fix double-charge race condition in payment flow").
5. Run `aig checkpoint "<your generated message>"` with your crafted message.
6. Report what you checkpointed.

### `session start <description>`
Start a new aig session:

1. Run `aig status` to check if a session is already active. If so, ask the user if they want to end it first.
2. Run `aig session start "<description>"` with the user's description.
3. Confirm the session was started.

### `session end`
End the current session with a summary:

1. Run `aig status` to confirm there's an active session.
2. Run `aig log` to see the checkpoints in the current session.
3. Summarize what was accomplished in this session.
4. Run `aig session end`.
5. Report the summary.

### `why <file:line>`
Explain why a line of code exists:

1. Run `aig why <file:line>` to get the intent and checkpoint metadata.
2. Read the relevant file around the specified line for context.
3. Run `aig log` to understand the broader intent history.
4. Synthesize a clear, human-readable explanation of **why** this code exists, what problem it solves, and how it fits into the broader intent. Go beyond what `aig why` alone provides by connecting the dots.

### `status`
Show enhanced status:

1. Run `aig status` for the current session state.
2. Run `aig diff --semantic` to show what's changed since last checkpoint.
3. Present a clean summary: current session intent, pending changes, and suggestion for next action.

### `log`
Show the intent history:

1. Run `aig log` and display the output.

## Guidelines

- Always run `aig status` first to understand the current state.
- If no session is active and the user tries to checkpoint, suggest starting a session first.
- Keep checkpoint messages under 80 characters when possible. Use a second line for details if needed.
- When generating checkpoint messages, focus on the semantic meaning, not file paths.
- If `aig` is not initialized in the repo, run `aig init --import` first and inform the user.
