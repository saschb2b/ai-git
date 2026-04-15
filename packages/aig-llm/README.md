# @aig/llm

LLM integration layer for aig. Provides intent inference, summary generation, and natural-language explanations via the [Anthropic Claude API](https://docs.anthropic.com/).

## What it does

This package is called by the Rust CLI as a child process over NDJSON (newline-delimited JSON) on stdin/stdout. It handles three commands:

| Command | Used by | Purpose |
|---------|---------|---------|
| `infer_intent` | `aig import` | Infer developer intent from git commit messages and diff stats |
| `generate_summary` | `aig session end` | Summarize code changes into a concise paragraph |
| `explain_line` | `aig why --explain` | Synthesize a natural-language explanation for why a line exists |

## Setup

Requires an `ANTHROPIC_API_KEY` environment variable.

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

## Architecture

```
src/
  index.ts              Package exports
  ipc.ts                NDJSON IPC server — reads commands from stdin, writes responses to stdout
  import.ts             Commit clustering heuristics for aig import
  providers/
    types.ts            LLMProvider interface and shared types
    anthropic.ts        Anthropic Claude implementation (default: claude-sonnet-4-20250514)
```

### IPC protocol

The Rust binary spawns `node dist/index.js` and communicates via newline-delimited JSON:

**Request** (Rust -> Node):
```json
{"command": "infer_intent", "params": {"commit_messages": [...], "diff_stats": [...]}}
```

**Response** (Node -> Rust):
```json
{"id": "...", "result": {"intent": "feature", "summary": "Add JWT authentication"}}
```

### Adding a new provider

Implement the `LLMProvider` interface from `providers/types.ts`:

```typescript
interface LLMProvider {
  inferIntent(commitMessages: string[], diffStats: string[]): Promise<IntentInference>;
  generateSummary(changes: string[]): Promise<string>;
  explainLine(context: ExplainLineContext): Promise<string>;
}
```

## Development

```bash
# From the repo root
pnpm install
pnpm --filter @aig/llm build    # build once
pnpm --filter @aig/llm dev      # watch mode
pnpm --filter @aig/llm test     # run tests
```
