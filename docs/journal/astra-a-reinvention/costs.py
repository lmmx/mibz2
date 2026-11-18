"""Reproduce a scalar-price sensitivity, separate from byte/line counts and hardware."""
import argparse
import csv
from pathlib import Path

HERE = Path(__file__).resolve().parent
DATA = HERE / "evidence"
OLD = HERE.parent / "2026-11-18-cells-endpoint-sweep"


def rows(path):
    with path.open() as stream:
        return [{k: int(v) if v.isdigit() else v for k,v in r.items()} for r in csv.DictReader(stream)]


if __name__ == "__main__":
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--data", type=Path, default=DATA)
    DATA=ap.parse_args().data
    roots = {r["block"]:r for r in rows(DATA/"whole.csv")}
    ops = rows(DATA/"operands.csv")
    old_mem = {(r["block"],r["model"]):r for r in rows(OLD/"traffic.csv")}
    grammars = {r["block"]:r for r in rows(HERE.parent/"2026-11-18-cells-live-reuse"/"blocks.csv")
                if r["corpus"]=="wikidata.bz2"}
    prices = []
    for r in rows(OLD/"prices.csv"):
        b, arm = r["block"],r["model"]
        if arm not in ("eager","lazy"):
            continue
        a, old, g = roots[b], old_mem[b,arm], grammars[b]
        assert a["n"] == g["n"] and a["capped"] == g["records"]
        payload = {x:sum(v["q8_reads"]+v["q8_writes"] for v in ops if v["block"]==b and v["arm"]==x)
                   for x in (arm,"whole")}
        R, K = g["rules"], g["copies"]
        edges = R-1+K
        terminals = R+1-K
        baseline_scalar = r["setup_core"] - sum(old[k] for k in (
            "setup_r","setup_w","core_r","core_w","hint_r","word_r","word_w"))
        writer = 6*R+4*R+edges+terminals+2*(R-1)+2*K+9*K
        w,m,t,p,I,D = (a[k] for k in ("nodes","capped","tiles","phrases","installs","retired"))
        u = a["roots"]-w
        setup = 10*m+13*a["roots"]+2*u+8*w+3*t+3*256
        walk = 16*p+2*a["unit_visits"]+6*a["wide_visits"]+5*(I+D)+7*I+2*D
        prices.append(dict(block=b, arm=arm, admitted=r[arm], cells_q8=payload[arm],
                           cells_setup_induce_scalar=baseline_scalar, cells_writer_scalar=writer,
                           whole_q8=payload["whole"],whole_setup_scalar=setup,whole_walk_scalar=walk,
                           whole_visits=p, extra_spans=p-R-1))
    with (DATA/"resident.csv").open("w") as stream:
        w = csv.DictWriter(stream, prices[0].keys(), lineterminator="\n")
        w.writeheader()
        w.writerows(prices)
