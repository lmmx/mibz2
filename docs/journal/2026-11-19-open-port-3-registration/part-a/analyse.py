"""Part A of open-port-3: the cachegrind causal diagnostic, v0.5.6 (530ed404) against the port
(9743bbe0), on the nine zero-route corpora. Reads raw/cg.<corpus>.<t threads | w workers>.<build>.<repeat>
and raw/out.<...> (the child's JSON line and cachegrind's summary), writes summary.json and
prints the table.

Per (corpus, threads): each event's mean over repeats per build; the port over v0.5.6; and the
same-build spread (max/min - 1 over that build's repeats), the A/A noise floor.
VOID if any output (size, digest) differs from the corpus's other runs, or any port run spends any
instruction in `mibz2::open::emit` (every opened block calls it: open.rs:361; the positive control
shows it nonzero when blocks open).
Triggers (QUEUED 7013818037): Ir differs by more than 0.1%, or any miss or mispredict count by
more than 2%. A trigger is an investigation item, never a pass or a fail by itself."""
import collections, glob, json, os, re, statistics, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cgfn  # noqa: E402

EV = ["Ir", "I1mr", "ILmr", "D1mr", "D1mw", "DLmr", "DLmw", "Bc", "Bcm", "Bi", "Bim"]
MISS = [e for e in EV if e not in ("Ir", "Bc", "Bi")]
here = os.path.dirname(os.path.abspath(__file__))
runs = collections.defaultdict(dict)
voids, outs, emits = [], collections.defaultdict(set), {}
for path in sorted(glob.glob(os.path.join(here, "raw", "cg.*"))):
    m = re.fullmatch(r"cg\.(.+)\.([tw]\d+)\.(port|v0\.5\.6)\.(\d+)", os.path.basename(path))
    corpus, t, build, rep = m.group(1), m.group(2), m.group(3), int(m.group(4))
    ev, per, tot = cgfn.load(path)
    tot = dict(zip(ev, tot))
    child = None
    for line in open(path.replace("/cg.", "/out.")):
        if line.startswith("{"):
            child = json.loads(line)
    if child is None or child.get("err"):
        voids.append(f"{os.path.basename(path)}: the child failed")
        continue
    outs[corpus].add((child["out"], child["hash"]))
    em = sum(xs[0] for fn, xs in per.items() if fn.startswith("_ZN5mibz24open4emit"))
    induced = sum(xs[0] for fn, xs in per.items() if "4open6Kernel7induced" in fn)
    if build == "port":
        emits[(corpus, t, rep)] = (em, induced)
        if em or induced:
            voids.append(f"{corpus} {t} #{rep}: the open kernel ran (emit Ir {em}, induced Ir {induced})")
    runs[(corpus, t)].setdefault(build, []).append({e: tot[e] for e in EV})
for c, o in outs.items():
    if len(o) != 1:
        voids.append(f"{c}: the outputs differ: {sorted(o)}")
rows, triggers = [], []
for (c, t), b in sorted(runs.items()):
    p, v = b.get("port", []), b.get("v0.5.6", [])
    if not p or not v:
        voids.append(f"{c} {t}: a build has no runs")
        continue
    row = {"corpus": c, "threads": t, "repeats": [len(p), len(v)], "out": sorted(outs[c])[0]}
    for e in EV:
        mp, mv = statistics.mean(r[e] for r in p), statistics.mean(r[e] for r in v)
        ratio = mp / mv if mv else (1.0 if mp == 0 else float("inf"))
        aa = max(
            (max(r[e] for r in xs) / min(r[e] for r in xs) - 1) if min(r[e] for r in xs) else 0.0
            for xs in (p, v)
        )
        row[e] = {"port": mp, "v056": mv, "ratio": ratio, "aa": aa}
        lim = 0.001 if e == "Ir" else 0.02 if e in MISS else None
        if lim is not None and abs(ratio - 1) > lim:
            triggers.append({"corpus": c, "threads": t, "event": e, "ratio": ratio, "aa": aa})
    rows.append(row)
json.dump({"voids": voids, "triggers": triggers, "rows": rows,
           "emit": {f"{c} {t} #{r}": v for (c, t, r), v in sorted(emits.items())}},
          open(os.path.join(here, "summary.json"), "w"), indent=1)
print(f"{len(rows)} cells; VOIDs {len(voids)}; triggers {len(triggers)}")
for v in voids:
    print("  VOID", v)
hdr = "corpus t  " + " ".join(f"{e:>15}" for e in EV)
print(hdr)
for r in rows:
    print(f"{r['corpus']:<12}{r['threads']:>3} " + " ".join(
        f"{r[e]['ratio']:>8.4f}({r[e]['aa']*100:4.1f})" for e in EV))
