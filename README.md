# aig — Version Control for the AI Age

Git was built for a world where humans typed every line. That world is ending.

**aig** is a research project exploring what version control should look like when AI is a first-class participant in software development — not an afterthought bolted onto a 2005-era data model.

## Installation

### From source (recommended)
```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
```

### Build locally
```bash
git clone https://github.com/saschb2b/ai-git.git
cd ai-git
cargo build --release
# Binary at target/release/aig (.exe on Windows)
```

### Pre-built binaries
Download from [GitHub Releases](https://github.com/saschb2b/ai-git/releases) — builds are available for Linux (x86_64), macOS (aarch64), and Windows (x86_64).

## The Core Idea

Replace git's line-based diffs and manual commits with:

- **Intent as the primary unit** — declare what you want to accomplish *before* writing code
- **Semantic change tracking** — understand changes at the structural level, not the text level
- **Conversations as history** — preserve the human-AI reasoning that produced the code
- **Continuous versioning** — stop worrying about when to commit; crystallize checkpoints when they matter
- **Impact-first review** — make AI-generated changes reviewable by design, not by heroic effort
- **`aig why`** — trace any line back to the intent, conversation, and alternatives that produced it

## Read the Research

The full vision is laid out in **[RESEARCH.md](RESEARCH.md)** — a ~5,000-word document covering:

1. Why git breaks down with AI (commit granularity, diffs, authorship, knowledge loss)
2. Intent-based version control as an alternative
3. How to regain human transparency and ownership
4. Collaboration models for humans, AIs, and teams
5. A technical architecture sketch with example CLI commands

## Repository Structure

```
ai-git/
├── crates/
│   ├── aig-core/            Rust CLI binary + library (21 commands)
│   └── aig-treesitter/      Rust library: semantic diff engine (11 languages)
├── packages/
│   ├── aig-llm/             TypeScript: LLM integration (intent inference, explanations)
│   └── aig-tui/             TypeScript: interactive terminal UI (React/Ink)
├── docs/                    VitePress documentation site
├── scripts/                 Build and doc generation scripts
├── RESEARCH.md              Vision document (~5,000 words)
└── TECH_STACK.md            Technology choices and rationale
```

The Rust binary (`aig`) is self-contained for most commands. Two optional features require Node.js:
- **LLM features** (`aig import` with inference, `aig why --explain`) — calls `@aig/llm` via IPC
- **TUI** (`aig review --tui`) — launches `@aig/tui` as a child process

## Status

aig is working software — 21 commands, semantic diff for 11 languages, trust scoring, LLM-powered explanations, interactive TUI review, git hooks for zero-friction tracking, and remote sync via git notes. Built in Rust + TypeScript, runs on Linux, macOS, and Windows. See the [Getting Started](https://saschb2b.github.io/ai-git/guide/getting-started) guide or the [Roadmap](https://saschb2b.github.io/ai-git/roadmap) for what's next.
