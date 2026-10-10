// A tiny terminal: a grid of cells (glyph, foreground, background, bold) and a canvas renderer.
// Box-drawing and block elements are drawn as geometry, so borders meet and bars stay crisp at any
// size and in any font; everything else is text in Spline Sans Mono, centred in its cell.

export type Ink = 'fg' | 'dim' | 'faint' | 'cyan' | 'violet' | 'amber' | 'rose' | 'green' | 'blue' | 'bright' | 'sel' | 'flash' | 'cur' | '';

export class Grid {
  cols: number; rows: number;
  ch: string[]; fg: Ink[]; bg: Ink[]; bold: boolean[];
  constructor(cols: number, rows: number) {
    this.cols = cols; this.rows = rows;
    const n = cols * rows;
    this.ch = new Array(n).fill(' '); this.fg = new Array(n).fill('fg'); this.bg = new Array(n).fill(''); this.bold = new Array(n).fill(false);
  }
  /** Writes text at (x, y); clips at the grid edge (or at maxX). Returns the column after it. */
  put(x: number, y: number, text: string, fg: Ink = 'fg', bg: Ink | null = null, bold = false, maxX = this.cols): number {
    if (y < 0 || y >= this.rows || !text) return x;
    for (const c of text) {
      if (x >= maxX) break;
      if (x >= 0) { const i = y * this.cols + x; this.ch[i] = c; this.fg[i] = fg; if (bg !== null) this.bg[i] = bg; this.bold[i] = bold; }
      x++;
    }
    return x;
  }
  /** Right-aligns text so it ends at column `end` (exclusive). */
  putR(end: number, y: number, text: string, fg: Ink = 'fg', bg: Ink | null = null, bold = false) {
    return this.put(end - [...text].length, y, text, fg, bg, bold);
  }
  paint(x: number, y: number, w: number, bg: Ink) {
    if (y < 0 || y >= this.rows) return;
    for (let i = Math.max(0, x); i < Math.min(this.cols, x + w); i++) this.bg[y * this.cols + i] = bg;
  }
  /** A rounded box with a title in the top border, like a fictty node with `title` and `border`. */
  box(x: number, y: number, w: number, h: number, title = '', right = '', rightInk: Ink = 'dim') {
    const x1 = x + w - 1, y1 = y + h - 1;
    for (let i = x + 1; i < x1; i++) { this.put(i, y, '─', 'faint'); this.put(i, y1, '─', 'faint'); }
    for (let j = y + 1; j < y1; j++) { this.put(x, j, '│', 'faint'); this.put(x1, j, '│', 'faint'); }
    this.put(x, y, '╭', 'faint'); this.put(x1, y, '╮', 'faint'); this.put(x, y1, '╰', 'faint'); this.put(x1, y1, '╯', 'faint');
    if (title) { this.put(x + 2, y, ' ', 'faint'); this.put(x + 3, y, title, 'violet', null, true, x1 - 1); this.put(x + 3 + [...title].length, y, ' ', 'faint'); }
    // the right-hand label only where it fits beside the title
    if (right && [...title].length + [...right].length + 8 <= w) this.putR(x1 - 1, y, ' ' + right + ' ', rightInk);
  }
}

export type Colors = Record<Exclude<Ink, ''>, string>;
export function readColors(el: Element = document.documentElement): Colors {
  const cs = getComputedStyle(el), v = (n: string) => cs.getPropertyValue(n).trim();
  return {
    fg: v('--term-fg'), dim: v('--ansi-8'), faint: v('--line-strong'), cyan: v('--ansi-6'), violet: v('--ansi-5'), amber: v('--ansi-3'),
    rose: v('--ansi-1'), green: v('--ansi-2'), blue: v('--ansi-4'), bright: v('--ink'), sel: v('--term-sel'), flash: v('--term-flash'), cur: v('--term-cursor'),
  };
}

// partial blocks: k/8 from the bottom, from the left; and our own top-hanging ones for negative bars
const LOWER = ' ▁▂▃▄▅▆▇█', LEFT = ' ▏▎▍▌▋▊▉█';
export const UPPER = (k: number) => String.fromCharCode(0xe000 + Math.max(0, Math.min(8, k)));

export interface RenderOpts { cw: number; chh: number; fs: number; colors: Colors }
export function render(ctx: CanvasRenderingContext2D, g: Grid, { cw, chh, fs, colors }: RenderOpts) {
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  const lw = Math.max(1, Math.round(fs * 0.075));
  let curFont = '';
  // backgrounds first, merged into runs
  for (let y = 0; y < g.rows; y++) {
    let x = 0;
    while (x < g.cols) {
      const b = g.bg[y * g.cols + x];
      if (!b) { x++; continue; }
      let e = x; while (e + 1 < g.cols && g.bg[y * g.cols + e + 1] === b) e++;
      ctx.fillStyle = colors[b]; ctx.fillRect(Math.round(x * cw), Math.round(y * chh), Math.round((e + 1) * cw) - Math.round(x * cw), Math.round((y + 1) * chh) - Math.round(y * chh));
      x = e + 1;
    }
  }
  for (let y = 0; y < g.rows; y++) {
    for (let x = 0; x < g.cols; x++) {
      const i = y * g.cols + x, c = g.ch[i];
      if (c === ' ') continue;
      const f = g.fg[i]; if (!f) continue;
      const X = x * cw, Y = y * chh, col = colors[f];
      ctx.fillStyle = col; ctx.strokeStyle = col;
      const code = c.charCodeAt(0);
      const lo = LOWER.indexOf(c), le = LEFT.indexOf(c);
      // blocks snap outward to whole pixels so neighbouring cells meet without seams
      const bx = Math.floor(X), bw = Math.ceil(X + cw) - bx, by = Math.floor(Y), bh = Math.ceil(Y + chh) - by;
      if (lo > 0) { const h = Math.round((bh * lo) / 8); ctx.fillRect(bx, by + bh - h, bw, h); continue; }
      if (le > 0) { ctx.fillRect(bx, by, Math.round((cw * le) / 8), bh); continue; }
      if (code >= 0xe000 && code <= 0xe008) { ctx.fillRect(bx, by, bw, Math.round((bh * (code - 0xe000)) / 8)); continue; }
      if (c === '▀') { ctx.fillRect(bx, by, bw, Math.round(bh / 2)); continue; }
      if (c === '▐') { ctx.fillRect(Math.round(X + cw / 2), by, bx + bw - Math.round(X + cw / 2), bh); continue; }
      if (c === '░' || c === '▒' || c === '▓') { ctx.globalAlpha = c === '░' ? 0.22 : c === '▒' ? 0.45 : 0.7; ctx.fillRect(X, Y, cw + 0.5, chh + 0.5); ctx.globalAlpha = 1; continue; }
      if (code >= 0x2500 && code <= 0x257f) { boxGlyph(ctx, c, X, Y, cw, chh, c === '━' || c === '┃' ? lw * 2 : lw); continue; }
      const font = `${g.bold[i] ? 600 : 450} ${fs}px "Spline Sans Mono Variable", ui-monospace, monospace`;
      if (font !== curFont) { ctx.font = font; curFont = font; }
      ctx.fillText(c, X + cw / 2, Y + chh / 2 + fs * 0.04);
    }
  }
}

function boxGlyph(ctx: CanvasRenderingContext2D, c: string, X: number, Y: number, w: number, h: number, lw: number) {
  const mx = Math.round(X + w / 2 - lw / 2) + lw / 2, my = Math.round(Y + h / 2 - lw / 2) + lw / 2, r = Math.min(w, h) * 0.5;
  const R = X + w + 0.5, B = Y + h + 0.5, L = X - 0.5, T = Y - 0.5;
  ctx.lineWidth = lw; ctx.beginPath();
  const H = (a: number, b: number) => { ctx.moveTo(a, my); ctx.lineTo(b, my); };
  const V = (a: number, b: number) => { ctx.moveTo(mx, a); ctx.lineTo(mx, b); };
  switch (c) {
    case '─': case '━': H(L, R); break;
    case '│': case '┃': V(T, B); break;
    case '╭': ctx.moveTo(R, my); ctx.arcTo(mx, my, mx, B, r); ctx.lineTo(mx, B); break;
    case '╮': ctx.moveTo(L, my); ctx.arcTo(mx, my, mx, B, r); ctx.lineTo(mx, B); break;
    case '╰': ctx.moveTo(R, my); ctx.arcTo(mx, my, mx, T, r); ctx.lineTo(mx, T); break;
    case '╯': ctx.moveTo(L, my); ctx.arcTo(mx, my, mx, T, r); ctx.lineTo(mx, T); break;
    case '┌': H(mx, R); V(my, B); break;
    case '┐': H(L, mx); V(my, B); break;
    case '└': H(mx, R); V(T, my); break;
    case '┘': H(L, mx); V(T, my); break;
    case '├': V(T, B); H(mx, R); break;
    case '┤': V(T, B); H(L, mx); break;
    case '┬': H(L, R); V(my, B); break;
    case '┴': H(L, R); V(T, my); break;
    case '┼': H(L, R); V(T, B); break;
    case '╌': case '┄': ctx.setLineDash([w * 0.3, w * 0.2]); H(L, R); break;
    default: H(L, R);
  }
  ctx.stroke(); ctx.setLineDash([]);
}

/** fictty's big font: glyphs five pixels tall, drawn as three rows of half blocks. */
const BIG: Record<string, string[]> = {
  '0': ['###', '#.#', '#.#', '#.#', '###'], '1': ['.#.', '##.', '.#.', '.#.', '###'], '2': ['###', '..#', '###', '#..', '###'],
  '3': ['###', '..#', '.##', '..#', '###'], '4': ['#.#', '#.#', '###', '..#', '..#'], '5': ['###', '#..', '###', '..#', '###'],
  '6': ['###', '#..', '###', '#.#', '###'], '7': ['###', '..#', '..#', '.#.', '.#.'], '8': ['###', '#.#', '###', '#.#', '###'],
  '9': ['###', '#.#', '###', '..#', '###'], '.': ['.', '.', '.', '.', '#'], ',': ['.', '.', '.', '#', '#'], '%': ['#.#', '..#', '.#.', '#..', '#.#'],
  '+': ['...', '.#.', '###', '.#.', '...'], '-': ['...', '...', '###', '...', '...'], '$': ['.##', '##.', '.#.', '.##', '##.'], 'K': ['#.#', '#.#', '##.', '#.#', '#.#'],
};
export function big(s: string): [string, string, string] {
  const out: [string, string, string] = ['', '', ''];
  [...s].forEach((c, i) => {
    const g = BIG[c] || ['..', '..', '..', '..', '..'], w = g[0].length;
    if (i > 0) for (let r = 0; r < 3; r++) out[r] += ' ';
    for (let r = 0; r < 3; r++) {
      const top = g[r * 2], bot = g[r * 2 + 1] || '.'.repeat(w);
      for (let x = 0; x < w; x++) out[r] += top[x] === '#' ? (bot[x] === '#' ? '█' : '▀') : bot[x] === '#' ? '▄' : ' ';
    }
  });
  return out;
}
