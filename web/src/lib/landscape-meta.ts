// The landscape's vocabulary: relations, scores, surfaces and fictty's own scores. Pure data, shared
// by the build (pages) and the browser (the radar, map and board on /research/).

export const FIELDS = ['slug', 'name', 'cluster', 'kind', 'surface', 'layer', 'maker', 'url', 'oneLiner', 'status', 'codeVsData', 'ephemerality', 'readback', 'liveData', 'learning', 'relation', 'threat', 'adoptFit', 'takeaway', 'evidence'] as const;
export interface Entry {
  slug: string; name: string; cluster: string; kind: string; surface: string; layer: string; maker: string; url: string;
  oneLiner: string; status: string; codeVsData: number; ephemerality: number; readback: number; liveData: number; learning: number;
  relation: string; threat: number; adoptFit: number; takeaway: string; evidence: string; id: string;
}
export interface Cluster { key: string; title: string; summary: string }

export const SHORT: Record<string, string> = { frame: 'Frames', harness: 'Harnesses', framework: 'Frameworks', 'terminal-gen': 'Terminal gen-UI', html: 'HTML camp', data: 'Data camp', ideas: 'Ideas' };
export const RELS: [string, string][] = [
  ['adopt', 'Speak it natively, as an input fictty accepts.'],
  ['build-on', 'A dependency fictty runs on or lifts code from.'],
  ['complement', 'Works beside fictty. Integrate, report status, ship recipes.'],
  ['compete', "Does part of fictty's job for some of its users. Win on the claims or concede."],
  ['inspire', 'Borrow the idea, not the code.'],
  ['watch', 'Not in the way today; could move toward fictty.'],
];
export const RELIDX: Record<string, number> = Object.fromEntries(RELS.map((r, i) => [r[0], i]));
/** key, label, max, meaning, low end, high end */
export const AXES: [keyof Entry, string, number, string, string, string][] = [
  ['codeVsData', 'code vs data', 10, '0 the UI is code, 10 it is pure data', 'code', 'data'],
  ['ephemerality', 'ephemerality', 10, '0 a durable app, 10 made for one question and thrown away', 'durable', 'throwaway'],
  ['readback', 'read-back', 10, "0 the agent can't see what's drawn, 10 exact state read back", 'blind', 'exact'],
  ['liveData', 'live data', 10, '0 frozen when written, 10 bound to live sources that skip the model', 'frozen', 'live'],
  ['learning', 'learning', 10, '0 no bearing on understanding, 10 built to teach', 'none', 'teaches'],
  ['threat', 'threat', 5, "0 to 5: how much of fictty's job it could take", 'none', 'high'],
  ['adoptFit', 'adopt fit', 5, '0 to 5: how well fictty could use it directly', 'none', 'high'],
];
export const FICTTY: Record<string, number> = { codeVsData: 9, ephemerality: 7, readback: 9, liveData: 9, learning: 3 };
export const SURF: [string, string][] = [['terminal', '--glass-cyan'], ['browser', '--glass-violet'], ['native', '--glass-amber'], ['chat', '--glass-rose'], ['any', '--ok'], ['n/a', '--glass-smoke']];
export const SURFC: Record<string, string> = Object.fromEntries(SURF.map(([k, v]) => [k, `var(${v})`]));
export const EVID = ['strong', 'mixed', 'thin'];
export const LAYERS = ['frame', 'harness', 'framework', 'renderer', 'runtime', 'format', 'idea'];

export const entryUrl = (e: { cluster: string; slug: string }) => `/research/${e.cluster}/${e.slug}/`;
export const clusterUrl = (key: string) => `/research/${key}/`;
export const shortName = (n: string) => n.replace(/\s*\(.*\)\s*$/, '').replace(/^"(.+)"$/, '$1');
export const byBoardOrder = (a: Entry, b: Entry) => RELIDX[a.relation] - RELIDX[b.relation] || b.threat - a.threat || (a.name < b.name ? -1 : 1);
export function nearest(all: Entry[], e: Entry, n = 4): Entry[] {
  const keys = ['codeVsData', 'ephemerality', 'readback', 'liveData', 'learning'] as const;
  return all.filter((o) => o.id !== e.id && o.slug !== e.slug)
    .map((o) => [keys.reduce((d, k) => d + (o[k] - e[k]) ** 2, 0), o] as const)
    .sort((a, b) => a[0] - b[0]).slice(0, n).map((x) => x[1]);
}
