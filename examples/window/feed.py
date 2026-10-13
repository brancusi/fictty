import json, random, subprocess, sys, time
F, sock = sys.argv[1], sys.argv[2]
def fictty(*a, data=None):
    subprocess.run([F, "--socket", sock, *a], input=json.dumps(data).encode() if data is not None else None, capture_output=True)
log, pct, t0 = [], 38, time.time()
events = ["<pos>ok</> health check eu-west", "<accent>scale</> search +2 replicas", "<warn>slow</> search p95 412 ms",
          "<pos>ok</> cache hit rate 97%", "<neg>error</> notifications 503", "<accent>deploy</> canary 10% healthy"]
while True:
    now = time.strftime("%H:%M:%S")
    pct = min(100, pct + random.randint(1, 4))
    fictty("merge", "stats", "-", data={"clock": now, "rps": f"{random.randint(11800, 13400):,}", "drps": round(random.uniform(-3, 9), 1),
           "p95": str(random.randint(96, 140)), "dp95": random.randint(-12, 10), "errs": f"{random.uniform(1.1, 1.9):.2f}",
           "derr": round(random.uniform(-0.4, 0.5), 2), "pct": pct})
    log.insert(0, {"t": now[:5], "msg": random.choice(events) if pct < 100 else "<pos>deployed</> api-gateway v2.14.0"})
    fictty("put", "log", "-", data=log[:200])
    if pct >= 100: pct = 0
    time.sleep(0.7)
