// Build-time loading of the landscape: entries and clusters from docs/research/landscape/_data/
// (the scores and takeaways), dossiers from the same folder (the prose).
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { FIELDS, type Cluster, type Entry } from './landscape-meta';

const DIR = path.resolve(process.cwd(), '../docs/research/landscape/_data');
const rows: unknown[][] = JSON.parse(readFileSync(path.join(DIR, 'entries.json'), 'utf8'));
export const CLUSTERS: Cluster[] = JSON.parse(readFileSync(path.join(DIR, 'clusters.json'), 'utf8'));
export const ENTRIES: Entry[] = rows.map((r) => {
  const o = Object.fromEntries(FIELDS.map((k, i) => [k, r[i]])) as unknown as Entry;
  o.id = `${o.cluster}--${o.slug}`;
  return o;
});
