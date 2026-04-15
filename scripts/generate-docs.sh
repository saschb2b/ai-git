#!/usr/bin/env bash
# Generate documentation from the actual aig binary and source code.
# Run this after changing CLI commands, flags, or supported languages.
#
# Usage:
#   ./scripts/generate-docs.sh          # regenerate docs
#   ./scripts/generate-docs.sh --check  # check if docs are up to date (for CI)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
AIG="$REPO_ROOT/target/release/aig"
GENERATED="$REPO_ROOT/docs/guide/cli-reference.md"

CHECK_MODE=false
if [[ "${1:-}" == "--check" ]]; then
  CHECK_MODE=true
fi

# Build the binary if needed
if [[ ! -f "$AIG" ]]; then
  echo "Building aig (release)..."
  cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml" 2>/dev/null
fi

# --- Generate CLI reference ---

generate_cli_reference() {
  cat << 'HEADER'
---
outline: deep
---

# CLI Reference

::: info Auto-generated
This page is generated from the actual `aig` binary. Run `./scripts/generate-docs.sh` to regenerate.
:::

## Commands

HEADER

  # Main help (normalize binary name: aig.exe -> aig)
  echo '```'
  "$AIG" --help | sed 's/aig\.exe/aig/g'
  echo '```'
  echo ""

  # Each subcommand's help
  for cmd in init session checkpoint status log diff why import conversation watch capture push pull review repair export import-bundle hooks trust reviewed release changelog; do
    echo "### \`aig $cmd\`"
    echo ""
    echo '```'
    "$AIG" "$cmd" --help 2>&1 | sed 's/aig\.exe/aig/g' || true
    echo '```'
    echo ""
  done

  # Session subcommands
  for sub in start end; do
    echo "### \`aig session $sub\`"
    echo ""
    echo '```'
    "$AIG" session "$sub" --help 2>&1 | sed 's/aig\.exe/aig/g' || true
    echo '```'
    echo ""
  done

  # Conversation subcommands
  echo "### \`aig conversation add\`"
  echo ""
  echo '```'
  "$AIG" conversation add --help 2>&1 | sed 's/aig\.exe/aig/g' || true
  echo '```'
  echo ""

  # Hooks subcommands
  for sub in install remove; do
    echo "### \`aig hooks $sub\`"
    echo ""
    echo '```'
    "$AIG" hooks "$sub" --help 2>&1 | sed 's/aig\.exe/aig/g' || true
    echo '```'
    echo ""
  done

  # Supported languages for semantic diff
  cat << 'LANGHEADER'
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
LANGHEADER
}

CONTENT="$(generate_cli_reference)"

if $CHECK_MODE; then
  if [[ ! -f "$GENERATED" ]]; then
    echo "FAIL: $GENERATED does not exist. Run ./scripts/generate-docs.sh to generate it."
    exit 1
  fi

  EXISTING="$(cat "$GENERATED")"
  if [[ "$CONTENT" != "$EXISTING" ]]; then
    echo "FAIL: CLI reference is out of date."
    echo "Run ./scripts/generate-docs.sh to regenerate."
    diff <(echo "$EXISTING") <(echo "$CONTENT") || true
    exit 1
  fi

  echo "OK: CLI reference is up to date."
  exit 0
fi

echo "$CONTENT" > "$GENERATED"
echo "Generated: $GENERATED"
