import { defineConfig } from 'astro/config';
import mdx from '@astrojs/mdx';
import sitemap from '@astrojs/sitemap';

export default defineConfig({
  site: 'https://openwarp.dev',
  integrations: [mdx(), sitemap()],
  trailingSlash: 'ignore',
  redirects: {
    '/docs': '/docs/introduction',
  },
  build: {
    format: 'directory',
  },
});
