#!/usr/bin/env python3
"""A made-up market, as a stream source: eight invented tickers on a random walk, printed as one
JSON line four times a second. The screen draws its table, charts and gauge from these lines;
the data goes from this process to the screen and never through a model.
"""
import json
import math
import random
import sys
import time

random.seed(2611)
NAMES = [
    ("GLAS", "Glassworks"), ("PANE", "Pane & Frame"), ("LUMN", "Lumen Labs"), ("TINT", "Tint Systems"),
    ("PRSM", "Prism Optics"), ("FROS", "Frost Data"), ("OPAL", "Opal Grid"), ("SHRD", "Shard Robotics"),
]
HIST = 90


def main():
    book = []
    for i, (sym, name) in enumerate(NAMES):
        p = random.uniform(18, 240)
        hist = []
        for _ in range(HIST):
            p *= math.exp(random.gauss(0.0002, 0.006))
            hist.append(p)
        book.append({"sym": sym, "name": name, "open": hist[0], "hist": hist, "vol": random.uniform(2, 9), "drift": random.gauss(0, 0.0006)})
    index = [sum(b["hist"][k] for b in book) / len(book) for k in range(HIST)]
    tick = 0
    while True:
        tick += 1
        for b in book:
            p = b["hist"][-1] * math.exp(random.gauss(b["drift"], 0.0045))
            b["hist"] = (b["hist"] + [p])[-HIST:]
            b["vol"] = max(0.5, b["vol"] * math.exp(random.gauss(0, 0.05)))
        index = (index + [sum(b["hist"][-1] for b in book) / len(book)])[-HIST:]
        rows = []
        for b in book:
            p = b["hist"][-1]
            pct = (p / b["open"] - 1) * 100
            rows.append({
                "sym": b["sym"], "name": b["name"], "price": f"{p:,.2f}", "pct": round(pct, 2),
                "chg": f"{p - b['open']:+.2f}", "vol": round(b["vol"], 2),
                "hi": f"{max(b['hist']):,.2f}", "lo": f"{min(b['hist']):,.2f}",
                "history": [round(x, 2) for x in b["hist"]],
            })
        up = sum(1 for r in rows if r["pct"] >= 0)
        idx_pct = (index[-1] / index[0] - 1) * 100
        print(json.dumps({
            "tickers": rows,
            "index": {"value": f"{index[-1]:,.2f}", "pct": round(idx_pct, 2), "series": [round(x, 2) for x in index]},
            "breadth": {"up": up, "down": len(rows) - up, "ratio": up / len(rows)},
            "clock": time.strftime("%H:%M:%S"),
            "tick": tick,
        }), flush=True)
        time.sleep(0.25)


if __name__ == "__main__":
    try:
        main()
    except (BrokenPipeError, KeyboardInterrupt):
        sys.exit(0)
