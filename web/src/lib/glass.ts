// The glass generator, ported from docs/design/identity/glass-v2.html. Every stroke of a word set
// in the nine-row cell font becomes one pane of tinted glass; the seed decides tint swaps, the one
// warm accent and the quarter-module slides. Pure geometry here; drawing reads the palette from
// the page's CSS tokens, so the CSS stays the one source of truth for colour.

export type Rgb = [number, number, number];
export type Tint = 'cyan' | 'violet' | 'amber' | 'rose' | 'smoke';
export type Orient = 'h' | 'v' | 'd';
export interface Rect { x: number; y: number; w: number; h: number }
export interface Pane extends Rect { o: Orient; tint: Tint; alpha: number }
export interface ScenePane extends Pane { li: number; enter: number; delay: number; rel: number }
export interface Palette { ground: Rgb; line: Rgb; ink3: Rgb; cyan: Rgb; violet: Rgb; amber: Rgb; rose: Rgb; smoke: Rgb; dark: boolean }

// ---------- the cell font: nine rows (two ascender, five x-height, two descender) ----------
export const FONT: Record<string, string[]> = {
  f: ['..##', '.#..', '###.', '.#..', '.#..', '.#..', '.#..', '....', '....'],
  i: ['.#.', '...', '##.', '.#.', '.#.', '.#.', '###', '...', '...'],
  c: ['....', '....', '.###', '#...', '#...', '#...', '.###', '....', '....'],
  t: ['.#..', '.#..', '####', '.#..', '.#..', '.#..', '..##', '....', '....'],
  y: ['....', '....', '#..#', '#..#', '#..#', '#..#', '.###', '...#', '###.'],
  // digits for the 404, on the same rows (cap height = ascender + x-height)
  '4': ['#..#', '#..#', '#..#', '####', '...#', '...#', '...#', '....', '....'],
  '0': ['.##.', '#..#', '#..#', '#..#', '#..#', '#..#', '.##.', '....', '....'],
};

interface Cell { x: number; y: number; li: number }
interface Stroke { x0: number; y0: number; x1: number; y1: number; o: Orient; li: number }

function wordCells(word: string): Cell[] {
  const out: Cell[] = [];
  let x = 0;
  [...word].forEach((ch, li) => {
    const g = FONT[ch], w = g[0].length;
    g.forEach((row, ry) => { for (let rx = 0; rx < w; rx++) if (row[rx] === '#') out.push({ x: x + rx, y: ry, li }); });
    x += w + 1;
  });
  return out;
}
export const wordCols = (word: string) => [...word].reduce((n, ch) => n + FONT[ch][0].length + 1, -1);

// runs of cells become panes: horizontal (h), vertical (v) or a single cell (d)
function strokes(cells: Cell[]): Stroke[] {
  const key = (x: number, y: number) => x + ',' + y;
  const on = new Map(cells.map((c) => [key(c.x, c.y), c]));
  const has = (x: number, y: number) => on.has(key(x, y));
  const used = new Set<string>(), out: Stroke[] = [];
  for (const c of cells) {
    if (!has(c.x - 1, c.y)) {
      let e = c.x; while (has(e + 1, c.y)) e++;
      if (e > c.x) { out.push({ x0: c.x, y0: c.y, x1: e + 1, y1: c.y + 1, o: 'h', li: c.li }); for (let x = c.x; x <= e; x++) used.add(key(x, c.y)); }
    }
    if (!has(c.x, c.y - 1)) {
      let e = c.y; while (has(c.x, e + 1)) e++;
      if (e > c.y) { out.push({ x0: c.x, y0: c.y, x1: c.x + 1, y1: e + 1, o: 'v', li: c.li }); for (let y = c.y; y <= e; y++) used.add(key(c.x, y)); }
    }
  }
  for (const c of cells) if (!used.has(key(c.x, c.y))) out.push({ x0: c.x, y0: c.y, x1: c.x + 1, y1: c.y + 1, o: 'd', li: c.li });
  return out;
}

// ---------- seeded randomness (mulberry32) ----------
export function rng(seed: number): () => number {
  let a = seed >>> 0;
  return () => { a = (a + 0x6D2B79F5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
}

// ---------- the v2 generator: rules, then the seed ----------
export const CHOSEN = 2611;
export const INSET = 0.07;
export function glassWord(seed: number, word = 'fictty'): ScenePane[] {
  const S = strokes(wordCells(word));
  const R = rng(seed);
  const swap = [...word].map(() => R() < 0.2);
  const lines = S.map((_, i) => i).filter((i) => S[i].o !== 'd');
  const accent = R() < 0.7 ? lines[Math.floor(R() * lines.length)] : -1;
  const accentTint: Tint = R() < 0.6 ? 'amber' : 'rose';
  return S.map((s, i) => {
    let tint: Tint = s.o === 'v' ? 'cyan' : s.o === 'h' ? 'violet' : 'amber';
    if (swap[s.li] && s.o !== 'd') tint = tint === 'cyan' ? 'violet' : 'cyan';
    if (i === accent) tint = accentTint;
    const r = R(), q = r < 0.18 ? -0.25 : r > 0.82 ? 0.25 : 0;
    const dx = s.o === 'h' ? q : 0, dy = s.o === 'v' ? q : 0;
    const alpha = 0.62 + R() * 0.14;
    const enter = (R() < 0.5 ? -1 : 1) * (3 + R() * 3);
    const delay = s.li * 0.05 + R() * 0.07;
    const rel = (R() < 0.5 ? -1 : 1) * (0.6 + R() * 0.9);
    return { x: s.x0 + INSET + dx, y: s.y0 + INSET + dy, w: s.x1 - s.x0 - 2 * INSET, h: s.y1 - s.y0 - 2 * INSET, o: s.o, li: s.li, tint, alpha, enter, delay, rel };
  });
}

// ---------- motion ----------
export const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);
export const expoOut = (x: number) => (x >= 1 ? 1 : 1 - Math.pow(2, -10 * x));
/** A damped spring from 0 to 1 that settles (within 1%) at u = 1. zeta < 1 overshoots a little. */
export function spring(u: number, zeta = 0.8): number {
  if (u <= 0) return 0;
  if (u >= 1) return 1;
  const w = 4.6 / zeta, wd = w * Math.sqrt(1 - zeta * zeta), t = u;
  return 1 - Math.exp(-zeta * w * t) * (Math.cos(wd * t) + ((zeta * w) / wd) * Math.sin(wd * t));
}
/** The same spring as a CSS linear() easing, for WAAPI and transitions. */
export function springCss(zeta = 0.8, n = 28): string {
  const pts: string[] = [];
  for (let i = 0; i <= n; i++) pts.push((i === n ? 1 : spring(i / n, zeta)).toFixed(4));
  return `linear(${pts.join(', ')})`;
}

/** Pose a scene: prog is seconds since the throw, release 0..1 slides panes apart along their axis. */
export function pose(scene: ScenePane[], prog = 9, release = 0, flat = false): Pane[] {
  return scene.map((p) => {
    const local = clamp01((prog - p.delay) / 0.7), e = spring(local, 0.86);
    const slide = p.enter * (1 - e) + p.rel * release * 2.2;
    const along = p.o === 'h' ? [slide, 0] : p.o === 'v' ? [0, slide] : [0, -Math.abs(slide)];
    let x = p.x + along[0], y = p.y + along[1];
    if (flat) {
      x = Math.round((p.x - INSET) * 4) / 4 + INSET; y = Math.round((p.y - INSET) * 4) / 4 + INSET;
      if (p.o === 'h') x = Math.round(p.x - INSET) + INSET;
      if (p.o === 'v') y = Math.round(p.y - INSET) + INSET;
    }
    return { x, y, w: p.w, h: p.h, o: p.o, tint: p.tint, alpha: (flat ? 0.85 : p.alpha) * Math.min(1, local * 2.5) * (1 - 0.45 * release) };
  });
}

// ---------- palette, read from the CSS tokens ----------
const hex = (h: string): Rgb => { const n = parseInt(h.trim().replace('#', ''), 16); return [(n >> 16) & 255, (n >> 8) & 255, n & 255]; };
export const rgba = ([r, g, b]: Rgb, a: number) => `rgba(${r},${g},${b},${a})`;
export function isDark(): boolean {
  const t = document.documentElement.dataset.theme;
  return t ? t === 'dark' : matchMedia('(prefers-color-scheme: dark)').matches;
}
export function readPalette(el: Element = document.documentElement): Palette {
  const cs = getComputedStyle(el), v = (n: string) => hex(cs.getPropertyValue(n));
  return {
    ground: v('--ground'), line: v('--line'), ink3: v('--ink-3'),
    cyan: v('--glass-cyan'), violet: v('--glass-violet'), amber: v('--glass-amber'), rose: v('--glass-rose'), smoke: v('--glass-smoke'),
    dark: isDark(),
  };
}

// ---------- the night wash, as cached nine-slice sprites (shadowBlur once, not every frame) ----------
const sprites = new Map<string, { cv: HTMLCanvasElement; pad: number }>();
function glowSprite(c: Rgb, blur: number) {
  const b = Math.max(2, Math.round(blur)), key = c.join(',') + '|' + b;
  let s = sprites.get(key);
  if (s) return s;
  const pad = Math.ceil(b * 1.25), size = pad * 4, cv = document.createElement('canvas');
  cv.width = cv.height = size;
  const ctx = cv.getContext('2d')!;
  // draw the rect far off-canvas and bring only its shadow back in
  ctx.shadowColor = rgba(c, 1); ctx.shadowBlur = b; ctx.shadowOffsetX = size * 4;
  ctx.fillStyle = '#000'; ctx.fillRect(pad - size * 4, pad, pad * 2, pad * 2);
  s = { cv, pad };
  if (sprites.size > 64) sprites.clear();
  sprites.set(key, s);
  return s;
}
function drawGlow(ctx: CanvasRenderingContext2D, spr: { cv: HTMLCanvasElement; pad: number }, x: number, y: number, w: number, h: number, alpha: number) {
  const { cv, pad } = spr, H = cv.width / 2;
  const dx = x - pad, dy = y - pad, dw = w + 2 * pad, dh = h + 2 * pad;
  const hw = Math.min(H, dw / 2), hh = Math.min(H, dh / 2), mw = dw - 2 * hw, mh = dh - 2 * hh;
  // a small pane throws less light than the edge of a long one
  ctx.globalAlpha = alpha * Math.sqrt(Math.min(1, w / (2 * pad) + 0.15) * Math.min(1, h / (2 * pad) + 0.15));
  const sx = [0, H - 1, H + 1], sw = [H - 1, 2, H - 1], ddx = [dx, dx + hw, dx + hw + mw], ddw = [hw, mw, hw];
  const sy = sx, sh = sw, ddy = [dy, dy + hh, dy + hh + mh], ddh = [hh, mh, hh];
  for (let j = 0; j < 3; j++) for (let i = 0; i < 3; i++) {
    if (ddw[i] <= 0 || ddh[j] <= 0) continue;
    ctx.drawImage(cv, sx[i], sy[j], sw[i], sh[j], ddx[i], ddy[j], ddw[i], ddh[j]);
  }
  ctx.globalAlpha = 1;
}

// ---------- drawing panes: multiply by day; screen, edge light and wash by night ----------
export interface DrawOpts { s: number; ox?: number; oy?: number; pal: Palette; detail?: boolean; glow?: number }
export function drawPanes(ctx: CanvasRenderingContext2D, panes: Pane[], { s, ox = 0, oy = 0, pal, detail = true, glow = 1 }: DrawOpts) {
  const dark = pal.dark;
  const R = (p: Pane): [number, number, number, number] => {
    const r: [number, number, number, number] = [ox + p.x * s, oy + p.y * s, p.w * s, p.h * s];
    if (detail) return r;
    const x = Math.round(r[0]), y = Math.round(r[1]);
    return [x, y, Math.max(1, Math.round(r[0] + r[2]) - x), Math.max(1, Math.round(r[1] + r[3]) - y)];
  };
  ctx.save();
  if (dark && detail && glow > 0) {
    // the wash: each pane's light falls on the ground around it (Flavin)
    ctx.globalCompositeOperation = 'screen';
    for (const p of panes) {
      if (p.alpha < 0.01) continue;
      const c = pal[p.tint], [x, y, w, h] = R(p);
      drawGlow(ctx, glowSprite(c, s * 2.2), x, y, w, h, 0.62 * p.alpha * glow);
      drawGlow(ctx, glowSprite(c, s * 0.6), x, y, w, h, 0.5 * p.alpha * glow);
    }
  }
  ctx.globalCompositeOperation = dark ? 'screen' : 'multiply';
  for (const p of panes) {
    if (p.alpha < 0.01) continue;
    const [x, y, w, h] = R(p);
    ctx.fillStyle = rgba(pal[p.tint], p.alpha * (dark ? (detail ? 0.78 : 0.95) : 1));
    ctx.fillRect(x, y, w, h);
  }
  if (detail) {
    for (const p of panes) {
      if (p.alpha < 0.01) continue;
      const [x, y, w, h] = R(p), c = pal[p.tint], t = Math.max(1, s * 0.055);
      if (dark) {
        // edge light: the whole rim, brighter than the body
        ctx.globalCompositeOperation = 'screen'; ctx.strokeStyle = rgba(c, 0.85 * p.alpha); ctx.lineWidth = Math.max(1, s * 0.04);
        ctx.strokeRect(x + ctx.lineWidth / 2, y + ctx.lineWidth / 2, w - ctx.lineWidth, h - ctx.lineWidth);
      } else {
        ctx.globalCompositeOperation = 'source-over'; ctx.fillStyle = `rgba(255,255,255,${0.5 * p.alpha})`;
        ctx.fillRect(x, y, w, Math.max(1, s * 0.03)); ctx.fillRect(x, y, Math.max(1, s * 0.03), h);
      }
      // Bell's coat: an iridescent edge along the long side
      const vert = h > w;
      const g = vert ? ctx.createLinearGradient(x, y, x, y + h) : ctx.createLinearGradient(x, y, x + w, y);
      g.addColorStop(0, rgba(pal.cyan, 0.9)); g.addColorStop(0.5, rgba(pal.violet, 0.9)); g.addColorStop(1, rgba(pal.amber, 0.9));
      ctx.globalCompositeOperation = dark ? 'screen' : 'multiply'; ctx.globalAlpha = (dark ? 0.75 : 0.55) * p.alpha; ctx.fillStyle = g;
      if (vert) ctx.fillRect(x + w - t, y, t, h); else ctx.fillRect(x, y + h - t, w, t);
      ctx.globalAlpha = 1;
    }
  }
  ctx.restore();
}

export const DPR = () => Math.min(2, window.devicePixelRatio || 1);
/** Size a canvas's backing store to its CSS width at cols:rows; returns pixels per module. */
export function fit(cv: HTMLCanvasElement, cols: number, rows: number): number {
  const w = Math.max(40, cv.clientWidth || cv.parentElement!.clientWidth);
  const W = Math.round(w * DPR()), H = Math.round((W * rows) / cols);
  if (cv.width !== W || cv.height !== H) { cv.width = W; cv.height = H; }
  return W / cols;
}

/** The wordmark, assembled, into a canvas with a transparent ground. */
export function drawWord(cv: HTMLCanvasElement, scene: ScenePane[], opts: { pal: Palette; prog?: number; cols?: number; rows?: number; pad?: [number, number] }) {
  const { pal, prog = 9, cols = 30, rows = 11, pad = [1, 1] } = opts;
  const s = fit(cv, cols, rows), ctx = cv.getContext('2d')!;
  ctx.clearRect(0, 0, cv.width, cv.height);
  const flat = s / DPR() < 8;
  drawPanes(ctx, pose(scene, prog, 0, flat), { s, ox: pad[0] * s, oy: pad[1] * s, pal, detail: !flat });
}

// ---------- the mark: a block cursor with a second pane over it ----------
export const CURSOR: Pane[] = [
  { x: 2.5, y: 1.5, w: 6.5, h: 12, o: 'v', tint: 'cyan', alpha: 0.74 },
  { x: 7, y: 4, w: 6.5, h: 10.5, o: 'v', tint: 'violet', alpha: 0.72 },
];
export function drawMark(cv: HTMLCanvasElement, pal: Palette, css: number) {
  const W = Math.round(css * DPR());
  if (cv.width !== W) { cv.width = cv.height = W; }
  const ctx = cv.getContext('2d')!, s = W / 16;
  ctx.clearRect(0, 0, W, W);
  drawPanes(ctx, CURSOR, { s, pal, detail: css >= 48, glow: 0 });
}
