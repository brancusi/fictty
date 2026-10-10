// The landscape's data for the board's radar and map: clusters and entries, in the compact
// row form of docs/research/landscape/_data/entries.json.
import type { APIRoute } from 'astro';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { CLUSTERS } from '../../lib/landscape';
const DIR = path.resolve(process.cwd(), '../docs/research/landscape/_data');
export const GET: APIRoute = () => new Response(JSON.stringify({ clusters: CLUSTERS.map(({ key, title }) => ({ key, title })), rows: JSON.parse(readFileSync(path.join(DIR, 'entries.json'), 'utf8')) }), { headers: { 'Content-Type': 'application/json' } });
