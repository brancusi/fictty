# Time travel an agent can use

*Proposal, 2026-10-09. Builds on the landscape research of the same date: the public board at
[thoughtcontrol.app/landscape](https://thoughtcontrol.app/landscape/).*

## Summary

fictty is a terminal UI that is one serializable value, driven by agents over a socket. Its
state is already serializable, every input is already a message, and `update` and `view` are
pure. That makes time travel cheap to build, and the research found a real gap: **no tool lets an
agent address a live UI's history.** Redux DevTools has the verbs for humans. Replay.io gives
agents a recording to read, after the fact. Rerun lets an agent move a time cursor over logged
data, not UI state. LangGraph forks agent state with no UI. Scrimba lets a learner take over a
recorded lesson, but people author it. A2UI, MCP Apps and AG-UI keep no history at all.

The proposal: record inputs, never pixels; snapshot sparsely; give agents seven operations
(`history`, `state_at`, `screen_at`, `diff`, `seek`, `fork`, `replay`) plus marks and
export/import; and build teaching tours as branches the learner can fork by touching anything.

Two cautions shape the order of work. Elm hoped its debugger would teach; the feature people
used was the exported bug report. And Vue removed time travel when nobody used it. So the first
release is **reproducible agent sessions** (record, export, import, replay, diff). Tours come
second, on the same log.

## 1. What already exists

| Piece | Where | What it gives time travel |
|---|---|---|
| `State` (ui, local view state by id, focus, data slots, hover, caps, stats, `now_ms`) | `state.rs` | A full snapshot is `serde_json::to_vec(&state)` |
| `Msg` (push, patch, key, click, hover, scroll, data, ran, stats, tick, layers, caps) | `state.rs` | Every change is already a value |
| `update(&mut State, Msg) -> Vec<Effect>`, pure | `state.rs` | Replaying messages reproduces state exactly |
| `view(&State) -> frame + hits + pictures`, pure | `view.rs` | `screen_at` is free: replay, then render |
| `Effect` (fetch, run, load, quit) run by the runtime | `runtime.rs` | Effects are the boundary not to cross on replay |
| `upgrade` (save state, exec, restore) | `runtime.rs` | "Change the code, keep the history" (Elm's swap) is upgrade + replay |
| Coalesced streams (newest value wins per frame) | `runtime.rs` | Bounds how much a high-rate source can write to the log |
| caretline trace/replay; Thought Control's `thc tui --trace` and `thc ui replay` | caretline, thc-tui | Same discipline, proven byte-identical with a pinned clock |
| caretline-tour (steps of layers, narration, the host's scene) | crates.io | The tour format to build on |

## 2. What to record

**Inputs only.** The view is pure, so frames are re-derived; never store pixels or cells.

One log entry per applied message:

```json
{"seq": 4812, "branch": "main", "parent": 4811, "t": 183420,
 "origin": "agent:claude", "msg": {"msg": "patch", "id": "ring", "node": {…}}}
```

- `t` is the screen's logical clock (`State.now_ms`), not the wall clock, so replays don't
  depend on when they run. Time spent paused doesn't count (Elm's rule).
- `origin` says who: `agent:<actor>`, `human`, `stream:<source>`, `timer`, `runtime`.
- **Data is an input.** A source's value is recorded as the `Data` message that carried it.
  Replay never re-runs a command or re-reads a stream (Elm: "your database is not coming
  along").
- **High-rate data is batched per frame.** The runtime already coalesces stream values to the
  newest per source per frame; record one `frame` entry holding those winners. That caps the log
  at the frame rate (120/s worst case, usually far less: it only ticks when something changed),
  however fast the data arrives.
- **Effects are not recorded as such.** A `Run` effect's result comes back as `Ran`, which is
  recorded; that is enough to replay its consequences.

**Snapshots** of the full `State`: at every whole-UI push, at every mark, and every 1,000
entries, or when the estimated replay cost passes a budget (Elm's later suggestion: snapshot by
cost, not count).

**Bounds.** A time-based ring (default: the last 10 minutes, `history.keep` in the UI), the
equivalent of Redux DevTools' `maxAge`. Anything marked, forked, exported or used by a tour is
pinned. At 120 frames/s for 10 minutes that's at most 72,000 entries and about 72 snapshots.
Worst-case seek replays 1,000 messages through a pure update; measured whole-UI pushes run at
~6,000/s, patches faster, so ~0.2 s worst case. **To measure** before committing to numbers.

## 3. Operations (socket ops, and MCP tools later)

Shaped after LangGraph's checkpoint vocabulary and Redux DevTools' verbs, with Replay.io's and
Rerun's advice for agent tools: an overview first, semantic tools before raw input, small
results.

| Op | Does | Notes |
|---|---|---|
| `history {from?, to?, origin?, kind?, limit?}` | Compact summaries: seq, t, origin, kind, the node id touched, marks | The "overview first" tool |
| `state_at {seq, path?}` | State (or a JSON pointer into it) at a past step | Nearest snapshot + replay |
| `screen_at {seq, size?}` | The frame at a past step, as text (or ANSI) | Free: the view is pure |
| `diff {a, b, path?}` | RFC 6902 ops between two states | Same patch dialect as input |
| `seek {seq}` | Show the past; live input pauses | `lock` decides whether live streams buffer or drop; `resume` returns to live |
| `fork {seq, label?}` | A new branch from a past step | Any write while seeking forks automatically |
| `replay {from, to, branch?, speed?}` | Play a range again, at a speed | Recorded data, not live sources |
| `mark {label}` / `marks` | Named stopping points | tldraw marks, Motion Canvas time events |
| `export {from?, to?}` / `import {file}` | JSONL: header (versions, ui schema), snapshot, entries | Import refuses a history the running engine can't replay (Elm's check) |
| later: `skip {seq}`, `test {from, to}` | Replay without a step; write a golden test from a range | Redux DevTools; ship only after the above |

**Every write carries `if_seq`** (the screen's current head). A mismatch is a `stale` error,
never a silent drift. This is AG-UI's "snapshot tax" avoided up front, and the same discipline
as thc's `--if-match`.

## 4. Teaching tours

A tour is a **branch** (its messages), **marks** (its steps) and **narration** (caretline-layers
callouts bound to the step). It runs on the same log.

- **Play:** the agent replays the tour branch; at each mark it narrates (a callout, a spotlight,
  an arrow) and waits or continues.
- **Take over:** the learner touches a key or the mouse. That forks at the current step
  (Scrimba's pause-and-edit). Live data keeps flowing in the learner's branch.
- **Rewind:** seek to the last mark.
- **"What did I do differently?"** `diff` between the learner's branch and the tour's, which the
  agent can read and explain.
- **Reproducible:** a replayed tour uses its recorded data, so it looks the same every time; a
  live tour uses live data.

## 5. What not to do

- Record pixels or frames. (The view is pure.)
- Re-run commands or effects on seek or replay.
- Keep unbounded history.
- Build a branch-graph UI for humans in v1; agents get the ops, people get marks and rewind.
- Let an agent seek while a person is typing without `lock`.
- Ship skip/reorder before jump and export/import (Elm's order: what got used first).
- Invent a patch dialect: inputs and diffs are RFC 6902; accept A2UI `updateComponents` as an
  adapter.
- Compete with Rerun at scrubbing dense time series; link to it.

## 6. Order of work

1. **Record:** the log (entries + snapshots + ring) in the runtime, behind `history.keep`.
   Measure log size and seek cost on the stress demos and the pixel tour.
2. **Read:** `history`, `state_at`, `screen_at`, `diff`.
3. **Move:** `seek`/`resume` with `lock`, `fork`, `replay`, `if_seq` on every write.
4. **Share:** `export`/`import` with version checks; a replayable bug report from any session.
5. **MCP server:** the socket ops as MCP tools (two families: semantic first, raw keys/mouse as
   the fallback; logs appended to results, after Rerun's viewer MCP).
6. **Tours:** branch + marks + narration, take-over as fork, on caretline-tour.

## 7. Open questions

- Does `upgrade` + replay of the old log on new code ("change history") hold when the UI schema
  changed? It needs the import version check, and probably a per-message schema version.
- Retention for very long sessions: snapshots only beyond the ring (a coarse timeline), or drop?
- Where command policy meets replay: an imported history from someone else must not get to run
  commands when it goes live again. Tie it to the command policy (allowlist, literal-only mode
  for untrusted UIs), which is needed anyway.

## Sources

The landscape research behind this (Elm's posts on time travel and the perfect bug report; Redux
DevTools' options; Replay.io and Rerun MCP docs; LangGraph time travel; rrweb's event model;
tldraw history; Scrimba; Motion Canvas time events; AG-UI events and the snapshot-tax write-up)
is linked from each tool's dossier on the board.
