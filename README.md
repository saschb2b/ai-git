# aig — Version Control for the AI Age

Git was built for a world where humans typed every line. That world is ending.

**aig** is a research project exploring what version control should look like when AI is a first-class participant in software development — not an afterthought bolted onto a 2005-era data model.

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

## Status

This is a research/vision document, not working software (yet). Contributions, critiques, and wild ideas welcome.
