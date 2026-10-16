# Daybook: a journal written as data, after Thought Control's: a page a day, pages for
# everything else, tasks in the text, and a zen mode that leaves only the writing. Built from
# what fictty has, to find what it lacks (README.md lists both). Writes journal.json beside this
# file:  python3 examples/journal/journal.py
import json, os
L = os.path.dirname(os.path.abspath(__file__))
def T(text, **k): return {"type": "text", "text": text, **k}

# What's open is `here`; the editor shows the entry it names, and the focus goes into it.
def open_day(row="{id}", long="{long}"):
    return {"do": [{"put": "here", "value": {"kind": "day", "id": row, "title": long}}, {"select": row, "in": "days"}, {"focus": "page"}]}
def open_page(name):
    return {"do": [{"put": "here", "value": {"kind": "page", "id": name, "title": name}}, {"select": name, "in": "pages"}, {"focus": "page"}]}
# A page named in a link or the new-page box joins the list if it isn't there (once, at the end).
def add_page(name): return [{"remove": "pages", "where": {"id": name}}, {"append": "pages", "value": {"id": name}}]
# The day before or after the open one: the journal list's row above or below it.
def step_day(way): return {"do": [{"put": "zen", "value": False}, {"focus": "days"}, way, open_day()]}

KEYS = {
    "d-t": {"do": [{"remove": "days", "where": {"id": "{today.id}"}}, {"append": "days", "value": "{today}"},
                   open_day("{today.id}", "{today.long}")]},
    "d-n": {"do": [{"put": "newname", "value": ""}, {"put": "naming", "value": True}, {"focus": "newname"}]},
    "d-1": {"do": [{"put": "zen", "value": False}, {"focus": "days"}]},
    "d-2": {"do": [{"put": "zen", "value": False}, {"focus": "pages"}]},
    "a-[": step_day("up"),
    "a-]": step_day("down"),
    "a-z": {"toggle": "store.zen"},
    "d-.": {"toggle": "store.zen"},
    "d-s-t": {"theme": "next"},
}

side = {"type": "col", "id": "side", "size": 28, "when": "!zen", "gap": 1, "children": [
    T("<heading>Daybook</>", size=2),
    {"type": "list", "id": "days", "title": "JOURNAL  ⌘1", "bind": "days", "item": "{short}", "size": "*",
     "keys": {"enter": open_day()}, "click": open_day()},
    {"type": "list", "id": "pages", "title": "PAGES  ⌘2", "bind": "pages", "item": "{id}", "size": "*",
     "keys": {"enter": open_page("{id}")}, "click": open_page("{id}")},
]}

editor = {"type": "editor", "id": "page", "bind": "here", "doc": "{kind}/{id}",
          "syntax": "markdown", "tasks": "cycle", "tab": "indent", "caret": "end",
          "links": [{"kind": "wiki", "click": {"do": add_page("{$match}") + [open_page("{$match}")]}}]}

# The writing column: the heading, the text and the hints share one measure, centred in what
# the sidebar leaves.
main = {"type": "row", "justify": "center", "children": [{"type": "col", "size": 72, "gap": 1, "children": [
    T("<dim>{kind|day=§;*=¶}</>  <heading>{title}</>", bind="here", size=2, when="!zen"),
    T("", size=6, when="zen"),
    editor,
    T("<dim>⌘T today · ⌘N new page · ⌥[ ⌥] days · ⌃T task · ⌥Z zen · esc leaves</>", size=1, when="!zen"),
]}]}

dialog = {"type": "input", "id": "newname", "bind": "newname", "title": "NEW PAGE", "border": True, "modal": True, "when": "naming",
          "float": {"width": 44, "height": 3, "at": "center"}, "placeholder": "A name, then Enter", "required": True,
          "keys": {"enter": {"do": [{"put": "naming", "value": False}] + add_page("{store.newname}") + [open_page("{store.newname}")]},
                   "esc": {"do": [{"put": "naming", "value": False}, {"focus": "$prev"}]}}}

ui = {
    "root": {"type": "stack", "children": [{"type": "row", "gap": 2, "pad": [1, 0], "children": [side, main]}, dialog]},
    # Today, from the clock, asked again every minute so a journal left open moves on at midnight.
    "data": {"today": {"cmd": ["date", '+{"id": "%Y-%m-%d", "short": "%a %-d %b", "long": "%A, %B %-d %Y"}'], "every": 60}},
    "keys": KEYS,
    "status_bar": False,
    "store": {
        "zen": False, "naming": False, "newname": "",
        "here": {"kind": "page", "id": "Welcome", "title": "Welcome"},
        "days": [{"id": "2026-10-11", "short": "Sun 11 Oct", "long": "Sunday, October 11 2026"},
                 {"id": "2026-10-12", "short": "Mon 12 Oct", "long": "Monday, October 12 2026"}],
        "pages": [{"id": "Welcome"}, {"id": "Reading list"}, {"id": "Garden"}],
        "page/Welcome": "Daybook keeps a page a day, and pages for everything else.\n\n- [ ] Write today's entry (⌘T)\n- [ ] Make a page for [[Garden]] notes\n- [ ] Try zen mode (⌥Z)\n",
        "page/Reading list": "- [x] The Peregrine\n- [ ] Pilgrim at Tinker Creek\n- [ ] The Living Mountain\n",
        "page/Garden": "- [ ] Plant the garlic before the frost\n- [ ] Mulch the raspberries\n  - [ ] Ask the farm for straw\n\nSee [[Reading list]].\n",
        "day/2026-10-11": "Rain all morning. Fixed the shed door.\n\n- [x] Call the library\n- [ ] Order seeds\n",
        "day/2026-10-12": "A walk by the river before work.\n\n- [ ] Order seeds\n- [x] Return the ladder\n",
    },
    "focus": "page",
}
with open(f"{L}/journal.json", "w") as f:
    json.dump(ui, f, indent=1, ensure_ascii=False)
    f.write("\n")
