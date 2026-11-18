"""Collate the native census and paired timing CSVs. Standard library only."""
import argparse
import csv
from pathlib import Path
from statistics import median


def read(path):
    with path.open() as f:
        return list(csv.DictReader(f))


def records(path):
    return [{k: int(v) for k, v in row.items()} for row in read(path)]


def write(path, keys, rows):
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=keys, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def collate(raw, out):
    detailed, sweep = [], []
    frozen = []
    for corpus in ["wikidata", "prose", "frozen"]:
        all_counts = {}
        for mode in ["census", "induced", "switches"]:
            rows = records(raw / f"{corpus}-{mode}.csv")
            all_counts[mode] = {(r["ordinal"], r["limit"]): r for r in rows}
            for threshold in sorted({r["limit"] for r in rows}):
                group = [r for r in rows if r["limit"] == threshold and r["n"] >= 65536]
                for key in ["r", "retained", "thick", "bands", "words", "copies", "traced",
                            "thick_area", "unit_work", "thin_traced", "open_nodes", "steps", "rules"]:
                    values = [r.get(key, 0) / r["n"] for r in group]
                    if values and max(values):
                        sweep.append(dict(corpus=corpus, mode=mode, limit=threshold,
                                          blocks=len(group), fraction=key + "/n",
                                          minimum=f"{min(values):.8f}", median=f"{median(values):.8f}",
                                          maximum=f"{max(values):.8f}"))
        for idx, r in all_counts["induced"].items():
            if r["limit"] != 16:
                continue
            d = all_counts["census"][idx]
            t = all_counts["switches"][idx]
            # The two constructors can split equivalent arrows differently
            # on general inputs. These are the invariant area and traffic count.
            assert all(r[k] == d[k] for k in ("n", "r", "retained", "thick", "copies", "thick_area"))
            row = {"corpus": corpus, "ordinal": r["ordinal"], "n": r["n"], "r": r["r"],
                   "run_front": r["admitted"], "m": r["retained"], "H": r["thick"],
                   "B": d["bands"], "nonempty": d["words"], "A": d["thick_area"],
                   "U": d["traced"], "unit_work": d["unit_work"], "h": d["copies"],
                   "induced_B": r["bands"], "induced_U": r["traced"],
                   "open_nodes": r["open_nodes"], "steps": r["steps"], "rules": r["rules"],
                   "switch_rows": t["retained"], "switch_bands": t["bands"],
                   "switch_narrow_U": t["thin_traced"], "switch_wide_U": t["traced"]}
            detailed.append(row)
        if corpus == "frozen":
            for mode, counts in all_counts.items():
                for r in counts.values():
                    frozen.append({"mode": mode, **r})
    write(out / "blocks.csv", detailed[0], detailed)
    write(out / "widths.csv", sweep[0], sweep)
    write(out / "frozen.csv", frozen[0], frozen)


def timing(paths):
    for path in paths:
        groups = {}
        for r in read(path):
            if int(r["round"]) == 0:
                continue
            group = (r.get("experiment", path.stem), r.get("input", ""), r["workers"])
            groups.setdefault(group, {}).setdefault(r["arm"], {})[r["round"]] = float(r["seconds"])
        for group, arms in groups.items():
            for arm, values in arms.items():
                wall = median(values.values())
                ratios = sorted(t / arms["cells"][i] for i, t in values.items())
                print(",".join((*group, arm, f"{wall:.6f}", f"{median(ratios):.4f}",
                                f"{min(ratios):.4f}", f"{max(ratios):.4f}")))


def cache(raw, output):
    rows = []
    for p in sorted(raw.glob("*.cg")):
        with p.open() as f:
            for line in f:
                if line.startswith("events:"):
                    keys = line.split()[1:]
                elif line.startswith("summary:"):
                    rows.append(dict(arm=p.stem, **dict(zip(keys, map(int, line.split()[1:])))))
    write(output, rows[0], rows)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    subs = parser.add_subparsers(dest="mode", required=True)
    p = subs.add_parser("counts")
    p.add_argument("raw", type=Path)
    p.add_argument("out", type=Path)
    p = subs.add_parser("timing")
    p.add_argument("csv", type=Path, nargs="+")
    p = subs.add_parser("cache")
    p.add_argument("raw", type=Path)
    p.add_argument("output", type=Path)
    a = parser.parse_args()
    if a.mode == "counts":
        collate(a.raw, a.out)
    elif a.mode == "timing":
        timing(a.csv)
    else:
        cache(a.raw, a.output)
