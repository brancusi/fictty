// Markdown on the site comes straight from the repo (docs/guide.md, docs/blog/*.md), so its
// relative links point at repo files. This rewrites them: pages the site renders go to their
// routes, everything else to the file on GitHub. It also wraps tables so they scroll on phones,
// and turns dialogue (paragraphs that open with **Aram:** or **Claude:**) into speaker turns.
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { demoHtml, demoProps } from './demo-link.mjs';

const REPO = path.resolve(fileURLToPath(new URL('../../../', import.meta.url)));
const GH = 'https://github.com/brancusi/fictty';

/** The site route for a repo file the site renders, or null. */
export function route(rel) {
  rel = rel.replace(/\/$/, '');
  if (rel === 'docs/guide.md') return '/docs/';
  if (rel === 'docs/PLAN.md') return '/plan/';
  if (rel === 'docs/design/scope.md') return '/scope/';
  if (rel === 'docs/design/adopt-or-build.md') return '/adopt-or-build/';
  if (rel === 'docs/design/state-and-theme.md') return '/state-and-theme/';
  if (rel === 'docs/design/prototype-notes.md') return '/prototype-notes/';
  if (rel === 'docs/research' || rel === 'docs/research/landscape') return '/research/';
  let m = rel.match(/^docs\/research\/landscape\/([a-z0-9-]+)--_overview\.md$/);
  if (m) return `/research/${m[1]}/`;
  m = rel.match(/^docs\/research\/landscape\/([a-z0-9-]+)--([a-z0-9._-]+)\.md$/);
  if (m) return `/research/${m[1]}/${m[2]}/`;
  m = rel.match(/^docs\/blog\/(?:\d{4}-\d{2}-)?([^/]+)\.md$/);
  return m ? `/blog/${m[1]}/` : null;
}
function rewrite(href, fileDir) {
  if (!href || /^[a-z]+:/i.test(href) || href.startsWith('#') || href.startsWith('/')) return href;
  const [p, hash = ''] = href.split('#');
  const abs = path.resolve(fileDir, p), rel = path.relative(REPO, abs).split(path.sep).join('/');
  if (rel.startsWith('..')) return href;
  const r = route(rel);
  if (r) return r + (hash ? '#' + hash : '');
  const kind = p.endsWith('/') || !path.extname(rel) ? 'tree' : 'blob';
  return `${GH}/${kind}/main/${rel}${hash ? '#' + hash : ''}`;
}

// ---------- dialogue ----------
// Speakers the site knows, each with a glass tint. "Aram (@brancusi):" and "Aram:" both match.
const SPEAKERS = { aram: { name: 'Aram', handle: '@brancusi', tint: 'cyan' }, claude: { name: 'Claude', handle: '', tint: 'amber' }, tilde: { name: 'Tilde', handle: 'Claude', tint: 'amber' } };
const text = (n) => (n.type === 'text' ? n.value : (n.children || []).map(text).join(''));
const isWs = (n) => n.type === 'text' && !n.value.trim();
/** If p opens with a bold speaker label, returns the speaker and strips the label from p. */
function speakerOf(p) {
  if (p.type !== 'element' || p.tagName !== 'p') return null;
  const first = p.children.findIndex((c) => !isWs(c));
  const s = p.children[first];
  if (!s || s.type !== 'element' || s.tagName !== 'strong') return null;
  let label = text(s).trim(), colonAfter = false;
  const next = p.children[first + 1];
  if (!label.endsWith(':') && next?.type === 'text' && next.value.startsWith(':')) colonAfter = true;
  else if (!label.endsWith(':')) return null;
  const m = label.replace(/:$/, '').match(/^([A-Za-z]+)(?:\s*\((@[\w-]+)\))?$/);
  const sp = m && SPEAKERS[m[1].toLowerCase()];
  if (!sp) return null;
  p.children.splice(0, first + 1);
  if (colonAfter) p.children[0].value = p.children[0].value.slice(1);
  if (p.children[0]?.type === 'text') p.children[0].value = p.children[0].value.replace(/^\s+/, '');
  return { ...sp, handle: m[2] || sp.handle };
}
function dialogue(root) {
  const out = [];
  let turn = null;
  for (const node of root.children) {
    const sp = speakerOf(node);
    if (sp) {
      const label = { type: 'element', tagName: 'div', properties: { className: ['speaker'] }, children: [
        { type: 'element', tagName: 'span', properties: { className: ['who'] }, children: [{ type: 'text', value: sp.name }] },
        ...(sp.handle ? [{ type: 'element', tagName: 'span', properties: { className: ['handle'] }, children: [{ type: 'text', value: sp.handle }] }] : []),
      ] };
      turn = { type: 'element', tagName: 'div', properties: { className: ['turn', `turn-${sp.name.toLowerCase()}`], style: `--tint: var(--glass-${sp.tint})` }, children: [label, { type: 'element', tagName: 'div', properties: { className: ['said'] }, children: [node] }] };
      out.push(turn);
      continue;
    }
    // a speaker keeps the floor until a heading, a rule or the next speaker
    if (turn && node.type === 'element' && /^(h[1-6]|hr)$/.test(node.tagName)) turn = null;
    if (turn && !isWs(node)) { turn.children[1].children.push(node); continue; }
    if (turn && isWs(node)) continue;
    out.push(node);
  }
  root.children = out;
}

// ---------- design documents and the plan: sections, option cards, labelled fields ----------
const el = (tagName, className, children = [], props = {}) => ({ type: 'element', tagName, properties: { ...(className ? { className: [].concat(className) } : {}), ...props }, children });
const FIELDS = { gain: 'Gain', lose: 'Lose', 'deciding-question': 'Deciding question', verdict: 'Verdict', build: 'Build', adopt: 'Adopt', 'interoperate-with': 'Interoperate with', 'dont-build': "Don't build", 'keep-ratatui': 'Keep ratatui' };
const keyOf = (label) => label.toLowerCase().replace(/[’']/g, '').replace(/[^a-z]+/g, '-').replace(/^-|-$/g, '');
function labelField(p) {
  if (p.type !== 'element' || p.tagName !== 'p') return;
  const first = p.children.find((c) => !isWs(c));
  if (!first || first.type !== 'element' || first.tagName !== 'strong') return;
  const t = text(first).trim(), m = t.match(/^(.+?)[:.]$/);
  if (!m) return;
  const k = keyOf(m[1]);
  if (!FIELDS[k]) return;
  p.properties.className = ['field', `field-${k}`];
  first.tagName = 'span'; first.properties = { className: ['flabel'] };
  first.children = [{ type: 'text', value: m[1] }];
  // the rest of the paragraph is one body, so the label and the body are the field's two columns
  const rest = p.children.slice(p.children.indexOf(first) + 1);
  if (rest[0]?.type === 'text') rest[0].value = rest[0].value.replace(/^\s+/, '');
  p.children = [first, el('span', 'fbody', rest)];
}
function structure(tree, rel) {
  const kids = tree.children.filter((n) => !isWs(n));
  const out = [];
  let sec = null, card = null, ledeDone = false;
  for (const n of kids) {
    const tag = n.type === 'element' ? n.tagName : '';
    if (tag === 'h1') { out.push(n); continue; }
    if (tag === 'h2') {
      const title = text(n);
      const done = /\(done/i.test(title), next = /\(next/i.test(title);
      sec = el('section', ['dsec', ...(done ? ['is-done'] : []), ...(next ? ['is-next'] : [])], [n], { dataTitle: keyOf(title) });
      card = null; out.push(sec); continue;
    }
    if (tag === 'h3' && sec) { card = el('article', 'dcard', [n]); sec.children.push(card); continue; }
    if (tag === 'p' && !ledeDone && !sec) { n.properties.className = ['lede']; ledeDone = true; out.push(n); continue; }
    labelField(n);
    (card || sec || { children: out }).children.push(n);
  }
  // the plan: progress per phase
  for (const s of out) {
    if (s.tagName !== 'section') continue;
    let total = 0, done = 0;
    const count = (n) => {
      if (n.type === 'element' && n.tagName === 'input' && n.properties.type === 'checkbox') { total++; if (n.properties.checked) done++; n.properties.ariaLabel = n.properties.checked ? 'done' : 'not done'; }
      (n.children || []).forEach(count);
    };
    count(s);
    if (total) s.children.splice(1, 0, el('div', 'progress', [el('span', 'pbar', [el('span', 'pfill', [], { style: `width:${Math.round((done / total) * 100)}%` })]), el('span', 'pnum', [{ type: 'text', value: `${done} of ${total} done` }])]));
  }
  // the verdict, lifted from the document's own words, goes up front
  // a section that is itself the pitch is styled as one, not repeated
  for (const s of out) if (s.tagName === 'section' && s.properties.dataTitle === 'the-pitch') s.properties.className.push('is-pitch');
  const lead = out.find((s) => s.tagName === 'section' && s.properties.dataTitle === 'recommendation');
  if (lead) {
    const p = lead.children.find((c) => c.type === 'element' && c.tagName === 'p');
    if (p) {
      const at = out.findIndex((n) => n.properties?.className?.includes('lede'));
      const label = 'The verdict';
      out.splice(at + 1, 0, el('aside', 'verdict', [el('span', 'vlabel', [{ type: 'text', value: label }]), structuredClone(p)], { ariaLabel: label }));
    }
  }
  tree.children = out;
}
// dossiers: the opening Maker / URL / Status list becomes a facts block
function dossier(tree) {
  const ul = tree.children.find((n) => n.type === 'element' && n.tagName === 'ul');
  if (ul && /Maker:/.test(text(ul))) ul.properties.className = ['facts'];
}

export default function rehypeLinks() {
  return (tree, file) => {
    const dir = file.path ? path.dirname(file.path) : path.join(REPO, 'docs');
    const walk = (node, parent) => {
      // a demo link placeholder becomes the demo link (src/lib/demo-link.mjs)
      if (node.type === 'raw' && /^\s*<div class="demo-link"[^>]*><\/div>\s*$/.test(node.value)) node.value = demoHtml(demoProps(node.value));
      if (node.type === 'element') {
        if (node.tagName === 'a' && node.properties?.href) {
          const after = rewrite(String(node.properties.href), dir);
          node.properties.href = after;
          if (/^https?:/.test(after)) node.properties.rel = ['noopener'];
        }
        if (node.tagName === 'table' && parent && !(parent.properties?.className || []).includes('table-wrap')) {
          const i = parent.children.indexOf(node);
          parent.children[i] = { type: 'element', tagName: 'div', properties: { className: ['table-wrap'], tabIndex: 0, role: 'region', ariaLabel: 'Table (scrolls sideways)' }, children: [node] };
        }
      }
      if (node.children) for (const c of [...node.children]) walk(c, node);
    };
    walk(tree, null);
    dialogue(tree);
    const rel = file.path ? path.relative(REPO, file.path).split(path.sep).join('/') : '';
    if (/^docs\/(design\/[^/]+|PLAN)\.md$/.test(rel)) structure(tree, rel);
    if (/^docs\/research\/landscape\/[^/]+\.md$/.test(rel)) dossier(tree);
  };
}
