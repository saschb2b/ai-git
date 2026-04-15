<template>
  <div class="terminal-compare">
    <div class="terminal before">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">git log</div>
      </div>
      <div class="terminal-body">
<pre><span class="t-dim">e4b21a7</span> Add mutex
<span class="t-dim">9c0f3d2</span> wip
<span class="t-dim">7a1bc85</span> fix test
<span class="t-dim">f82ca91</span> Add google oauth
<span class="t-dim">3d6e0f4</span> rotate tokens
<span class="t-dim">a90c1b3</span> fix lint
<span class="t-dim">c44df87</span> WIP oauth
<span class="t-dim">1e8b6a0</span> refactor auth middleware
<span class="t-dim">bb712f5</span> Add postgres migration
<span class="t-dim">6d0ea93</span> fix connection pool
<span class="t-dim">f1c4e72</span> Update user model
<span class="t-dim">8823a1d</span> remove sqlite refs
<span class="t-dim">02efc9a</span> fix tests
<span class="t-dim">d4917cb</span> cleanup
<span class="t-muted">...</span>

<span class="t-muted">47 commits. Which ones matter?</span></pre>
      </div>
    </div>
    <div class="terminal after">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">aig log</div>
      </div>
      <div class="terminal-body">
<pre><span class="t-yellow">[a1f8c932]</span> <span class="t-white t-bold">Fix payment race condition</span> <span class="t-green">(active)</span>
<span class="t-dim">  3 checkpoint(s)</span>
    <span class="t-muted">(e4b21a7)</span> <span class="t-cyan">Add mutex lock on charge_customer</span>
    <span class="t-muted">(9c0f3d2)</span> <span class="t-cyan">Add idempotency key to Stripe call</span>
    <span class="t-muted">(7a1bc85)</span> <span class="t-cyan">Add regression test for double-charge</span>

<span class="t-yellow">[d47e0b15]</span> <span class="t-white t-bold">Add OAuth2 login flow</span>
<span class="t-dim">  5 checkpoint(s)</span>
    <span class="t-muted">(f82ca91)</span> <span class="t-cyan">Add Google OAuth provider</span>
    <span class="t-muted">(3d6e0f4)</span> <span class="t-cyan">Add session token rotation</span>
    <span class="t-dim">...</span>

<span class="t-yellow">[8b3f21c7]</span> <span class="t-white t-bold">Migrate user store to PostgreSQL</span>
<span class="t-dim">  8 checkpoint(s)</span>
    <span class="t-dim">...</span>

<span class="t-green t-bold">3 intents. The story of what actually happened.</span></pre>
      </div>
    </div>
  </div>
</template>
