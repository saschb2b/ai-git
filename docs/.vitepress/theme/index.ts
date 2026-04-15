import DefaultTheme from 'vitepress/theme'
import type { Theme } from 'vitepress'
import './custom.css'
import Layout from './components/Layout.vue'
import TrustBar from './components/TrustBar.vue'
import LogCompare from './components/LogCompare.vue'
import BlameCompare from './components/BlameCompare.vue'
import CheckpointDemo from './components/CheckpointDemo.vue'
import CtaInstall from './components/CtaInstall.vue'

export default {
  extends: DefaultTheme,
  Layout,
  enhanceApp({ app }) {
    app.component('TrustBar', TrustBar)
    app.component('LogCompare', LogCompare)
    app.component('BlameCompare', BlameCompare)
    app.component('CheckpointDemo', CheckpointDemo)
    app.component('CtaInstall', CtaInstall)
  }
} satisfies Theme
