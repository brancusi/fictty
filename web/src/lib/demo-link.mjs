// The demo link: a name, a date, the one line that runs it, and a copy button. One markup for
// pages (components/DemoLink.astro) and for Markdown, where a post writes
//   <div class="demo-link" data-name="…" data-date="2026-10-11" data-script="try/slug.sh" data-commit="7c918e6"></div>
// and rehype-links.mjs expands it. The copy button is wired by Base.astro (any [data-copy]).
const SITE = 'https://fictty.com';
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
const day = (d) => new Date(d).toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone: 'UTC' });

/** name, date (ISO), script (a path on the site: try.sh, try/slug.sh), commit (pinned) or latest */
export function demoHtml({ name, date, script, commit = '', latest = false, needs = 'git and Rust', compact = false }) {
  const cmd = `curl -fsSL ${SITE}/${script.replace(/^\//, '')} | sh`;
  const version = latest
    ? `<span class="dl-tag dl-latest">latest</span>`
    : commit
      ? `<span class="dl-tag">pinned · <a href="https://github.com/brancusi/fictty/commit/${esc(commit)}" rel="noopener">${esc(commit.slice(0, 7))}</a></span>`
      : '';
  return `<div class="demo-link${compact ? ' dl-compact' : ''}" role="group" aria-label="${esc(`Demo: ${name}`)}">
<div class="dl-head"><span class="dl-label">Demo</span><b class="dl-name">${esc(name)}</b>${date ? `<time class="dl-date" datetime="${esc(date)}">${esc(day(date))}</time>` : ''}${version}</div>
<div class="dl-cmd"><code><span class="d">$ </span>${esc(cmd)}</code><button type="button" data-copy="${esc(cmd)}" aria-label="Copy the demo command">copy</button></div>
${compact ? '' : `<p class="dl-foot">Paste it in a terminal: it runs right there. Needs ${esc(needs)}; everything goes in <code>~/.fictty</code>.</p>`}
</div>`;
}

/** The attributes of a Markdown placeholder, as demoHtml's props. */
export function demoProps(tag) {
  const get = (k) => tag.match(new RegExp(`data-${k}="([^"]*)"`))?.[1];
  return { name: get('name') || 'Demo', date: get('date'), script: get('script') || 'try.sh', commit: get('commit') || '', latest: get('latest') === 'true', needs: get('needs') || undefined };
}
