"""What master's recorded bench runs already said about CPU per byte against threads.

For each commit on the bench-results branch: CPU per byte at 4 threads over 1 on the
Wikidata blocks (each round's ratio, their geometric mean and 95% t-interval), for mibz2,
lbzip2 and pbz2, on the same runner in the same rounds; and mibz2's over lbzip2's, paired
by round. Run from the repository's root:

    git fetch origin bench-results:refs/remotes/origin/bench-results
    mkdir -p /tmp/results && git archive origin/bench-results | tar -x -C /tmp/results
    python3 docs/journal/2026-10-28-early-warning/retro.py /tmp/results
"""

import json
import math
import statistics
import sys
from collections import defaultdict
from pathlib import Path

T95 = {4: 2.776, 5: 2.571, 6: 2.447, 7: 2.365, 8: 2.306, 9: 2.262}


def ci(logs: list[float]) -> str:
    m = statistics.fmean(logs)
    h = T95.get(len(logs) - 1, 2.0) * statistics.stdev(logs) / math.sqrt(len(logs))
    return f"{math.exp(m):.2f} ({math.exp(m - h):.2f}-{math.exp(m + h):.2f})"


def main(store: Path) -> None:
    print("| commit | runner | mibz2 | lbzip2 | pbz2 | mibz2 over lbzip2 |")
    print("|---|---|--:|--:|--:|--:|")
    for line in (store / "records.jsonl").read_text().splitlines():
        rec = json.loads(line)
        by = defaultdict(dict)  # (arm, threads) -> round -> run
        for l in (store / "raw" / f"{rec['commit']}.jsonl").read_text().splitlines():
            r = json.loads(l)
            if r["corpus"] == "wikidata" and not r["err"]:
                by[(r["arm"], r["threads"])][r["round"]] = r

        def amp(arm: str, k: int) -> float:
            return by[(arm, 4)][k]["cpu"] / by[(arm, 1)][k]["cpu"]

        def rounds(*arms: str) -> list[int]:
            return [k for k in by[(arms[0], 1)] if all(k in by[(a, t)] for a in arms for t in (1, 4))]

        cells = [ci([math.log(amp(a, k)) for k in rounds(a)]) for a in ("mibz2", "lbzip2", "pbz2")]
        ks = rounds("mibz2", "lbzip2")
        rel = ci([math.log(amp("mibz2", k) / amp("lbzip2", k)) for k in ks])
        print(f"| {rec['commit'][:7]} | {rec['cpu']} | " + " | ".join(cells) + f" | {rel} |")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
