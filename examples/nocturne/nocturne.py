# Nocturne: a listening room, made to see how far the window can be pushed. Every album's cover
# and backdrop are drawn here (Pillow), and the theme and the UI written beside them, into a
# folder outside the repo, so nothing binary is checked in. Every name, title and line is made
# up.
#   python3 examples/nocturne/nocturne.py [out] [--no-art]   # default out: $TMPDIR/fictty-nocturne
# --no-art writes only the theme and the UI (the tests' way: no Pillow needed, pictures missing).
import json, math, os, random, sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
ART = "--no-art" not in sys.argv
OUT = args[0] if args else os.path.join(os.environ.get("TMPDIR", "/tmp"), "fictty-nocturne")
os.makedirs(OUT, exist_ok=True)
if ART:
    try:
        from PIL import Image, ImageDraw, ImageEnhance, ImageFilter, ImageFont, ImageStat
    except ImportError:
        sys.exit("Nocturne draws its covers with Pillow: python3 -m pip install pillow")
BG = (11, 9, 16)
SIZE = 640


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def mix(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def gradient(w, h, stops, angle=90):
    """A linear gradient through `stops` (colours, evenly spaced), at `angle` degrees: worked
    out small and scaled up, which a smooth gradient doesn't mind."""
    if max(w, h) > 128:
        k = 128 / max(w, h)
        return gradient(max(2, round(w * k)), max(2, round(h * k)), stops, angle).resize((w, h), Image.BICUBIC)
    img = Image.new("RGB", (w, h))
    px = img.load()
    a = math.radians(angle)
    dx, dy = math.cos(a), math.sin(a)
    span = abs(w * dx) + abs(h * dy)
    for y in range(h):
        for x in range(w):
            t = ((x - w / 2) * dx + (y - h / 2) * dy) / span + 0.5
            t = min(max(t, 0), 1) * (len(stops) - 1)
            i = min(int(t), len(stops) - 2)
            px[x, y] = mix(stops[i], stops[i + 1], t - i)
    return img


def soft(img, shapes, blur):
    """Shapes drawn on their own layer, blurred, then laid over `img`."""
    layer = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    for draw in shapes:
        draw(d)
    layer = layer.filter(ImageFilter.GaussianBlur(blur))
    out = img.convert("RGBA")
    out.alpha_composite(layer)
    return out.convert("RGB")


def vignette(img, strength):
    w, h = img.size
    mask = Image.new("L", (w, h), 0)
    ImageDraw.Draw(mask).ellipse((-w * 0.25, -h * 0.25, w * 1.25, h * 1.25), fill=255)
    mask = mask.filter(ImageFilter.GaussianBlur(w / 6))
    dark = Image.new("RGB", (w, h), (0, 0, 0))
    return Image.composite(img, Image.blend(img, dark, strength), mask)


def face(name, size):
    for path in [f"/System/Library/Fonts/Supplemental/{name}", f"/System/Library/Fonts/{name}", f"/Library/Fonts/{name}"]:
        if os.path.exists(path):
            return ImageFont.truetype(path, size)
    return None


# The motifs: each draws a cover's picture over its gradient.
def sun(img, c, rnd):
    w, h = img.size
    img = soft(img, [lambda d: d.ellipse((w * .22, h * .28, w * .78, h * .84), fill=c["glow"] + (150,))], 60)
    d = ImageDraw.Draw(img)
    d.ellipse((w * .3, h * .36, w * .7, h * .76), fill=c["sun"])
    # The horizon cuts the sun into slats, wider apart lower down.
    y, gap = h * .58, 6
    while y < h:
        d.rectangle((0, y, w, y + gap * .55), fill=mix(c["sun"], c["sea"], .55))
        y += gap
        gap *= 1.22
    d.rectangle((0, h * .76, w, h), fill=c["sea"])
    for k in range(9):
        yy = h * .78 + k * k * 2.2
        d.line((w * (.5 - .03 * (9 - k)), yy, w * (.5 + .03 * (9 - k)), yy), fill=mix(c["sun"], c["sea"], .3 + k * .07), width=2)
    return img


def waves(img, c, rnd):
    w, h = img.size
    d = ImageDraw.Draw(img)
    for k in range(26):
        y0 = h * (.18 + k * .028)
        col = mix(c["line"], c["sea"], k / 26)
        pts = [(x, y0 + math.sin(x / w * math.pi * 3 + k * .45) * (14 + k * 1.4) + math.sin(x / 37 + k) * 3) for x in range(0, w + 8, 8)]
        d.line(pts, fill=col, width=2)
    return soft(img, [lambda d: d.ellipse((w * .55, h * .05, w * .95, h * .45), fill=c["glow"] + (120,))], 50)


def rings(img, c, rnd):
    w, h = img.size
    img = soft(img, [lambda d: d.ellipse((w * .15, h * .15, w * .85, h * .85), fill=c["glow"] + (90,))], 70)
    d = ImageDraw.Draw(img)
    cx, cy = w * .5, h * .54
    for k in range(18, 0, -1):
        r = k * w * .026
        d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=mix(c["line"], c["sun"], k / 18), width=2 if k % 3 else 4)
    d.ellipse((cx - 22, cy - 22, cx + 22, cy + 22), fill=c["sun"])
    return img


def blobs(img, c, rnd):
    w, h = img.size
    shapes = []
    for k in range(7):
        x, y, r = rnd.uniform(0, w), rnd.uniform(0, h), rnd.uniform(w * .15, w * .38)
        col = [c["sun"], c["glow"], c["line"]][k % 3]
        shapes.append(lambda d, x=x, y=y, r=r, col=col: d.ellipse((x - r, y - r, x + r, y + r), fill=col + (200,)))
    return soft(img, shapes, 70)


def arches(img, c, rnd):
    w, h = img.size
    d = ImageDraw.Draw(img)
    n = 5
    for k in range(n):
        x0 = w * (.08 + k * .17)
        col = mix(c["sun"], c["line"], k / (n - 1))
        d.rounded_rectangle((x0, h * (.32 + .06 * (k % 2)), x0 + w * .13, h * .86), radius=w * .065, fill=col)
    d.rectangle((0, h * .86, w, h), fill=c["sea"])
    return soft(img, [lambda d: d.ellipse((w * .3, h * .02, w * .7, h * .3), fill=c["glow"] + (140,))], 40)


def dots(img, c, rnd):
    w, h = img.size
    d = ImageDraw.Draw(img)
    step = w / 15
    for i in range(16):
        for j in range(16):
            x, y = i * step, j * step
            t = math.hypot(x - w * .62, y - h * .4) / w
            r = max(0.5, 11 - t * 18)
            d.ellipse((x - r, y - r, x + r, y + r), fill=mix(c["sun"], c["line"], min(t * 1.6, 1)))
    return soft(img, [lambda d: d.ellipse((w * .4, h * .2, w * .85, h * .65), fill=c["glow"] + (70,))], 60)


def cover(slug, title, artist, c, motif, seed):
    path, room_path = os.path.join(OUT, f"{slug}.png"), os.path.join(OUT, f"{slug}-room.png")
    if not ART:
        return path, room_path
    rnd = random.Random(seed)
    stops = [hexrgb(s) for s in c["sky"]]
    col = {k: hexrgb(v) for k, v in c.items() if k not in ("sky", "angle")}
    img = gradient(SIZE, SIZE, stops, c.get("angle", 90))
    img = motif(img, col, rnd)
    img = vignette(img, .35)
    noise = Image.effect_noise(img.size, 28).convert("RGB")
    img = Image.blend(img, noise, .06)
    # A small mark, as a sleeve has: the title in a serif, the artist in small caps.
    d = ImageDraw.Draw(img)
    serif, caps = face("Didot.ttc", 34), face("Futura.ttc", 15)
    ink = col.get("ink", (250, 244, 236))
    if serif and caps:
        d.text((40, SIZE - 92), title, font=serif, fill=ink)
        d.text((42, SIZE - 46), " ".join(artist.upper()), font=caps, fill=mix(ink, (0, 0, 0), .25))
    # Rounded corners, so the window shows the art the shape its shadow is.
    mask = Image.new("L", (SIZE * 4, SIZE * 4), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, SIZE * 4, SIZE * 4), radius=36 * 4, fill=255)
    out = img.convert("RGBA")
    out.putalpha(mask.resize((SIZE, SIZE), Image.LANCZOS))
    out.save(path)
    # The room behind it: the art's colours spread wide and blurred away, richer than the art,
    # lit from where the hero sits (top right of the sidebar) and sunk into the night at the
    # edges and under the sidebar, so text stays easy to read over it.
    w, h = 1600, 1066
    small = img.resize((6, 6), Image.LANCZOS)
    small = ImageEnhance.Color(small).enhance(1.7)
    # Every room as dark as the others, whatever its art: a pale sleeve mustn't wash out the text.
    mean = sum(ImageStat.Stat(small).mean) / 3
    small = ImageEnhance.Brightness(small).enhance(min(1.2, 62 / max(mean, 1)))
    light = Image.new("L", (160, 107), 0)
    ImageDraw.Draw(light).ellipse((10, -60, 150, 70), fill=255)
    small = small.resize((160, 107), Image.BICUBIC)
    room = Image.composite(small, Image.new("RGB", (160, 107), BG), light.filter(ImageFilter.GaussianBlur(26)))
    # Blurred at full size, then a little noise, so the dark steps don't band.
    room = room.resize((w, h), Image.BICUBIC).filter(ImageFilter.GaussianBlur(40))
    room = Image.blend(room, Image.effect_noise((w, h), 40).convert("RGB"), .035)
    room.save(room_path)
    return path, room_path


def clock(s):
    return f"{int(s) // 60}:{int(s) % 60:02}"


def tracks(names, rnd):
    out = []
    for k, t in enumerate(names):
        s = rnd.randint(150, 330)
        out.append({"id": str(k), "n": f"{k + 1:02}", "t": t, "len": s, "d": clock(s)})
    return out


ALBUMS = [
    ("harbour-lights", "Harbour Lights", "Mara Vey", "Blue Hours", 2025, "Gullwing Records",
     {"sky": ["#1b1446", "#5b2a6e", "#d9605a", "#f5a25d"], "angle": 90, "sun": "#ffd9a0", "sea": "#20163f", "glow": "#ff8a5c", "line": "#f3c18a"}, sun,
     ["Blue Hours", "Ropewalk", "The Last Ferry", "Lamplighter", "Salt in the Seams", "Harbour Lights", "Tidewater", "Morning, Eventually"],
     ["the cranes go quiet one by one", "and the water keeps the colour of the sun", "I count the boats that don't come in", "and call it patience, call it hymn"]),
    ("salt-and-static", "Salt & Static", "The Lumen", "Coastline", 2024, "Northbound",
     {"sky": ["#06262b", "#0d4f57", "#1d8a87"], "angle": 120, "sun": "#bff3e6", "sea": "#031618", "glow": "#5ce0c8", "line": "#9ff0dc"}, waves,
     ["Coastline", "Static Bloom", "Breakwater", "Radio Silence", "Seaglass", "Undertow", "Low Signal", "Shoreline Hymn"],
     ["a radio left on in an empty house", "plays the weather to the walls", "somewhere a lighthouse learns your name", "and keeps it, and it calls"]),
    ("paper-moons", "Paper Moons", "Ilse Okafor", "Kite Season", 2025, "Fold & Crease",
     {"sky": ["#f1e2c6", "#e7c28f", "#c9895b"], "angle": 90, "sun": "#7a2e1f", "sea": "#3d1a12", "glow": "#fff3dc", "line": "#a4553a", "ink": "#3a1a10"}, rings,
     ["Kite Season", "Paper Moons", "Ink on Linen", "The Long Afternoon", "Orchard", "Small Hours", "Letters Home", "Folding"],
     ["we cut the moon from yesterday's news", "and hung it where the real one used to be", "it never rose, it never set", "it only ever looked at me"]),
    ("night-ferry", "Night Ferry", "Oskar Brandt", "Crossing", 2023, "Kestrel Tapes",
     {"sky": ["#030b1c", "#0b2348", "#163a6b"], "angle": 80, "sun": "#7fd4ff", "sea": "#020713", "glow": "#3a8dff", "line": "#4fb3ff"}, dots,
     ["Crossing", "Deck Lights", "Night Ferry", "Engine Room", "Gulls at 3am", "Bearing North", "The Far Pier", "Arrival"],
     ["the deck lights hum a single note", "the sea holds everything we wrote", "we're halfway to a different shore", "we'll be someone else by four"]),
    ("velvet-arcade", "Velvet Arcade", "Juno Park", "Coin Op", 2024, "Pinball Social",
     {"sky": ["#1a0626", "#4a0f4f", "#8a1c6b"], "angle": 135, "sun": "#ff5fa8", "sea": "#12041c", "glow": "#b26bff", "line": "#ff9bd2"}, blobs,
     ["Coin Op", "High Score", "Velvet Arcade", "Tilt", "Neon Sundae", "Two Player", "Continue?", "Game Over (Reprise)"],
     ["you were the last light on the strip", "a quarter spinning on its rim", "we played until the machines got tired", "and the morning wandered in"]),
    ("hollow-pines", "Field Recordings II", "Hollow Pines", "Fern Light", 2022, "Understorey",
     {"sky": ["#0b1a10", "#1f3a22", "#4f6b2c"], "angle": 100, "sun": "#e8d48a", "sea": "#08130b", "glow": "#c7d36b", "line": "#9fbf6a"}, arches,
     ["Fern Light", "Moss Clock", "Under the Canopy", "Rainshadow", "Woodsmoke", "The Clearing", "Lichen", "Dusk Chorus"],
     ["the forest keeps the time it likes", "and never tells the clocks", "I left a song under a stone", "the rain has learned the chords"]),
    ("glasshouse", "Glasshouse", "Sora Lind", "Condensation", 2025, "Greenroom",
     {"sky": ["#d8efe4", "#b8e0cf", "#f2c6b0"], "angle": 60, "sun": "#ff9e7a", "sea": "#2d4a43", "glow": "#ffffff", "line": "#5f9a8a"}, sun,
     ["Condensation", "Glasshouse", "Fig", "Terracotta", "Warm Panes", "Overgrown", "Sunroom", "Water Me"],
     ["the windows learn to cry in June", "the figs grow fat on afternoon", "I talk to plants the way I'd talk to you", "and they grow, and they grow"]),
    ("low-tide", "Low Tide", "Wren & Ash", "Mudflats", 2023, "Estuary",
     {"sky": ["#14161d", "#2e3443", "#6b5f6e", "#c48a8a"], "angle": 90, "sun": "#f7c1b5", "sea": "#0e1016", "glow": "#e08a8a", "line": "#d9a3a3"}, waves,
     ["Mudflats", "Wader", "Low Tide", "Samphire", "The Channel", "Heron", "Slack Water", "Turning"],
     ["the tide went out and took the noise", "and left us standing in the shine", "two shapes upon the silver ground", "one of them yours, one of them mine"]),
    ("copper-weather", "Copper Weather", "Ada Mireille", "Verdigris", 2024, "Foundry Hall",
     {"sky": ["#2a120a", "#6b2f17", "#b8622c"], "angle": 110, "sun": "#ffd2a6", "sea": "#1c0b06", "glow": "#ff9b54", "line": "#f0a46a"}, rings,
     ["Verdigris", "Copper Weather", "Kiln", "The Long Pour", "Patina", "Bellfounder", "Rust Hymn", "Cooling"],
     ["the weather turns the roof to green", "the colour of a coin you've kept", "we hammer out a little light", "and hang it where the others slept"]),
    ("small-planets", "Small Planets", "Teo Varga", "Orbiter", 2025, "Satellite Hum",
     {"sky": ["#0c0820", "#241a52", "#4b3a8f"], "angle": 70, "sun": "#e3d7ff", "sea": "#07051a", "glow": "#9b7bff", "line": "#c2b0ff"}, dots,
     ["Orbiter", "Small Planets", "Perihelion", "Ground Control", "Moonlet", "Escape Velocity", "Far Side", "Re-entry"],
     ["I built a planet in my room", "it fits inside a coffee cup", "it has a weather and a moon", "and it is waiting to wake up"]),
    ("winter-orchard", "Winter Orchard", "Nell Harrow", "First Frost", 2023, "Bare Branch",
     {"sky": ["#dfe8f1", "#a9bfd6", "#5d7896"], "angle": 90, "sun": "#ffffff", "sea": "#24344a", "glow": "#ffffff", "line": "#7e98b8"}, arches,
     ["First Frost", "Winter Orchard", "Pruning", "Cold Frame", "Snow on the Hives", "Thaw", "Grafting", "Blossom, Later"],
     ["the apple trees have kept their names", "beneath the snow, beneath the frost", "I walk the rows and say them back", "so nothing over winter's lost"]),
    ("sodium", "Sodium", "Kiri Osei", "Streetlamp", 2024, "Late Bus",
     {"sky": ["#1a0d05", "#5a2a08", "#d26a10"], "angle": 120, "sun": "#ffcf6b", "sea": "#140903", "glow": "#ff8f1f", "line": "#ffb347"}, blobs,
     ["Streetlamp", "Sodium", "Night Bus", "Ring Road", "Amber Alert", "Underpass", "Last Orders", "Home by Two"],
     ["the city's lit the colour of a match", "held up against the dark too long", "we wait beneath the orange hum", "for the bus, for the song"]),
]

NOTES = {
    "harbour-lights": "Recorded over four nights in a converted net loft, with the doors open to the harbour. You can hear the gulls on side two.",
    "salt-and-static": "A coastal record made on two tape machines and a shortwave radio that wouldn't stay tuned. The static stayed in.",
    "paper-moons": "Songs written on the backs of old maps, then sung in a kitchen with the kettle on. Nothing was played twice.",
    "night-ferry": "Written on the overnight crossing, eight times there and back. Every song is as long as a stretch of water.",
    "velvet-arcade": "Synths borrowed from a shuttered arcade, played in the order the machines were wired. Loud, then very soft.",
    "hollow-pines": "Microphones left in the woods for a season, then a few quiet instruments laid over what they caught.",
    "glasshouse": "Tracked in a Victorian glasshouse in late spring. The panes were warm enough to fog the strings.",
    "low-tide": "Two voices and a cello on the mudflats at slack water, between one tide and the next.",
    "copper-weather": "Made in a disused bell foundry, where every note rings a little longer than it should.",
    "small-planets": "A bedroom record with a planetarium's patience, built from toy synths and a borrowed telescope motor.",
    "winter-orchard": "Recorded in an orchard shed through one cold month, with a stove, a harmonium and the wind in the eaves.",
    "sodium": "Late songs for the last bus, written under streetlights and sung quietly so as not to wake the flat upstairs.",
}

albums = []
for k, (slug, title, artist, single, year, label, palette, motif, names, lines) in enumerate(ALBUMS):
    art, room = cover(slug, title, artist, palette, motif, k)
    rnd = random.Random(k)
    ts = tracks(names, rnd)
    albums.append({
        "id": slug, "title": title, "artist": artist, "single": single, "year": year, "label": label,
        "art": art, "room": room, "tracks": ts, "count": len(ts),
        "l1": lines[0], "l2": lines[1], "l3": lines[2], "l4": lines[3], "note": NOTES[slug],
    })


# The theme: a night room, warm white on plum-black, coral and amber for what's live, and glass
# (colours with an alpha) for the cards that sit over the room.
tokens = {
    "bg": "#0b0910", "fg": "#f4ede5", "dim": "#a99fb2", "faint": "#6e6579", "border": "#2b2533",
    "panel": "#15121c", "deep": "#0d0b13", "ring": "#2a2233", "hover": "#1d1925", "plate": "#221c2b",
    "accent": "#ff8a5c", "amber": "#ffc46b", "rose": "#ff6b8b", "violet": "#a283ff", "pos": "#7fe0a8",
    "neg": "#ff6b8b", "warn": "#ffc46b", "track": "#ffffff1f", "scrim": "#07060a",
    "glass": "#1a1524b8", "glass2": "#0b0910d9", "rim": "#ffffff14", "veil": "#ffffff0d", "lift": "#ffffff1a",
}
theme = {
    "tokens": tokens,
    "fonts": {"display": "Bodoni 72", "ui": "Avenir Next", "lyric": "Big Caslon", "mono": "Menlo"},
    "type": {
        "brand": {"font": "display", "size": 30, "weight": 400, "line": 40},
        "display": {"font": "display", "size": 58, "weight": 400, "line": 66},
        "headline": {"font": "display", "size": 36, "weight": 400, "line": 44},
        "subtitle": {"font": "ui", "size": 17, "weight": 500, "line": 26},
        "heading": {"font": "ui", "size": 15, "weight": 600, "line": 24},
        "label": {"font": "ui", "size": 11, "weight": 700, "line": 24, "tracking": 1.6},
        "body": {"font": "ui", "size": 14, "weight": 400, "line": 24},
        "caption": {"font": "ui", "size": 12, "weight": 500, "line": 20},
        "num": {"font": "ui", "size": 13, "weight": 500, "line": 24, "tabular": True},
        "lyric": {"font": "lyric", "size": 19, "weight": 400, "line": 30},
        "now": {"font": "lyric", "size": 23, "weight": 400, "line": 34},
        "mono": {"font": "mono", "size": 13, "line": 24},
    },
    "space": {"s1": "4px", "s2": "8px", "s3": "12px", "s4": "16px", "s5": "20px", "s6": "24px", "s8": "32px"},
    "radius": {"r1": 8, "r2": 12, "r3": 18, "pill": 99},
    "depth": {
        "e1": [{"y": 1, "blur": 2, "color": "black", "alpha": 0.45}],
        "e2": [{"y": 18, "blur": 48, "color": "black", "alpha": 0.45}, {"y": 2, "blur": 6, "color": "black", "alpha": 0.3}],
        "art": [{"y": 28, "blur": 70, "color": "black", "alpha": 0.7}, {"y": 0, "blur": 90, "color": "accent", "alpha": 0.22}],
        "card": [{"y": 12, "blur": 26, "color": "black", "alpha": 0.6}],
        "glow": [{"y": 8, "blur": 26, "color": "accent", "alpha": 0.45}, {"y": 1, "blur": 3, "color": "black", "alpha": 0.4}],
    },
    "parts": {
        "screen": {"type": "body"},
        "panel": {"bg": {"from": "glass", "to": "glass2", "angle": 180}, "radius": "r3", "pad": ["s5", "s4"], "border": {"color": "rim"}, "shadow": "e2"},
        "title": {"fg": "faint", "type": "label"}, "title.focused": {"fg": "accent", "type": "label"},
        "border.focused": "rim",
        "button": {"fg": "fg", "bg": "veil", "radius": "pill", "pad": ["s4", 0], "type": "heading", "cursor": "pointer", "border": {"color": "rim"}},
        "button.hover": {"bg": "lift"},
        "button.focused": {"ring": {"width": 2, "offset": 2, "color": "accent"}},
        "input": {"bg": "veil", "radius": "pill", "pad": ["s4", 0], "border": {"color": "rim"}},
        "input.focused": {"border": {"color": "accent"}, "ring": {"width": 3, "offset": 0, "color": "ring"}},
        "input.placeholder": "faint", "editor.caret": "on-accent", "editor.selection": "fg+on-ring",
        "row": {"radius": "r2", "pad": ["s3", 0], "cursor": "pointer"},
        "row.hover": {"bg": "veil"},
        "row.selected": {"fg": "fg", "bg": "lift"}, "row.selected.blur": {"fg": "fg", "bg": "veil"},
        "row.active": {"fg": "fg", "bg": "lift"},
        "gauge": {"bg": "track", "radius": "pill"},
        "gauge.fill": {"bg": {"from": "amber", "to": "accent", "angle": 90}, "radius": "pill"},
        "scrollbar": {"shape": "overlay"}, "scrollbar.thumb": "faint",
        "toast": {"fg": "fg", "bg": "plate", "radius": "r2", "border": {"color": "border"}, "shadow": "e2"},
        "empty": "faint+i", "spinner": "accent",
    },
    "variants": {
        "card": {"style": {"bg": {"from": "glass", "to": "glass2", "angle": 170}, "radius": "r3", "pad": ["s5", "s4"], "border": {"color": "rim"}, "shadow": "e2"}, "title": "faint"},
        "mini": {"style": {"bg": {"from": "glass", "to": "glass2", "angle": 170}, "radius": "r2", "pad": "s3", "border": {"color": "rim"}}},
        "art": {"style": {"radius": "r3", "shadow": "art"}},
        # A cover on the shelf: no glow, which its card would clip (a node clips what it holds).
        "cover": {"style": {"radius": "r3"}},
        # The shelf's rows are cards: room all round, and corners that follow the cover's (its
        # radius and the padding between).
        "shelf": {"parts": {"row": {"radius": 26, "pad": "s2"}}},
        "thumb": {"style": {"radius": "r1", "shadow": "card"}},
        "play": {"style": {"fg": "#1a0f0a", "bg": {"from": "amber", "to": "accent", "angle": 135}, "border": {"color": "accent"}, "shadow": "glow", "pad": ["s6", 0]}},
        # A quiet button that lights up while it's on (loved, shuffling).
        "ghost": {"style": {"fg": "dim", "bg": "#00000000", "border": {"color": "#00000000"}}, "parts": {"button.on": {"fg": "accent"}}},
    },
}
with open(os.path.join(OUT, "themes.json"), "w") as f:
    json.dump({"nocturne": theme}, f, indent=1)


def T(text, **k):
    return {"type": "text", "text": text, **k}


def label(text, **k):
    return T(f"<label>{text}</>", tone="faint", size="fit", **k)


def button(id, face, click, **k):
    return {"type": "button", "id": id, "label": face, "size": "fit", "click": click, **k}


songs = sum(a["count"] for a in albums)
secs = sum(t["len"] for a in albums for t in a["tracks"])
WIDE, SLIM = "$screen.width>=230", "$screen.width<230"
NOW = "@nav.id=now"
PLAY = {"toggle": "store.playing"}
# The playing track is the one Up Next has selected: ↑/↓ on it, a click, or these.
NEXT = {"select": "next", "in": "tracks"}
PREV = {"select": "prev", "in": "tracks"}

# What's playing, small, at the foot of the sidebar.
mini = {"type": "row", "id": "mini", "variant": "mini", "size": "fit", "gap": 2, "items": "center", "children": [
    {"type": "image", "id": "thumb", "src": "{art}", "bind": "@albums", "aspect": 1, "size": "52px", "variant": "thumb", "fit": "cover"},
    {"type": "col", "gap": 0, "children": [
        T("<heading>{t}</>", bind="@tracks", size="fit", wrap=False),
        T("<caption>{artist}</>", bind="@albums", tone="dim", size="fit", wrap=False),
        T("<caption>{$store.playing|true=<accent>▶︎  playing</>;*=<faint>paused</>}</>", size="fit"),
    ]},
]}

# Where you are: each place names its page's heading and its shelf's.
places = [{"id": "now", "t": "◉   Listen Now", "name": "Listen Now", "shelf": "Recently played"},
          {"id": "browse", "t": "▦   Browse", "name": "Browse", "shelf": "Every album"}]
sidebar = {"type": "col", "id": "side", "size": 25, "justify": "between", "when": WIDE, "children": [
    {"type": "col", "size": "fit", "gap": 0, "children": [
        T("<brand>Nocturne</>", size="fit", margin=[0, 0, "s6", 0]),
        label("LISTEN", margin=[0, 0, "s1", 0]),
        {"type": "list", "id": "nav", "size": "fit", "items": places, "item": "{t}"},
        label("LIBRARY", margin=["s6", 0, "s1", 0]),
        T(f"<dim>{len(albums)} albums\n{songs} songs\n{secs // 3600} hr {secs % 3600 // 60} min</>", size="fit", pad=[1, 0])]},
    mini,
]}

# The controls share one measure under the title, at most 560 pixels: the bar, the times, the buttons.
controls = {"type": "row", "size": "fit", "children": [{"type": "col", "max": "560px", "gap": 0, "children": [
    {"type": "gauge", "id": "progress", "bind": "player", "value": "{at}", "of": "{len}", "size": "5px"},
    T("<num>{el}</>\t<num>−{left}</>", bind="player", tone="faint", size="fit", margin=["s1", 0, "s4", 0]),
    {"type": "row", "size": 2, "justify": "between", "items": "center", "children": [
        {"type": "row", "size": "fit", "gap": 2, "items": "center", "children": [
            button("prev", "⏮︎", PREV),
            button("play", "{$store.playing|true=❚❚   Pause;*=▶︎   Play}", PLAY, variant="play", on="playing"),
            button("next", "⏭︎", NEXT)]},
        {"type": "row", "size": "fit", "gap": 2, "items": "center", "children": [
            button("love", "♥", {"toggle": "store.loved/{id}"}, bind="@albums", variant="ghost", on="loved/{id}"),
            button("shuffle", "⇄", {"toggle": "store.shuffle"}, variant="ghost", on="shuffle")]},
    ]},
]}]}

hero = {"type": "row", "id": "hero", "size": 14, "gap": 5, "items": "center", "when": NOW, "children": [
    {"type": "image", "id": "art", "src": "{art}", "bind": "@albums", "aspect": 1, "self": "stretch", "variant": "art", "fit": "cover"},
    {"type": "col", "id": "playing", "justify": "center", "gap": 0, "children": [
        T("<label>{$store.playing|true=NOW PLAYING;*=PAUSED}</>     <caption>{label} · {year}</>", bind="@albums", tone="faint", size="fit", wrap=False, id="status"),
        T("<display>{t}</>", bind="@tracks", size="fit", wrap=False),
        T("<subtitle>{artist}</>    <dim>from</>   <subtitle><i>{title}</></>", bind="@albums", size="fit", wrap=False, margin=[0, 0, "s6", 0]),
        controls,
    ]},
]}

card = {"type": "col", "gap": 0, "children": [
    {"type": "image", "src": "{$item.art}", "aspect": 1, "variant": "cover", "fit": "cover", "margin": [0, 0, "s2", 0]},
    T("<heading>{$item.title}</>", size="fit", wrap=False),
    T("<caption>{$item.artist}</>", tone="dim", size="fit", wrap=False),
]}


def shelf():
    """The shelf: as many covers to a line as fit at 200 pixels or more each, each card as tall as
    its cover (square, as wide as the card) and its two lines; in the scrolling main column the
    shelf is as tall as all its lines, and the room scrolls."""
    return {"type": "list", "id": "albums", "variant": "shelf", "bind": "albums", "filter": "find", "row": {**card, "size": "fit"},
            "cols": {"min": "200px"}}


main = {"type": "col", "id": "main", "gap": 1, "scroll": True, "children": [
    {"type": "row", "size": 2, "items": "center", "justify": "between", "children": [
        {"type": "row", "size": "fit", "gap": 3, "items": "center", "children": [
            T("<brand>Nocturne</>", size="fit", wrap=False, when=SLIM),
            T("<headline>{name}</>", bind="@nav", size="fit", wrap=False)]},
        {"type": "input", "id": "find", "size": 28, "placeholder": "Search"}]},
    hero,
    {"type": "row", "size": 1, "items": "center", "justify": "between", "margin": ["s4", 0, 0, 0], "children": [
        T("<heading>{shelf}</>", bind="@nav", size="fit", wrap=False),
        T("<caption>arrows choose  ·  space plays  ·  n and p skip</>", tone="faint", size="fit", wrap=False)]},
    shelf(),
]}

rail = {"type": "col", "id": "rail", "size": "24%", "min": 36, "max": 44, "gap": 2, "when": "$screen.width>=150", "children": [
    {"type": "list", "id": "tracks", "title": "UP NEXT", "variant": "card", "bind": "@albums.tracks", "size": "fit",
     "item": "<num>{n}</>     {t}\t<num>{d}</>", "click": {"put": "playing", "value": True}, "keys": {"enter": {"put": "playing", "value": True}}},
    {"type": "text", "id": "lyrics", "title": "LYRICS", "variant": "card", "size": "fit", "bind": "@albums",
     "text": "<lyric>{l1}</>\n<now>{l2}</>\n<lyric>{l3}</>\n<lyric>{l4}</>", "tone": "dim"},
    {"type": "text", "id": "about", "title": "ABOUT THE RECORD", "variant": "card", "size": "fit", "bind": "@albums",
     "text": "{note}\n\n<caption>{count} songs · {label} · {year}</>", "tone": "dim"},
]}

first = albums[0]["tracks"][0]
ui = {
    "root": {"type": "stack", "children": [
        {"type": "image", "id": "room", "src": "{room}", "bind": "@albums", "fit": "cover"},
        {"type": "row", "gap": 4, "pad": [2, "s3"], "children": [sidebar, main, rail]},
    ]},
    "keys": {"space": PLAY, "n": NEXT, "p": PREV},
    "store": {"albums": albums, "playing": False, "shuffle": False,
              "player": {"at": 98, "len": first["len"], "el": clock(98), "left": clock(first["len"] - 98)}},
    "focus": "albums",
    "status_bar": False,
}
# What the player knows, as an audio engine would: each album's track lengths, by its room.
with open(os.path.join(OUT, "library.json"), "w") as f:
    json.dump({a["room"]: [t["len"] for t in a["tracks"]] for a in albums}, f)
with open(os.path.join(OUT, "nocturne.json"), "w") as f:
    json.dump(ui, f, indent=1, ensure_ascii=False)
print(OUT)
