"""The spin audit's tables from examples/wait_audit.rs's JSON lines (counts, never times).

Usage: python3 summarise.py counts.jsonl
"""

import collections
import json
import sys


def main(path: str) -> None:
    """Print each label's per-site counts, averaged over its rounds."""
    rows = [json.loads(line) for line in open(path)]
    by = collections.defaultdict(list)
    for r in rows:
        by[r["label"]].append(r)
    print("outputs", {(r["bytes"], r["hash"]) for r in rows})
    for label, rs in by.items():
        n = len(rs)
        spins = sum(r["spins_all"] for r in rs) / n
        est = sum(r["spin_cpu_s_estimate"] for r in rs) / n
        print(
            f"== {label}: rounds {n}, spins/decode {spins:,.0f}, "
            f"spin CPU-s/decode {est:.3f} (estimate: spins x {rs[0]['spin_ns_estimate']} ns)"
        )
        for site in rs[0]["sites"]:
            t = collections.Counter()
            for r in rs:
                t.update(r["sites"][site])
            w = t["waits"] or 1
            useful = (t["waits"] - t["immediate"]) or 1
            print(
                f"   {site:18s} waits {t['waits'] / n:6.0f}"
                f" immediate {t['immediate'] / w:6.1%}"
                f" ended spinning {t['ended_spinning'] / w:6.1%}"
                f" parked {t['parked'] / w:6.1%} yielded {t['yielded'] / w:6.1%}"
                f" spins/wait {t['spins'] / useful:6.0f}"
                f" wasted {t['spins_wasted'] / (t['spins'] or 1):6.1%}"
                f" parks/parked {t['parks'] / (t['parked'] or 1):.2f}"
            )


if __name__ == "__main__":
    main(sys.argv[1])
