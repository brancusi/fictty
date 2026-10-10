// /research/: the radar and the map (drawn here from /research/data.json), plus views, filters and
// search over the board, which the page renders statically. Every dot is a link to its dossier.
// Ported from the landscape board in the research notes.
import { AXES, entryUrl, FICTTY, FIELDS, RELS, SHORT, shortName, SURF, SURFC, type Entry } from './landscape-meta';

const SVGNS = 'http://www.w3.org/2000/svg';
const $ = (id: string) => document.getElementById(id)!;
const esc = (s: string) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
function S(tag: string, attrs: Record<string, string | number>, parent?: Element) {
  const n = document.createElementNS(SVGNS, tag);
  for (const k in attrs) n.setAttribute(k, String(attrs[k]));
  if (parent) parent.appendChild(n);
  return n;
}
function store(k: string, v?: string): string | null {
  try { if (v === undefined) return localStorage.getItem(k); localStorage.setItem(k, v); } catch { /* storage off: fine */ }
  return null;
}
const AX = Object.fromEntries(AXES.map((a) => [a[0], a]));

export async function initResearch() {
  const data = await fetch('/research/data.json').then((r) => r.json()) as { clusters: { key: string }[]; rows: unknown[][] };
  const CLUSTERS = data.clusters;
  const E: (Entry & { on: boolean })[] = data.rows.map((r) => {
    const o = Object.fromEntries(FIELDS.map((k, i) => [k, r[i]])) as unknown as Entry & { on: boolean };
    o.id = `${o.cluster}--${o.slug}`; o.on = true; return o;
  });
  const BY = Object.fromEntries(E.map((e) => [e.id, e]));
  const state = { q: '', f: { surface: {}, layer: {}, relation: {}, evidence: {} } as Record<string, Record<string, 1>>, labels: false };
  let HAY: Record<string, string> | null = null;

  // legends
  for (const ul of document.querySelectorAll('.surf-legend')) {
    ul.innerHTML = SURF.map(([k, v]) => `<li><span class="sw" style="background:var(${v})"></span>${k} <span class="n">${E.filter((e) => e.surface === k).length}</span></li>`).join('');
  }

  // tooltip
  const tip = $('tip');
  function showTip(e: Entry, x: number, y: number) {
    tip.innerHTML = `<b>${esc(e.name)}</b>${esc(e.oneLiner)}<small>${esc(SHORT[e.cluster])} · ${e.relation} · ${e.surface}</small>`;
    tip.hidden = false;
    const w = tip.offsetWidth, h = tip.offsetHeight;
    let left = x + 14, top = y + 14;
    if (left + w > innerWidth - 8) left = Math.max(8, x - w - 14);
    if (top + h > innerHeight - 8) top = Math.max(8, y - h - 14);
    tip.style.left = left + 'px'; tip.style.top = top + 'px';
  }
  const hideTip = () => { tip.hidden = true; };
  function dot(parent: Element, e: Entry & { on: boolean }) {
    const a = S('a', { href: entryUrl(e), class: 'dot' + (e.on ? '' : ' off'), 'data-id': e.id, 'aria-label': `${e.name}: ${e.oneLiner}` }, parent);
    if (!e.on) a.setAttribute('tabindex', '-1');
    a.addEventListener('mousemove', (ev) => showTip(e, (ev as MouseEvent).clientX, (ev as MouseEvent).clientY));
    a.addEventListener('mouseleave', hideTip);
    a.addEventListener('focus', () => { const r = a.getBoundingClientRect(); showTip(e, r.right, r.top); });
    a.addEventListener('blur', hideTip);
    return a;
  }
  const defaultLabel = (e: Entry) => e.threat >= 2.5 || e.relation === 'adopt' || e.relation === 'build-on' || e.relation === 'compete';

  // ---------- radar ----------
  const RW = [46, 46, 92, 62, 118, 86], R0 = 62;
  function drawRadar() {
    const svg = $('radar'); svg.innerHTML = '';
    const cx = 500, cy = 500, nS = CLUSTERS.length, GAP = 0.2, span = (2 * Math.PI - GAP) / nS, A0 = -Math.PI / 2 + GAP / 2;
    const bounds = [R0]; RW.forEach((w) => bounds.push(bounds[bounds.length - 1] + w));
    const gr = S('g', { class: 'grid' }, svg);
    for (let i = RW.length - 1; i >= 0; i--) S('circle', { cx, cy, r: bounds[i + 1], fill: i % 2 ? 'var(--raised)' : 'var(--ground)', 'fill-opacity': 0.55, stroke: 'var(--line)' }, gr);
    CLUSTERS.forEach((c, s) => {
      const a = A0 + s * span;
      S('line', { x1: cx + R0 * Math.cos(a), y1: cy + R0 * Math.sin(a), x2: cx + bounds[6] * Math.cos(a), y2: cy + bounds[6] * Math.sin(a) }, gr);
      if (s === nS - 1) { const ae = a + span; S('line', { x1: cx + R0 * Math.cos(ae), y1: cy + R0 * Math.sin(ae), x2: cx + bounds[6] * Math.cos(ae), y2: cy + bounds[6] * Math.sin(ae) }, gr); }
      const am = a + span / 2, lr = bounds[6] + 34, lx = cx + lr * Math.cos(am), ly = cy + lr * Math.sin(am);
      const anchor = Math.abs(Math.cos(am)) < 0.25 ? 'middle' : Math.cos(am) > 0 ? 'start' : 'end';
      const link = S('a', { href: `/research/${c.key}/` }, svg);
      S('text', { x: lx, y: ly, 'text-anchor': anchor, class: 'seclbl' }, link).textContent = SHORT[c.key];
      S('text', { x: lx, y: ly + 18, 'text-anchor': anchor, class: 'secsub' }, svg).textContent = `${E.filter((e) => e.cluster === c.key).length} entries`;
    });
    RELS.forEach((r, i) => { S('text', { x: cx, y: cy - (bounds[i] + bounds[i + 1]) / 2 + 4, class: 'ringlbl', 'text-anchor': 'middle' }, svg).textContent = r[0]; });
    // fictty at the centre: the cursor mark, two panes of glass
    S('circle', { cx, cy, r: R0 - 6, fill: 'var(--glass-violet)', opacity: 0.12 }, svg);
    S('rect', { x: cx - 24, y: cy - 34, width: 26, height: 48, fill: 'var(--glass-cyan)', opacity: 0.74, style: 'mix-blend-mode:var(--blend)' }, svg);
    S('rect', { x: cx - 6, y: cy - 24, width: 26, height: 42, fill: 'var(--glass-violet)', opacity: 0.72, style: 'mix-blend-mode:var(--blend)' }, svg);
    S('text', { x: cx, y: cy + 40, 'text-anchor': 'middle', style: 'font-weight:600;font-size:17px;fill:var(--ink);paint-order:stroke;stroke:var(--raised);stroke-width:4px' }, svg).textContent = 'fictty';

    const layer = S('g', {}, svg), labels: [Element, number, number, Entry, number][] = [];
    CLUSTERS.forEach((c, s) => {
      const a0 = A0 + s * span;
      RELS.forEach((r, ri) => {
        const items = E.filter((e) => e.cluster === c.key && e.relation === r[0]).sort((x, y) => y.threat - x.threat || (x.name < y.name ? -1 : 1));
        if (!items.length) return;
        const rIn = bounds[ri], rOut = bounds[ri + 1], pad = 0.07, mid = (rIn + rOut) / 2, arc = (span - 2 * pad) * mid;
        const cap = Math.max(1, Math.floor(arc / 26)), rows = Math.ceil(items.length / cap), perRow = Math.ceil(items.length / rows);
        items.forEach((e, k) => {
          const row = Math.floor(k / perRow), idx = k % perRow, inRow = Math.min(perRow, items.length - row * perRow);
          const rr = rIn + ((row + 0.5) * (rOut - rIn)) / rows;
          let t = (idx + 0.5 + (row % 2 ? 0.25 : 0)) / inRow; if (t > 1) t = 1 - 0.5 / inRow;
          const ang = a0 + pad + t * (span - 2 * pad), x = cx + rr * Math.cos(ang), y = cy + rr * Math.sin(ang), rad = 5.5 + e.threat * 1.8;
          const g = dot(layer, e as Entry & { on: boolean });
          S('circle', { cx: x, cy: y, r: rad, fill: SURFC[e.surface] }, g);
          if (state.labels || defaultLabel(e)) labels.push([g, x, y, e, rad]);
        });
      });
    });
    for (const [g, x, y, e, rad] of labels) { const right = x >= 500; S('text', { x: x + (right ? rad + 4 : -rad - 4), y: y + 4, 'text-anchor': right ? 'start' : 'end' }, g).textContent = shortName(e.name); }
  }

  // ---------- map ----------
  let mx = store('fx-x') || 'codeVsData', my = store('fx-y') || 'ephemerality';
  if (!AX[mx]) mx = 'codeVsData'; if (!AX[my]) my = 'ephemerality';
  const sx = $('ax-x') as HTMLSelectElement, sy = $('ax-y') as HTMLSelectElement;
  sx.value = mx; sy.value = my;
  sx.addEventListener('change', () => { mx = sx.value; store('fx-x', mx); drawMap(); });
  sy.addEventListener('change', () => { my = sy.value; store('fx-y', my); drawMap(); });
  $('swap').addEventListener('click', () => { [mx, my] = [my, mx]; sx.value = mx; sy.value = my; store('fx-x', mx); store('fx-y', my); drawMap(); });
  const hash = (s: string) => { let h = 2166136261; for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); } return ((h >>> 0) % 10000) / 10000; };
  function drawMap() {
    const svg = $('map'); svg.innerHTML = '';
    const W = 1000, H = 680, L = 64, R = 30, T = 30, B = 70, ax = AX[mx], ay = AX[my];
    const px0 = ax[2] * 0.04, py0 = ay[2] * 0.04;
    const X = (v: number) => L + ((v + px0) / (ax[2] + 2 * px0)) * (W - L - R), Y = (v: number) => H - B - ((v + py0) / (ay[2] + 2 * py0)) * (H - T - B);
    const g = S('g', { class: 'grid' }, svg), stepX = ax[2] === 10 ? 2 : 1, stepY = ay[2] === 10 ? 2 : 1;
    for (let v = 0; v <= ax[2]; v += stepX) { S('line', { x1: X(v), x2: X(v), y1: T, y2: H - B }, g); S('text', { x: X(v), y: H - B + 18, 'text-anchor': 'middle', class: 'tick' }, svg).textContent = String(v); }
    for (let v = 0; v <= ay[2]; v += stepY) { S('line', { x1: L, x2: W - R, y1: Y(v), y2: Y(v) }, g); S('text', { x: L - 10, y: Y(v) + 4, 'text-anchor': 'end', class: 'tick' }, svg).textContent = String(v); }
    S('rect', { x: L, y: T, width: W - L - R, height: H - T - B, fill: 'none', stroke: 'var(--line-strong)' }, svg);
    S('text', { x: (L + W - R) / 2, y: H - 22, 'text-anchor': 'middle', class: 'axislbl' }, svg).textContent = ax[1] + ' →';
    S('text', { x: X(0), y: H - 22, class: 'endlbl' }, svg).textContent = ax[4];
    S('text', { x: X(ax[2]), y: H - 22, 'text-anchor': 'end', class: 'endlbl' }, svg).textContent = ax[5];
    S('text', { x: 0, y: 0, 'text-anchor': 'middle', class: 'axislbl', transform: `translate(18 ${(T + H - B) / 2}) rotate(-90)` }, svg).textContent = ay[1] + ' →';
    S('text', { x: 0, y: 0, class: 'endlbl', transform: `translate(34 ${Y(0)}) rotate(-90)` }, svg).textContent = ay[4];
    S('text', { x: 0, y: 0, 'text-anchor': 'end', class: 'endlbl', transform: `translate(34 ${Y(ay[2])}) rotate(-90)` }, svg).textContent = ay[5];
    const jx = ax[2] * 0.032, jy = ay[2] * 0.032, layer = S('g', {}, svg), lab: [Element, number, number, Entry][] = [];
    for (const e of E) {
      const x = X(Math.max(0, Math.min(ax[2], (e[mx as keyof Entry] as number) + (hash(e.id + 'x') - 0.5) * 2 * jx)));
      const y = Y(Math.max(0, Math.min(ay[2], (e[my as keyof Entry] as number) + (hash(e.id + 'y') - 0.5) * 2 * jy)));
      const gg = dot(layer, e);
      S('circle', { cx: x, cy: y, r: 7.5, fill: SURFC[e.surface], 'fill-opacity': 0.9 }, gg);
      if (state.labels || defaultLabel(e)) lab.push([gg, x, y, e]);
    }
    for (const [gg, x, y, e] of lab) { const right = x < W - 200; S('text', { x: x + (right ? 11 : -11), y: y + 4, 'text-anchor': right ? 'start' : 'end' }, gg).textContent = shortName(e.name); }
    const fx = FICTTY[mx], fy = FICTTY[my];
    if (fx !== undefined && fy !== undefined) {
      const px = X(fx), py = Y(fy), fg = S('g', { 'pointer-events': 'none' }, svg);
      S('path', { d: `M${px} ${py - 13}L${px + 13} ${py}L${px} ${py + 13}L${px - 13} ${py}Z`, fill: 'var(--ink)', stroke: 'var(--raised)', 'stroke-width': 2 }, fg);
      S('text', { x: px + (px < W - 160 ? 18 : -18), y: py + 5, 'text-anchor': px < W - 160 ? 'start' : 'end', style: 'font-weight:600;font-size:15px;fill:var(--ink);paint-order:stroke;stroke:var(--raised);stroke-width:4px' }, fg).textContent = 'fictty';
      $('map-note').textContent = 'Points are nudged slightly apart so equal scores stay visible. The diamond is fictty, scored from the scope.';
    } else {
      $('map-note').textContent = `fictty isn't plotted: ${fx === undefined ? ax[1] : ay[1]} measures other projects against fictty, so it has no value for fictty itself.`;
    }
  }

  // ---------- filters and search ----------
  const cards = [...document.querySelectorAll<HTMLElement>('#view-board .rcard')];
  function matches(e: Entry) {
    for (const g in state.f) { const keys = Object.keys(state.f[g]); if (keys.length && !state.f[g][e[g as keyof Entry] as string]) return false; }
    if (state.q) {
      const hay = `${e.name} ${e.maker} ${e.oneLiner} ${e.takeaway} ${e.status} ${e.kind} ${e.layer} ${e.surface} ${HAY?.[e.id] || ''}`.toLowerCase();
      for (const t of state.q.split(/\s+/)) if (!hay.includes(t)) return false;
    }
    return true;
  }
  function apply() {
    let n = 0;
    for (const e of E) { e.on = matches(e); if (e.on) n++; }
    $('count').textContent = n === E.length ? `${E.length} entries` : `${n} of ${E.length} shown`;
    for (const d of document.querySelectorAll<SVGElement>('.dot[data-id]')) { const e = BY[d.dataset.id!]; d.classList.toggle('off', !e.on); if (e.on) d.removeAttribute('tabindex'); else d.setAttribute('tabindex', '-1'); }
    for (const c of cards) c.hidden = !BY[c.dataset.id!].on;
    for (const sec of document.querySelectorAll<HTMLElement>('.rcluster')) {
      const all = sec.querySelectorAll('.rcard').length, on = sec.querySelectorAll('.rcard:not([hidden])').length;
      sec.querySelector('small')!.textContent = on === all ? String(all) : `${on} of ${all}`;
      (sec.querySelector('.empty') as HTMLElement).hidden = on > 0;
    }
  }
  const fg = document.querySelector('.fgroups')!;
  fg.addEventListener('click', (ev) => {
    const b = (ev.target as Element).closest<HTMLElement>('.chip');
    if (b) { const g = b.dataset.g!, v = b.dataset.v!; if (state.f[g][v]) delete state.f[g][v]; else state.f[g][v] = 1; b.setAttribute('aria-pressed', String(!!state.f[g][v])); apply(); }
    if ((ev.target as Element).id === 'clear') { for (const g in state.f) state.f[g] = {}; fg.querySelectorAll('.chip').forEach((c) => c.setAttribute('aria-pressed', 'false')); (q as HTMLInputElement).value = ''; state.q = ''; apply(); }
  });
  const q = $('q') as HTMLInputElement;
  let qt = 0;
  q.addEventListener('input', () => {
    if (!HAY) fetch('/research/search.json').then((r) => r.json()).then((h) => { HAY = h; apply(); }).catch(() => { HAY = {}; });
    clearTimeout(qt); qt = window.setTimeout(() => { state.q = q.value.trim().toLowerCase(); apply(); }, 120);
  });
  ($('all-labels') as HTMLInputElement).addEventListener('change', (ev) => { state.labels = (ev.target as HTMLInputElement).checked; drawRadar(); drawMap(); apply(); });
  if (!matchMedia('(max-width: 640px)').matches) $('filters').setAttribute('open', '');

  // ---------- views ----------
  const tabs = [...document.querySelectorAll<HTMLButtonElement>('.rtabs button')];
  function setView(v: string) {
    for (const k of ['radar', 'map', 'board']) { $('view-' + k).hidden = k !== v; $('tab-' + k).setAttribute('aria-selected', String(k === v)); $('tab-' + k).tabIndex = k === v ? 0 : -1; }
    store('fx-view', v);
    try { history.replaceState(null, '', '#' + v); } catch { /* fine */ }
  }
  tabs.forEach((b, i) => {
    b.addEventListener('click', () => setView(b.dataset.view!));
    b.addEventListener('keydown', (ev) => { if (ev.key === 'ArrowRight' || ev.key === 'ArrowLeft') { const n = tabs[(i + (ev.key === 'ArrowRight' ? 1 : tabs.length - 1)) % tabs.length]; n.focus(); setView(n.dataset.view!); } });
  });

  drawRadar(); drawMap(); apply();
  const h0 = location.hash.slice(1), v0 = store('fx-view');
  setView(['radar', 'map', 'board'].includes(h0) ? h0 : v0 && ['map', 'board'].includes(v0) ? v0 : 'radar');
}
