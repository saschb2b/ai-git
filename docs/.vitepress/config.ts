import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'aig',
  description: 'Version Control for the AI Age — intent-based, semantically-aware, conversation-preserving',
  base: '/ai-git/',

  head: [
    ['meta', { name: 'theme-color', content: '#646cff' }],
    ['meta', { property: 'og:title', content: 'aig — Version Control for the AI Age' }],
    ['meta', { property: 'og:description', content: 'Intent-based version control that captures why code changes, not just what changed.' }],
  ],

  themeConfig: {
    logo: undefined,
    siteTitle: 'aig',

    nav: [
      { text: 'Guide', link: '/guide/' },
      { text: 'Comparison', link: '/comparison' },
      { text: 'Roadmap', link: '/roadmap' },
      { text: 'Research', link: '/research' },
      {
        text: 'GitHub',
        link: 'https://github.com/saschb2b/ai-git'
      }
    ],

    sidebar: [
      {
        text: 'Introduction',
        items: [
          { text: 'What is aig?', link: '/guide/' },
          { text: 'Git vs aig', link: '/comparison' },
        ]
      },
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' },
          { text: 'Daily Workflow', link: '/guide/daily-workflow' },
          { text: 'Migrating from Git', link: '/guide/migration' },
          { text: 'CLI Reference', link: '/guide/cli-reference' },
        ]
      },
      {
        text: 'Deep Dive',
        items: [
          { text: 'Related Tools', link: '/related-tools' },
          { text: 'Roadmap', link: '/roadmap' },
          { text: 'Research & Vision', link: '/research' },
          { text: 'Tech Stack', link: '/tech-stack' },
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/saschb2b/ai-git' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Built with intent.'
    },

    search: {
      provider: 'local'
    },

    outline: {
      level: [2, 3]
    }
  }
})
