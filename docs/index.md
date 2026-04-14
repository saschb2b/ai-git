---
layout: home
hero:
  name: "aig"
  text: "Version Control for the AI Age"
  tagline: "Ever run git blame and found a useless commit message from 6 months ago? aig captures the intent, reasoning, and AI conversation behind every change — so you always know why code exists."
  actions:
    - theme: brand
      text: Try It in 60 Seconds
      link: /guide/getting-started#try-it-on-your-existing-repo
    - theme: alt
      text: Full Guide
      link: /guide/getting-started
    - theme: alt
      text: Why not Git?
      link: /comparison
features:
  - icon: "\U0001F50D"
    title: "git blame tells you WHO. aig why tells you WHY."
    details: "Trace any line back to the intent that created it, the semantic changes around it, and the conversation that shaped it."
  - icon: "\U0001F333"
    title: Semantic Diffs, Not Line Noise
    details: "See 'function authenticate() added' instead of 300 lines of red and green. Supports TypeScript, Python, Rust, Go, Java, C#, C++, and Ruby."
  - icon: "\U0001F4AC"
    title: AI Conversations Auto-Captured
    details: "AI conversations are auto-captured into your version history. Supports Claude Code out of the box, with a generic import format for any AI tool."
  - icon: "\U0001F4E6"
    title: Works With Your Existing Repo
    details: "Run 'aig import' in any git repo to build an intent graph from your commit history. Non-destructive — your git history is untouched."
  - icon: "\U0001F91D"
    title: Fully Git Compatible
    details: "aig layers on top of git. Push, pull, branch, merge — everything works. Your repo stays a valid git repo. Teammates who don't use aig are unaffected."
  - icon: "\U0001F680"
    title: Share Context Across Clones
    details: "aig push/pull syncs intent history via git notes. Clone a repo, run 'aig pull', and the full reasoning behind every change is there."
---

<div class="vp-doc" style="max-width: 800px; margin: 0 auto; padding: 2rem 1.5rem;">

## This Is What You'll See

Real output from aig running on its own repository.

### git log vs aig log

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
<div>

**git log**
```
43ed456 Initial commit
fd990ed Add aig MVP
d74a8cd Fix GitHub Pages CI
6b42cef Add CLI reference
7564dca Complete core loop
6bdca12 Add conversation capture
4c9fb74 Add CI pipeline
4bcf69e Fix CLI reference
d57407e Update all docs
00cd0ef Add roadmap page
7975e6d Add remote sync
9ee934f Add 4 languages
...
```
19 commits. A flat list. No narrative.

</div>
<div>

**aig log**
```
[44d3ab98] Rewrite docs (active)
  7 checkpoint(s)
    (120ab1e) Rewrite docs
    (3e8142a) Add Daily Workflow
    (ed3a9c5) Add aig repair
    (635eb92) Add Related Tools
    ...

[479d2692] Initial commit (active)
  12 checkpoint(s)
    (fd990ed) Add aig MVP
    (7564dca) Complete core loop
    (6bdca12) Add conversation capture
    (7975e6d) Add remote sync
    ...
```
2 intents. The story of how this project was built.

</div>
</div>

### git blame vs aig why

<div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem;">
<div>

**git blame capture.rs**
```
6bdca12 (Sascha Becker 2026-04-14)
```
A name and a date.

</div>
<div>

**aig why capture.rs:1**
```
Intent:     [479d2692] Initial commit
Checkpoint: Add conversation capture,
            file watching, cargo install
Commit:     6bdca120
Time:       2026-04-14T01:06:24+00:00
```
The intent, the checkpoint, the full context.

</div>
</div>

### aig checkpoint (no message needed)

```
$ aig checkpoint

  auto-message: added generate_token, added validate_token, added AuthMiddleware
  semantic:
    + added generate_token (auth.py)
    + added validate_token (auth.py)
    + added AuthMiddleware (auth.py)
Checkpoint created
  intent:     Add authentication
  git commit: 8d5b5ff9
```

The code describes its own changes. You just type `aig checkpoint`.

### Install and try it

```bash
cargo install --git https://github.com/saschb2b/ai-git.git aig-core
cd your-repo
aig import
aig log
```

Four commands. Your git history is untouched. [Get started](/guide/getting-started).

</div>
