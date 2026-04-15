# @aig/tui

Interactive terminal UI for reviewing aig intents. Built with [React](https://react.dev/) and [Ink](https://github.com/vadimdemedes/ink).

## What it does

`aig review --tui` opens a split-pane interface in your terminal:

- **Left panel** — navigable list of intents (scrolls automatically for large histories)
- **Right panel** — details for the selected intent: checkpoints, semantic changes, trust/provenance, and conversation notes

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `Down` | Next intent |
| `k` / `Up` | Previous intent |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `q` | Quit |

## How it works

The TUI reads `.aig/aig.db` directly via [better-sqlite3](https://github.com/WiseLibs/better-sqlite3) in read-only mode. It does not modify any data. The Rust CLI (`aig review --tui`) spawns this as a Node.js child process.

## Development

```bash
# From the repo root
pnpm install
pnpm run dev:tui     # watch mode

# Or build once
pnpm --filter @aig/tui build
```

## Architecture

```
src/
  index.tsx   Entry point + React components (App, IntentList, DetailPanel)
  db.ts       Data layer — reads the .aig SQLite database
```

The TUI is a workspace package within the aig monorepo. It requires Node.js >= 18 at runtime.
