// Shared runtime helpers: reduced motion, the theme, and "is this worth animating right now"
// (on screen and the tab visible). Every animated thing on the site goes through `liveWhen` so it
// stops costing anything the moment nobody can see it.

export const reducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

/** Calls fn() whenever the theme changes (the toggle, or the system scheme under auto). */
export function onTheme(fn: () => void) {
  addEventListener('fictty:theme', fn);
  matchMedia('(prefers-color-scheme: dark)').addEventListener('change', fn);
}

/** Tracks whether el is on screen and the page is visible; calls fn(live) on every change. */
export function liveWhen(el: Element, fn: (live: boolean) => void, rootMargin = '80px') {
  let onScreen = false, last: boolean | null = null;
  const emit = () => { const live = onScreen && !document.hidden; if (live !== last) { last = live; fn(live); } };
  const io = new IntersectionObserver((es) => { for (const e of es) onScreen = e.isIntersecting; emit(); }, { rootMargin });
  io.observe(el);
  document.addEventListener('visibilitychange', emit);
  return () => { io.disconnect(); document.removeEventListener('visibilitychange', emit); };
}
