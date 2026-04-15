<template>
  <div class="terminal hero-terminal">
    <div class="terminal-header">
      <div class="terminal-dots">
        <span></span><span></span><span></span>
      </div>
      <div class="terminal-title">aig log</div>
    </div>
    <div class="terminal-body">
      <span class="terminal-line" :style="d(0)">
        <span class="t-purple t-bold">$ aig log</span>
      </span>
      <span class="terminal-line" :style="d(1)">&nbsp;</span>
      <span class="terminal-line" :style="d(2)">
        <span class="t-yellow">[a1f8c932]</span> <span class="t-white t-bold">Fix payment race condition</span> <span class="t-green">(active)</span>
      </span>
      <span class="terminal-line" :style="d(3)">
        <span class="t-dim">  3 checkpoint(s)</span>
      </span>
      <span class="terminal-line" :style="d(4)">
        <span class="t-muted">    (e4b21a7)</span> <span class="t-cyan">Add mutex lock on charge_customer</span>
      </span>
      <span class="terminal-line" :style="d(5)">
        <span class="t-muted">    (9c0f3d2)</span> <span class="t-cyan">Add idempotency key to Stripe call</span>
      </span>
      <span class="terminal-line" :style="d(6)">
        <span class="t-muted">    (7a1bc85)</span> <span class="t-cyan">Add regression test for double-charge</span>
      </span>
      <span class="terminal-line" :style="d(7)">&nbsp;</span>
      <span class="terminal-line" :style="d(8)">
        <span class="t-yellow">[d47e0b15]</span> <span class="t-white t-bold">Add OAuth2 login flow</span>
      </span>
      <span class="terminal-line" :style="d(9)">
        <span class="t-dim">  5 checkpoint(s)</span>
      </span>
      <span class="terminal-line" :style="d(10)">
        <span class="t-muted">    (f82ca91)</span> <span class="t-cyan">Add Google OAuth provider</span>
      </span>
      <span class="terminal-line" :style="d(11)">
        <span class="t-muted">    (3d6e0f4)</span> <span class="t-cyan">Add session token rotation</span>
      </span>
      <span class="terminal-line" :style="d(12)">
        <span class="t-dim">    ...</span>
      </span>
      <span class="terminal-line" :style="d(13)">&nbsp;</span>
      <span class="terminal-line" :style="d(14)">
        <span class="t-yellow">[8b3f21c7]</span> <span class="t-white t-bold">Migrate user store to PostgreSQL</span>
      </span>
      <span class="terminal-line" :style="d(15)">
        <span class="t-dim">  8 checkpoint(s)</span>
      </span>
      <span class="terminal-line" :style="d(16)">
        <span class="t-dim">    ...</span>
      </span>
      <span class="terminal-line terminal-cursor" :style="d(17)"></span>
    </div>
  </div>
</template>

<script setup>
const d = (i) => `--i: ${i}`
</script>

<style scoped>
.hero-terminal {
  width: 100%;
  max-width: 400px;
}

@media (max-width: 960px) {
  .hero-terminal {
    max-width: 100%;
    margin: 1rem auto 0;
  }
}
</style>
