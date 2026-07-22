// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';
import starlightLinksValidator from 'starlight-links-validator';

// Deployed to GitHub Pages as a project site at
// https://pegasusheavy.github.io/bitbucket-cli/
export default defineConfig({
  site: 'https://pegasusheavy.github.io',
  base: '/bitbucket-cli',
  integrations: [
    sitemap(),
    starlight({
      title: 'Bitbucket CLI',
      description:
        'A command-line interface for Bitbucket Cloud — manage repos, pull requests, issues, and pipelines from your terminal.',
      logo: {
        src: './src/assets/logo.svg',
        alt: 'Bitbucket CLI',
      },
      favicon: '/favicon.ico',
      plugins: [starlightLinksValidator()],
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/pegasusheavy/bitbucket-cli',
        },
      ],
      customCss: ['./src/styles/theme.css'],
      editLink: {
        baseUrl:
          'https://github.com/pegasusheavy/bitbucket-cli/edit/develop/docs/',
      },
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', slug: 'index' },
            { label: 'Installation', slug: 'installation' },
            { label: 'Authentication', slug: 'authentication' },
            { label: 'Configuration', slug: 'configuration' },
          ],
        },
        {
          label: 'Commands',
          items: [{ autogenerate: { directory: 'commands' } }],
        },
        {
          label: 'Advanced',
          items: [
            { label: 'Interactive TUI', slug: 'tui' },
            { label: 'Scripting with JSON', slug: 'scripting' },
          ],
        },
      ],
    }),
  ],
});
