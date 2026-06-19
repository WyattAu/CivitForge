import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import solid from '@astrojs/solid-js';

// CivitForge documentation site
// Built with Astro + Starlight for the landing page and docs.
// Uses SolidJS for interactive components (API explorer, playground).
export default defineConfig({
  site: 'https://wyattau.github.io',
  base: '/CivitForge',
  integrations: [
    starlight({
      title: 'CivitForge',
      description: 'Federated, Rust-native software forge for large-scale monorepos',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/WyattAu/CivitForge' },
      ],
      customCss: ['./src/styles/custom.css'],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Overview', slug: 'overview' },
            { label: 'Quick Start', slug: 'quick-start' },
            { label: 'Configuration', slug: 'configuration' },
          ],
        },
        {
          label: 'Architecture',
          items: [
            { label: 'Architecture', slug: 'architecture' },
            { label: 'Database', slug: 'database' },
            { label: 'Federation', slug: 'federation' },
          ],
        },
        {
          label: 'Features',
          items: [
            { label: 'CI/CD Pipeline', slug: 'ci-cd' },
            { label: 'API Reference', slug: 'api-reference' },
          ],
        },
        {
          label: 'Operations',
          items: [
            { label: 'Operator Guide', slug: 'operator-guide' },
            { label: 'Security', slug: 'security' },
          ],
        },
      ],
    }),
    solid(),
  ],
});
