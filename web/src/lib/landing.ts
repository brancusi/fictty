// The landing page's smaller motions: the three steps sliding apart from the centre line as they
// scroll in, the background panes drifting in mirrored pairs, the scripted demo, and the alphabet
// tiles swapping in mirrored pairs. Scroll work is read-then-write in one rAF, eased toward its
// target so it never snaps, and stops when it settles.
import { frameHtml, STEPS } from './demo';
import { liveWhen, reducedMotion } from './live';

const reduced = reducedMotion();
const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);
const inOut = (x: number) => (x < 0.5 ? 4 * x * x * x : 1 - Math.pow(-2 * x + 2, 3) / 2);

function scrollMotion() {
  const site = document.getElementById('site')!, steps = document.getElementById('steps')!;
  const cards = [...steps.children] as HTMLElement[], ghosts = [...site.querySelectorAll<HTMLElement>('.ghost')];
  if (reduced) return;
  let spread = 0, spreadT = 0, prog = 0, progT = 0, raf = 0, last = 0, step = 0;
  const m = 24;
  const measure = () => { step = cards.length > 1 ? cards[1].offsetLeft - cards[0].offsetLeft : 0; };
  const targets = () => {
    const vh = innerHeight, r = steps.getBoundingClientRect(), sr = site.getBoundingClientRect();
    spreadT = innerWidth <= 860 ? 1 : clamp01((vh * 0.95 - r.top) / (vh * 0.5));
    progT = clamp01(-sr.top / Math.max(1, sr.height - vh));
  };
  const tick = (now: number) => {
    raf = 0;
    const dt = Math.min(0.05, (now - (last || now)) / 1000); last = now;
    const k = 1 - Math.exp(-dt * 10);
    spread += (spreadT - spread) * k; prog += (progT - prog) * k;
    if (Math.abs(spreadT - spread) < 0.001) spread = spreadT;
    if (Math.abs(progT - prog) < 0.0005) prog = progT;
    const e = inOut(spread), wide = innerWidth > 860;
    cards.forEach((c, i) => {
      const side = i - 1; // -1, 0, 1: the outer cards start stacked on the middle one
      c.style.transform = wide && side ? `translate3d(${(-side * (1 - e) * step).toFixed(1)}px, ${(side * e * m).toFixed(1)}px, 0)` : '';
      c.style.opacity = wide && side ? String(0.35 + 0.65 * e) : '';
    });
    ghosts.forEach((g) => {
      const n = +g.dataset.g!, dir = n % 2 ? -1 : 1, amp = n < 2 ? 140 : 90;
      g.style.transform = `translate3d(${(dir * (prog - 0.3) * amp).toFixed(1)}px, ${(-prog * 140).toFixed(1)}px, 0)`;
    });
    if (spread !== spreadT || prog !== progT) raf = requestAnimationFrame(tick); else last = 0;
  };
  const onScroll = () => { targets(); if (!raf) raf = requestAnimationFrame(tick); };
  addEventListener('scroll', onScroll, { passive: true });
  addEventListener('resize', () => { measure(); onScroll(); });
  measure(); targets(); spread = spreadT; prog = progT; tick(performance.now());
}

function demo() {
  const frame = document.getElementById('demoFrame')!, log = document.getElementById('demoLog')!;
  const pauseBtn = document.getElementById('demoPause') as HTMLButtonElement, state = document.getElementById('demoState')!;
  const items = [...log.querySelectorAll('li')];
  let i = 0, paused = reduced, timer = 0, live = false;
  const show = (n: number) => {
    i = n; frame.innerHTML = frameHtml(n);
    items.forEach((li, j) => {
      li.classList.toggle('past', j < n);
      const b = li.querySelector('button')!;
      if (j === n) b.setAttribute('aria-current', 'step'); else b.removeAttribute('aria-current');
      const rb = li.querySelector<HTMLElement>('.readback'); if (rb) rb.hidden = j !== n;
    });
  };
  const sync = () => { pauseBtn.textContent = paused ? 'play' : 'pause'; state.textContent = paused ? `step ${i + 1} of ${STEPS.length}` : 'auto'; };
  const tick = () => {
    clearTimeout(timer);
    if (paused || !live) return;
    timer = window.setTimeout(() => { show((i + 1) % STEPS.length); sync(); tick(); }, i === STEPS.length - 1 ? 3600 : 2700);
  };
  log.addEventListener('click', (e) => {
    const b = (e.target as Element).closest<HTMLButtonElement>('button[data-step]'); if (!b) return;
    paused = true; show(+b.dataset.step!); sync(); tick();
  });
  pauseBtn.addEventListener('click', () => { paused = !paused; sync(); tick(); });
  liveWhen(frame, (l) => { live = l; tick(); });
  sync();
}

function blocks() {
  const host = document.getElementById('blocks')!;
  if (reduced) return;
  const easing = CSS.supports('transition-timing-function', 'linear(0, 1)') ? getComputedStyle(document.documentElement).getPropertyValue('--spring').trim() || 'cubic-bezier(.65,0,.35,1)' : 'cubic-bezier(.65,0,.35,1)';
  let timer = 0, n = 0, live = false;
  const swap = () => {
    const els = [...host.children] as HTMLElement[], cols = getComputedStyle(host).gridTemplateColumns.split(' ').length, half = Math.floor(cols / 2);
    if (!half) return;
    const first = new Map(els.map((e) => [e, e.getBoundingClientRect()]));
    const order = els.slice(), rows = Math.ceil(order.length / cols);
    // swap a mirrored pair in one row and its point reflection in the opposite row: rotational symmetry
    const c = n % half, r = Math.floor(n / half) % rows; n++;
    const sw = (a: number, b: number) => { const t = order[a]; order[a] = order[b]; order[b] = t; };
    sw(r * cols + c, r * cols + (cols - 1 - c));
    const r2 = rows - 1 - r; if (r2 !== r) sw(r2 * cols + c, r2 * cols + (cols - 1 - c));
    order.forEach((e) => host.append(e));
    for (const e of order) {
      const a = first.get(e)!, b = e.getBoundingClientRect(), dx = a.left - b.left, dy = a.top - b.top;
      if (!dx && !dy) continue;
      e.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: 'none' }], { duration: 1100, easing });
    }
  };
  const tick = () => { clearTimeout(timer); if (!live) return; timer = window.setTimeout(() => { swap(); tick(); }, 4200); };
  liveWhen(host, (l) => { live = l; tick(); }, '0px');
}

export function initLanding() { scrollMotion(); demo(); blocks(); }
