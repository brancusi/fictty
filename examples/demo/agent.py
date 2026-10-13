#!/usr/bin/env python3
"""The demo's agent: it builds Kestrel Cycles, a bike workshop's app, in a running fictty window,
one command at a time, and logs each command on the screen as it goes. It's a script, not a
model, so the demo plays the same every time; every step is a command anyone could type.

    examples/demo/open.sh                # the window and this agent, together
    python3 examples/demo/agent.py FICTTY SOCKET [--fast]

The company, the people and the numbers are made up.
"""
import json, random, subprocess, sys, time

F, SOCK = sys.argv[1], sys.argv[2]
FAST = "--fast" in sys.argv
LOG = []


def fictty(*args, data=None):
    """Run a fictty command against the screen; return what it printed."""
    raw = None if data is None else (data if isinstance(data, str) and args[0] in ("push", "patch", "layers") else json.dumps(data))
    r = subprocess.run([F, "--socket", SOCK, *args], input=raw.encode() if raw is not None else None, capture_output=True)
    return r.stdout.decode()


def pause(s):
    time.sleep(s / 6 if FAST else s)


def step(msg, cmd, *args, data=None, wait=1.2):
    """Do one thing, and say so in the log on screen: what, and the command that did it."""
    out = fictty(*args, data=data) if args else ""
    LOG.insert(0, {"t": time.strftime("%H:%M:%S"), "msg": msg, "cmd": cmd})
    fictty("put", "agent_log", "-", "--by", "agent", data=LOG[:60])
    pause(wait)
    return out


def toast(text, tone="accent"):
    fictty("toast", text, "--tone", tone)


# The app, as data. Each beat pushes or patches more of it.

HEADER = {"type": "text", "size": 1, "bind": "shop",
          "text": "<heading+accent>◆ KESTREL CYCLES</>   <dim>workshop · {open|true=<pos>● open</>;*=closed}</>\t<dim>t theme · tab move · q quit</>"}

LOGPANE = {"type": "list", "id": "log", "size": 46, "title": "AGENT", "bind": "agent_log", "lines": 2,
           "empty": "waiting for the agent…", "item": "<accent>{t}</>  {msg}\n<dim>  $ {cmd}</>"}

INBOX = {"type": "row", "gap": 2, "children": [
    {"type": "col", "size": "40%", "gap": 1, "children": [
        {"type": "input", "id": "find", "size": 1, "placeholder": "Search messages"},
        {"type": "list", "id": "inbox", "bind": "inbox", "filter": "find", "lines": 2, "empty": "no messages yet",
         "item": "{unread|true=<accent>●</> ;*=  }<b>{from}</>\t<dim>{when}</>\n  {subject}",
         "keys": {"e": {"do": [{"remove": "inbox"}, {"toast": "Archived", "tone": "pos"}]}}},
    ]},
    {"type": "col", "gap": 1, "children": [
        {"type": "text", "id": "msg", "bind": "@inbox", "pad": [1, 0],
         "text": "<b>{subject}</>\n<dim>{from} · {when}</>\n\n{body}"},
        {"type": "editor", "id": "reply", "title": "REPLY", "size": "35%", "syntax": "markdown", "pad": [1, 0],
         "placeholder": "Write a reply…"},
        {"type": "row", "size": 1, "gap": 1, "children": [
            {"type": "button", "id": "send", "label": "Send ↵", "size": 12, "click": {"toast": "Sent", "tone": "pos"}},
            {"type": "button", "id": "archive", "label": "Archive", "size": 12,
             "click": {"do": [{"remove": "inbox"}, {"toast": "Archived", "tone": "pos"}]}},
            {"type": "text", "text": ""},
        ]},
    ]},
]}

JOB = "<b>{bike}</>\t<dim>{due}</>\n  {work} <dim>· {who}</>"
JOBS = {"type": "row", "gap": 2, "children": [
    {"type": "list", "id": f"jobs_{k}", "title": t, "bind": f"jobs_{k}", "lines": 2, "empty": "nothing here", "item": JOB}
    for k, t in [("booked", "BOOKED"), ("stand", "ON THE STAND"), ("ready", "READY TO COLLECT")]
]}

STOCK = {"type": "col", "gap": 1, "children": [
    {"type": "input", "id": "part", "size": 1, "placeholder": "Filter parts (try “chain”)"},
    {"type": "table", "id": "stock", "bind": "stock", "filter": "part", "columns": [
        {"title": "Part", "value": "<b>{name}</>"},
        {"title": "Brand", "value": "<dim>{brand}</>"},
        {"title": "Bin", "value": "{bin}"},
        {"title": "On hand", "value": "{qty|0=<neg>out</>;*={qty}}", "align": "right"},
        {"title": "Price", "value": "£{price}", "align": "right"},
    ]},
]}

FLOOR = {"type": "col", "gap": 1, "children": [
    {"type": "row", "size": "120px", "gap": 2, "children": [
        {"type": "text", "id": f"stat{k}", "title": t, "bind": "floor", "variant": "card", "pad": [1, 0], "text": s} for k, (t, s) in enumerate([
            ("TODAY", "<stat>£{takings}</>\n{±dtakings}% <dim>vs last Tuesday</>"),
            ("ON THE STANDS", "<stat>{stands}</> <dim>of 4</>\n<dim>{waiting} waiting</>"),
            ("AVERAGE JOB", "<stat>{avg}</> <dim>min</>\n{±davg} min <dim>vs last week</>"),
            ("WALK-INS", "<stat>{walkins}</>\n<dim>since 9:00</>"),
        ])
    ]},
    {"type": "list", "id": "floorlog", "title": "THE FLOOR, LIVE", "bind": "floorlog", "item": "<dim>{t}</>  {kind|done=<pos>done</>;paid=<pos>paid</>;low=<warn>low</>;in=<accent>in</>;sold=<accent>sold</>;stand=<accent>on stand</>}  {msg}"},
]}

NOTES = {"type": "editor", "id": "notes", "title": "NOTES", "syntax": "markdown", "pad": [1, 0],
         "placeholder": "Notes for the shop…"}

TABS = [("Inbox <accent>{inbox}</>", INBOX), ("Jobs", JOBS), ("Stock", STOCK), ("Floor", FLOOR), ("Notes", NOTES)]


def app(upto):
    """The app with its first `upto` tabs, and the agent's log beside it."""
    tabs = TABS[:upto]
    return {
        "root": {"type": "col", "gap": 1, "children": [
            HEADER,
            {"type": "row", "gap": 2, "children": [main(upto), LOGPANE]},
        ]},
        "store": {"agent_log": [], "shop": {"open": True}},
        "keys": {"t": {"theme": "next"}},
        "focus": "inbox" if upto == 1 else None,
    }


def main(upto):
    tabs = TABS[:upto]
    return {"type": "tabs", "id": "main", "bind": "inbox_count", "tabs": [t for t, _ in tabs], "children": [n for _, n in tabs]}


# The data. Made up.

MAIL = [
    {"id": "m1", "from": "Priya Natarajan", "when": "08:12", "unread": True, "subject": "Squeaky disc brakes again",
     "body": "Hi, the rear brake on my gravel bike is squealing again after the rain this weekend. Can you fit it in this week? I can drop it off Thursday morning."},
    {"id": "m2", "from": "Tom Okafor", "when": "08:40", "unread": True, "subject": "Wheel build quote",
     "body": "Looking for a pair of hand-built wheels for touring: 36 spokes, dynamo front. What would that run to, and how long?"},
    {"id": "m3", "from": "Hollis & Rowe Supply", "when": "09:05", "unread": False, "subject": "Your order #4471 has shipped",
     "body": "12 × 11-speed chains, 6 × tubeless sealant 500 ml, 20 × brake pads (organic). Arriving Wednesday."},
    {"id": "m4", "from": "Maya Lindqvist", "when": "09:31", "unread": True, "subject": "Is my bike ready?",
     "body": "Just checking whether the service on the blue commuter is done. I could come by after work."},
    {"id": "m5", "from": "Riverside Cycling Club", "when": "10:02", "unread": False, "subject": "Saturday maintenance class",
     "body": "Twelve signed up for the puncture class. Could we borrow two stands and a box of levers?"},
    {"id": "m6", "from": "Dev Patel", "when": "10:47", "unread": True, "subject": "Cracked frame?",
     "body": "Found a hairline mark near the bottom bracket on my carbon road bike. Should I stop riding it until you've had a look?"},
]

JOBS_DATA = {
    "booked": [
        {"id": "j1", "bike": "Gravel · Priya N.", "due": "Thu", "work": "brake pads, rotor clean", "who": "Sam"},
        {"id": "j2", "bike": "Road · Dev P.", "due": "today", "work": "frame inspection", "who": "Ana"},
        {"id": "j3", "bike": "Cargo · Lee W.", "due": "Fri", "work": "full service", "who": "Sam"},
    ],
    "stand": [
        {"id": "j4", "bike": "Commuter · Maya L.", "due": "today", "work": "service, new chain", "who": "Ana"},
        {"id": "j5", "bike": "Kids' 20\" · the Rossis", "due": "Wed", "work": "puncture, gears", "who": "Jo"},
    ],
    "ready": [
        {"id": "j6", "bike": "Tourer · Tom O.", "due": "done", "work": "headset, bar tape", "who": "Jo"},
    ],
}

STOCK_DATA = [
    {"id": "p1", "name": "11-speed chain", "brand": "Hollis", "bin": "A3", "qty": 4, "price": "34.00"},
    {"id": "p2", "name": "12-speed chain", "brand": "Hollis", "bin": "A4", "qty": 0, "price": "46.00"},
    {"id": "p3", "name": "Chain checker", "brand": "Parkline", "bin": "T1", "qty": 2, "price": "12.50"},
    {"id": "p4", "name": "Brake pads, organic", "brand": "Stopwell", "bin": "B1", "qty": 18, "price": "14.00"},
    {"id": "p5", "name": "Brake pads, sintered", "brand": "Stopwell", "bin": "B2", "qty": 7, "price": "17.00"},
    {"id": "p6", "name": "Rotor 160 mm", "brand": "Stopwell", "bin": "B5", "qty": 3, "price": "29.00"},
    {"id": "p7", "name": "Tubeless sealant 500 ml", "brand": "Greenseal", "bin": "C2", "qty": 6, "price": "19.00"},
    {"id": "p8", "name": "Inner tube 700×28", "brand": "Roundway", "bin": "C4", "qty": 31, "price": "6.50"},
    {"id": "p9", "name": "Bar tape, cork", "brand": "Wrapco", "bin": "D1", "qty": 9, "price": "16.00"},
    {"id": "p10", "name": "Cassette 11–34", "brand": "Hollis", "bin": "A7", "qty": 2, "price": "58.00"},
    {"id": "p11", "name": "Spokes 2.0 mm (box)", "brand": "Truewheel", "bin": "W1", "qty": 5, "price": "42.00"},
    {"id": "p12", "name": "Dynamo hub", "brand": "Truewheel", "bin": "W3", "qty": 1, "price": "129.00"},
]

# What happens on the floor: a kind, which the log's template styles, and plain text. Data carries
# no markup; the UI decides how a kind looks.
FLOOR_EVENTS = [("done", "Commuter · Maya L. back on the rack"), ("in", "walk-in: puncture, 700×28"),
                ("sold", "2 × inner tube 700×28"), ("low", "11-speed chain: 3 left"),
                ("stand", "Kids' 20\" · the Rossis"), ("sold", "bar tape, cork"),
                ("paid", "Tourer · Tom O. collected"), ("in", "walk-in: gears skipping")]


def type_into(key, text, every=0.05):
    """Type text into a text entry a few characters at a time, as a person would."""
    for i in range(1, len(text) + 1, 3):
        fictty("put", key, "-", "--by", "agent", data=text[:i + 2])
        pause(every)


def run():
    pause(1.5)
    step("Hello. I'm going to build an app in this window.", "fictty screen --nodes", "screen", "--nodes", wait=2)

    # 1. A whole UI, as one value.
    step("Pushed the app: a header, an inbox, my log. It's one JSON value.", "fictty push app.json",
         "push", "-", data=json.dumps(app(1)), wait=1.6)

    # 2. Data arrives in the store; the UI only names it.
    for i in range(1, len(MAIL) + 1):
        fictty("put", "inbox", "-", "--by", "agent", data=MAIL[:i])
        fictty("put", "inbox_count", "-", "--by", "agent", data={"inbox": sum(m["unread"] for m in MAIL[:i])})
        pause(0.35)
    step("Six messages came in. The list is bound to the store entry `inbox`.", "fictty put inbox mail.json", wait=1.6)

    # 3. Read it back the way an agent checks its work.
    nodes = fictty("screen", "--nodes")
    shown = next((l for l in nodes.splitlines() if "list inbox" in l), "")
    count = shown.split("/", 1)[1].split(")", 1)[0] if "/" in shown else "?"
    step(f"Read the screen back: the inbox shows {count} rows, the first selected.", "fictty screen --nodes", wait=1.8)

    # 4. Select, then reply in the person's editor.
    step("Selected Dev's message: the reader follows the selection.", "echo '{\"selected\": 5}' | fictty set inbox -",
         "set", "inbox", "-", data={"selected": 5}, wait=1.6)
    type_into("reply", "Hi Dev, please don't ride it until we've looked. Bring it in today and Ana will inspect the frame; no charge for the check.")
    step("Wrote a reply in the editor. Each write is an edit you can undo.", "fictty put reply -", wait=1.6)
    fictty("put", "reply", "-", "--by", "agent", data="")
    toast("Sent to Dev", "pos")

    # 5. More of the app: patch one node, keep everything else.
    for k, rows in JOBS_DATA.items():
        fictty("put", f"jobs_{k}", "-", "--by", "agent", data=rows)
    step("Added a Jobs board by replacing one node. Your selection stayed put.", "fictty patch main jobs.json",
         "patch", "main", "-", data=json.dumps(main(2)), wait=0.8)
    step("Switched to it.", "echo '{\"tab\": 1}' | fictty set main -", "set", "main", "-", data={"tab": 1}, wait=1.6)
    booked = JOBS_DATA["booked"][:]
    stand = JOBS_DATA["stand"][:]
    dev = booked.pop(1)
    fictty("put", "jobs_booked", "-", "--by", "agent", data=booked)
    pause(0.4)
    fictty("put", "jobs_stand", "-", "--by", "agent", data=[dev] + stand)
    step("Moved Dev's bike onto a stand: two store writes.", "fictty put jobs_stand -", wait=1.8)

    # 6. A table with a filter.
    fictty("put", "stock", "-", "--by", "agent", data=STOCK_DATA)
    step("Added Stock, a table.", "fictty patch main stock.json", "patch", "main", "-", data=json.dumps(main(3)), wait=0.6)
    fictty("set", "main", "-", data={"tab": 2})
    pause(1.2)
    type_into("part", "chain", every=0.2)
    step("Filtered it by typing into the field, as you would.", "echo '\"chain\"' | fictty put part -", wait=2)
    fictty("put", "part", "-", "--by", "agent", data="")

    # 7. Live numbers.
    step("Added the Floor: numbers that change while you watch.", "fictty patch main floor.json",
         "patch", "main", "-", data=json.dumps(main(4)), wait=0.4)
    fictty("set", "main", "-", data={"tab": 3})
    floor = {"takings": 1240, "stands": 3, "waiting": 2, "avg": 47, "walkins": 5}
    events = []
    for i in range(12):
        live(floor, events)
        pause(0.7)
    step("Each tick is one `merge` into the store, nothing else.", "fictty merge floor -", wait=0.4)

    # 8. Themes: the agent never picks a colour.
    for _ in range(3):
        step("Changed the theme. The UI only names roles; the theme says how they look.", "fictty theme next", "theme", "next", wait=1.6)
        live(floor, events)
    style = fictty("screen", "--nodes", "--style").splitlines()
    card = next((style[i + 1] for i, l in enumerate(style[:-1]) if '"TODAY"' in l), "")
    # `border <width> <colour>`.
    border = card.split("border ", 1)[1].split(" ")[1] if "border " in card else "?"
    step(f"Read back the styles: the TODAY card's border is {border} now.", "fictty screen --nodes --style", wait=1.6)

    # 9. A typo: wrong names are problems, on the status bar and in the state, never silence.
    card = FLOOR["children"][0]["children"][0]
    step("Patched a card with a typo in its variant…", "fictty patch stat0 card.json",
         "patch", "stat0", "-", data=json.dumps({**card, "variant": "crad"}), wait=1.4)
    problems = json.loads(fictty("get", "--state") or "{}").get("problems", [])
    step(f"…and it says so: “{problems[0] if problems else 'no problems'}”. Fixed.", "fictty get --state",
         "patch", "stat0", "-", data=json.dumps(card), wait=1.8)

    # 10. Notes, then hand over.
    step("Added Notes, a Markdown editor kept in the store.", "fictty patch main notes.json",
         "patch", "main", "-", data=json.dumps(main(5)), wait=0.4)
    fictty("set", "main", "-", data={"tab": 4})
    type_into("notes", "# Tuesday\n\n- [x] order 12-speed chains\n- [ ] call Priya about Thursday\n- [ ] stands for the club class\n")
    step("That's the app. It's yours now: click, type, tab around. I'll keep the floor live.", "", wait=0.5)
    toast("Your turn", "pos")
    while True:
        live(floor, events)
        pause(1.5)


def live(floor, events):
    """One tick of the shop floor: a merge of the numbers, and a line in its log."""
    floor["takings"] += random.choice([0, 0, 6, 13, 34, 58])
    floor["walkins"] += random.random() < 0.2
    floor["stands"] = random.randint(2, 4)
    floor["waiting"] = random.randint(0, 3)
    floor["avg"] = random.randint(41, 55)
    fictty("merge", "floor", "-", "--by", "agent", data={**floor, "takings": f"{floor['takings']:,}",
           "dtakings": round(random.uniform(-4, 12), 1), "davg": random.randint(-6, 4)})
    kind, msg = random.choice(FLOOR_EVENTS)
    events.insert(0, {"t": time.strftime("%H:%M:%S"), "kind": kind, "msg": msg})
    fictty("put", "floorlog", "-", "--by", "agent", data=events[:40])


if __name__ == "__main__":
    try:
        run()
    except (KeyboardInterrupt, BrokenPipeError):
        pass
