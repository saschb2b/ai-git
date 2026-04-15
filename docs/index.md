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
  - icon:
      src: /icons/search.svg
      width: 40
      height: 40
    title: "git blame tells you WHO. aig why tells you WHY."
    details: "Trace any line back to the intent that created it, the semantic changes around it, and the conversation that shaped it."
  - icon:
      src: /icons/tree.svg
      width: 40
      height: 40
    title: Semantic Diffs, Not Line Noise
    details: "See 'function authenticate() added' instead of 300 lines of red and green. Supports TypeScript, Python, Rust, Go, Java, C#, C++, and Ruby."
  - icon:
      src: /icons/chat.svg
      width: 40
      height: 40
    title: AI Conversations Auto-Captured
    details: "AI conversations are auto-captured into your version history. Supports Claude Code out of the box, with a generic import format for any AI tool."
  - icon:
      src: /icons/package.svg
      width: 40
      height: 40
    title: Works With Your Existing Repo
    details: "Run 'aig import' in any git repo to build an intent graph from your commit history. Non-destructive — your git history is untouched."
  - icon:
      src: /icons/git-merge.svg
      width: 40
      height: 40
    title: Fully Git Compatible
    details: "aig layers on top of git. Push, pull, branch, merge — everything works. Your repo stays a valid git repo. Teammates who don't use aig are unaffected."
  - icon:
      src: /icons/rocket.svg
      width: 40
      height: 40
    title: Share Context Across Clones
    details: "aig push/pull syncs intent history via git notes. Clone a repo, run 'aig pull', and the full reasoning behind every change is there."
---

<TrustBar />

<div class="showcase">
<div class="showcase-inner">

<div class="showcase-block">
<h2 class="showcase-heading">Your git log is a wall of noise</h2>
<p class="showcase-sub">47 commits. Half of them say "wip". aig groups them into intents — the actual units of work.</p>
<LogCompare />
</div>

<div class="showcase-block">
<h2 class="showcase-heading">git blame answers the wrong question</h2>
<p class="showcase-sub">You don't need to know who wrote line 42. You need to know why it exists.</p>
<BlameCompare />
</div>

<div class="showcase-block">
<h2 class="showcase-heading">Stop writing commit messages</h2>
<p class="showcase-sub">aig reads your code changes and describes them for you. Just type <code>aig checkpoint</code>.</p>
<CheckpointDemo />
</div>

</div>
</div>

<CtaInstall />
