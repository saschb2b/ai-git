<template>
  <div class="cta-section">
    <div>
      <h2 class="cta-title">Try it in 60 seconds</h2>
      <p class="cta-subtitle">Your git history stays untouched. Zero risk.</p>

      <div class="cta-terminal">
        <div class="terminal">
          <div class="terminal-header">
            <div class="terminal-dots">
              <span></span><span></span><span></span>
            </div>
            <div class="terminal-title">terminal</div>
          </div>
          <div class="terminal-body">
            <pre><span class="t-muted"># Install aig</span>
<span class="t-purple t-bold">$</span> <span class="t-white">curl -fsSL https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.sh | sh</span>

<span class="t-muted"># Try it on your repo</span>
<span class="t-purple t-bold">$</span> <span class="t-white">cd</span> <span class="t-cyan">your-repo</span>
<span class="t-purple t-bold">$</span> <span class="t-white">aig import</span>
<span class="t-purple t-bold">$</span> <span class="t-white">aig log</span></pre>
          </div>
        </div>
      </div>

      <div class="cta-buttons">
        <a href="/ai-git/guide/getting-started" class="cta-button primary">
          Get Started
        </a>
        <a href="https://github.com/saschb2b/ai-git" class="cta-button secondary" target="_blank" rel="noopener">
          View on GitHub
        </a>
      </div>
    </div>
  </div>
</template>
