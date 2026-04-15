import { defineConfig } from 'vitepress'

const siteUrl = 'https://saschb2b.github.io/ai-git'
const ogImage = `${siteUrl}/og-image.png`

export default defineConfig({
  title: 'aig',
  description: 'Intent-based version control for AI-assisted development. Captures why code changes, not just what changed. Semantic diffs, conversation history, and git compatibility.',
  base: '/ai-git/',
  lang: 'en-US',
  lastUpdated: true,
  cleanUrls: true,

  sitemap: {
    hostname: 'https://saschb2b.github.io/ai-git',
  },

  head: [
    // Charset and viewport
    ['meta', { charset: 'utf-8' }],
    ['meta', { name: 'viewport', content: 'width=device-width, initial-scale=1' }],

    // Primary meta
    ['meta', { name: 'theme-color', content: '#646cff' }],
    ['meta', { name: 'author', content: 'Sascha Becker' }],
    ['meta', { name: 'keywords', content: 'aig, version control, git alternative, AI coding, semantic diff, intent tracking, Claude Code, developer tools, Rust, open source' }],

    // Open Graph
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'aig' }],
    ['meta', { property: 'og:title', content: 'aig - Version Control for the AI Age' }],
    ['meta', { property: 'og:description', content: 'Intent-based version control that captures why code changes, not just what changed. Semantic diffs, AI conversation capture, and full git compatibility.' }],
    ['meta', { property: 'og:image', content: ogImage }],
    ['meta', { property: 'og:url', content: siteUrl }],

    // Twitter Card
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'aig - Version Control for the AI Age' }],
    ['meta', { name: 'twitter:description', content: 'Intent-based version control that captures why code changes, not just what changed.' }],
    ['meta', { name: 'twitter:image', content: ogImage }],

    // Canonical
    ['link', { rel: 'canonical', href: siteUrl }],
  ],

  // Per-page meta via transformPageData
  transformPageData(pageData) {
    const canonicalUrl = `${siteUrl}/${pageData.relativePath}`
      .replace(/index\.md$/, '')
      .replace(/\.md$/, '')

    pageData.frontmatter.head ??= []
    pageData.frontmatter.head.push(
      ['link', { rel: 'canonical', href: canonicalUrl }],
      ['meta', { property: 'og:url', content: canonicalUrl }],
    )

    // Use page title + description for OG if available
    if (pageData.frontmatter.description) {
      pageData.frontmatter.head.push(
        ['meta', { property: 'og:description', content: pageData.frontmatter.description }],
      )
    }
  },

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
          { text: 'Claude Code Integration', link: '/guide/claude-code' },
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
    },

    editLink: {
      pattern: 'https://github.com/saschb2b/ai-git/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    }
  }
})
