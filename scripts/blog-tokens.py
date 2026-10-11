#!/usr/bin/env python3
"""Tokens a stretch of work used, from Claude Code session transcripts, as the `tokens:` front
matter a blog post shows in its "Tokens" panel.

    scripts/blog-tokens.py SESSION.jsonl \
        --phase "I'm not sure about the project=Explore the options" \
        --phase "Ok, let's make a simple ui library=Proof of concept" \
        --end "Is it possible to analyze"

Each --phase is "START=NAME": the phase starts at the first message from the person (typed, or
queued mid-turn) that begins with START. --end stops the count at the first message that begins
with its text (leave it out to count to the end). Subagents and workflows that ran inside the
window, in the session's own folder, count too. Every model response is counted once.

Prints YAML to paste into the post's front matter. Nothing here reads anything but the files given.
"""
import argparse
import datetime as dt
import json
import pathlib
import sys


def said(e):
    """What the person typed in this transcript line, if anything."""
    m = e.get("message") or {}
    if e.get("type") == "user" and isinstance(m.get("content"), str):
        return m["content"]
    if e.get("type") == "queue-operation" and e.get("operation") == "enqueue":
        return e.get("content", "")
    return ""


def lines(path):
    for line in open(path, encoding="utf-8"):
        try:
            yield json.loads(line)
        except json.JSONDecodeError:
            continue


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("session", type=pathlib.Path)
    ap.add_argument("--phase", action="append", required=True, help='"START=NAME"')
    ap.add_argument("--end", help="text the first message after the window starts with")
    a = ap.parse_args()

    wanted = [p.split("=", 1) for p in a.phase]
    starts, end = {}, None
    for e in lines(a.session):
        t = said(e).strip()
        for start, name in wanted:
            if t.startswith(start) and name not in starts:
                starts[name] = e["timestamp"]
        if a.end and t.startswith(a.end) and starts and end is None:
            end = e["timestamp"]
    missing = [n for _, n in wanted if n not in starts]
    if missing:
        sys.exit(f"no message starts any of: {', '.join(missing)}")
    bounds = sorted((ts, n) for n, ts in starts.items())
    end = end or "9999"

    def phase(ts):
        if ts < bounds[0][0] or ts >= end:
            return None
        return [n for s, n in bounds if ts >= s][-1]

    files = [a.session] + sorted(a.session.with_suffix("").glob("**/*.jsonl"))
    seen, models = {}, set()
    for f in files:
        for e in lines(f):
            m = e.get("message") or {}
            if e.get("type") != "assistant" or not m.get("usage") or not m.get("id"):
                continue
            p = phase(e.get("timestamp", ""))
            if p:
                seen[m["id"]] = (p, m["usage"])
                models.add(m.get("model"))

    tot = {n: dict(calls=0, input=0, cached=0, output=0) for _, n in bounds}
    for p, u in seen.values():
        d = tot[p]
        d["calls"] += 1
        d["input"] += u.get("input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
        d["cached"] += u.get("cache_read_input_tokens", 0)
        d["output"] += u.get("output_tokens", 0)

    day = lambda ts: dt.datetime.fromisoformat(ts.replace("Z", "+00:00")).strftime("%Y-%m-%dT%H:%MZ")
    last = end if end != "9999" else None
    print("tokens:")
    print(f"  models: [{', '.join(sorted(m for m in models if m and not m.startswith('<')))}]")
    print(f"  from: {day(bounds[0][0])}")
    if last:
        print(f"  to: {day(last)}")
    print("  phases:")
    for _, n in bounds:
        d = tot[n]
        print(f'    - {{ name: "{n}", calls: {d["calls"]}, input: {d["input"]}, cached: {d["cached"]}, output: {d["output"]} }}')


if __name__ == "__main__":
    main()
