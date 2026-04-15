<template>
  <div class="terminal hero-terminal">
    <div class="terminal-header">
      <div class="terminal-dots">
        <span></span><span></span><span></span>
      </div>
      <div class="terminal-title">~/acme-app</div>
    </div>
    <div class="terminal-body">
      <span class="terminal-line" :style="d(0)">
        <span class="t-muted"># Start a new unit of work</span>
      </span>
      <span class="terminal-line" :style="d(1)">
        <span class="t-purple t-bold">$</span> <span class="t-white">aig session start</span> <span class="t-cyan">"Add API rate limiting"</span>
      </span>
      <span class="terminal-line" :style="d(2)">
        <span class="t-green">Session started</span> <span class="t-yellow">[c83f1a09]</span>
      </span>
      <span class="terminal-line" :style="d(3)">&nbsp;</span>
      <span class="terminal-line" :style="d(4)">
        <span class="t-muted"># You code... then save progress</span>
      </span>
      <span class="terminal-line" :style="d(5)">
        <span class="t-purple t-bold">$</span> <span class="t-white">aig checkpoint</span>
      </span>
      <span class="terminal-line" :style="d(6)">
        <span class="t-dim">  semantic:</span> <span class="t-green">+ added rate_limit</span> <span class="t-dim">(middleware.rs)</span>
      </span>
      <span class="terminal-line" :style="d(7)">
        <span class="t-dim">           </span> <span class="t-green">+ added RateLimiter</span> <span class="t-dim">(middleware.rs)</span>
      </span>
      <span class="terminal-line" :style="d(8)">
        <span class="t-white">Checkpoint:</span> <span class="t-cyan">added rate_limit, added RateLimiter</span>
      </span>
      <span class="terminal-line" :style="d(9)">&nbsp;</span>
      <span class="terminal-line" :style="d(10)">
        <span class="t-muted"># Keep coding... checkpoint again</span>
      </span>
      <span class="terminal-line" :style="d(11)">
        <span class="t-purple t-bold">$</span> <span class="t-white">aig checkpoint</span>
      </span>
      <span class="terminal-line" :style="d(12)">
        <span class="t-dim">  semantic:</span> <span class="t-green">+ added rate_limit_test</span> <span class="t-dim">(tests/api.rs)</span>
      </span>
      <span class="terminal-line" :style="d(13)">
        <span class="t-dim">           </span> <span class="t-yellow">~ modified api_router</span> <span class="t-dim">(main.rs)</span>
      </span>
      <span class="terminal-line" :style="d(14)">
        <span class="t-white">Checkpoint:</span> <span class="t-cyan">added rate_limit_test, modified api_router</span>
      </span>
      <span class="terminal-line" :style="d(15)">&nbsp;</span>
      <span class="terminal-line" :style="d(16)">
        <span class="t-muted"># Done? Wrap up</span>
      </span>
      <span class="terminal-line" :style="d(17)">
        <span class="t-purple t-bold">$</span> <span class="t-white">aig session end</span>
      </span>
      <span class="terminal-line" :style="d(18)">
        <span class="t-green">Session closed</span> <span class="t-yellow">[c83f1a09]</span> <span class="t-white">Add API rate limiting</span>
      </span>
      <span class="terminal-line" :style="d(19)">
        <span class="t-dim">  2 checkpoints, 3 files changed</span>
      </span>
      <span class="terminal-line" :style="d(20)">&nbsp;</span>
      <span class="terminal-line" :style="d(21)">
        <span class="t-muted"># See the clean history</span>
      </span>
      <span class="terminal-line" :style="d(22)">
        <span class="t-purple t-bold">$</span> <span class="t-white">aig log</span>
      </span>
      <span class="terminal-line" :style="d(23)">
        <span class="t-yellow">[c83f1a09]</span> <span class="t-white t-bold">Add API rate limiting</span>
      </span>
      <span class="terminal-line" :style="d(24)">
        <span class="t-dim">  2 checkpoint(s)</span>
      </span>
      <span class="terminal-line" :style="d(25)">
        <span class="t-muted">    (a4f82c1)</span> <span class="t-cyan">added rate_limit, added RateLimiter</span>
      </span>
      <span class="terminal-line" :style="d(26)">
        <span class="t-muted">    (b7d93e2)</span> <span class="t-cyan">added rate_limit_test, modified api_router</span>
      </span>
      <span class="terminal-line terminal-cursor" :style="d(27)"></span>
    </div>
  </div>
</template>

<script setup>
const d = (i) => `--i: ${i}`
</script>

<style scoped>
.hero-terminal {
  width: 100%;
  max-width: 440px;
}

@media (max-width: 960px) {
  .hero-terminal {
    max-width: 100%;
    margin: 1rem auto 0;
  }
}
</style>
