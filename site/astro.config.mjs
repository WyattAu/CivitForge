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
    }),
    solid(),
  ],
});
