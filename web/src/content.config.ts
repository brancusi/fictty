import { defineCollection } from 'astro:content';
import { glob } from 'astro/loaders';
import { z } from 'astro/zod';

// The site renders the repo's own Markdown, so there's one source of truth:
// - blog: docs/blog/ (a YYYY-MM- prefix on the file name is dropped from the URL)
// - docs: the guide; pages: the scope, adopt-or-build and the plan
// - dossiers: docs/research/landscape/, one per project plus a cluster overview each
export const collections = {
  blog: defineCollection({
    loader: glob({ pattern: '*.md', base: '../docs/blog', generateId: ({ entry }) => entry.replace(/\.md$/, '').replace(/^\d{4}-\d{2}-/, '') }),
    schema: z.object({ title: z.string(), description: z.string(), date: z.coerce.date(), kicker: z.string().optional() }),
  }),
  docs: defineCollection({
    loader: glob({ pattern: 'guide.md', base: '../docs' }),
  }),
  pages: defineCollection({
    loader: glob({ pattern: ['PLAN.md', 'design/scope.md', 'design/adopt-or-build.md'], base: '../docs', generateId: ({ entry }) => entry.replace(/\.md$/, '').replace(/^design\//, '').toLowerCase() }),
  }),
  dossiers: defineCollection({
    loader: glob({ pattern: '*.md', base: '../docs/research/landscape', generateId: ({ entry }) => entry.replace(/\.md$/, '') }),
  }),
};
