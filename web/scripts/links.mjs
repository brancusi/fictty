// Link check over dist/: every internal href and src in every built page must resolve to a file the
// site serves (a page, its index.html, or an asset), and every #fragment to an id on that page.
// Also fails on links to GitHub for content the site hosts (posts, dossiers, design docs, the plan).
// Run after a build: npm run links
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const dist = path.resolve(fileURLToPath(new URL('../dist', import.meta.url)));
if (!existsSync(dist)) { console.error('links: no dist/ (npm run build first)'); process.exit(1); }
const pages = [];
const walk = (d) => { for (const f of readdirSync(d)) { const p = path.join(d, f); if (statSync(p).isDirectory()) walk(p); else if (p.endsWith('.html')) pages.push(p); } };
walk(dist);
const ids = new Map();
const idsOf = (file) => { if (!ids.has(file)) ids.set(file, new Set([...readFileSync(file, 'utf8').matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]))); return ids.get(file); };
const resolve = (url) => {
  const p = decodeURIComponent(url.split(/[?#]/)[0]);
  const f = path.join(dist, p);
  if (p.endsWith('/')) return existsSync(path.join(f, 'index.html')) ? path.join(f, 'index.html') : null;
  if (existsSync(f) && statSync(f).isFile()) return f;
  if (existsSync(path.join(f, 'index.html'))) return path.join(f, 'index.html');
  return null;
};
const hosted = /github\.com\/brancusi\/fictty\/(blob|tree)\/main\/docs\/(blog|research|design\/(scope|adopt-or-build|state-and-theme|prototype-notes|architecture)\.md|PLAN\.md|components\.md|CHANGELOG\.md|releasing\.md|guide\.md)/;
const bad = [];
let checked = 0;
for (const page of pages) {
  const html = readFileSync(page, 'utf8'), rel = '/' + path.relative(dist, page).replace(/index\.html$/, '');
  for (const m of html.matchAll(/<[a-z]+[^>]*?\s(?:href|src)="([^"]+)"[^>]*>/g)) {
    const url = m[1].replace(/&amp;/g, '&');
    // a deliberate "the source of this page" link carries data-source
    if (hosted.test(url) && !/\sdata-source[\s>=]/.test(m[0])) { bad.push(`${rel}: links GitHub for hosted content: ${url}`); continue; }
    if (/^(https?:|mailto:|data:|javascript:)/.test(url)) continue;
    checked++;
    if (url.startsWith('#')) { if (url.length > 1 && !idsOf(page).has(decodeURIComponent(url.slice(1)))) bad.push(`${rel}: missing anchor ${url}`); continue; }
    if (!url.startsWith('/')) { bad.push(`${rel}: relative link ${url}`); continue; }
    const target = resolve(url);
    if (!target) { bad.push(`${rel}: broken ${url}`); continue; }
    const hash = url.split('#')[1];
    if (hash && target.endsWith('.html') && !idsOf(target).has(decodeURIComponent(hash))) bad.push(`${rel}: missing anchor ${url}`);
  }
}
if (bad.length) { console.error([...new Set(bad)].join('\n')); console.error(`links: ${new Set(bad).size} problems in ${pages.length} pages`); process.exit(1); }
console.log(`links: ${checked} internal links in ${pages.length} pages all resolve`);
