#!/usr/bin/env python3
"""Run a demo one-liner the way a reader does, `curl … | sh` in a terminal, and check it works.

    scripts/try-test.py https://fictty.com/try.sh
    scripts/try-test.py web/public/try/slug.sh           # a local file, before it's deployed
    scripts/try-test.py https://fictty.com/try.sh --expect Showcase --key 'enter' --expect 'New note'

It pipes the script into `sh` inside a fresh 160×46 pseudo-terminal with an empty FICTTY_HOME,
answers the terminal's cursor queries the way a real terminal would, waits for the first
--expect text to be drawn, then sends each --key and waits for the next --expect. Exits 0 when
every expected text appeared, 1 (with what was drawn) when one didn't. A build from source takes
a minute or so the first time; --timeout is per step.
"""
import argparse
import os
import pty
import re
import select
import signal
import struct
import sys
import tempfile
import time
import fcntl
import termios

KEYS = {"enter": b"\r", "esc": b"\x1b", "tab": b"\t", "down": b"\x1b[B", "up": b"\x1b[A", "q": b"q"}


def visible(raw: bytes) -> str:
    text = raw.decode("utf-8", "replace")
    return re.sub(r"\x1b\[[0-9;?]*[A-Za-z]|\x1b[\]_P][^\x07\x1b]*(\x07|\x1b\\)?", "", text)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("script", help="a URL or a local file")
    ap.add_argument("--expect", action="append", default=None, help="text that must appear (in order)")
    ap.add_argument("--key", action="append", default=[], help="keys to send after the matching --expect, comma-separated (enter,down,esc,tab,up or text)")
    ap.add_argument("--second", action="append", default=[], help="a command for a second terminal, run with only ~/.fictty/bin added to PATH once the screen is up; it must succeed")
    ap.add_argument("--timeout", type=float, default=240)
    a = ap.parse_args()
    expects = a.expect or ["Showcase"]
    fetch = f"curl -fsSL {a.script}" if re.match(r"https?://", a.script) else f"cat {a.script}"
    home = tempfile.mkdtemp(prefix="fictty-try-")

    pid, fd = pty.fork()
    if pid == 0:
        os.environ.update(TERM="xterm-256color", FICTTY_HOME=home)
        os.execvp("sh", ["sh", "-c", f"{fetch} | sh"])
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 46, 160, 0, 0))
    seen, ok = b"", True
    try:
        for i, want in enumerate(expects):
            end = time.time() + a.timeout
            while want not in visible(seen) and time.time() < end:
                r, _, _ = select.select([fd], [], [], 0.2)
                if r:
                    try:
                        chunk = os.read(fd, 65536)
                    except OSError:
                        break
                    if not chunk:
                        break
                    seen += chunk
                    if b"\x1b[6n" in chunk:
                        os.write(fd, b"\x1b[1;1R")
            if want not in visible(seen):
                ok = False
                print(f"✗ never drew {want!r}. Last output:\n{visible(seen)[-1500:]}")
                break
            print(f"✓ drew {want!r}")
            seen = b""
            if i < len(a.key):
                time.sleep(0.5)
                for k in a.key[i].split(","):
                    os.write(fd, KEYS.get(k, k.encode()))
                    time.sleep(0.3)
        # keep reading the screen's output, as a terminal would, while the second terminal works
        import subprocess, threading
        stop = threading.Event()
        def drain():
            while not stop.is_set():
                if select.select([fd], [], [], 0.2)[0]:
                    try:
                        chunk = os.read(fd, 65536)
                    except OSError:
                        return
                    if b"\x1b[6n" in chunk:
                        os.write(fd, b"\x1b[1;1R")
        threading.Thread(target=drain, daemon=True).start()
        for cmd in a.second if ok else []:
            env = {k: v for k, v in os.environ.items() if not k.startswith("FICTTY")}
            env["PATH"] = f"{home}/bin:" + env.get("PATH", "")
            r = subprocess.run(["sh", "-c", cmd], env=env, capture_output=True, text=True, timeout=60)
            ok = ok and r.returncode == 0
            print(f"{'✓' if r.returncode == 0 else '✗'} second terminal: {cmd} -> {(r.stdout or r.stderr).strip()[:160]!r}")
        stop.set()
    finally:
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        os.system(f"pkill -f '{home}/' >/dev/null 2>&1")
    print(f"(home: {home}; delete it when done)")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
