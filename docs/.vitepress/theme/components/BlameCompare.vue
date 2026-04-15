<template>
  <div class="terminal-compare">
    <div class="terminal before">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">git blame payment.rs</div>
      </div>
      <div class="terminal-body">
<pre><span class="t-dim">9c0f3d2</span> (<span class="t-muted">Sarah Chen 2026-01-15</span>) <span class="t-white">let key = IdempotencyKey::new(user_id, cart_id);</span>

<span class="t-muted">Who wrote it and when.</span>
<span class="t-muted">But why? The commit message says "wip".</span></pre>
      </div>
    </div>
    <div class="terminal after">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">aig why payment.rs:42</div>
      </div>
      <div class="terminal-body">
<pre><span class="t-muted">Intent:</span>     <span class="t-yellow">[a1f8c932]</span> <span class="t-white">Fix payment race condition</span>
<span class="t-muted">Checkpoint:</span> <span class="t-cyan">Add idempotency key to Stripe call</span>
<span class="t-muted">Commit:</span>     <span class="t-dim">9c0f3d2</span>
<span class="t-muted">Semantic:</span>   <span class="t-green">+ added IdempotencyKey::new</span> <span class="t-dim">(payment.rs)</span>
<span class="t-muted">AI context:</span> <span class="t-dim">Claude suggested idempotency keys</span>
            <span class="t-dim">after analyzing the Stripe webhook logs</span>
            <span class="t-dim">showing duplicate charge events.</span>

<span class="t-green t-bold">The intent, the reasoning, the full story.</span></pre>
      </div>
    </div>
  </div>
</template>
