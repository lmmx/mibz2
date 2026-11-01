"""Read-only: per-block, per-phase thread CPU time from the attribution archive's full
traces (occupancy example, level 2), at 1 and 4 workers, paired by block; and, within the
4-worker runs, each phase's CPU time against what the other workers were doing meanwhile."""
import collections, glob, json, statistics, sys
from pathlib import Path
PH = [("mtf", "DecodeStart", "InvertStart"), ("build", "InvertStart", "WalkStart"), ("walk", "WalkStart", "InvertEnd"), ("post", "InvertEnd", "DecodeEnd")]
def load(path):
    rows = []
    for l in open(path):
        r = l.rstrip("\n").split("\t")
        if r[0] == "thread": continue
        rows.append(dict(name=r[1], ns=int(r[2]), kind=r[3], place=None if r[4] == "-" else int(r[4]), cpu=None if r[5] == "-" else int(r[5]), len=None if r[9] == "-" else int(r[9])))
    return rows
def blocks(rows):
    """per worker: [(place, rows, {kind: (ns, cpu)})]"""
    out = []
    by = collections.defaultdict(list)
    for r in rows:
        if r["name"].startswith("mibz2-") and r["name"][6:].isdigit(): by[r["name"]].append(r)
    for w, rs in by.items():
        cur = None
        for r in rs:
            if r["kind"] == "DecodeStart": cur = dict(worker=w, place=r["place"], m={})
            if cur is not None and r["kind"] in ("DecodeStart", "InvertStart", "WalkStart", "InvertEnd", "DecodeEnd"):
                cur["m"][r["kind"]] = (r["ns"], r["cpu"])
                if r["kind"] == "InvertStart": cur["rows"] = r["len"]
                if r["kind"] == "DecodeEnd": out.append(cur); cur = None
    return out


def main():
    d = Path(sys.argv[1])
    res = {}
    for t in (1, 4):
        per = collections.defaultdict(lambda: collections.defaultdict(list))  # place -> phase -> [cpu ns]
        wall = collections.defaultdict(lambda: collections.defaultdict(list))
        for f in sorted(d.glob(f"r??-t{t}-full.tsv")):
            for b in blocks(load(f)):
                if b.get("rows") != 900000: continue
                for name, a, z in PH:
                    if a in b["m"] and z in b["m"] and b["m"][a][1] is not None and b["m"][z][1] is not None:
                        per[b["place"]][name].append(b["m"][z][1] - b["m"][a][1])
                        wall[b["place"]][name].append(b["m"][z][0] - b["m"][a][0])
        res[t] = (per, wall)
    common = sorted(set(res[1][0]) & set(res[4][0]))
    out = dict(blocks_paired=len(common))
    tot1 = tot4 = 0
    for name, _, _ in PH:
        a = [statistics.median(res[1][0][p][name]) for p in common if res[1][0][p][name] and res[4][0][p][name]]
        b = [statistics.median(res[4][0][p][name]) for p in common if res[1][0][p][name] and res[4][0][p][name]]
        wa = [statistics.median(res[1][1][p][name]) for p in common if res[1][1][p][name] and res[4][1][p][name]]
        wb = [statistics.median(res[4][1][p][name]) for p in common if res[1][1][p][name] and res[4][1][p][name]]
        s1, s4 = sum(a), sum(b); tot1 += s1; tot4 += s4
        ratios = sorted(y / x for x, y in zip(a, b) if x)
        out[name] = dict(cpu_ns_per_row_1=round(s1 / len(a) / 9e5, 3), cpu_ns_per_row_4=round(s4 / len(b) / 9e5, 3), ratio=round(s4 / s1, 3),
                         block_ratio_median=round(statistics.median(ratios), 3), block_ratio_p10_p90=(round(ratios[len(ratios)//10], 3), round(ratios[9*len(ratios)//10], 3)),
                         wall_over_cpu_4=round(sum(wb) / s4, 3))
        out[name]["_sum1"], out[name]["_sum4"] = s1, s4
    inc = tot4 - tot1
    for name, _, _ in PH:
        out[name]["share_of_increase"] = round((out[name]["_sum4"] - out[name]["_sum1"]) / inc, 3)
        del out[name]["_sum1"], out[name]["_sum4"]
    out["all_phases_ratio"] = round(tot4 / tot1, 3)
    print(json.dumps(out, indent=1))


if __name__ == "__main__":
    main()
