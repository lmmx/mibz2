"""Part A's attribution: for each (corpus, threads), each event's difference, the port's mean
over repeats minus v0.5.6's, split by function (demangled; the worker's monomorph, `launch<
with_inverse<OpenCells>>` against `launch<with_inverse<CellsVariant<1>>>`, as one name), with each
build's same-binary spread over its repeats beside it. Also the cycle estimate
CEst = Ir + 10 (I1mr + D1mr + D1mw) + 100 (ILmr + DLmr + DLmw) + 10 (Bcm + Bim)
(cachegrind's documented rough cost model), its relative difference and its A/A spread."""
import collections, glob, os, re, statistics, subprocess, sys, json
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cgfn  # noqa: E402

EV = ["Ir", "I1mr", "ILmr", "D1mr", "D1mw", "DLmr", "DLmw", "Bc", "Bcm", "Bi", "Bim"]
here = os.path.dirname(os.path.abspath(__file__))
names = {}


def norm(d):
    d = re.sub(r"with_inverse<mibz2::open::OpenCellsOf<false>>", "with_inverse<KERNEL>", d)
    d = re.sub(r"with_inverse<mibz2::cells::CellsVariant<1>>", "with_inverse<KERNEL>", d)
    d = re.sub(r"Scratch<mibz2::open::OpenCellsOf<false>>", "Scratch<KERNEL>", d)
    d = re.sub(r"Scratch<mibz2::cells::CellsKernel<false>>", "Scratch<KERNEL>", d)
    return d


# generic std code that only the harness's sampler calls in this process (it parses /proc; the
# decoder reads no directories and splits no strings)
SAMPLER = ("SplitWhitespace", "IsWhitespace", "from_utf8_lossy", "memrchr", "std::fs::", "ReadDir",
           "DirStream", "core::fmt::", "read_to_string", "str::converts::from_utf8", "__vfscanf")


def group(f):
    """decoder: the mibz2 crate's own functions (the worker's monomorph among them); harness: the
    `ab` binary's, with the std code only its sampler calls (`SAMPLER`); runtime: everything else (libc, the allocator,
    the dynamic loader, generic core/std code, which either may call)"""
    if f.startswith(("ab::", "<ab::")) or any(k in f for k in SAMPLER):
        return "harness"
    if "mibz2::" in f:
        return "decoder"
    return "runtime"


def cest(t):
    return t["Ir"] + 10 * (t["I1mr"] + t["D1mr"] + t["D1mw"]) + 100 * (t["ILmr"] + t["DLmr"] + t["DLmw"]) + 10 * (t["Bcm"] + t["Bim"])


data = collections.defaultdict(lambda: collections.defaultdict(list))
for path in sorted(glob.glob(os.path.join(here, "raw", "cg.*"))):
    m = re.fullmatch(r"cg\.(.+)\.([tw]\d+)\.(port|v0\.5\.6)\.(\d+)", os.path.basename(path))
    ev, per, tot = cgfn.load(path)
    mangled = [n for n in per if n not in names]
    if mangled:
        for a, b in zip(mangled, cgfn.demangle(mangled)):
            names[a] = norm(b)
    fns = collections.defaultdict(lambda: [0] * len(ev))
    for n, xs in per.items():
        fns[names[n]] = [a + b for a, b in zip(fns[names[n]], xs)]
    data[(m.group(1), m.group(2))][m.group(3)].append((dict(zip(ev, tot)), {k: dict(zip(ev, v)) for k, v in fns.items()}))
out = {}
want = sys.argv[1:] or None
for (c, t), b in sorted(data.items()):
    p, v = b["port"], b["v0.5.6"]
    cp = [cest(x[0]) for x in p]
    cv = [cest(x[0]) for x in v]
    rec = {"cest": {"ratio": statistics.mean(cp) / statistics.mean(cv),
                    "aa_port": max(cp) / min(cp) - 1, "aa_v056": max(cv) / min(cv) - 1}}
    for g in ("decoder", "harness", "runtime"):
        def gsum(x, e):
            return sum(vals.get(e, 0) for f, vals in x[1].items() if group(f) == g)
        rec[g] = {}
        for e in EV:
            gp = [gsum(x, e) for x in p]
            gv = [gsum(x, e) for x in v]
            rec[g][e] = {"port": statistics.mean(gp), "v056": statistics.mean(gv),
                         "aa": [max(gp) - min(gp), max(gv) - min(gv)]}
        gp = [cest({e: gsum(x, e) for e in EV}) for x in p]
        gv = [cest({e: gsum(x, e) for e in EV}) for x in v]
        rec[g]["CEst"] = {"port": statistics.mean(gp), "v056": statistics.mean(gv), "aa": [max(gp) - min(gp), max(gv) - min(gv)]}
    for e in EV:
        tp = statistics.mean(x[0][e] for x in p)
        tv = statistics.mean(x[0][e] for x in v)
        fnames = set().union(*(x[1] for x in p + v))
        diffs = []
        for f in fnames:
            fp = statistics.mean(x[1].get(f, {}).get(e, 0) for x in p)
            fv = statistics.mean(x[1].get(f, {}).get(e, 0) for x in v)
            sd = max(statistics.pstdev([x[1].get(f, {}).get(e, 0) for x in p]),
                     statistics.pstdev([x[1].get(f, {}).get(e, 0) for x in v]))
            if fp != fv:
                diffs.append((fp - fv, fp, fv, sd, f))
        diffs.sort(key=lambda d: -abs(d[0]))
        rec[e] = {"port": tp, "v056": tv, "delta": tp - tv,
                  "aa": [max(x[0][e] for x in xs) - min(x[0][e] for x in xs) for xs in (p, v)],
                  "top": [{"fn": d[4][:200], "delta": d[0], "port": d[1], "v056": d[2], "sd": d[3]} for d in diffs[:8]]}
    out[f"{c} {t}"] = rec
json.dump(out, open(os.path.join(here, "attribution.json"), "w"), indent=1)
for k, rec in out.items():
    print(f"{k}: CEst x{rec['cest']['ratio']:.5f} (A/A spread port {rec['cest']['aa_port']*100:.3f}%, v0.5.6 {rec['cest']['aa_v056']*100:.3f}%)")
    for g in ("decoder", "harness", "runtime"):
        print(f"  {g}: " + "  ".join(
            f"{e} {rec[g][e]['port'] / rec[g][e]['v056']:.4f}" if rec[g][e]['v056'] else f"{e} {rec[g][e]['v056']:.0f}->{rec[g][e]['port']:.0f}"
            for e in ["CEst"] + EV))
    for e in (want or []):
        r = rec[e]
        print(f"  {e}: {r['v056']:,.0f} -> {r['port']:,.0f} (delta {r['delta']:+,.0f}; A/A ranges {r['aa'][0]:,.0f} / {r['aa'][1]:,.0f})")
        for d in r["top"][:5]:
            print(f"     {d['delta']:+14,.0f}  ({d['v056']:,.0f} -> {d['port']:,.0f}, sd {d['sd']:,.0f})  {d['fn'][:120]}")
