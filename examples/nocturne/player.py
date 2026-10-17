#!/usr/bin/env python3
# Nocturne's player. It plays no sound: it keeps time the way a player would, from outside the
# screen, over its socket, reading the screen back as an agent does. While `playing` is true it
# moves `player` on (where the track is, how long it is, the times shown); at a track's end it
# selects the next one in Up Next (any other with shuffle on), and after the last it stops.
# Choosing another track or album starts it from the top. It knows the albums' track lengths
# from library.json, as an audio engine knows its files, so it keeps time when the window is too
# narrow to show Up Next (the track is then the row Up Next last had).
#   python3 examples/nocturne/player.py [socket] [library.json]   # open.sh starts it
import json, os, random, socket, sys, time

SOCK = sys.argv[1] if len(sys.argv) > 1 else os.environ.get("FICTTY_SOCKET", "/tmp/fnoct.sock")
LIBRARY = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.environ.get("TMPDIR", "/tmp"), "fictty-nocturne", "library.json")
TICK = 0.25


def ask(req):
    with socket.socket(socket.AF_UNIX) as s:
        s.connect(SOCK)
        s.sendall((json.dumps(req) + "\n").encode())
        buf = b""
        while not buf.endswith(b"\n"):
            chunk = s.recv(1 << 16)
            if not chunk:
                break
            buf += chunk
    return json.loads(buf)


def clock(s):
    return f"{int(s) // 60}:{int(s) % 60:02}"


def main():
    with open(LIBRARY) as f:
        library = json.load(f)
    at = ask({"op": "read", "key": "player"}).get("value", {}).get("at", 0)
    was = shown = None
    k = 0
    while True:
        nodes = {n["id"]: n for n in ask({"op": "readback"})["nodes"] if n.get("id")}
        tracks, room = nodes.get("tracks"), nodes.get("room")
        if not room or room["text"] not in library:
            time.sleep(TICK)
            continue
        # The track is the album's room (its picture) and the row Up Next has selected, or,
        # with Up Next hidden, the row it last had (its selection stays while it's hidden).
        lens = library[room["text"]]
        if tracks:
            k = tracks.get("selected", 0)
        k = min(k, len(lens) - 1)
        now = (room["text"], k)
        length = lens[k]
        if was is not None and now != was:
            at = 0
        was = now
        playing = "pause" in nodes
        if playing:
            at += TICK
        if at >= length:
            at, n = 0, len(lens)
            if "shuffled" in nodes and n > 1:
                k = random.choice([i for i in range(n) if i != k])
            elif k + 1 < n:
                k += 1
            else:
                k = 0
                ask({"op": "write", "write": "put", "key": "playing", "value": False, "by": "player"})
            ask({"op": "set", "id": "tracks", "selected": k})
            was = (room["text"], k)
            continue
        if playing or now != shown:
            shown = now
            ask({"op": "write", "write": "put", "key": "player", "by": "player",
                 "value": {"at": at, "len": length, "el": clock(at), "left": clock(length - at)}})
        time.sleep(TICK)


try:
    main()
except (ConnectionRefusedError, FileNotFoundError, KeyboardInterrupt):
    pass  # the screen has gone
