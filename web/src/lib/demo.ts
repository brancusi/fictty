// The scripted demo: one screen from first push to thrown away. Frames are composed in cells so
// every column lines up, then turned into HTML spans. Pure functions: the page renders the first
// frame at build time and the client steps through the rest.

type Seg = [string, string];
type Line = Seg[];
const esc = (t: string) => t.replace(/&/g, '&amp;').replace(/</g, '&lt;');
const W = 79;
const S = (t: string, c = ''): Seg => [t, c];
const wlen = (line: Line) => line.reduce((n, [t]) => n + [...t].length, 0);
const padTo = (line: Line, w: number, c = ''): Line => { const n = wlen(line); return n < w ? [...line, S(' '.repeat(w - n), c)] : line; };

function box(title: string, w: number, rows: (Line | { line: Line; sel: boolean })[]): Line[] {
  const out: Line[] = [[S('╭─ ', 'd'), S(title, 'b'), S(' ' + '─'.repeat(Math.max(0, w - 5 - [...title].length)) + '╮', 'd')]];
  for (const r of rows) {
    const line = 'line' in r ? r.line : r, sel = 'sel' in r && r.sel ? ' sel' : '';
    const inner = padTo([S(' ', sel), ...line.map(([t, c]): Seg => [t, c + sel])], w - 3, sel);
    inner.push(S(' ', sel));
    out.push([S('│', 'd'), ...inner, S('│', 'd')]);
  }
  out.push([S('╰' + '─'.repeat(w - 2) + '╯', 'd')]);
  return out;
}
function hcat(a: Line[], b: Line[], gap = 1): Line[] {
  const n = Math.max(a.length, b.length), wa = Math.max(...a.map(wlen)), out: Line[] = [];
  for (let i = 0; i < n; i++) out.push([...padTo(a[i] || [], wa), S(' '.repeat(gap)), ...(b[i] || [])]);
  return out;
}
export const html = (lines: Line[]) => lines.map((l) => l.map(([t, c]) => (c.trim() ? `<span class="${c.trim()}">${esc(t)}</span>` : esc(t))).join('')).join('\n');
const titleBar = (left: string, right: Line): Line => { const l: Line = [S('▌', 'b'), S('fictty', 'w'), S(' · ' + left, 'd')]; return [...padTo(l, W - wlen(right)), ...right]; };

const SVC: [string, string, string, Seg][] = [
  ['api-1', 'eu-west', '112ms', ['● healthy', 'g']], ['api-2', 'eu-west', '948ms', ['▲ slow', 'y']], ['web', 'us-east', ' 64ms', ['● healthy', 'g']],
  ['worker', 'us-east', ' 31ms', ['● healthy', 'g']], ['cron', 'eu-west', '    -', ['○ idle', 'd']],
];
function deployFrame({ sel = 1, callout = false } = {}): Line[] {
  const rows: (Line | { line: Line; sel: boolean })[] = [[S('service   region    p95     status', 'd')]];
  SVC.forEach(([n, r, p, st], i) => rows.push({ line: [S(n.padEnd(10) + r.padEnd(10) + p.padEnd(8)), S(st[0], st[1])], sel: i === sel }));
  const ev: [string, string, string, string][] = [['14:02:11', 'api-1', 'healthy', 'g'], ['14:02:12', 'api-2', 'rollout 3/4', 'c'], ['14:02:14', 'web', 'healthy', 'g'], ['14:02:15', 'api-2', 'p95 948ms', 'y'], ['14:02:17', 'worker', 'queue 12', '']];
  const er: Line[] = [[S('time      pod     event', 'd')], ...ev.map(([t, p, e, c]): Line => [S(t + '  ', 'd'), S(p.padEnd(8)), S(e, c)])];
  const lines: Line[] = [titleBar('deploy · release 4.12', [S('● live', 'g'), S('  60 fps', 'd')]), [S('')]];
  lines.push(...hcat(box('Services', 44, rows), box('Events', 34, er)));
  if (callout) lines.push([S('     ╰─▶ ', 'b'), S('◆ agent', 'b'), S('  api-2 is 8× slower than the rest since rollout 3/4.')]);
  else lines.push([S('')]);
  lines.push([S('')], [S('j/k move · enter logs · r roll back · q close', 'd')]);
  return lines;
}
function confirmFrame(pressed = false): Line[] {
  const w = 48, padL = ' '.repeat(Math.floor((W - w) / 2));
  const b = box('Roll back?', w, [
    [S('api-2  '), S('4.12', 'y'), S(' → '), S('4.11', 'g')], [S('p95 948ms since 14:02 · 3 of 4 pods', 'd')], [S('')],
    [S('[y] roll back', pressed ? 'cur' : 'b'), S('        '), S('[n] keep 4.12', 'd')],
  ]);
  const lines: Line[] = [titleBar('deploy · question', [S('waiting for you', 'y')]), [S('')], [S('')]];
  b.forEach((l) => lines.push([S(padL), ...l]));
  lines.push([S('')], [S('')], [S('y/n answer · esc dismiss', 'd')]);
  return lines;
}
function doneFrame(): Line[] {
  return [titleBar('deploy', [S('done', 'g')]), [S('')], [S('  ✓ ', 'g'), S('api-2 rolled back to 4.11')], [S('    p95 118ms · 4 of 4 pods healthy', 'd')], [S('')], [S('  this screen closes in 3s', 'd')]];
}

export interface Step { cmd: string; note: string; frame: () => Line[]; readback?: string }
export const STEPS: Step[] = [
  { cmd: '$ fictty push deploy.json', note: 'A screen from one value. Rows stream from kubectl, never through the model.', frame: () => deployFrame({ sel: 0 }) },
  { cmd: '· person presses j', note: 'Keys run locally, at frame rate.', frame: () => deployFrame({ sel: 1 }) },
  { cmd: '$ fictty patch note callout.json', note: 'The agent replaces one node by id. The selection stays put.', frame: () => deployFrame({ sel: 1, callout: true }) },
  { cmd: '$ fictty screen', note: 'It reads back what it drew, as text.', frame: () => deployFrame({ sel: 1, callout: true }), readback: '│ api-2     eu-west   948ms   ▲ slow\n     ╰─▶ ◆ agent  api-2 is 8× slower…' },
  { cmd: '$ fictty push confirm.json', note: 'The question replaces the dashboard in one frame.', frame: () => confirmFrame(false) },
  { cmd: '· person presses y', note: 'The key runs the command the agent bound to it.', frame: () => confirmFrame(true) },
  { cmd: '$ fictty push done.json', note: 'Done. The screen is thrown away.', frame: () => doneFrame() },
];
export const frameHtml = (i: number) => html(STEPS[i].frame());
