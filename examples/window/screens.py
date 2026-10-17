# The wringer: screens a designer might hand over, built from what fictty has today. Each one
# notes the tricks it needs; examples/window/README.md says what rebuilding it without them takes.
# Writes <screen>.json beside this file:  python3 examples/window/screens.py
import json, sys, os
L = os.path.dirname(os.path.abspath(__file__))
def T(text, **k): return {"type": "text", "text": text, **k}
def save(name, ui):
    with open(f"{L}/{name}.json", "w") as f: json.dump(ui, f, indent=1); f.write("\n")

# 1 · Settings form: labels, fields, toggles, a radio group, a select, a slider, a save bar. The
# form scrolls; a label sits beside its field, centred on it. The radio group is tabs without
# pages. Tricks left: the checkboxes are list rows that toggle, the select a pop-up list, the
# slider a gauge (recipes and `slider` are phase 2's).
def field(label, node): return {"type": "row", "size": "fit", "gap": 2, "items": "center", "children": [T(f"<dim>{label}</>", size=18, align="right"), node]}
save("settings", {
 "root": {"type": "stack", "children": [
  {"type": "row", "gap": 3, "children": [
   {"type": "list", "id": "nav", "size": 22, "title": "SETTINGS", "items": ["General", "Appearance", "Notifications", "Billing", "Danger zone"]},
   {"type": "col", "id": "form", "scroll": True, "gap": 1, "children": [
    T("<heading>Profile</>\n<dim>How you appear to your team.</>", size="fit"),
    field("Display name", {"type": "input", "id": "name", "text": "Ada Lovelace"}),
    field("Email", {"type": "input", "id": "email", "text": "ada@example.com"}),
    field("Seats", {"type": "input", "id": "seats", "text": "12", "accept": "number", "size": 10}),
    T("<heading>Preferences</>", size=1),
    {"type": "list", "id": "prefs", "size": 4, "bind": "prefs",
     "item": "{on|true=<accent>[■]</>;*=<dim>[ ]</>}  {label}\t<dim>{hint}</>", "keys": {"space": {"toggle": "on"}, "enter": {"toggle": "on"}}, "click": {"toggle": "on"}},
    T("<b>Density</>", size=1),
    {"type": "tabs", "id": "density", "size": 1, "tabs": ["Compact", "Comfortable", "Spacious"]},
    T("<b>Volume</>  {vol}%   <dim>(a slider: ←/→ can't add 5 without the host)</>", bind="ui", size=1),
    {"type": "gauge", "id": "vol", "size": 1, "bind": "ui", "value": "{vol}", "of": 100},
    T("<b>Region</>", size=1),
    {"type": "button", "id": "region", "label": "{region}  ▾", "bind": "ui", "size": 1, "click": {"put": "picking", "value": True}},
    {"type": "row", "size": 1, "gap": 2, "justify": "end", "margin": [1, 0, 0, 0], "children": [{"type": "button", "id": "cancel", "label": "Cancel", "size": "fit"}, {"type": "button", "id": "save", "label": "Save changes", "size": "fit", "click": {"toast": "Saved", "tone": "pos"}}]}
   ]}
  ]},
  {"type": "list", "id": "regions", "modal": True, "when": "picking", "border": True, "items": ["Europe (Frankfurt)", "US East (Virginia)", "Asia (Singapore)"],
   "float": {"width": 30, "height": 5, "at": "center"}, "keys": {"enter": {"do": [{"put": "ui.region", "value": "{.}"}, {"put": "picking", "value": False}, {"focus": "region"}]}, "esc": {"put": "picking", "value": False}}}
 ]},
 "store": {"ui": {"vol": 70, "region": "Europe (Frankfurt)"}, "picking": False,
   "prefs": [{"id": "a", "label": "Email me a weekly digest", "hint": "Mondays", "on": True}, {"id": "b", "label": "Desktop notifications", "hint": "when mentioned", "on": False},
             {"id": "c", "label": "Show avatars", "hint": "", "on": True}, {"id": "d", "label": "Reduce motion", "hint": "accessibility", "on": False}]},
 "focus": "name"})

# 2 · Chat: bubbles left and right, a typing line, a composer. A message is a row template: a
# bubble as wide as its text (up to 44 cells), on its side, mine in the accent, from its data. (The
# column is a share held to 44, its bubble only as wide as its text: a `fit` held by `max` in a
# `fit` row is measured on one line, Taffy's, see rows-step.md.)
msgs = [("them", "Grace", "Did the build go out?", "09:12"), ("me", "You", "Yes, v2.14 is live in eu-west.", "09:13"),
        ("them", "Grace", "Latency looks better already 🎉", "09:13"), ("me", "You", "p95 dropped from 140 to 96 ms, and the error rate is flat since the deploy.", "09:14"),
        ("them", "Grace", "Ship the US region after lunch?", "09:15"), ("me", "You", "Deal.", "09:15")]
bubble = {"type": "row", "size": "fit", "justify": "{$item.side|me=end;*=start}", "children": [
    {"type": "col", "max": 44, "margin": [0, 0, 1, 0], "children": [
        T("<dim>{$item.who} · {$item.at}</>", size="fit", wrap=False, self="{$item.side|me=end;*=start}"),
        T("{$item.text}", variant="{$item.side|me=hero;*=card}", pad=[1, 0], size="fit", self="{$item.side|me=end;*=start}")]}]}
# Enter in the composer and the Send button do the same: add the message, empty the composer.
send = {"do": [{"append": "msgs", "value": {"side": "me", "who": "You", "text": "{store.compose.text}", "at": "now"}}, {"put": "compose", "value": ""}]}
save("chat", {
 "root": {"type": "row", "gap": 2, "children": [
  {"type": "list", "id": "rooms", "size": 34, "title": "MESSAGES", "lines": 2, "bind": "rooms", "item": "<b>{name}</>\t<dim>{when}</>\n<dim>{last}</>"},
  {"type": "col", "gap": 1, "children": [
   {"type": "row", "size": 1, "justify": "between", "children": [T("<heading>Grace Hopper</>  <pos>●</> <dim>online</>", size="fit", wrap=False), T("<dim>⌕  ⋯</>", size="fit", wrap=False)]},
   {"type": "rule", "size": 1},
   {"type": "list", "id": "thread", "bind": "msgs", "row": bubble},
   T("<dim>Grace is typing…</>", size=1),
   {"type": "row", "size": 1, "gap": 1, "children": [{"type": "input", "id": "compose", "placeholder": "Message Grace…",
     "keys": {"enter": send}},
     {"type": "button", "id": "send", "label": "Send", "size": "fit", "click": send}]}
  ]}
 ]},
 "store": {"msgs": [{"id": str(i), "side": s, "who": w, "text": t, "at": a} for i, (s, w, t, a) in enumerate(msgs)],
   "rooms": [{"id": "g", "name": "Grace Hopper", "when": "09:15", "last": "Deal."}, {"id": "a", "name": "Ada Lovelace", "when": "Mon", "last": "The table on page four…"}, {"id": "t", "name": "#launch", "when": "Sun", "last": "Alan: rollout plan attached"}]},
 "focus": "compose"})

# 3 · Kanban: columns of cards with a tag and a size, counts. Each column is a list of cards, a row
# template as tall as its title wraps to, its title counting its rows.
cards = {"todo": [("Ellipsis for cut lines", "window", "S"), ("Cards: radius, gradient and a shadow in every theme", "themes", "M"), ("IME composition", "input", "S")],
         "doing": [("Parts as styles", "themes", "L"), ("Charts on canvas", "charts", "M")],
         "review": [("Layout parity", "window", "M")], "done": [("Window under test", "tests", "M"), ("Fuzz through the window", "tests", "S")]}
cols = [{"id": k, "title": t} for k, t in [("todo", "TO DO"), ("doing", "IN PROGRESS"), ("review", "REVIEW"), ("done", "DONE")]]
card = {"type": "col", "size": "fit", "variant": "card", "border": True, "pad": [1, 0], "margin": [0, 0, 1, 0], "children": [
    T("<b>{$item.t}</>", size="fit"),
    {"type": "row", "size": 1, "justify": "between", "children": [T("<accent>#{$item.tag}</>", size="fit", wrap=False), T("<dim>{$item.size}</>", size="fit", wrap=False)]}]}
save("kanban", {
 "root": {"type": "col", "gap": 1, "children": [
  {"type": "row", "size": 1, "justify": "between", "children": [T("<heading>Phase 1 · the window</>   <dim>8 cards · 3 people</>", size="fit", wrap=False), T("<dim>+ New card   ⌕ Filter</>", size="fit", wrap=False)]},
  {"type": "each", "id": "board", "bind": "cols", "row": True, "gap": 2, "template":
   {"type": "list", "id": "col-{$item.id}", "title": "{$item.title} · {$rows}", "bind": "cards.{$item.id}", "row": card, "empty": "drop cards here"}}
 ]},
 "store": {"cols": cols, "cards": {k: [{"id": f"{k}{i}", "t": c[0], "tag": c[1], "size": c[2]} for i, c in enumerate(v)] for k, v in cards.items()}}})

# 4 · Media player: art, big title, progress, transport, up next.
save("player", {
 "root": {"type": "row", "gap": 3, "children": [
  {"type": "col", "size": "45%", "gap": 1, "children": [
   {"type": "svg", "id": "art", "size": "60%", "svg": "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='#ff2079'/><stop offset='1' stop-color='#3a0ca3'/></linearGradient></defs><rect width='100' height='100' rx='6' fill='url(#g)'/><circle cx='50' cy='50' r='22' fill='none' stroke='#fff' stroke-width='2'/></svg>", "alt": "[album art]"},
   T("<b>Neon Rain</>\n<dim>Vela Drive · Afterglow (Deluxe)</>", size=2, align="center"),
   {"type": "gauge", "id": "pos", "size": 1, "value": "0.42"},
   T("<dim>1:38</>\t<dim>3:54</>", size=1),
   {"type": "row", "size": 1, "gap": 2, "justify": "center", "children": [{"type": "button", "id": "prev", "label": "⏮", "size": 6}, {"type": "button", "id": "play", "label": "⏸", "size": 6}, {"type": "button", "id": "next", "label": "⏭", "size": 6}]},
   {"type": "sparkline", "id": "eq", "size": 2, "values": [3, 5, 8, 6, 9, 4, 7, 10, 6, 3, 5, 8, 9, 7, 4, 6]}
  ]},
  {"type": "list", "id": "queue", "title": "UP NEXT", "bind": "q", "item": "{n:>2}  <b>{t}</>  <dim>{a}</>\t<dim>{d}</>"}
 ]},
 "store": {"q": [{"id": str(i), "n": i + 1, "t": t, "a": a, "d": d} for i, (t, a, d) in enumerate([("Afterglow", "Vela Drive", "4:18"), ("Static Bloom", "Halcyon Grid", "3:32"), ("Overclock", "Mono Ferro", "4:07"), ("Low Orbit", "Mono Ferro", "4:33"), ("Coastline", "The Lumen", "5:21")])]},
 "focus": "queue"})

# 5 · Landing page: hero, CTA, a feature grid, pricing. The page scrolls; the features and the
# prices are grids of as many as fit, so they wrap on a narrow window with their columns lined up.
def feat(t, d): return {"type": "text", "variant": "card", "border": True, "pad": [1, 0], "size": "fit", "text": f"<heading+accent>{t}</>\n{d}"}
def price(n, p, items, hi=False): return {"type": "col", "border": True, "title": n, "pad": [1, 0], "variant": "hero" if hi else "card", "children": [T(f"<stat>{p}</>  <dim>/ month</>", size="40px"), T("\n".join("✓ " + i for i in items), size="fit"), {"type": "button", "id": f"buy-{n}", "label": "Choose " + n, "size": 1, "align": "center", "margin": [1, 0, 0, 0]}]}
save("landing", {
 "root": {"type": "col", "id": "page", "scroll": True, "gap": 1, "children": [
  {"type": "row", "size": 1, "justify": "between", "children": [T("<heading>◆ fictty</>", size="fit", wrap=False), T("<dim>Docs   Blog   Pricing</>   <accent+b>Get started</>", size="fit", wrap=False)]},
  T("<display>UI as data</>", id="hero", size="84px", align="center"),
  T("<dim>A screen is one serializable value. Push it from anywhere; agents can read it back.</>", size="fit", align="center"),
  {"type": "row", "size": 1, "gap": 2, "justify": "center", "children": [{"type": "button", "id": "cta", "label": "Try it in one line", "size": "fit"}, {"type": "button", "id": "cta2", "label": "Read the docs", "size": "fit"}]},
  {"type": "grid", "id": "features", "cols": {"min": "240px", "max": 3}, "gap": 2, "children": [feat("One value", "The whole screen is JSON you can push, patch and diff."), feat("Agent-native", "Read the screen back as text or nodes."), feat("Fast", "Thousands of updates a second, drawn at 60 fps.")]},
  {"type": "grid", "id": "prices", "cols": {"min": "240px", "max": 3}, "gap": 2, "children": [price("Hobby", "$0", ["1 screen", "Community"]), price("Pro", "$12", ["Unlimited screens", "Themes", "Priority support"], True), price("Team", "$40", ["SSO", "Audit log", "Shared screens"])]}
 ]}})

# 6 · Calendar: a week grid with events at times, of different lengths. Each day's events are
# data, floated where they fall: `y` from the start (two rows an hour), the height from the length.
days = ["MON 12", "TUE 13", "WED 14", "THU 15", "FRI 16"]
hours = [f"{h:02d}:00" for h in range(9, 18)]
ev = {0: [(1, 2, "Standup"), (4, 3, "Design review")], 1: [(0, 1, "1:1 Grace"), (5, 1, "Lunch")], 2: [(2, 4, "Focus: themes")], 3: [(1, 1, "Hiring"), (6, 2, "Demo prep")], 4: [(3, 2, "Ship v0.3")]}
event = T("<b>{$item.title}</>\n<dim>{$item.at}</>", id="ev-{$item.id}", variant="card", border=True, pad=[1, 0],
          float={"y": "{$item.y}", "height": "{$item.h}", "width": "100%"})
def day(i): return {"type": "col", "title": days[i], "border": True, "children": [{"type": "each", "id": f"day{i}", "bind": f"day{i}", "stack": True, "template": event}]}
save("calendar", {
 "root": {"type": "col", "gap": 1, "children": [
  {"type": "row", "size": 1, "justify": "between", "children": [T("<heading>October 2026</>  <dim>Week 42</>", size="fit", wrap=False), T("<dim>‹ Today ›   Day  Week  Month</>", size="fit", wrap=False)]},
  {"type": "row", "children": [{"type": "col", "size": 7, "margin": [1, 0, 0, 0], "children": [{"type": "each", "bind": "hours", "template": T("<dim>{$item}</>", size=2)}]}] + [day(i) for i in range(5)]}
 ]},
 "data": {"hours": {"value": hours}, **{f"day{i}": {"value": [{"id": f"{i}-{s}", "title": t, "at": hours[s], "y": s * 2, "h": n * 2} for s, n, t in sorted(ev.get(i, []))]} for i in range(5)}}})

# 7 · Spreadsheet: dense numbers, heat, totals, sort marks.
import random
random.seed(4)
rows = [{"id": str(i), "rep": n, "q1": random.randint(20, 90), "q2": random.randint(20, 90), "q3": random.randint(20, 90), "q4": random.randint(20, 90), "d": round(random.uniform(-12, 18), 1)}
        for i, n in enumerate(["Ada", "Alan", "Barbara", "Claude", "Dennis", "Edsger", "Frances", "Grace", "Hedy", "John", "Ken", "Linus", "Margaret", "Radia"])]
save("sheet", {
 "root": {"type": "col", "gap": 1, "children": [
  T("<heading>Q-by-Q pipeline</>  <dim>14 reps · $k</>\t<dim>Sorted by name ▲</>", size=1),
  {"type": "table", "id": "t", "bind": "rows", "columns": [
    {"title": "Rep ▲", "value": "<b>{rep}</>"},
    {"title": "Q1", "value": "{q1}", "align": "right", "heat": {"value": "q1", "min": 20, "max": 90}, "total": "q1"},
    {"title": "Q2", "value": "{q2}", "align": "right", "heat": {"value": "q2", "min": 20, "max": 90}, "total": "q2"},
    {"title": "Q3", "value": "{q3}", "align": "right", "heat": {"value": "q3", "min": 20, "max": 90}, "total": "q3"},
    {"title": "Q4", "value": "{q4}", "align": "right", "heat": {"value": "q4", "min": 20, "max": 90}, "total": "q4"},
    {"title": "Δ YoY", "value": "{±d}%", "align": "right"}]}
 ]},
 "store": {"rows": rows}, "focus": "t"})
print("ok")
