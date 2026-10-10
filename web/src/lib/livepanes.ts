// Live glass panes (src/components/LivePanes.astro): a plain tinted pane opens into a live fictty
// screen on hover, focus or tap, and settles back on leave. CSS does the geometry; this file decides
// which pane is open, keeps each open pane clear of the row's edges, and runs the mini screens
// (src/lib/minis.ts) only while someone can see them. Reduced motion: an instant swap, a still frame.
import { Grid, readColors, render, type Colors } from './cells';
import { liveWhen, onTheme, reducedMotion } from './live';
import { MINIS, type Mini } from './minis';

interface P {
  el: HTMLElement; glass: HTMLElement; cv: HTMLCanvasElement; ctx: CanvasRenderingContext2D;
  mini: Mini; next: number; open: boolean; until: number; grid: Grid | null;
  fs: number; cw: number; chh: number; ox: number; oy: number; dpr: number; W: number; H: number;
}

export function initLivePanes(row: HTMLElement) {
  const reduced = reducedMotion();
  let colors: Colors = readColors(), raf = 0, rowLive = false;
  const panes: P[] = [...row.querySelectorAll<HTMLElement>('.lp')].map((el) => {
    const cv = el.querySelector('canvas')!;
    const mini = MINIS[el.dataset.kind!]();
    for (let i = 0; i < 24; i++) mini.step(-1e9 + i); // some history, so it never opens empty
    return { el, glass: el.querySelector<HTMLElement>('.lp-glass')!, cv, ctx: cv.getContext('2d')!, mini, next: 0, open: false, until: 0, grid: null, fs: 11, cw: 7, chh: 15, ox: 0, oy: 0, dpr: 1, W: 0, H: 0 };
  });

  // keep every opened pane inside the row: edge panes open inward
  function place() {
    const rw = row.clientWidth;
    for (const p of panes) {
      const cell = p.el.parentElement!, cx = cell.offsetLeft + cell.offsetWidth / 2, sw = p.el.offsetWidth;
      const shift = Math.max(sw / 2 - cx, Math.min(0, rw - cx - sw / 2));
      p.el.style.setProperty('--SHIFT', String(Math.round(shift)));
      p.grid = null; // re-measure the screen on next open
    }
  }

  function layout(p: P) {
    p.dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = p.el.offsetWidth, h = p.el.offsetHeight;
    p.W = Math.round(w * p.dpr); p.H = Math.round(h * p.dpr);
    p.cv.width = p.W; p.cv.height = p.H;
    p.fs = Math.max(8.5, Math.min(11.5, w / 33));
    p.ctx.font = `450 ${p.fs}px "Spline Sans Mono Variable", ui-monospace, monospace`;
    p.cw = p.ctx.measureText('0').width || p.fs * 0.6; p.chh = Math.round(p.fs * 1.42);
    const cols = Math.max(20, Math.floor((w - 20) / p.cw)), rows = Math.max(8, Math.floor((h - 16) / p.chh));
    p.grid = new Grid(cols, rows);
    p.ox = Math.round((w - cols * p.cw) / 2); p.oy = Math.round((h - rows * p.chh) / 2);
  }

  function draw(p: P, now: number) {
    if (!p.grid) layout(p);
    const g = p.grid!;
    g.ch.fill(' '); g.bg.fill(''); g.bold.fill(false);
    p.mini.draw(g, now);
    const c = p.ctx;
    c.setTransform(1, 0, 0, 1, 0, 0); c.clearRect(0, 0, p.W, p.H);
    c.setTransform(p.dpr, 0, 0, p.dpr, p.ox * p.dpr, p.oy * p.dpr);
    render(c, g, { cw: p.cw, chh: p.chh, fs: p.fs, colors });
  }

  let lastDraw = 0;
  function frame(now: number) {
    raf = 0;
    let any = false;
    for (const p of panes) {
      if (!p.open && now > p.until) continue;
      any = true;
      if (!reduced) {
        if (now - p.next > 1500) p.next = now;
        while (now >= p.next) { p.mini.step(now); p.next += p.mini.every; }
      }
    }
    // the screens draw at 30 Hz: what a terminal at a brisk refresh looks like
    if (any && now - lastDraw >= 32) { lastDraw = now; for (const p of panes) if (p.open || now <= p.until) draw(p, now); }
    if (any && rowLive && !reduced) raf = requestAnimationFrame(frame);
  }
  const kick = () => { if (!raf) raf = requestAnimationFrame(frame); };

  function setOpen(p: P, open: boolean) {
    if (p.open === open) return;
    p.open = open;
    p.el.classList.toggle('open', open);
    p.glass.setAttribute('aria-expanded', String(open));
    if (open) { for (const q of panes) if (q !== p && q.open) setOpen(q, false); p.next = performance.now(); draw(p, performance.now()); }
    else p.until = performance.now() + 500; // keep drawing while it closes
    kick();
  }

  for (const p of panes) {
    const g = p.glass;
    g.setAttribute('aria-expanded', 'false');
    g.addEventListener('pointerenter', (e) => { if (e.pointerType === 'mouse') setOpen(p, true); });
    g.addEventListener('mousedown', (e) => e.preventDefault()); // pointers open by hover or tap; focus is for the keyboard
    g.addEventListener('pointerleave', (e) => { if (e.pointerType === 'mouse' && document.activeElement !== g) setOpen(p, false); });
    g.addEventListener('click', (e) => { if ((e as PointerEvent).pointerType !== 'mouse') setOpen(p, !p.open); });
    g.addEventListener('focus', () => setOpen(p, true));
    g.addEventListener('blur', () => setOpen(p, false));
    g.addEventListener('keydown', (e) => { if (e.key === 'Escape') { setOpen(p, false); g.blur(); } });
  }
  // a tap outside closes a tapped-open pane
  document.addEventListener('pointerdown', (e) => { if (!(e.target as Element).closest?.('.lp-glass')) for (const p of panes) if (p.open && document.activeElement !== p.glass) setOpen(p, false); });

  place();
  new ResizeObserver(place).observe(row);
  liveWhen(row, (l) => { rowLive = l; row.classList.toggle('live', l); if (l) kick(); });
  onTheme(() => { colors = readColors(); for (const p of panes) if (p.open) draw(p, performance.now()); });
  if (document.fonts) document.fonts.load('450 12px "Spline Sans Mono Variable"').then(() => { for (const p of panes) p.grid = null; }).catch(() => {});
}
