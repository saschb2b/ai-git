<template>
  <div style="max-width: 600px; margin: 0 auto;">
    <div class="terminal">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">aig checkpoint</div>
      </div>
      <div class="terminal-body">
<pre><span class="t-purple t-bold">$</span> <span class="t-white">aig checkpoint</span>

  <span class="t-muted">auto-message:</span> <span class="t-cyan">added rate_limit middleware, modified api router, added RateLimiter</span>
  <span class="t-muted">semantic:</span>
    <span class="t-green">+ added rate_limit</span> <span class="t-dim">(middleware.rs)</span>
    <span class="t-green">+ added RateLimiter</span> <span class="t-dim">(middleware.rs)</span>
    <span class="t-yellow">~ modified api_router</span> <span class="t-dim">(main.rs)</span>
    <span class="t-green">+ added rate_limit_test</span> <span class="t-dim">(tests/api.rs)</span>
<span class="t-white">Checkpoint created</span>
  <span class="t-muted">intent:</span>     <span class="t-yellow">Add API rate limiting</span>
  <span class="t-muted">git commit:</span> <span class="t-dim">c83f1a09</span></pre>
      </div>
    </div>
  </div>
</template>
