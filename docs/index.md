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
    details: "Claude Code conversations are saved into your version history automatically. The reasoning behind AI-generated code is never lost."
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
