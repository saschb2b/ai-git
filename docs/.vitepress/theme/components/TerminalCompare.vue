<template>
  <div class="terminal-compare">
    <div class="terminal before">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">{{ leftTitle }}</div>
      </div>
      <div class="terminal-body">
        <slot name="left" />
      </div>
    </div>
    <div class="terminal after">
      <div class="terminal-header">
        <div class="terminal-dots">
          <span></span><span></span><span></span>
        </div>
        <div class="terminal-title">{{ rightTitle }}</div>
      </div>
      <div class="terminal-body">
        <slot name="right" />
      </div>
    </div>
  </div>
</template>

<script setup>
defineProps({
  leftTitle: { type: String, default: 'Before' },
  rightTitle: { type: String, default: 'After' },
})
</script>
