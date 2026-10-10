// The hero: the wordmark's twenty panes in three arrangements. Assembled; a Flavin row of tubes
// with heights in a symmetric (Judd) progression; and a mirrored terminal layout. On idle it
// cycles; every move staggers out from the centre axis and lands on a restrained spring.
import { CHOSEN, clamp01, drawPanes, fit, glassWord, pose, readPalette, spring, type Palette, type Rect } from './glass';
import { liveWhen, onTheme, reducedMotion } from './live';

const HC = 40, HR = 15, HOX = 6, HOY = 3;
const BLEED = 3; // modules of canvas past the layout box on each side, for the night wash
type StateName = 'word' | 'row' | 'screen';
const ORDER: StateName[] = ['word', 'row', 'screen'];

export function initHero(cv: HTMLCanvasElement, buttons: HTMLButtonElement[]) {
  const scene = glassWord(CHOSEN);
  const wordRects: Rect[] = pose(scene, 9, 0).map((p) => ({ x: p.x + HOX, y: p.y + HOY, w: p.w, h: p.h }));
  const byX = wordRects.map((_, i) => i).sort((a, b) => (wordRects[a].x + wordRects[a].w / 2) - (wordRects[b].x + wordRects[b].w / 2) || wordRects[a].y - wordRects[b].y);

  function rowState(): Rect[] {
    const n = wordRects.length, pitch = 1.6, w = 0.62, x0 = HC / 2 - (pitch * (n - 1)) / 2 - w / 2, out: Rect[] = [];
    byX.forEach((idx, k) => { const d = Math.abs(k - (n - 1) / 2) / ((n - 1) / 2); const h = 2 + 9 * Math.pow(1 - d, 1.15); out[idx] = { x: x0 + k * pitch, y: HR / 2 - h / 2, w, h }; });
    return out;
  }
  function screenState(): Rect[] {
    const g = 0.25, S: number[][] = [];
    S.push([2, 1, 20 - 2 - g, 1], [20 + g, 1, 18 - g, 1]); // title bar, split on the axis
    for (const [x, w] of [[2, 4.75], [7.25, 4.75], [28, 4.75], [33.25, 4.75]]) for (const [y, h] of [[3, 3], [6.5, 3], [10, 2]]) S.push([x, y, w, h]); // side tiles, mirrored
    S.push([13, 3, 14, 9]); // the main pane
    S.push([15, 4.5, 10, 1]); // its header line
    S.push([14, 8, 5.5, 2.5], [20.5, 8, 5.5, 2.5]); // two layers, mirrored
    S.push([2, 13, 18 - g, 1], [20 + g, 13, 18 - g, 1]); // status bar
    const slots = S.map(([x, y, w, h]) => ({ x, y, w, h })).sort((a, b) => (a.x + a.w / 2) - (b.x + b.w / 2) || a.y - b.y);
    const out: Rect[] = []; byX.forEach((idx, k) => { out[idx] = slots[k]; }); return out;
  }
  const STATES: Record<StateName, Rect[]> = { word: wordRects, row: rowState(), screen: screenState() };

  const reduced = reducedMotion();
  let pal: Palette = readPalette();
  let cur: Rect[] = wordRects.map((r) => ({ ...r }));
  let from: Rect[] = cur, to: StateName = 'word', t0 = 0, raf = 0, idle = 0, live = false, flying = false;
  let introT0 = reduced ? -1e9 : performance.now();

  function draw(now: number): boolean {
    const s = fit(cv, HC + 2 * BLEED, HR + 2 * BLEED), ctx = cv.getContext('2d')!;
    ctx.clearRect(0, 0, cv.width, cv.height);
    const intro = (now - introT0) / 1000;
    const busy = intro < 1.4;
    const panes = cur.map((r, i) => {
      const p = scene[i], local = clamp01((intro - p.delay) / 0.75), e = spring(local, 0.86);
      const sl = p.enter * (1 - e);
      return { ...r, x: r.x + (p.o === 'h' ? sl : 0), y: r.y + (p.o === 'v' ? sl : p.o === 'd' ? -Math.abs(sl) : 0), o: (r.h > r.w ? 'v' : 'h') as 'v' | 'h', tint: p.tint, alpha: p.alpha * Math.min(1, local * 2.5) };
    });
    drawPanes(ctx, panes, { s, pal, ox: BLEED * s, oy: BLEED * s });
    return busy;
  }

  const D = 1.05; // seconds per pane, after its stagger
  function frame(now: number) {
    raf = 0;
    const t = (now - t0) / 1000;
    let moving = false;
    if (flying) {
      const target = STATES[to];
      cur = from.map((a, i) => {
        const b = target[i], cx = a.x + a.w / 2, delay = (Math.abs(cx - HC / 2) / (HC / 2)) * 0.3; // radiate from the axis
        const ux = clamp01((t - delay) / D), uy = clamp01((t - delay - 0.08) / D);
        if (uy < 1) moving = true;
        const ex = spring(ux), ey = spring(uy);
        return { x: a.x + (b.x - a.x) * ex, y: a.y + (b.y - a.y) * ey, w: a.w + (b.w - a.w) * ex, h: a.h + (b.h - a.h) * ey };
      });
      if (!moving) { flying = false; from = cur = target.map((r) => ({ ...r })); }
    }
    const intro = draw(now);
    if ((moving || intro) && live) raf = requestAnimationFrame(frame);
  }
  const kick = () => { if (!raf) raf = requestAnimationFrame(frame); };

  function go(name: StateName) {
    to = name;
    for (const b of buttons) b.setAttribute('aria-pressed', String(b.dataset.state === name));
    if (reduced) { from = cur = STATES[name].map((r) => ({ ...r })); draw(performance.now()); return; }
    from = cur.map((r) => ({ ...r })); t0 = performance.now(); flying = true;
    kick();
  }
  const next = () => go(ORDER[(ORDER.indexOf(to) + 1) % ORDER.length]);
  function scheduleIdle() {
    clearTimeout(idle);
    if (reduced || !live) return;
    idle = window.setTimeout(() => { next(); scheduleIdle(); }, to === 'word' ? 5600 : 3600);
  }

  for (const b of buttons) b.addEventListener('click', () => { go(b.dataset.state as StateName); scheduleIdle(); });
  cv.addEventListener('click', () => { next(); scheduleIdle(); });

  liveWhen(cv, (l) => {
    live = l;
    if (l) { kick(); scheduleIdle(); } else { clearTimeout(idle); cancelAnimationFrame(raf); raf = 0; }
  });
  onTheme(() => { pal = readPalette(); draw(performance.now()); });
  new ResizeObserver(() => draw(performance.now())).observe(cv);
  draw(performance.now());
}
