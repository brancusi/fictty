// @ts-check
import { defineConfig } from 'astro/config';
import { unified } from '@astrojs/markdown-remark';
import rehypeLinks from './src/lib/rehype-links.mjs';

// fictty's public site. Static output in dist/, served by Cloudflare (wrangler.jsonc).
export default defineConfig({
  site: 'https://fictty.com',
  trailingSlash: 'ignore',
  build: { format: 'directory', inlineStylesheets: 'auto' },
  markdown: { syntaxHighlight: false, processor: unified({ rehypePlugins: [rehypeLinks] }) },
});
