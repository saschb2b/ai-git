---
outline: deep
---

# CLI Reference

::: info Auto-generated
This page is generated from the actual `aig` binary. Run `./scripts/generate-docs.sh` to regenerate.
:::

## Commands

```
AI-native version control for intent-driven development

Usage: aig.exe <COMMAND>

Commands:
  init          Initialize a new .aig directory in the current repo
  session       Manage development sessions
  checkpoint    Create a checkpoint with a message
  status        Show current aig status
  log           Show intent-level history
  diff          Show changes since last checkpoint
  why           Explain why a line/region was changed
  import        Import existing git history into aig
  conversation  Manage conversation records
  watch         Watch for file changes and auto-checkpoint
  capture       Capture the current Claude Code conversation into the active session
  help          Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig init`

```
Initialize a new .aig directory in the current repo

Usage: aig.exe init

Options:
  -h, --help  Print help
```

### `aig session`

```
Manage development sessions

Usage: aig.exe session <COMMAND>

Commands:
  start  Start a new session with an intent
  end    End the current session
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig checkpoint`

```
Create a checkpoint with a message

Usage: aig.exe checkpoint <MESSAGE>

Arguments:
  <MESSAGE>  Checkpoint message

Options:
  -h, --help  Print help
```

### `aig status`

```
Show current aig status

Usage: aig.exe status

Options:
  -h, --help  Print help
```

### `aig log`

```
Show intent-level history

Usage: aig.exe log

Options:
  -h, --help  Print help
```

### `aig diff`

```
Show changes since last checkpoint

Usage: aig.exe diff [OPTIONS]

Options:
      --semantic  Use semantic (tree-sitter) diff instead of line diff
  -h, --help      Print help
```

### `aig why`

```
Explain why a line/region was changed

Usage: aig.exe why <LOCATION>

Arguments:
  <LOCATION>  Location in the form "src/main.rs:42"

Options:
  -h, --help  Print help
```

### `aig import`

```
Import existing git history into aig

Usage: aig.exe import

Options:
  -h, --help  Print help
```

### `aig conversation`

```
Manage conversation records

Usage: aig.exe conversation <COMMAND>

Commands:
  add   Add a conversation message to the current session
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig session start`

```
Start a new session with an intent

Usage: aig.exe session start <INTENT>

Arguments:
  <INTENT>  Description of the development intent

Options:
  -h, --help  Print help
```

### `aig session end`

```
End the current session

Usage: aig.exe session end

Options:
  -h, --help  Print help
```

### `aig conversation add`

```
Add a conversation message to the current session

Usage: aig.exe conversation add <MESSAGE>

Arguments:
  <MESSAGE>  The message content

Options:
  -h, --help  Print help
```

## Supported Languages (Semantic Diff)

| Language | Extensions | Definition Types Tracked |
|---|---|---|
| TypeScript / JavaScript | `.ts`, `.tsx` | functions, classes, interfaces, type aliases, methods |
| Python | `.py` | functions, classes |
| Rust | `.rs` | functions, structs, enums, impls, traits, types |
| Go | `.go` | functions, methods, types |

All other file types fall back to line-based diffing automatically.
