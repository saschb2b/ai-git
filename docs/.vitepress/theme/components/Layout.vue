<script setup>
import DefaultTheme from 'vitepress/theme'
import HeroTerminal from './HeroTerminal.vue'

const { Layout } = DefaultTheme
</script>

<template>
  <Layout>
    <template #home-hero-image>
      <HeroTerminal />
    </template>
  </Layout>
</template>
