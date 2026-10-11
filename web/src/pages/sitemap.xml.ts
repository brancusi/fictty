import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { CLUSTERS, ENTRIES } from '../lib/landscape';
export const GET: APIRoute = async ({ site }) => {
  const posts = await getCollection('blog');
  const urls = ['/', '/docs/', '/blog/', '/research/', '/scope/', '/plan/', '/adopt-or-build/', '/state-and-theme/', '/prototype-notes/', ...posts.map((p) => `/blog/${p.id}/`), ...CLUSTERS.map((c) => `/research/${c.key}/`), ...ENTRIES.map((e) => `/research/${e.cluster}/${e.slug}/`)];
  const body = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.map((u) => `  <url><loc>${new URL(u, site)}</loc></url>`).join('\n')}\n</urlset>\n`;
  return new Response(body, { headers: { 'Content-Type': 'application/xml; charset=utf-8' } });
};
