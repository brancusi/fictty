// Full-text search over every dossier, loaded by the board only when someone starts typing.
import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
export const GET: APIRoute = async () => {
  const docs = await getCollection('dossiers');
  const text = Object.fromEntries(docs.filter((d) => !d.id.endsWith('--_overview')).map((d) => [d.id, (d.body || '').replace(/\]\([^)]*\)/g, ']').replace(/[#*`>|_[\]-]+/g, ' ').replace(/\s+/g, ' ').toLowerCase()]));
  return new Response(JSON.stringify(text), { headers: { 'Content-Type': 'application/json' } });
};
