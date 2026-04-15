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

Usage: aig <COMMAND>

Commands:
  init           Initialize a new .aig directory in the current repo
  session        Manage development sessions
  checkpoint     Create a checkpoint (auto-generates message from semantic diff if omitted)
  status         Show current aig status
  log            Show intent-level history
  diff           Show changes since last checkpoint
  why            Explain why a line/region was changed
  import         Import existing git history into aig
  conversation   Manage conversation records
  watch          Watch for file changes and auto-checkpoint
  capture        Capture AI conversation into the active session
  push           Push aig metadata to remote via git notes
  pull           Pull aig metadata from remote via git notes
  review         Review an intent — show summary, semantic changes, and conversation
  repair         Repair aig metadata after rebase (re-attaches orphaned notes)
  export         Export all .aig metadata to a portable bundle file
  import-bundle  Import .aig metadata from a bundle file
  hooks          Install or remove git hooks for automatic aig tracking
  trust          Show trust and provenance information for files
  reviewed       Mark a file or intent as human-reviewed
  release        Create a release from completed intents
  changelog      Generate a changelog from intent history
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig init`

```
Initialize a new .aig directory in the current repo

Usage: aig init [OPTIONS]

Options:
      --import  Also run aig import after initialization
  -h, --help    Print help
```

### `aig session`

```
Manage development sessions

Usage: aig session <COMMAND>

Commands:
  start  Start a new session with an intent
  end    End the current session
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig checkpoint`

```
Create a checkpoint (auto-generates message from semantic diff if omitted)

Usage: aig checkpoint [MESSAGE]

Arguments:
  [MESSAGE]  Checkpoint message (optional — auto-generated from changes if omitted)

Options:
  -h, --help  Print help
```

### `aig status`

```
Show current aig status

Usage: aig status

Options:
  -h, --help  Print help
```

### `aig log`

```
Show intent-level history

Usage: aig log

Options:
  -h, --help  Print help
```

### `aig diff`

```
Show changes since last checkpoint

Usage: aig diff [OPTIONS]

Options:
      --semantic  Use semantic (tree-sitter) diff instead of line diff
  -h, --help      Print help
```

### `aig why`

```
Explain why a line/region was changed

Usage: aig why [OPTIONS] <LOCATION>

Arguments:
  <LOCATION>  Location in the form "src/main.rs:42"

Options:
      --explain  Use LLM to synthesize a natural-language explanation
  -h, --help     Print help
```

### `aig import`

```
Import existing git history into aig

Usage: aig import

Options:
  -h, --help  Print help
```

### `aig conversation`

```
Manage conversation records

Usage: aig conversation <COMMAND>

Commands:
  add   Add a conversation message to the current session
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig watch`

```
Watch for file changes and auto-checkpoint

Usage: aig watch [OPTIONS]

Options:
      --auto-checkpoint  Automatically create checkpoints after quiet periods
  -h, --help             Print help
```

### `aig capture`

```
Capture AI conversation into the active session

Usage: aig capture [OPTIONS]

Options:
      --source <SOURCE>  Source to capture from: auto (default), claude-code, or a file path [default: auto]
      --file <FILE>      Import conversation from a file (JSONL with role/content per line)
  -h, --help             Print help
```

### `aig push`

```
Push aig metadata to remote via git notes

Usage: aig push [REMOTE]

Arguments:
  [REMOTE]  Remote name (default: origin) [default: origin]

Options:
  -h, --help  Print help
```

### `aig pull`

```
Pull aig metadata from remote via git notes

Usage: aig pull [REMOTE]

Arguments:
  [REMOTE]  Remote name (default: origin) [default: origin]

Options:
  -h, --help  Print help
```

### `aig review`

```
Review an intent — show summary, semantic changes, and conversation

Usage: aig review [OPTIONS] [INTENT_ID]

Arguments:
  [INTENT_ID]  Intent ID (first 8 chars). Omit to review the most recent intent

Options:
      --tui   Open interactive terminal UI
  -h, --help  Print help
```

### `aig repair`

```
Repair aig metadata after rebase (re-attaches orphaned notes)

Usage: aig repair

Options:
  -h, --help  Print help
```

### `aig export`

```
Export all .aig metadata to a portable bundle file

Usage: aig export [OUTPUT]

Arguments:
  [OUTPUT]  Output file path (default: aig-bundle.tar.gz) [default: aig-bundle.tar.gz]

Options:
  -h, --help  Print help
```

### `aig import-bundle`

```
Import .aig metadata from a bundle file

Usage: aig import-bundle [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the .aig-bundle.tar.gz file

Options:
      --force  Overwrite existing .aig directory if present
  -h, --help   Print help
```

### `aig hooks`

```
Install or remove git hooks for automatic aig tracking

Usage: aig hooks <COMMAND>

Commands:
  install  Install git hooks for automatic aig tracking
  remove   Remove aig git hooks
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `aig trust`

```
Show trust and provenance information for files

Usage: aig trust [FILE]

Arguments:
  [FILE]  File path to inspect (omit for project-wide summary)

Options:
  -h, --help  Print help
```

### `aig reviewed`

```
Mark a file or intent as human-reviewed

Usage: aig reviewed <TARGET>

Arguments:
  <TARGET>  File path or intent ID to mark as reviewed

Options:
  -h, --help  Print help
```

### `aig release`

```
Create a release from completed intents

Usage: aig release [OPTIONS] <TAG>

Arguments:
  <TAG>  Tag name (e.g., v0.2.0)

Options:
      --title <TITLE>  Release title (defaults to tag name)
  -h, --help           Print help
```

### `aig changelog`

```
Generate a changelog from intent history

Usage: aig changelog [RANGE]

Arguments:
  [RANGE]  Range in the form "v0.1.0..v0.2.0" (omit for latest release)

Options:
  -h, --help  Print help
```

### `aig session start`

```
Start a new session with an intent

Usage: aig session start <INTENT>

Arguments:
  <INTENT>  Description of the development intent

Options:
  -h, --help  Print help
```

### `aig session end`

```
End the current session

Usage: aig session end

Options:
  -h, --help  Print help
```

### `aig conversation add`

```
Add a conversation message to the current session

Usage: aig conversation add <MESSAGE>

Arguments:
  <MESSAGE>  The message content

Options:
  -h, --help  Print help
```

### `aig hooks install`

```
Install git hooks for automatic aig tracking

Usage: aig hooks install

Options:
  -h, --help  Print help
```

### `aig hooks remove`

```
Remove aig git hooks

Usage: aig hooks remove

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
| Java | `.java` | classes, interfaces, methods, constructors, enums |
| C# | `.cs` | classes, interfaces, methods, structs, enums, constructors |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.h` | functions, classes, structs, enums, templates |
| Ruby | `.rb` | methods, classes, modules |
| PHP | `.php` | functions, methods, classes, interfaces, traits, enums |
| Kotlin | `.kt`, `.kts` | functions, classes, objects |
| Swift | `.swift` | functions, classes, protocols, type aliases |

All other file types fall back to line-based diffing automatically.
