import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
const x = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
export const GET: APIRoute = async ({ site }) => {
  const posts = (await getCollection('blog')).sort((a, b) => +b.data.date - +a.data.date);
  const items = posts.map((p) => {
    const link = new URL(`/blog/${p.id}/`, site);
    return `<item><title>${x(p.data.title)}</title><link>${link}</link><guid>${link}</guid><pubDate>${p.data.date.toUTCString()}</pubDate><description>${x(p.data.description)}</description></item>`;
  }).join('');
  const body = `<?xml version="1.0" encoding="UTF-8"?><rss version="2.0"><channel><title>fictty blog</title><link>${new URL('/blog/', site)}</link><description>Notes from building fictty.</description><language>en</language>${items}</channel></rss>`;
  return new Response(body, { headers: { 'Content-Type': 'application/rss+xml; charset=utf-8' } });
};
