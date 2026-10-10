// Six tiny fictty screens, one per live glass pane, each built from a real component in fictty's
// alphabet (sparkline, table, gauge, list, bars, big). Each is a little state machine: `step`
// advances its made-up data on its own clock, `draw` lays it out for whatever grid the pane has.
// They are deliberately live: values tick, rows patch in by id, a patch flashes past.
import { big, Grid, UPPER, type Ink } from './cells';
import { rng } from './glass';

export interface Mini {
  every: number; // ms between data steps
  step(now: number): void;
  draw(g: Grid, now: number): void;
}

const LOWER = ' ▁▂▃▄▅▆▇█';
const pad2 = (n: number) => String(n).padStart(2, '0');
const clock = (s: number) => `${pad2(Math.floor(s / 3600) % 24)}:${pad2(Math.floor(s / 60) % 60)}:${pad2(s % 60)}`;
function status(g: Grid, keys: [string, string][], right: string, rightInk: Ink = 'green') {
  const y = g.rows - 1;
  let x = 1;
  for (const [k, what] of keys) {
    if (x + k.length + what.length + 2 > g.cols - right.length - 2) break;
    x = g.put(x, y, k, 'violet', null, true); x = g.put(x + 1, y, what, 'dim') + 2;
  }
  g.putR(g.cols - 1, y, right, rightInk);
}
function gauss(R: () => number) { return (R() + R() + R() - 1.5) * 1.15; }

// ---------- sparkline: a ticker, eight samples a second ----------
function sparkline(): Mini {
  const R = rng(7), vals: number[] = [];
  let price = 412.4;
  const open = 407.9;
  const push = () => { price += gauss(R) * 0.55 + (410 - price) * 0.012; vals.push(price); if (vals.length > 160) vals.shift(); };
  for (let i = 0; i < 160; i++) push();
  return {
    every: 125,
    step: push,
    draw(g) {
      const W = g.cols, H = g.rows - 1;
      g.box(0, 0, W, H, 'ACME', 'sparkline');
      const chg = ((price - open) / open) * 100, up = chg >= 0;
      let x = g.put(2, 1, price.toFixed(2), 'bright', null, true);
      g.put(x + 1, 1, `${up ? '▲' : '▼'} ${up ? '+' : ''}${chg.toFixed(2)}%`, up ? 'green' : 'rose');
      if (W > 34) g.putR(W - 2, 1, '8 Hz', 'dim');
      const top = 2, h = Math.max(2, H - 4), w = W - 4, data = vals.slice(-w);
      const lo = Math.min(...data), hi = Math.max(...data), span = Math.max(0.01, hi - lo);
      data.forEach((v, i) => {
        let e = Math.max(1, Math.round(((v - lo) / span) * (h * 8 - 1)) + 1);
        const ink: Ink = i === data.length - 1 ? 'amber' : 'cyan';
        for (let r = h - 1; r >= 0 && e > 0; r--) { g.put(2 + i, top + r, LOWER[Math.min(8, e)], ink); e -= 8; }
      });
      const ly = top + h;
      if (ly < H - 1) { x = g.put(2, ly, 'lo ', 'dim'); x = g.put(x, ly, lo.toFixed(2), 'fg'); x = g.put(x + 2, ly, 'hi ', 'dim'); g.put(x, ly, hi.toFixed(2), 'fg'); }
      status(g, [['r', 'refresh'], ['q', 'quit']], '● live');
    },
  };
}

// ---------- table: rows patched in by id ----------
interface Svc { id: string; region: string; p95: number; st: 'healthy' | 'slow' | 'idle'; born: number; touched: number; col: 'p95' | 'st' | 'row' | '' }
function table(): Mini {
  const R = rng(11);
  const rows: Svc[] = [
    { id: 'api-1', region: 'eu-west', p95: 112, st: 'healthy', born: -1e9, touched: -1e9, col: '' },
    { id: 'api-2', region: 'eu-west', p95: 948, st: 'slow', born: -1e9, touched: -1e9, col: '' },
    { id: 'web', region: 'us-east', p95: 64, st: 'healthy', born: -1e9, touched: -1e9, col: '' },
    { id: 'worker', region: 'us-east', p95: 31, st: 'healthy', born: -1e9, touched: -1e9, col: '' },
  ];
  const pool = ['queue', 'auth', 'search', 'mail', 'cdn', 'cron', 'billing'];
  let sel = 1, last = '', lastAt = -1e9, now = 0;
  return {
    every: 950,
    step(t) {
      now = t;
      const r = R();
      if (r < 0.45) {
        const s = rows[Math.floor(R() * rows.length)];
        s.p95 = Math.max(18, Math.round(s.st === 'slow' ? 700 + R() * 400 : s.p95 * (0.7 + R() * 0.6) + 4));
        s.touched = t; s.col = 'p95'; last = `patch ${s.id}.p95`;
      } else if (r < 0.6) {
        const s = rows[Math.floor(R() * rows.length)];
        s.st = s.st === 'slow' ? 'healthy' : R() < 0.5 ? 'slow' : 'idle';
        if (s.st === 'slow') s.p95 = 640 + Math.round(R() * 400); else if (s.p95 > 300) s.p95 = 90 + Math.round(R() * 60);
        s.touched = t; s.col = 'st'; last = `patch ${s.id}.status`;
      } else if (r < 0.78 && rows.length < 7) {
        const free = pool.filter((p) => !rows.some((s) => s.id === p));
        const id = free[Math.floor(R() * free.length)];
        rows.splice(1 + Math.floor(R() * rows.length), 0, { id, region: R() < 0.5 ? 'eu-west' : 'us-east', p95: 20 + Math.round(R() * 140), st: 'healthy', born: t, touched: t, col: 'row' });
        last = `insert ${id}`;
      } else if (r < 0.88 && rows.length > 3) {
        const i = rows.findIndex((_, k) => k !== sel && k > 0);
        last = `remove ${rows[i].id}`; rows.splice(i, 1); if (sel >= rows.length) sel = rows.length - 1;
      } else {
        sel = (sel + (R() < 0.6 ? 1 : -1) + rows.length) % rows.length; last = `key ${R() < 0.6 ? 'j' : 'k'}`;
      }
      lastAt = t;
    },
    draw(g, t) {
      const W = g.cols, H = g.rows - 1, wide = W >= 40;
      g.box(0, 0, W, H, 'Services', t - lastAt < 1400 ? last : 'table', t - lastAt < 1400 ? 'violet' : 'dim');
      const cN = 3, cR = 12, pEnd = wide ? 26 : 18, cS = pEnd + 2, inner = W - 1;
      g.put(cN, 1, 'service', 'dim'); if (wide) g.put(cR, 1, 'region', 'dim'); g.putR(pEnd, 1, 'p95', 'dim'); g.put(cS, 1, 'status', 'dim');
      rows.slice(0, H - 3).forEach((s, i) => {
        const y = 2 + i, on = i === sel, fresh = t - s.touched < 900;
        const bg: Ink | null = on ? 'sel' : s.col === 'row' && fresh ? 'flash' : null;
        if (bg) g.paint(1, y, inner - 1, bg);
        g.put(1, y, on ? '▸' : ' ', 'violet', bg);
        g.put(cN, y, s.id, on ? 'bright' : 'fg', bg, on);
        if (wide) g.put(cR, y, s.region, 'dim', bg);
        const p = `${s.p95}ms`;
        const pBg: Ink | null = s.col === 'p95' && fresh ? 'flash' : bg;
        g.putR(pEnd, y, p, s.p95 > 500 ? 'amber' : 'fg', pBg);
        const st: [string, Ink] = s.st === 'healthy' ? ['● healthy', 'green'] : s.st === 'slow' ? ['▲ slow', 'amber'] : ['○ idle', 'dim'];
        g.put(cS, y, st[0], st[1], s.col === 'st' && fresh ? 'flash' : bg, false, inner);
      });
      status(g, [['j/k', 'move'], ['enter', 'logs']], `${rows.length} rows`, 'dim');
    },
  };
}

// ---------- gauge: a rollout and two meters ----------
function gauge(): Mini {
  const R = rng(5);
  let roll = 0.18, hold = 0, cpu = 0.42, mem = 0.61, cpuT = 0.5, memT = 0.6, n = 0;
  return {
    every: 90,
    step() {
      n++;
      if (roll >= 1) { if (++hold > 24) { roll = 0; hold = 0; } } else roll = Math.min(1, roll + 0.0045 + R() * 0.004);
      if (n % 14 === 0) { cpuT = 0.2 + R() * 0.7; memT = 0.45 + R() * 0.35; }
      cpu += (cpuT - cpu) * 0.12 + gauss(R) * 0.01; mem += (memT - mem) * 0.05;
      cpu = Math.min(1, Math.max(0, cpu));
    },
    draw(g) {
      const W = g.cols, H = g.rows - 1;
      g.box(0, 0, W, H, 'deploy 4.12', 'gauge');
      const lab = 9, bw = W - lab - 9;
      const bar = (y: number, name: string, v: number, ink: Ink) => {
        g.put(2, y, name, 'dim');
        const cells = v * bw, full = Math.floor(cells), part = Math.min(7, Math.round((cells - full) * 8));
        for (let i = 0; i < bw; i++) g.put(2 + lab + i, y, i < full ? '█' : i === full && part > 0 ? ' ▏▎▍▌▋▊▉'[part] : '░', i <= full ? ink : 'faint');
        g.putR(W - 2, y, `${Math.round(v * 100)}%`.padStart(4), 'fg', null, true);
      };
      const gap = H >= 12 ? 2 : 1;
      let y = 2;
      bar(y, 'rollout', roll, roll >= 1 ? 'green' : 'violet'); y += gap;
      bar(y, 'cpu', cpu, cpu > 0.8 ? 'rose' : 'cyan'); y += gap;
      bar(y, 'mem', mem, 'amber'); y += gap;
      if (y < H - 1) {
        const pods = Math.min(4, Math.floor(roll * 4 + 0.001));
        let x = g.put(2, y, 'pods', 'dim'); x = 2 + lab;
        for (let i = 0; i < 4; i++) x = g.put(x, y, i < pods ? '● ' : '○ ', i < pods ? 'green' : 'dim');
        g.put(x + 1, y, roll >= 1 ? '✓ done' : `${pods} of 4`, roll >= 1 ? 'green' : 'fg');
      }
      status(g, [['r', 'roll back'], ['p', 'pause']], roll >= 1 ? 'done' : 'rolling', roll >= 1 ? 'green' : 'amber');
    },
  };
}

// ---------- list: a log tail, following ----------
function logtail(): Mini {
  const R = rng(3);
  const pods = ['api-1', 'api-2', 'web', 'worker', 'queue', 'auth'];
  const kinds: [string, Ink][] = [['healthy', 'green'], ['ready', 'green'], ['p95 {n}ms', 'amber'], ['rollout {k}/4', 'cyan'], ['queue {q}', 'fg'], ['restart', 'rose'], ['GET /v1/orders 200', 'fg'], ['cache hit {p}%', 'blue']];
  const lines: { t: number; pod: string; text: string; ink: Ink; at: number }[] = [];
  let s = 14 * 3600 + 2 * 60 + 11, total = 0;
  const add = (at: number) => {
    s += 1 + Math.floor(R() * 3); total++;
    const [k, ink] = kinds[Math.floor(R() * kinds.length)];
    const text = k.replace('{n}', String(300 + Math.floor(R() * 700))).replace('{k}', String(1 + Math.floor(R() * 4))).replace('{q}', String(Math.floor(R() * 40))).replace('{p}', String(80 + Math.floor(R() * 19)));
    lines.push({ t: s, pod: pods[Math.floor(R() * pods.length)], text, ink, at });
    if (lines.length > 40) lines.shift();
  };
  for (let i = 0; i < 14; i++) add(-1e9);
  return {
    every: 620,
    step(t) { add(t); if (R() < 0.3) add(t); },
    draw(g, t) {
      const W = g.cols, H = g.rows - 1;
      g.box(0, 0, W, H, 'events', 'list · follow');
      const n = H - 2, shown = lines.slice(-n), narrow = W < 34;
      shown.forEach((l, i) => {
        const y = 1 + i + (n - shown.length), fresh = t - l.at < 700;
        const bg: Ink | null = fresh ? 'flash' : null;
        if (bg) g.paint(1, y, W - 2, bg);
        let x = 2;
        if (!narrow) x = g.put(x, y, clock(l.t), 'dim', bg) + 1;
        x = g.put(x, y, l.pod.padEnd(7), 'fg', bg) + 1;
        g.put(x, y, l.text, l.ink, bg, false, W - 1);
      });
      status(g, [['f', 'follow'], ['/', 'filter']], `${total} rows`, 'dim');
    },
  };
}

// ---------- bars: returns, negatives hanging below zero ----------
function bars(): Mini {
  const R = rng(13);
  const names = ['tech', 'enrg', 'hlth', 'fin', 'util', 'matl', 'cons'];
  const v = names.map(() => gauss(R) * 2), tgt = v.slice();
  let n = 0;
  return {
    every: 70,
    step() {
      if (n++ % 22 === 0) for (let i = 0; i < tgt.length; i++) tgt[i] = Math.max(-3.6, Math.min(4.2, tgt[i] * 0.4 + gauss(R) * 2.2));
      for (let i = 0; i < v.length; i++) v[i] += (tgt[i] - v[i]) * 0.14;
    },
    draw(g) {
      const W = g.cols, H = g.rows - 1;
      g.box(0, 0, W, H, 'returns 1D', 'bars');
      const k = v.length, inner = W - 4, bw = Math.max(1, Math.floor((inner - (k - 1)) / k)), used = bw * k + (k - 1), x0 = 2 + Math.floor((inner - used) / 2);
      const top = 1, h = H - 4, lo = -4, hi = 4.5, zero = top + Math.round((hi / (hi - lo)) * h);
      for (let x = 2; x < W - 2; x++) g.put(x, zero, '─', 'faint');
      v.forEach((val, i) => {
        const x = x0 + i * (bw + 1), ink: Ink = val >= 0 ? 'cyan' : 'rose';
        if (val >= 0) {
          let e = Math.round((val / hi) * (zero - top) * 8);
          for (let r = zero - 1; r >= top && e > 0; r--) { for (let c = 0; c < bw; c++) g.put(x + c, r, LOWER[Math.min(8, e)], ink); e -= 8; }
        } else {
          let e = Math.round((-val / -lo) * (top + h - 1 - zero) * 8);
          for (let r = zero + 1; r < top + h && e > 0; r++) { for (let c = 0; c < bw; c++) g.put(x + c, r, UPPER(Math.min(8, e)), ink); e -= 8; }
        }
        const lab = bw >= 4 ? names[i] : names[i].slice(0, Math.max(1, bw));
        g.put(x + Math.max(0, Math.floor((bw - lab.length) / 2)), top + h, lab, 'dim');
        if (bw >= 5) { const t = `${val >= 0 ? '+' : ''}${val.toFixed(1)}`; g.put(x + Math.max(0, Math.floor((bw - t.length) / 2)), val >= 0 ? zero + 1 : zero - 1, t, val >= 0 ? 'green' : 'rose'); }
      });
      status(g, [['←/→', 'sector'], ['t', 'tabs']], '● 10 Hz');
    },
  };
}

// ---------- big: a KPI, patched by id, then read back ----------
function kpi(): Mini {
  const R = rng(17);
  let orders = 1284, shown = 1284, phase = 0, at = 0, changedAt = -1e9;
  const fmt = (n: number) => n.toLocaleString('en-US');
  return {
    every: 100,
    step(t) {
      // 0 hold → 1 type the patch → 2 the JSON flashes and the node updates → 3 read back
      const d = t - at;
      if (phase === 0 && d > 1300) { phase = 1; at = t; orders += 3 + Math.floor(R() * 9); }
      else if (phase === 1 && d > 650) { phase = 2; at = t; shown = orders; changedAt = t; }
      else if (phase === 2 && d > 900) { phase = 3; at = t; }
      else if (phase === 3 && d > 1500) { phase = 0; at = t; }
    },
    draw(g, t) {
      const W = g.cols, H = g.rows - 1, boxH = Math.min(H - 3, 7);
      g.box(0, 0, W, boxH, 'orders today', 'big · id kpi');
      const b = big(fmt(shown)), fresh = t - changedAt < 900;
      const bx = Math.max(2, Math.floor((W - [...b[0]].length) / 2)), by = 1 + Math.max(0, Math.floor((boxH - 2 - 3) / 2));
      b.forEach((row, i) => g.put(bx, by + i, row, fresh ? 'amber' : 'bright'));
      let y = boxH;
      const cmd = '$ fictty patch kpi';
      const json = `{"type":"big","id":"kpi","text":"${fmt(orders)}"}`;
      const d = t - at;
      if (y < H) {
        if (phase === 1) g.put(1, y, cmd.slice(0, Math.min(cmd.length, Math.floor(d / 22))), 'fg');
        else if (phase >= 2) g.put(1, y, cmd, phase === 3 ? 'dim' : 'fg');
        else g.put(1, y, '$ ', 'dim');
      }
      y++;
      if (y < H && phase >= 2) g.put(1, y, json, phase === 2 ? 'violet' : 'dim', phase === 2 && d < 500 ? 'flash' : null, false, W - 1);
      y++;
      if (y < H && phase === 3) { const x = g.put(1, y, '$ fictty screen', 'fg'); g.put(x, y, '', 'fg'); }
      y++;
      if (y < H && phase === 3) g.put(1, y, `│ ${fmt(shown)} orders today`, 'green', null, false, W - 1);
      status(g, [['patch', 'by id'], ['screen', 'read back']], phase === 2 ? 'patched' : '● live', phase === 2 ? 'violet' : 'green');
    },
  };
}

export const MINIS: Record<string, () => Mini> = { sparkline, table, gauge, list: logtail, bars, big: kpi };
