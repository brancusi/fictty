// Renders the social card (public/og.png, 1200x630) and the PNG icons (favicon-32.png,
// apple-touch-icon.png) from the real generator in src/lib/glass.ts, in headless Chrome, so the
// pictures and the live site are one drawing. Run after changing the identity:
//   npm run icons            (CHROME=/path/to/chrome to pick a browser)
// The outputs are committed (they're small); nothing here runs during the build.
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const chrome = process.env.CHROME || ['/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', '/usr/bin/google-chrome', '/usr/bin/chromium'].find(existsSync);
if (!chrome) { console.error('icons: no Chrome found (set CHROME)'); process.exit(1); }

const glass = stripTypeScriptTypes(readFileSync(path.join(root, 'src/lib/glass.ts'), 'utf8'));
const font = (p) => readFileSync(path.join(root, 'node_modules', p)).toString('base64');
const sans = font('@fontsource-variable/spline-sans/files/spline-sans-latin-wght-normal.woff2');
const mono = font('@fontsource-variable/spline-sans-mono/files/spline-sans-mono-latin-wght-normal.woff2');
// the tokens, from the site's stylesheet (one source of truth)
const css = readFileSync(path.join(root, 'src/styles/site.css'), 'utf8');
const block = (sel) => { const i = css.indexOf(sel); return css.slice(css.indexOf('{', i) + 1, css.indexOf('\n}', i)); };
const dayVars = block(':root {'), nightVars = block(':root[data-theme="dark"] {');

function page(theme, w, h, body, script) {
  return `<!doctype html><html data-theme="${theme}"><head><meta charset="utf-8"><style>
@font-face { font-family: "Spline Sans Variable"; src: url(data:font/woff2;base64,${sans}) format("woff2"); font-weight: 300 700; }
@font-face { font-family: "Spline Sans Mono Variable"; src: url(data:font/woff2;base64,${mono}) format("woff2"); font-weight: 300 700; }
:root { ${dayVars} }
:root[data-theme="dark"] { ${nightVars} }
html, body { margin: 0; width: ${w}px; height: ${h}px; overflow: hidden; }
body { background: ${theme === 'clear' ? 'transparent' : 'var(--ground)'}; color: var(--ink); font-family: var(--sans); -webkit-font-smoothing: antialiased; }
${body.css}
</style></head><body>${body.html}<script type="module">
${glass}
window.devicePixelRatio = 1;
await document.fonts.ready;
${script}
</script></body></html>`;
}

const jobs = [
  {
    out: 'public/og.png', w: 1200, h: 630, theme: 'dark',
    body: {
      css: `
.axis { position: absolute; left: 600px; top: 0; bottom: 0; width: 1px; background: linear-gradient(var(--line), transparent 30%, transparent 70%, var(--line)); }
canvas { position: absolute; left: 90px; top: -14px; width: 1020px; height: 450px; }
h1 { position: absolute; left: 0; right: 0; top: 404px; margin: 0; text-align: center; font-weight: 500; font-size: 50px; line-height: 1.08; letter-spacing: -0.025em; padding: 0 140px; text-wrap: balance; }
.url { position: absolute; left: 0; right: 0; bottom: 34px; text-align: center; font: 500 20px/1 var(--mono); color: var(--ink-3); letter-spacing: 0.06em; }`,
      html: `<div class="axis"></div><canvas id="c"></canvas><h1>Terminal screens agents write, patch and read back.</h1><div class="url">fictty.com</div>`,
    },
    script: `const cv = document.getElementById('c'); drawWord(cv, glassWord(CHOSEN), { pal: readPalette(), cols: 34, rows: 15, pad: [3, 3] });`,
  },
  {
    out: 'public/apple-touch-icon.png', w: 180, h: 180, theme: 'dark',
    body: { css: `canvas { position: absolute; left: 22px; top: 22px; width: 136px; height: 136px; }`, html: `<canvas id="c"></canvas>` },
    script: `const cv = document.getElementById('c'); cv.width = cv.height = 136; const ctx = cv.getContext('2d'); drawPanes(ctx, CURSOR, { s: 136 / 16, pal: readPalette(), detail: true, glow: 0.8 });`,
  },
  {
    out: 'public/favicon-32.png', w: 32, h: 32, theme: 'clear',
    body: { css: `canvas { position: absolute; inset: 0; width: 32px; height: 32px; }`, html: `<canvas id="c"></canvas>` },
    script: `const cv = document.getElementById('c'); cv.width = cv.height = 32; drawPanes(cv.getContext('2d'), CURSOR.map((p) => ({ ...p, alpha: 0.92 })), { s: 2, pal: readPalette(), detail: false, glow: 0 });`,
  },
];

const dir = mkdtempSync(path.join(tmpdir(), 'fictty-icons-'));
try {
  for (const j of jobs) {
    const file = path.join(dir, path.basename(j.out) + '.html');
    writeFileSync(file, page(j.theme, j.w, j.h, j.body, j.script));
    const out = path.join(root, j.out);
    execFileSync(chrome, [
      '--headless', '--disable-gpu', '--hide-scrollbars', '--force-device-scale-factor=1', `--window-size=${j.w},${j.h}`,
      '--default-background-color=00000000', '--virtual-time-budget=4000', `--user-data-dir=${path.join(dir, 'profile')}`,
      `--screenshot=${out}`, 'file://' + file,
    ], { stdio: 'ignore' });
    console.log(`icons: ${j.out} (${Math.round(readFileSync(out).length / 1024)} KB)`);
  }
} finally { rmSync(dir, { recursive: true, force: true }); }
