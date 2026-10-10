#!/usr/bin/env python3
"""The ledger's bookkeeper: a stream source. It reads the `ledger` entry from the screen's store
(the screen gives its commands $FICTTY and $FICTTY_SOCKET) and prints a summary as one JSON
line whenever the ledger changes: the balance, money in and out, spending by category and the
running balance by date. The screen binds its summary panels to it; the model never sees it.
"""
import json
import os
import subprocess
import sys
import time

FICTTY = os.environ.get("FICTTY", "fictty")


def amount(row):
    try:
        return float(str(row.get("amount", "0")).replace(",", ""))
    except ValueError:
        return 0.0


def money(x):
    return f"{x:,.2f}".replace("-", "−") if x % 1 else f"{x:,.0f}"


def summary(rows):
    rows = sorted(rows, key=lambda r: str(r.get("date", "")))
    total = sum(amount(r) for r in rows)
    income = sum(amount(r) for r in rows if amount(r) > 0)
    spent = -sum(amount(r) for r in rows if amount(r) < 0)
    by_cat = {}
    for r in rows:
        if amount(r) < 0:
            c = r.get("category") or "Other"
            by_cat[c] = by_cat.get(c, 0.0) - amount(r)
    cats = sorted(by_cat.items(), key=lambda kv: -kv[1])
    running, bal = [], 0.0
    for r in rows:
        bal += amount(r)
        running.append(round(bal, 2))
    largest = max(rows, key=lambda r: -amount(r), default=None)
    return {
        "balance": f"{total:,.2f}",
        "income": f"{income:,.2f}",
        "spent": f"{spent:,.2f}",
        "saved": f"{(income - spent) / income * 100:.0f}%" if income else "–",
        "entries": len(rows),
        "categories": [{"name": c, "amount": round(v, 2), "label": f"{v:,.0f}"} for c, v in cats],
        "running": running,
        "largest": f"{largest.get('payee', '')} {-amount(largest):,.2f}" if largest and amount(largest) < 0 else "–",
    }


def read_ledger():
    try:
        out = subprocess.run([FICTTY, "read", "ledger"], capture_output=True, text=True, timeout=5)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0:
        return None
    entry = json.loads(out.stdout)
    return entry.get("rev"), entry.get("value") or []


def main():
    last, said = None, False
    while True:
        got = read_ledger()
        if got and got[0] != last:
            last, said = got[0], True
            print(json.dumps(summary(got[1])), flush=True)
        elif not got and not said:
            # No screen to read from (yet): an empty summary, so the panels draw.
            said = True
            print(json.dumps(summary([])), flush=True)
        time.sleep(0.4)


if __name__ == "__main__":
    try:
        main()
    except (BrokenPipeError, KeyboardInterrupt):
        sys.exit(0)
