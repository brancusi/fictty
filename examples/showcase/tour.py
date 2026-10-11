#!/usr/bin/env python3
"""Writes the tour's chapters (tour-*.json) beside this script.

The tour is plain JSON like every other screen here; this script only keeps its chapters
consistent (the header, the quest marks, the footer, the keys). Edit it, run it, commit both:

    python3 examples/showcase/tour.py

Each chapter is a screen with a lesson on the left, something to play with on the right and a
quest. A quest is a `when` condition over the store, so finishing one shows a mark in every
chapter's header, and the marks survive a restart with the store.
"""
import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
MARK = json.load(open(os.path.join(HERE, "home.json")))["root"]["children"][1]["children"][1]["children"][0]["text"]

# (file, title, quest condition or None, what the quest asks)
CHAPTERS = [
    ("tour-1-hello.json", "Hello", None, None),
    ("tour-2-value.json", "One value", "seen=Saturn", "Select <b>Saturn</> and press <b>Enter</>."),
    ("tour-3-parts.json", "Components", "plan.0.children.0.done", "In <b>Outline</>, tick <i>Write the post</> with <b>x</>."),
    ("tour-4-store.json", "The store", "items.3", "Add a fourth thing to the list."),
    ("tour-5-layers.json", "Layers", "trash", "Delete a file, and say yes."),
    ("tour-6-themes.json", "Themes", "$theme.name!=glass", "Press <b>t</> to change the theme."),
    ("tour-7-agents.json", "Agents", "agent_done", "Run the autopilot and keep your hands off."),
    ("tour-8-pages.json", "Pages", "open", "Click <b>[[Groceries]]</> to open it beside."),
    ("tour-9-speed.json", "Speed", "big_q", "Filter the 100,000 rows."),
    ("tour-10-yours.json", "Your turn", None, None),
]
QUESTS = [c for c in CHAPTERS if c[2]]

# Everything the tour writes in the store, at its start: "Start over" puts these back.
SEEDS = {
    "seen": "",
    "plan": [
        {"id": "launch", "text": "Launch the tour", "children": [
            {"id": "post", "text": "Write the post"},
            {"id": "demo", "text": "Record the demo", "children": [
                {"id": "script", "text": "Script it", "done": True},
                {"id": "shoot", "text": "Shoot it"},
            ]},
            {"id": "ship", "text": "Ship it"},
        ]},
        {"id": "rest", "text": "Rest", "children": [
            {"id": "walk", "text": "A long walk"},
            {"id": "read", "text": "A short book"},
        ]},
        {"id": "later", "text": "Someday", "children": [{"id": "sail", "text": "Learn to sail"}]},
    ],
    "items": [
        {"name": "Coffee beans", "qty": "1", "done": True},
        {"name": "Oat milk", "qty": "2", "done": False},
        {"name": "Lemons", "qty": "6", "done": False},
    ],
    "files": [{"name": n} for n in ["notes.md", "draft-v1.md", "draft-v2.md", "draft-final.md", "draft-final-FINAL.md", "todo.txt", "screenshot.png"]],
    "trash": [],
    "confirm": False,
    "doomed": "",
    "agent_done": False,
    "agent_tasks": [
        {"id": "read", "text": "Read the screen", "done": False},
        {"id": "write", "text": "Write a note", "done": False},
        {"id": "point", "text": "Point at something", "done": False},
        {"id": "theme", "text": "Try a theme", "done": False},
    ],
}
TEXT_SEEDS = {"item_name": "", "item_qty": "", "items_q": "", "big_q": "", "agentpad": ""}

# The Pages chapter's notebook: pages are store entries, links between them open them.
PAGES = {
    "Home": "# Home\n\nA tiny notebook. Click a link to open it beside this one: [[Groceries]], [[Ideas]], [[Trip]].\nShift-click opens it here instead. Tags are links too: #fictty #demo\n\n"
            "- Tab nests a line under the one above it\n- Enter on a list item starts the next one\n- [ ] Ctrl-T ticks a task, or makes a line one",
    "Groceries": "# Groceries\n\n- [ ] Oat milk\n- [ ] Lemons\n- [x] Coffee beans\n\nBack to [[Home]].",
    "Ideas": "# Ideas\n\n- A screen that is one value\n  - agents write it\n  - people type in it\n- More in [[Trip]]",
    "Trip": "# Trip\n\n1. Pack light\n2. Train at nine\n3. Read [[Ideas]] on the way",
}
SEEDS.update({"here": "Home", "open": [], "flash": ""})
SEEDS.update({f"page/{k}": v for k, v in PAGES.items()})


def text(t, **kw):
    return {"type": "text", "text": t, **kw}


def header(i):
    """The top line: the name, a mark per quest (filled when done), the chapter."""
    marks = []
    for (f, title, cond, _) in QUESTS:
        here = f == CHAPTERS[i][0]
        on = "<accent+b>◆</>" if here else "<pos>◆</>"
        off = "<accent+b>◇</>" if here else "<d>◇</>"
        marks.append(text(on, size=2, when=cond))
        marks.append(text(off, size=2, when="!" + cond))
    n, title = i + 1, CHAPTERS[i][1]
    return {"type": "row", "size": 1, "children": [
        text(" <accent+b>fictty</> <d>tour</>", size=14),
        *marks,
        text(f"<d>{n}/{len(CHAPTERS)}</>  <b>{title}</> ", align="right"),
    ]}


def footer():
    """The keys and the meter, and the toasts over them while one shows."""
    return {"type": "stack", "size": 1, "children": [
        {"type": "row", "children": [
            text(" <b>n</> <d>next</>  <b>p</> <d>back</>  <b>tab</> <d>move</>  <b>esc</> <d>leaves a field</>  <b>t</> <d>theme</>  <b>a</> <d>apps</>  <b>q</> <d>quit</>"),
            text("<d>{fps} fps · {msgs} msg/s · {draw_ms} ms a frame</> ", bind="$stats", align="right", size=44),
        ]},
        {"type": "toasts"},
    ]}


def keys(i):
    k = {"t": {"theme": "next"}, "a": {"load": "home.json"}}
    if i + 1 < len(CHAPTERS):
        k["n"] = {"load": CHAPTERS[i + 1][0]}
    if i > 0:
        k["p"] = {"load": CHAPTERS[i - 1][0]}
    return k


def lesson(i, body, tries):
    """The left panel: what this chapter shows, what to try, and the quest."""
    f, title, cond, quest = CHAPTERS[i]
    kids = [text(body, pad=[1, 0])]
    if tries:
        kids.append(text("\n<accent+b>Try</>\n" + "\n".join(f"<d>›</> {t}" for t in tries), pad=[1, 0], size=len(tries) + 3))
    if cond:
        kids.append({"type": "col", "size": 5, "edge": "rounded", "children": [
            text(f"<pos+b>◆ Quest done.</>\n<b>n</> for the next chapter.", when=cond, pad=[1, 0]),
            text(f"<warn+b>◇ Quest</>\n{quest}", when="!" + cond, pad=[1, 0]),
        ]})
    return {"type": "col", "title": f"{i + 1} · {title}", "size": 44, "variant": "card", "children": kids}


def chapter(i, stage, extra=None, focus=None):
    ui = {
        "root": {"type": "col", "children": [header(i), text("<border>{*─}</>", size=1), stage, footer()]},
        "keys": keys(i),
        "store": {**SEEDS, **TEXT_SEEDS},
        "status_bar": False,
    }
    if focus:
        ui["focus"] = focus
    ui.update(extra or {})
    return ui


def body(i, lesson_text, tries, stage_children, direction="row"):
    return {"type": "row", "children": [lesson(i, lesson_text, tries), {"type": direction, "children": stage_children}]}


# ---- 1 · Hello ----------------------------------------------------------------------------

def hello():
    cards = [
        ("One value", "c1", "Every screen in this tour is a JSON file. An agent writes one, pushes it in, patches it and reads it back."),
        ("Live", "c2", "Data streams in from commands, people type into editors, and agents write the same store, all at once."),
        ("Fast", "c3", "A Rust runtime on ratatui. Thousands of updates a second, drawn only when something changed."),
    ]
    stage = {"type": "col", "children": [
        text("", size="*"),
        {"type": "row", "size": 6, "children": [text("", size="*"), text(MARK, size=72), text("", size="*")]},
        text("<d>Terminal screens agents write, patch and read back.</>", size=2, align="center"),
        {"type": "row", "size": 7, "children": [
            text("", size="*"),
            *[{"type": "text", "size": 30, "title": t, "variant": "card", "pad": [1, 0], "text": f"<{c}>●</> {b}"} for t, c, b in cards],
            text("", size="*"),
        ]},
        text("", size=1),
        {"type": "row", "size": 1, "children": [text("", size="*"), {"type": "spinner", "size": 44, "text": "<b>n</> starts the tour  <d>·</>  <b>a</> opens the apps"}, text("", size="*")]},
        text("", size="*"),
    ]}
    return chapter(0, stage)


# ---- 2 · One value ------------------------------------------------------------------------

PLANETS = [
    {"name": "Mercury", "au": 0.39, "moons": 0, "ring": False},
    {"name": "Venus", "au": 0.72, "moons": 0, "ring": False},
    {"name": "Earth", "au": 1.0, "moons": 1, "ring": False},
    {"name": "Mars", "au": 1.52, "moons": 2, "ring": False},
    {"name": "Jupiter", "au": 5.2, "moons": 95, "ring": True},
    {"name": "Saturn", "au": 9.58, "moons": 146, "ring": True},
    {"name": "Uranus", "au": 19.2, "moons": 28, "ring": True},
    {"name": "Neptune", "au": 30.1, "moons": 16, "ring": True},
]


def value():
    planets = {"type": "list", "id": "planets", "title": "Planets", "size": "40%", "items": PLANETS,
               "item": "{name}\t<d>{moons} moons</>",
               "keys": {"enter": {"do": [{"put": "seen", "value": "{name}"}, {"toast": "You picked {name}", "tone": "pos"}]}}}
    detail = {"type": "text", "bind": "@planets", "title": "Detail", "pad": [1, 0],
              "text": "<b>{name}</>\n\n{au} AU from the sun\n{moons} moons\n{ring|true=<accent>rings</>;*=<d>no rings</>}"}
    mini = {"type": "row", "children": [planets, detail]}
    source = json.dumps(mini, indent=1, ensure_ascii=False)
    stage = [
        {"type": "col", "children": [mini]},
        {"type": "editor", "id": "src", "title": "…is this JSON", "size": "45%", "readonly": True, "text": source},
    ]
    return chapter(1, body(1,
        "A fictty screen is a <b>value</>: a tree of nodes, the data they show and the keys they take. "
        "The little app on the right is the JSON beside it, nothing more.\n\n"
        "An agent builds one the way it writes any JSON, pushes it into a running screen "
        "(<accent>fictty push ui.json</>), and reads back exactly what it shows "
        "(<accent>fictty screen</>, <accent>fictty get --state</>).",
        ["<b>j</>/<b>k</> or the mouse to move", "<b>tab</> into the JSON, scroll it", "the detail follows the selection (<accent>\"bind\": \"@planets\"</>)"],
        stage))


# ---- 3 · Components -----------------------------------------------------------------------

def parts():
    sales = [
        {"m": "Jan", "v": 42, "d": 3.1}, {"m": "Feb", "v": 51, "d": 9.0}, {"m": "Mar", "v": 47, "d": -4.2},
        {"m": "Apr", "v": 63, "d": 16.0}, {"m": "May", "v": 58, "d": -5.1}, {"m": "Jun", "v": 71, "d": 13.4},
        {"m": "Jul", "v": 69, "d": -2.0}, {"m": "Aug", "v": 80, "d": 11.0},
    ]
    data = {"type": "row", "gap": 1, "children": [
        {"type": "table", "id": "sales", "title": "table · totals · heat", "items": sales, "size": "45%", "columns": [
            {"title": "month", "value": "{m}", "size": 7},
            {"title": "sales", "value": "{v}", "size": 8, "align": "right", "total": "v"},
            {"title": "change", "value": "{±d}%", "size": 9, "align": "right", "heat": {"value": "{d}", "min": -10, "max": 10}},
        ]},
        {"type": "col", "children": [
            {"type": "big", "title": "big", "text": "80", "size": 5, "align": "center"},
            {"type": "gauge", "title": "gauge", "value": "0.72", "label": "72% of the goal", "size": 3},
            {"type": "sparkline", "title": "sparkline", "values": [3, 5, 4, 8, 6, 9, 7, 12, 10, 14, 12, 16], "size": 4},
            {"type": "bars", "title": "bars", "horizontal": True, "items": sales[-4:], "label": "{m}", "value": "v"},
        ]},
    ]}
    outline = {"type": "row", "gap": 1, "children": [
        {"type": "col", "children": [
            {"type": "input", "id": "plan_q", "size": 3, "title": "filter", "placeholder": "type to filter the outline"},
            {"type": "tree", "id": "plan", "bind": "plan", "filter": "plan_q", "mark": "▌", "lead": " ", "title": "tree · x ticks · ←/→ fold",
             "item": "{done|true=<pos>☑</>;*=<d>☐</>} {text}\t<d>{$kids|0=;*=$}</>",
             "keys": {"x": {"toggle": "done"}}},
        ]},
        {"type": "text", "bind": "@plan", "title": "@plan", "size": "35%", "pad": [1, 0],
         "text": "<b>{text}</>\n\ndepth {$depth}\n{$kids} under it\n{done|true=<pos>done</>;*=<d>not done</>}\n\n<d>at {$ptr}</>"},
    ]}
    charts = {"type": "col", "gap": 1, "children": [
        {"type": "chart", "title": "chart", "series": [
            {"name": "sales", "bind": "sales_s", "field": "v", "color": "accent"},
            {"name": "costs", "bind": "costs_s", "field": "v", "color": "rose"}], "labels": ["Jan", "Apr", "Aug"]},
        {"type": "bars", "title": "bars · columns", "items": sales, "label": "{m}", "value": "d", "size": 10},
    ]}
    texts = {"type": "row", "gap": 1, "children": [
        {"type": "editor", "id": "page", "title": "editor · markdown · ctrl-t ticks", "syntax": "markdown",
         "text": "# Saturday\n\n- [x] Coffee\n- [ ] Call Ada about the boat\n- [ ] Fix the bike\n\n> Small steps, every day.\n\nRun `fictty render` to see a screen without a terminal."},
        {"type": "editor", "id": "longread", "title": "editor · readonly", "readonly": True,
         "text": "\n".join(f"{i:>3}  A read-only editor scrolls, selects and copies, but never changes." for i in range(1, 121))},
    ]}
    tabs = {"type": "tabs", "id": "gallery", "tabs": ["Data", "Outline", "Charts", "Text"], "children": [data, outline, charts, texts]}
    return chapter(2, body(2,
        "A closed set of components, each with its behaviour built in: lists, tables, trees, charts, "
        "editors, inputs, tabs, buttons, dialogs, toasts, spinners.\n\n"
        "Anything that only <i>arranges</> them is data, not code: a project component or a recipe.",
        ["<b>tab</> to the tab bar, <b>←</>/<b>→</> to switch", "in <b>Outline</>: type to filter, <b>space</> folds", "in <b>Text</>: click a <b>[ ]</> box"],
        [tabs]),
        {"data": {"sales_s": {"value": sales}, "costs_s": {"value": [{"v": v} for v in [30, 33, 35, 38, 41, 44, 46, 48]]}}})


# ---- 4 · The store ------------------------------------------------------------------------

def store():
    add = {"do": [
        {"append": "items", "value": {"name": "{store.item_name}", "qty": "{store.item_qty}", "done": False}},
        {"toast": "Added {store.item_name}", "tone": "pos"},
        {"put": "item_name", "value": ""}, {"put": "item_qty", "value": ""}, {"focus": "item_name"}]}
    form = {"type": "col", "size": 5, "children": [
        {"type": "row", "size": 3, "gap": 1, "children": [
            {"type": "input", "id": "item_name", "title": "thing", "placeholder": "a thing to buy", "keys": {"enter": add}},
            {"type": "input", "id": "item_qty", "title": "how many", "size": 12, "accept": "number", "placeholder": "1", "keys": {"enter": add}},
            {"type": "button", "id": "add", "label": "Add", "size": 9, "click": add, "border": True},
        ]},
    ]}
    lst = {"type": "col", "children": [
        {"type": "input", "id": "items_q", "size": 3, "title": "filter", "placeholder": "type to filter"},
        {"type": "list", "id": "items", "bind": "items", "filter": "items_q", "title": "list · x ticks · d deletes",
         "item": "{done|true=<pos>☑</>;*=<d>☐</>} {name}\t<d>×{qty}</>",
         "keys": {"x": {"toggle": "done"}, "d": {"remove": "items"}}},
    ]}
    revs = {"type": "table", "id": "revs", "bind": "$store", "title": "$store · every entry, live", "size": "42%", "columns": [
        {"title": "key", "value": "{key}", "size": 12}, {"title": "rev", "value": "{rev}", "size": 4, "align": "right"},
        {"title": "by", "value": "{by}", "size": 7}, {"title": "holds", "value": "<d>{holds}</>"}]}
    return chapter(3, body(3,
        "The app's data lives in a <b>store</>, apart from the UI: a push never touches it. "
        "Every write, by you or by an agent, gets a revision and says who made it.\n\n"
        "Agents write with <accent>--if-rev</>: if you typed since they read, their write is refused "
        "and they read again. Nobody's work is lost.",
        ["<b>tab</> to the form, fill it, <b>enter</> or click <b>Add</>", "watch <b>rev</> and <b>by</> change on the right", "<b>x</> ticks, <b>d</> deletes, type to filter"],
        [{"type": "col", "children": [form, lst]}, revs]), focus="items")


# ---- 5 · Layers ---------------------------------------------------------------------------

def layers():
    files = {"type": "list", "id": "files", "bind": "files", "title": "files · d deletes", "item": "{name}",
             "keys": {"d": {"do": [{"put": "doomed", "value": "{name}"}, {"put": "confirm", "value": True}]}}}
    trash = {"type": "list", "id": "trash", "bind": "trash", "title": "trash", "size": "30%", "item": "<d>{name}</>", "empty": "Nothing deleted yet."}
    wide = {"type": "text", "title": "only when wide", "size": 30, "when": "$screen.width>=150", "pad": [1, 0],
            "text": "This pane shows only while your terminal is at least 150 columns wide:\n\n<accent>\"when\": \"$screen.width>=150\"</>\n\nMake the window narrower and watch it go."}
    dialog = {"type": "col", "id": "ask", "title": "Delete?", "modal": True, "when": "confirm", "variant": "alert",
              "float": {"width": 44, "height": 8}, "pad": [2, 1], "keys": {"esc": {"put": "confirm", "value": False}},
              "children": [
                  text("Delete <b>{.}</> for good?", bind="doomed", size=2),
                  {"type": "row", "size": 1, "gap": 2, "children": [
                      {"type": "button", "id": "yes", "label": "Delete", "click": {"do": [
                          {"append": "trash", "value": {"name": "{store.doomed}"}},
                          {"remove": "files", "where": {"name": "{store.doomed}"}},
                          {"put": "confirm", "value": False},
                          {"toast": "Deleted {store.doomed}", "tone": "neg"}]}},
                      {"type": "button", "id": "no", "label": "Keep it", "click": {"do": [{"put": "confirm", "value": False}, {"toast": "Kept it"}]}},
                  ]},
              ]}
    stage = {"type": "stack", "children": [{"type": "row", "gap": 1, "children": [files, trash, wide]}, dialog]}
    ui = chapter(4, body(4,
        "A <b>stack</> draws its children over each other. One that <b>float</>s sits where it says; "
        "a <b>modal</> one dims what's behind it and keeps the focus. "
        "With <b>when</>, a layer comes and goes with the store.\n\n"
        "Toasts, callouts and rings are layers too: agents use them to point at things.",
        ["<b>d</> on a file opens the dialog", "<b>tab</> between its buttons, <b>esc</> backs out", "resize the window"],
        [stage]))
    ui["layers"] = [{"on": "files", "row": 3, "title": "Try it", "text": "Press d to delete a file.", "arrow": True, "place": ["right", "below"],
                     "width": 26, "by": "tour", "when": "!trash & !confirm"}]
    return ui


# ---- 6 · Themes ---------------------------------------------------------------------------

def themes():
    variants = [{"type": "text", "variant": v, "title": v, "pad": [1, 0], "text": f"variant <b>{v}</>"} for v in ["card", "quiet", "hero", "alert"]]
    tones = text("  ".join(f"<{t}>● {t}</>" for t in ["accent", "pos", "neg", "warn", "violet", "dim"]), size=1, pad=[1, 0])
    mini = {"type": "row", "gap": 1, "children": [
        {"type": "table", "title": "a table", "items": [{"k": "rev", "v": "12"}, {"k": "by", "v": "agent"}, {"k": "fps", "v": "120"}],
         "columns": [{"title": "key", "value": "{k}", "size": 8}, {"title": "value", "value": "{v}"}]},
        {"type": "col", "children": [
            {"type": "gauge", "title": "a gauge", "value": "0.6", "size": 3},
            {"type": "sparkline", "title": "a sparkline", "size": 6, "tone": "accent",
             "values": [3, 4, 3, 5, 6, 5, 7, 6, 8, 7, 9, 8, 10, 9, 8, 10, 12, 11, 13, 12, 14, 13, 15, 14, 16, 15, 17, 16, 18, 20,
                        19, 21, 20, 22, 21, 23, 22, 24, 23, 25, 27, 26, 28, 27, 29, 28, 30, 29, 31, 30]},
        ]},
    ]}
    current = text("Now: <accent+b>{name}</>   <d>of {names}</>", bind="$theme", size=1, pad=[1, 0])
    stage = {"type": "col", "gap": 1, "children": [current, {"type": "row", "size": 4, "gap": 1, "children": variants}, tones, mini]}
    return chapter(5, body(5,
        "Screens ask for <b>meanings</>, never colours: a <b>variant</> (card, quiet, alert) and a "
        "<b>tone</> (pos, neg, warn). A theme says what those look like.\n\n"
        "So one key restyles everything, every screen, at once, and an agent never picks a colour.",
        ["<b>t</> cycles glass, paper and phosphor", "the theme stays when you change chapter"],
        [stage]))


# ---- 7 · Agents ---------------------------------------------------------------------------

def agents():
    cmds = [
        {"cmd": "fictty screen", "about": "the frame, as text"},
        {"cmd": "fictty read agentpad", "about": "read what's in the note"},
        {"cmd": "echo '\"hello from a terminal\"' | fictty put agentpad -", "about": "write the note"},
        {"cmd": "fictty toast 'hi from another terminal' --tone pos", "about": "say hi"},
        {"cmd": "fictty theme next", "about": "change the theme"},
        {"cmd": "fictty get --state", "about": "everything, as JSON"},
    ]
    left = {"type": "col", "children": [
        {"type": "button", "id": "go", "label": "▶  Run the autopilot", "size": 3, "border": True,
         "click": {"do": [{"toast": "Autopilot on. Hands off!", "tone": "warn"}, {"run": ["sh", "autopilot.sh"]}]}},
        {"type": "row", "gap": 1, "children": [
            {"type": "editor", "id": "agentpad", "title": "the agent writes here", "syntax": "markdown", "placeholder": "(empty, for now)"},
            {"type": "list", "id": "agent_tasks", "bind": "agent_tasks", "title": "and ticks these", "size": "40%",
             "item": "{done|true=<pos>☑</>;*=<d>☐</>} {text}", "keys": {"x": {"toggle": "done"}}},
        ]},
        {"type": "list", "id": "cmds", "title": "or drive it yourself · enter copies", "size": 9, "items": cmds,
         "item": "<accent>{cmd}</>\t<d>{about}</>",
         "keys": {"enter": {"do": [{"copy": "{$self.env} && {cmd}"}, {"toast": "Copied. Paste it in another terminal.", "tone": "pos"}]}},
         "click": {"copy": "{$self.env} && {cmd}"}},
    ]}
    log = {"type": "table", "id": "log", "bind": "$log", "title": "$log · what agents did", "size": "36%", "columns": [
        {"title": "at", "value": "<d>{at}</>", "size": 8}, {"title": "op", "value": "<accent>{op}</>", "size": 7}, {"title": "what", "value": "{what}"}]}
    return chapter(6, body(6,
        "Agents drive a screen over a socket, with the same verbs you just used: push a UI, write "
        "the store, move the selection, point with callouts, read the frame back as text.\n\n"
        "The autopilot is a shell script doing exactly that. Everything it does lands in "
        "<accent>$log</>, so you always see what an agent did.",
        ["press the button, then watch", "copy a command, run it in another terminal"],
        [left, log]), focus="go")


# ---- 8 · Pages ----------------------------------------------------------------------------

def pages():
    open_beside = {"do": [
        {"remove": "open", "where": {"id": "{$match}"}},
        {"append": "open", "value": {"id": "{$match}"}},
        {"put": "flash", "value": "{$match}", "for_ms": 2500},
        {"toast": "Opened {$match} beside", "tone": "accent"}]}
    links = [
        {"kind": "wiki", "click": open_beside, "s-click": {"put": "here", "value": "{$match}"}},
        {"kind": "tag", "click": {"toast": "#{$match}: tags are links too", "tone": "violet"}},
    ]
    main = {"type": "col", "children": [
        text("<b>{.}</>", bind="here", size=1, pad=[1, 0]),
        text("<d>wide: the sidebar is closed, so this pane has the room</>", size=1, pad=[1, 0], when="$width>=72"),
        text("<d>narrow: the sidebar took room, and this pane switched on its own width</>", size=1, pad=[1, 0], when="$width<72"),
        {"type": "editor", "id": "here_page", "bind": "here", "doc": "page/{.}", "syntax": "markdown",
         "tab": "indent-lines", "caret": "end", "links": links, "border": True},
    ]}
    panel = {"type": "col", "id": "panel-{$item.id}", "title": "{$item.id}", "children": [
        {"type": "button", "id": "close-{$item.id}", "label": "close", "size": 1, "align": "right",
         "click": {"remove": "open", "where": {"id": "{$item.id}"}}},
        {"type": "editor", "id": "side-{$item.id}", "doc": "page/{$item.id}", "syntax": "markdown",
         "tab": "indent-lines", "links": links},
    ]}
    side = {"type": "col", "size": 40, "when": "open", "children": [
        text("<accent>• {.} just opened</>", bind="flash", size=1, when="flash", pad=[1, 0]),
        {"type": "each", "id": "beside", "bind": "open", "template": panel},
    ]}
    return chapter(7, body(7,
        "Pages are store entries; links between them are data. A click on <b>[[Page]]</> runs an "
        "action, here: open it beside, as one more panel in a sidebar made with <b>each</> "
        "(a template per open page). Two editors on one page are one text with one undo.\n\n"
        "The main pane's layout reads <b>$width</>, its own width, not the screen's.",
        ["click a <b>[[link]]</>, shift-click to go there", "<b>Tab</> nests a line, <b>Enter</> continues a list", "type in either copy of a page"],
        [main, side]), focus="here_page")


# ---- 9 · Speed ----------------------------------------------------------------------------

FIREHOSE = r"""
BEGIN {
  srand(); n = split("ACME BOLT CRUX DUSK EMBR FLUX GLOW HALO IRIS JADE KILN LUMA MOSS NOVA ONYX PIKE QUAY RUNE SAGA TIDE", sym, " ");
  for (i = 1; i <= n; i++) px[i] = 40 + rand() * 160;
  for (k = 0; k < 120; k++) hist[k] = 100;
  idx = 100; t = 0;
  while (1) {
    t++; s = "{\"seq\":" t ",\"rows\":[";
    for (i = 1; i <= n; i++) {
      d = (rand() - 0.5) * px[i] * 0.004; px[i] += d; c = (px[i] / (px[i] - d) - 1) * 1000;
      s = s (i > 1 ? "," : "") sprintf("{\"sym\":\"%s\",\"px\":%.2f,\"chg\":%.2f}", sym[i], px[i], c);
    }
    idx += (rand() - 0.5) * 2; hist[t % 120] = idx; h = "";
    for (k = 1; k <= 120; k++) h = h (k > 1 ? "," : "") sprintf("%.2f", hist[(t + k) % 120]);
    print s "],\"index\":" sprintf("%.2f", idx) ",\"hist\":[" h "]}"; fflush();
  }
}
"""

BIGLIST = r"""
BEGIN {
  srand(7); na = split("Ada Ben Cleo Dev Eli Fay Gus Hana Ivo June Kai Lena Milo Nia Otto Pia Quin Rosa Sami Tess Ugo Vera Wes Xia Yuri Zoe", a, " ");
  nb = split("Lisbon Oslo Lagos Lima Kyoto Quito Accra Hanoi Perth Malmo Cork Sucre Turku Nara Fez Bergen Cusco Hue Leeds Pune", b, " ");
  nc = split("violin kite garden tram lantern atlas comet harbor pepper meadow", c, " ");
  printf "[";
  for (i = 1; i <= 100000; i++) {
    printf "%s{\"n\":%d,\"who\":\"%s\",\"city\":\"%s\",\"likes\":\"%s\"}", (i > 1 ? "," : ""), i, a[int(rand()*na)+1], b[int(rand()*nb)+1], c[int(rand()*nc)+1];
  }
  print "]";
}
"""


def speed():
    kpi = lambda title, t, tone: {"type": "big", "title": title, "text": t, "bind": "$stats", "tone": tone, "align": "center", "variant": "card"}
    kpis = {"type": "row", "size": 5, "gap": 1, "children": [
        kpi("frames a second", "{fps}", "accent"), kpi("messages a second", "{msgs}", "pos"), kpi("ms to draw a frame", "{draw_ms}", "warn")]}
    fire = {"type": "row", "gap": 1, "children": [
        {"type": "table", "id": "book", "bind": "fire.rows", "title": "a stream, as fast as awk can print", "size": "55%", "columns": [
            {"title": "sym", "value": "{sym}", "size": 6},
            {"title": "price", "value": "{px}", "size": 9, "align": "right"},
            {"title": "‰", "value": "{±chg}", "size": 7, "align": "right", "heat": {"value": "{chg}", "min": -2, "max": 2}}]},
        {"type": "col", "children": [
            text("seq <b>{seq}</>   index <accent+b>{index}</>", bind="fire", size=1, pad=[1, 0]),
            {"type": "sparkline", "bind": "fire.hist", "title": "index", "tone": "accent"},
        ]},
    ]}
    big = {"type": "col", "size": 40, "children": [
        {"type": "input", "id": "big_q", "size": 3, "title": "filter", "placeholder": "filter 100,000 rows: try ada kyoto"},
        {"type": "list", "id": "big", "bind": "big", "filter": "big_q", "title": "100,000 rows", "item": "<d>{n}</> {who} <d>in</> {city}\t<d>{likes}</>"},
    ]}
    return chapter(8, body(8,
        "Updates are messages; the screen draws only when something changed, at most 120 times a second, "
        "and only the newest value of a busy source is drawn.\n\n"
        "The table on the right is fed by one <b>awk</> process printing JSON lines as fast as it can. "
        "The list holds 100,000 rows and filters on every key.",
        ["watch the counters", "<b>tab</> to the filter and type: every word must match", "<b>G</> jumps to the last row"],
        [{"type": "col", "children": [kpis, fire]}, big]), focus="big",
        extra={"data": {"fire": {"stream": ["awk", FIREHOSE]}, "big": {"cmd": ["awk", BIGLIST]}}})


# ---- 10 · Your turn ------------------------------------------------------------------------

def yours():
    reset = {"do": [*[{"put": k, "value": v} for k, v in SEEDS.items()], *[{"put": k, "value": ""} for k in TEXT_SEEDS],
                    {"toast": "Back to the start", "tone": "warn"}, {"load": CHAPTERS[0][0]}]}
    lines = [
        ("Make a screen", "fictty run ui.json --watch", "edit the file, the screen follows"),
        ("Push one in", "fictty push ui.json", "into the screen that's running"),
        ("See it as text", "fictty render ui.json --size 100x30", "no terminal needed"),
        ("Embed it", "fictty::App::new(ui).host(my_app).run()", "a Rust library too"),
    ]
    rows = [{"type": "row", "size": 3, "children": [
        text(f"<b>{t}</>", size=18, pad=[1, 1]), text(f"<accent>{c}</>", pad=[1, 1]), text(f"<d>{w}</>", size=36, pad=[1, 1])]} for t, c, w in lines]
    stage = {"type": "col", "children": [
        text("", size=1),
        {"type": "row", "size": 6, "children": [text("", size="*"), text(MARK, size=72), text("", size="*")]},
        text("<b>That's all of it.</>  <d>The marks at the top are your quests.</>", size=2, align="center"),
        {"type": "row", "children": [text("", size="*"), {"type": "col", "size": 110, "variant": "card", "pad": [1, 0], "children": rows}, text("", size="*")]},
        {"type": "row", "size": 3, "gap": 2, "children": [
            text("", size="*"),
            {"type": "button", "id": "apps", "label": "Open the apps", "size": 20, "click": {"load": "home.json"}},
            {"type": "button", "id": "again", "label": "Start over", "size": 18, "click": reset},
            {"type": "button", "id": "bye", "label": "Quit", "size": 10, "click": "quit"},
            text("", size="*"),
        ]},
        text("<d>fictty.com · github.com/brancusi/fictty</>", size=1, align="center"),
        text("", size="*"),
    ]}
    return chapter(9, stage)


def main():
    for (f, *_), make in zip(CHAPTERS, [hello, value, parts, store, layers, themes, agents, pages, speed, yours]):
        with open(os.path.join(HERE, f), "w") as out:
            json.dump(make(), out, indent=1, ensure_ascii=False)
            out.write("\n")
    print(f"wrote {len(CHAPTERS)} chapters")


if __name__ == "__main__":
    main()
