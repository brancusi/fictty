# The wringer's six themes, as whole styles. Each is a palette plus a handful of decisions (type,
# spacing, edges, depth, states); everything a screen asks for by name (type roles, variants,
# tokens) exists in all six, so any screen wears any theme.
# Writes themes.json beside this file:  python3 examples/window/themes.py
import json, os
L = os.path.dirname(os.path.abspath(__file__))

# What every theme names. Inline roles fit the 24px row; `display` and `stat` need a node sized
# in pixels.
ROLES = ["display", "stat", "heading", "label", "body", "caption", "num", "mono"]
VARIANTS = ["card", "quiet", "hero", "alert", "bare"]


def types(ui, display, mono, *, weight=600, display_weight=700, display_size=48, tracking=0.8):
    return {
        "display": {"font": "display", "size": display_size, "weight": display_weight, "line": round(display_size * 1.2)},
        "stat": {"font": "display", "size": 28, "weight": display_weight, "line": 36, "tabular": True},
        "heading": {"font": "ui", "size": 16, "weight": weight, "line": 24},
        "label": {"font": "ui", "size": 11, "weight": weight, "line": 24, "tracking": tracking},
        "body": {"font": "ui", "size": 14, "line": 24},
        "caption": {"font": "ui", "size": 12, "line": 24},
        "num": {"font": "ui", "size": 14, "line": 24, "tabular": True},
        "mono": {"font": "mono", "size": 13, "line": 24},
    }, {"ui": ui, "display": display, "mono": mono}


def theme(tokens, fonts_types, *, space, radius, depth, parts, variants):
    t, fonts = fonts_types
    return {"tokens": tokens, "fonts": fonts, "type": t, "space": space, "radius": radius, "depth": depth, "parts": parts, "variants": variants}


def palette(**k):
    # Every theme has the screens' tokens; the chart colours follow the accents.
    k.setdefault("c1", k["accent"])
    k.setdefault("c2", k["violet"])
    k.setdefault("c3", k["amber"])
    k.setdefault("c4", k["rose"])
    k.setdefault("cyan", k["accent"])
    k.setdefault("scrim", k["bg"])
    return k


SPACE = {"s1": "4px", "s2": "8px", "s3": "12px", "s4": "16px", "s6": "24px"}

themes = {}

# Night City: hard edges, acid yellow on near-black, condensed type, neon glow on what's live.
themes["night-city"] = theme(
    palette(edge="square", bg="#050506", fg="#e6e1d3", dim="#6d6858", border="#2c2a1f", panel="#0b0b0a", ring="#3a0d1f", hover="#16140c",
            accent="#f3e600", violet="#b026ff", amber="#ff9f1c", rose="#ff2079", cyan="#00f0ff", pos="#00ff9c", neg="#ff2079", warn="#f3e600"),
    types("Avenir Next Condensed", "Avenir Next Condensed", "Menlo", weight=700, display_weight=800, display_size=52, tracking=1.6),
    space=SPACE, radius={"r1": 0, "r2": 0, "r3": 0},
    depth={"e1": [], "e2": [], "glow": [{"blur": 14, "color": "accent", "alpha": 0.45}], "neon": [{"blur": 10, "color": "rose", "alpha": 0.5}]},
    parts={
        "screen": {"type": "body"},
        "panel": {"bg": "panel", "pad": "s2", "border": {"color": "border"}},
        "title": {"fg": "dim", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "accent",
        "button": {"fg": "accent", "bg": "bg", "type": "label", "pad": ["s3", 0], "border": {"color": "accent"}},
        "button.hover": {"bg": "hover"},
        "button.focused": {"fg": "bg", "bg": "accent", "shadow": "glow"},
        "input": {"bg": "bg", "border": {"color": "border"}}, "input.focused": {"border": {"color": "accent"}, "shadow": "glow"},
        "row.hover": {"bg": "hover"}, "row.selected": {"fg": "bg", "bg": "accent", "bold": True}, "row.selected.blur": {"fg": "accent", "bg": "panel"},
        "tabs": {"shape": "underline"}, "tabs.active": {"fg": "accent", "type": "label"}, "tabs.idle": {"fg": "dim", "type": "label"}, "tabs.underline": "accent",
        "table.header": {"fg": "dim", "type": "label"}, "table.total": "fg+b",
        "editor.caret": "on-accent", "editor.selection": "fg+on-ring", "input.placeholder": "dim", "empty": "dim",
        "status": {"fg": "dim", "bg": "panel", "type": "label"}, "status.error": "rose",
        "toast": {"fg": "fg", "bg": "panel", "border": {"color": "accent"}, "shadow": "glow"},
        "md.heading": "accent+b", "md.task": "accent+b", "md.done": "dim+s", "md.link": "cyan+u", "md.code": "cyan", "md.quote": "dim+i",
        "spinner": "accent", "tree.guide": "dim", "scrollbar": "border", "scrollbar.thumb": "accent",
    },
    variants={
        "card": {"style": {"bg": "panel", "border": {"color": "border"}}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"border": {"color": "accent"}, "shadow": "glow"}, "title": "accent+b"},
        "alert": {"style": {"fg": "rose", "border": {"color": "rose"}, "shadow": "neon"}, "title": "rose+b"},
        "bare": {},
    },
)

# Glass: a dark, calm SaaS console. Soft gradients, rounded corners, two-layer shadows, a focus ring.
themes["glass"] = theme(
    palette(bg="#0a0b10", fg="#d6dae3", dim="#6b7385", border="#20263a", panel="#11131b", ring="#1b2533", hover="#151a26",
            accent="#39d6e6", violet="#8c7eff", amber="#ffbd4f", rose="#ff7aa2", pos="#4fd18b", neg="#ff7aa2", warn="#ffbd4f", deep="#0d0f16"),
    types("Avenir Next", "Avenir Next", "Menlo", weight=600, display_weight=700, display_size=50),
    space=SPACE, radius={"r1": 6, "r2": 10, "r3": 16},
    depth={"e1": [{"y": 1, "blur": 2, "color": "black", "alpha": 0.4}], "e2": [{"y": 10, "blur": 30, "color": "black", "alpha": 0.5}, {"y": 1, "blur": 3, "color": "black", "alpha": 0.4}],
           "lift": [{"y": 4, "blur": 16, "color": "accent", "alpha": 0.25}]},
    parts={
        "screen": {"type": "body"},
        "panel": {"bg": {"from": "panel", "to": "deep", "angle": 180}, "radius": "r2", "pad": "s3", "border": {"color": "border"}, "shadow": "e2"},
        "title": {"fg": "dim", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "accent",
        "button": {"fg": "fg", "bg": "ring", "radius": "r1", "pad": ["s4", 0], "type": "heading", "shadow": "e1"},
        "button.hover": {"bg": "hover"},
        "button.focused": {"fg": "bg", "bg": {"from": "accent", "to": "violet", "angle": 120}, "ring": {"width": 2, "offset": 2, "color": "accent"}, "shadow": "lift"},
        "input": {"bg": "deep", "radius": "r1", "border": {"color": "border"}}, "input.focused": {"border": {"color": "accent"}, "ring": {"width": 2, "offset": 1, "color": "ring"}},
        "row": {"radius": "r1", "pad": ["s3", 0]}, "row.hover": {"bg": "hover"}, "row.selected": {"fg": "fg", "bg": "ring", "bold": True}, "row.selected.blur": {"fg": "fg", "bg": "panel"},
        "tabs": {"shape": "pill"}, "tabs.active": {"fg": "bg", "bg": "accent", "radius": 99, "pad": ["s3", 0], "bold": True}, "tabs.idle": {"fg": "dim", "radius": 99, "pad": ["s3", 0]},
        "table.header": {"fg": "violet", "type": "label"}, "table.total": "fg+b",
        "editor.caret": "bg+on-accent", "editor.selection": "fg+on-ring", "input.placeholder": "dim+i", "empty": "dim+i",
        "status": {"fg": "dim", "bg": "deep", "type": "caption"}, "status.error": "rose",
        "toast": {"fg": "fg", "bg": "panel", "radius": "r2", "border": {"color": "border"}, "shadow": "e2"},
        "md.task": "accent+b", "md.done": "dim+s", "spinner": "accent", "tree.guide": "dim", "tabs.underline": "accent", "scrollbar": "border", "scrollbar.thumb": "accent",
    },
    variants={
        "card": {"style": {"bg": {"from": "panel", "to": "deep", "angle": 160}, "radius": "r3", "shadow": "e2", "border": {"color": "border"}}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"bg": {"from": "ring", "to": "panel", "angle": 160}, "radius": "r3", "border": {"color": "accent"}, "shadow": "lift"}, "title": "accent+b"},
        "alert": {"style": {"fg": "rose", "radius": "r2", "border": {"color": "rose"}}, "title": "rose+b"},
        "bare": {},
    },
)

# Paper: editorial. A serif for reading, small caps sans labels, hairlines, no shadow at all.
themes["paper"] = theme(
    palette(bg="#f6f1e7", fg="#1d1a16", dim="#7a7064", border="#d8ccb8", panel="#f6f1e7", ring="#ebdfca", hover="#efe7d8",
            accent="#b4442b", violet="#4b4b8f", amber="#a8670a", rose="#b4442b", pos="#3d7a3a", neg="#b4442b", warn="#a8670a"),
    types("Charter", "Didot", "Menlo", weight=600, display_weight=700, display_size=56, tracking=1.2),
    space=SPACE, radius={"r1": 2, "r2": 2, "r3": 2},
    depth={"e1": [], "e2": []},
    parts={
        "screen": {"type": "body"},
        "panel": {"bg": "panel", "radius": 0, "pad": ["s2", "s1"], "border": {"color": "border"}},
        "title": {"fg": "dim", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "fg",
        "button": {"fg": "fg", "bg": "bg", "radius": "r1", "pad": ["s4", 0], "border": {"color": "fg"}, "type": "label"},
        "button.hover": {"bg": "hover"},
        "button.focused": {"fg": "bg", "bg": "fg"},
        "input": {"bg": "bg", "radius": 0, "border": {"color": "border"}}, "input.focused": {"border": {"color": "fg"}},
        "row": {"radius": 0, "pad": ["s2", 0]}, "row.hover": {"bg": "hover"}, "row.selected": {"fg": "accent", "bg": "ring"}, "row.selected.blur": {"fg": "fg", "bg": "hover"},
        "tabs": {"shape": "underline"}, "tabs.active": {"fg": "fg", "type": "label"}, "tabs.idle": {"fg": "dim", "type": "label"}, "tabs.underline": "accent",
        "table.header": {"fg": "dim", "type": "label"}, "table.total": "fg+b",
        "editor.caret": "on-fg", "editor.selection": "fg+on-ring", "input.placeholder": "dim+i", "empty": "dim+i",
        "status": {"fg": "dim", "bg": "bg", "type": "caption", "border": {"color": "border"}}, "status.error": "accent",
        "toast": {"fg": "fg", "bg": "bg", "radius": 0, "border": {"color": "fg"}},
        "md.heading": "fg+b", "md.task": "accent+b", "md.done": "dim+s", "md.link": "accent+u", "md.quote": "dim+i",
        "spinner": "accent", "tree.guide": "border", "scrollbar": "border", "scrollbar.thumb": "dim",
    },
    variants={
        "card": {"style": {"bg": "bg", "radius": 0, "border": {"color": "border"}}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"bg": "ring", "radius": 0, "border": {"color": "fg"}}, "title": "fg+b"},
        "alert": {"style": {"fg": "accent", "border": {"color": "accent"}}, "title": "accent+b"},
        "bare": {},
    },
)

# Phosphor: a green CRT. All monospace, dashed rules, everything that's live glows.
themes["phosphor"] = theme(
    palette(edge="square", bg="#040904", fg="#9dffae", dim="#3a7a47", border="#1c3b24", panel="#071207", ring="#123a1c", hover="#0d1f11",
            accent="#c4ff6a", violet="#6affd0", amber="#ecff7a", rose="#ff8f6a", pos="#9dffae", neg="#ff7a6a", warn="#ecff7a", c3="#9dffae", c4="#ecff7a"),
    types("Menlo", "Monaco", "Menlo", weight=700, display_weight=400, display_size=44, tracking=0),
    space=SPACE, radius={"r1": 0, "r2": 0, "r3": 0},
    depth={"e1": [], "e2": [], "glow": [{"blur": 12, "color": "fg", "alpha": 0.3}], "hot": [{"blur": 16, "color": "accent", "alpha": 0.5}]},
    parts={
        "screen": {"type": "mono"},
        "panel": {"bg": "bg", "pad": "s2", "border": {"color": "dim", "style": "dashed"}},
        "title": {"fg": "dim", "bold": True}, "title.focused": {"fg": "accent", "bold": True},
        "border.focused": "accent",
        "button": {"fg": "fg", "bg": "bg", "pad": ["s2", 0], "border": {"color": "dim", "style": "dashed"}},
        "button.hover": {"fg": "accent", "shadow": "glow"},
        "button.focused": {"fg": "bg", "bg": "accent", "shadow": "hot"},
        "input": {"bg": "bg", "border": {"color": "dim", "style": "dashed"}}, "input.focused": {"border": {"color": "accent", "style": "solid"}, "shadow": "glow"},
        "row.hover": {"bg": "hover"}, "row.selected": {"fg": "bg", "bg": "fg"}, "row.selected.blur": {"fg": "fg", "bg": "ring"},
        "tabs": {"shape": "segment"}, "tabs.active": {"fg": "bg", "bg": "fg"}, "tabs.idle": {"fg": "dim"},
        "table.header": "accent+b", "table.total": "fg+b",
        "editor.caret": "bg+on-accent", "editor.selection": "bg+on-fg", "input.placeholder": "dim", "empty": "dim",
        "status": {"fg": "dim", "bg": "panel"}, "status.error": "rose",
        "toast": {"fg": "fg", "bg": "bg", "border": {"color": "accent", "style": "dashed"}, "shadow": "glow"},
        "md.task": "accent+b", "md.done": "dim+s", "spinner": "accent", "tree.guide": "dim", "tabs.underline": "accent", "scrollbar": "border", "scrollbar.thumb": "accent",
    },
    variants={
        "card": {"style": {"bg": "panel", "border": {"color": "dim", "style": "dashed"}}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"border": {"color": "accent"}, "shadow": "glow"}, "title": "accent+b"},
        "alert": {"style": {"fg": "rose", "border": {"color": "rose", "style": "dashed"}}, "title": "rose+b"},
        "bare": {},
    },
)

# Brutal: a poster. Black two-pixel borders, hard offset shadows, heavy grotesque type, primaries.
themes["brutal"] = theme(
    palette(edge="square", bg="#ffffff", fg="#000000", dim="#555555", border="#000000", panel="#ffffff", ring="#ffe600", hover="#f0f0f0",
            accent="#0000ff", violet="#0000ff", amber="#ff5500", rose="#ff0033", pos="#008a00", neg="#ff0033", warn="#ff5500", scrim="#ffffff"),
    types("Helvetica Neue", "Arial Black", "Menlo", weight=700, display_weight=900, display_size=54, tracking=0.4),
    space=SPACE, radius={"r1": 0, "r2": 0, "r3": 0},
    depth={"e1": [{"x": 3, "y": 3, "color": "black"}], "e2": [{"x": 6, "y": 6, "color": "black"}], "pop": [{"x": 4, "y": 4, "color": "accent"}]},
    parts={
        "screen": {"type": "body"},
        "panel": {"bg": "panel", "pad": "s2", "border": {"width": 2, "color": "border"}, "shadow": "e1"},
        "title": {"fg": "fg", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "accent",
        "button": {"fg": "fg", "bg": "ring", "pad": ["s3", 0], "border": {"width": 2, "color": "fg"}, "shadow": "e1", "type": "label"},
        "button.hover": {"bg": "amber"},
        "button.focused": {"fg": "bg", "bg": "accent", "shadow": "e2"},
        "input": {"bg": "bg", "border": {"width": 2, "color": "fg"}}, "input.focused": {"shadow": "pop"},
        "row.hover": {"bg": "hover"}, "row.selected": {"fg": "fg", "bg": "ring", "bold": True}, "row.selected.blur": {"fg": "fg", "bg": "hover"},
        "tabs": {"shape": "segment"}, "tabs.active": {"fg": "bg", "bg": "fg", "type": "label"}, "tabs.idle": {"fg": "fg", "type": "label"},
        "table.header": {"fg": "fg", "type": "label"}, "table.total": "fg+b",
        "editor.caret": "on-accent", "editor.selection": "fg+on-ring", "input.placeholder": "dim", "empty": "dim",
        "status": {"fg": "bg", "bg": "fg", "type": "label"}, "status.error": "ring",
        "toast": {"fg": "fg", "bg": "ring", "border": {"width": 2, "color": "fg"}, "shadow": "e1"},
        "md.task": "accent+b", "md.done": "dim+s", "spinner": "accent", "tree.guide": "fg", "tabs.underline": "fg", "scrollbar": "fg", "scrollbar.thumb": "accent",
    },
    variants={
        "card": {"style": {"bg": "bg", "border": {"width": 2, "color": "fg"}, "shadow": "e1"}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"bg": "ring", "border": {"width": 2, "color": "fg"}, "shadow": "e2"}, "title": "fg+b"},
        "alert": {"style": {"fg": "fg", "bg": "rose", "border": {"width": 2, "color": "fg"}, "shadow": "e1"}, "title": "fg+b"},
        "bare": {},
    },
)

# Soft: a friendly consumer app. Big radii, pills, no borders, lavender-tinted soft shadows.
themes["soft"] = theme(
    palette(bg="#f6f4fb", fg="#2d2a3e", dim="#9a96ad", border="#e4e0ef", panel="#ffffff", ring="#ece7ff", hover="#efebfa",
            accent="#7c5cff", violet="#7c5cff", amber="#f4a340", rose="#f2668b", cyan="#38bdf8", pos="#22b07d", neg="#f2668b", warn="#f4a340"),
    types("Avenir Next", "Avenir Next", "Menlo", weight=600, display_weight=600, display_size=50, tracking=0.6),
    space=SPACE, radius={"r1": 10, "r2": 14, "r3": 20},
    depth={"e1": [{"y": 2, "blur": 8, "color": "accent", "alpha": 0.08}, {"y": 1, "blur": 2, "color": "black", "alpha": 0.05}],
           "e2": [{"y": 12, "blur": 32, "color": "accent", "alpha": 0.14}, {"y": 2, "blur": 6, "color": "black", "alpha": 0.04}]},
    parts={
        "screen": {"type": "body"},
        "panel": {"bg": "panel", "radius": "r3", "pad": "s4", "border": {"width": 0}, "shadow": "e2"},
        "title": {"fg": "dim", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "accent",
        "button": {"fg": "accent", "bg": "ring", "radius": 99, "pad": ["s4", 0], "type": "heading"},
        "button.hover": {"bg": "hover"},
        "button.focused": {"fg": "panel", "bg": {"from": "accent", "to": "rose", "angle": 120}, "shadow": "e2"},
        "input": {"bg": "panel", "radius": "r1", "border": {"color": "border"}, "shadow": "e1"}, "input.focused": {"border": {"color": "accent"}, "ring": {"width": 3, "color": "ring"}},
        "row": {"radius": "r1", "pad": ["s3", 0]}, "row.hover": {"bg": "hover"}, "row.selected": {"fg": "accent", "bg": "ring", "bold": True}, "row.selected.blur": {"fg": "fg", "bg": "hover"},
        "tabs": {"shape": "pill"}, "tabs.active": {"fg": "panel", "bg": "accent", "radius": 99, "pad": ["s3", 0], "shadow": "e1"}, "tabs.idle": {"fg": "dim", "radius": 99, "pad": ["s3", 0]},
        "table.header": {"fg": "dim", "type": "label"}, "table.total": "fg+b",
        "editor.caret": "on-accent", "editor.selection": "fg+on-ring", "input.placeholder": "dim", "empty": "dim+i",
        "status": {"fg": "dim", "bg": "bg", "type": "caption"}, "status.error": "rose",
        "toast": {"fg": "fg", "bg": "panel", "radius": "r2", "shadow": "e2"},
        "md.task": "accent+b", "md.done": "dim+s", "spinner": "accent", "tree.guide": "border", "tabs.underline": "accent", "scrollbar": "border", "scrollbar.thumb": "accent",
    },
    variants={
        "card": {"style": {"bg": "panel", "radius": "r3", "shadow": "e2", "border": {"width": 0}}},
        "quiet": {"style": {"fg": "dim"}, "title": "dim"},
        "hero": {"style": {"bg": {"from": "ring", "to": "panel", "angle": 160}, "radius": "r3", "shadow": "e2", "border": {"width": 0}}, "title": "accent+b"},
        "alert": {"style": {"fg": "rose", "radius": "r2", "border": {"color": "rose"}}, "title": "rose+b"},
        "bare": {},
    },
)

for name, t in themes.items():
    assert sorted(t["type"]) == sorted(ROLES) and sorted(t["variants"]) == sorted(VARIANTS), name
with open(f"{L}/themes.json", "w") as f:
    json.dump(themes, f, indent=1)
    f.write("\n")
print("ok")
