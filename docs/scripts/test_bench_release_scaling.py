"""The release lineage (bench_release_scaling.lineage): every release against every other,
from pairwise release-scaling runs with lbzip2 as the anchor in each.

    python3 docs/scripts/test_bench_release_scaling.py
"""

import json
import math
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench_release_scaling as brs  # noqa: E402

ANCHOR = "lbzip2 2.5"
# each round's multiplicative noise, the same pattern in every arm but shifted per arm
NOISE = [0.01, -0.01, 0.02, -0.02, 0.0, 0.015, -0.015, 0.005, -0.005, 0.0]


def write_run(root: Path, name: str, new: str, old: str, walls: dict, cpu="CPU A", date="2026-11-01"):
    """A run of three arms; walls maps an arm's label to its wall time per thread count."""
    d = root / name
    d.mkdir()
    old_arm = f"mibz2@/build-v{old}/ab"
    arms = {"mibz2": new, old_arm: old, "lbzip2": ANCHOR}
    meta = {"date": date, "cpu": cpu, "cpus": 4, "cores": 4, "ref": "mibz2", "arms": arms,
            "rounds": len(NOISE), "threads": sorted(walls[new])}
    (d / "meta.json").write_text(json.dumps(meta))
    lines = []
    for k, (arm, label) in enumerate(arms.items()):
        for t, w in walls[label].items():
            for i, e in enumerate(NOISE):
                wall = w * math.exp(NOISE[(i + 3 * k) % len(NOISE)])
                lines.append(json.dumps({"round": i, "corpus": "c", "threads": t, "arm": arm,
                                         "wall": wall, "out": 1e8, "hash": "h", "err": ""}))
    (d / "runs.jsonl").write_text("\n".join(lines) + "\n")
    return d


class Lineage(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_recovers_ratios_chain_and_closure(self):
        # true walls relative to lbzip2 at 1 thread: 2.0, 1.6, 1.0; at 2: 1.8, 1.2, 0.9
        rel = {"0.1.0": {1: 2.0, 2: 1.8}, "0.2.0": {1: 1.6, 2: 1.2}, "0.3.0": {1: 1.0, 2: 0.9}}
        lb = {1: 1.0, 2: 0.5}
        walls = lambda r: {t: rel[r][t] * lb[t] for t in lb}  # noqa: E731
        # 0.2.0 drifts by 2% between its two runs, at 1 thread only
        drifted = {1: walls("0.2.0")[1] * 1.02, 2: walls("0.2.0")[2]}
        dirs = [
            write_run(self.root, "a", "0.2.0", "0.1.0", {"0.2.0": walls("0.2.0"), "0.1.0": walls("0.1.0"), ANCHOR: lb}),
            write_run(self.root, "b", "0.3.0", "0.2.0", {"0.3.0": walls("0.3.0"), "0.2.0": drifted, ANCHOR: lb}, date="2026-11-02"),
            # another machine: never combined with the first
            write_run(self.root, "c", "0.3.0", "0.2.0", {"0.3.0": walls("0.3.0"), "0.2.0": walls("0.2.0"), ANCHOR: lb}, cpu="CPU B"),
        ]
        out = brs.lineage(dirs)
        self.assertEqual(len(out), 1, "a machine with only two releases makes no lineage")
        g = out[0]
        self.assertEqual(g["cpu"], "CPU A")
        self.assertEqual(g["releases"], ["0.1.0", "0.2.0", "0.3.0"])
        self.assertEqual([r["id"] for r in g["runs"]], ["a", "b"])
        p1, p2 = g["per_thread"]["1"], g["per_thread"]["2"]
        # x is log(release wall / lbzip2 wall); the oldest over the newest
        self.assertAlmostEqual(math.exp(p2["x"][0] - p2["x"][2]), 2.0, places=5)
        self.assertAlmostEqual(math.exp(p1["x"][0] - p1["x"][2]), 2.0, places=5)
        self.assertEqual(p1["n"], [1, 2, 1])
        # 0.2.0 is pooled from its two runs: between 1.6 and 1.6 * 1.02
        self.assertTrue(math.log(1.6) < p1["x"][1] < math.log(1.6 * 1.02))
        # the chain multiplies the direct pairs: 1.6/1.0 then 1.6*1.02/1.0
        self.assertAlmostEqual(math.exp(p1["chain"][2]), 2.0 / 1.0 * 1.02, places=5)
        self.assertAlmostEqual(math.exp(p2["chain"][2]), 2.0, places=5)
        # the closure residual is the drift, later run minus earlier
        (c1,), (c2,) = p1["closure"], p2["closure"]
        self.assertEqual((c1["release"], c1["runs"]), ("0.2.0", ["a", "b"]))
        self.assertAlmostEqual(c1["d"], math.log(1.02), places=5)
        self.assertAlmostEqual(c2["d"], 0.0, places=5)
        self.assertAlmostEqual(p2["anchor_mbps"], 1e8 / 0.5 / 1e6, places=0)

    def test_recorded_runs(self):
        """The recorded i9 runs reproduce the coordinator's figures for 0.2.1 over 0.3.0."""
        dirs = sorted(p for p in brs.RUNS.glob("*/") if (p / "meta.json").exists())
        g = next((g for g in brs.lineage(dirs) if "i9-10900K" in g["cpu"] and g["corpus"] == "wikidata"), None)
        if g is None or not {"0.2.1", "0.3.0"} <= set(g["releases"]):
            self.skipTest("the i9 runs are not recorded here")
        i, j = g["releases"].index("0.2.1"), g["releases"].index("0.3.0")
        anchor = {1: 1.319, 4: 1.406, 10: 1.329, 20: 1.311}
        chain = {1: 1.335, 4: 1.403, 10: 1.303, 20: 1.240}
        for t in anchor:
            p = g["per_thread"][str(t)]
            self.assertAlmostEqual(math.exp(p["x"][i] - p["x"][j]), anchor[t], places=3)
            self.assertAlmostEqual(math.exp(p["chain"][j] - p["chain"][i]), chain[t], places=3)


if __name__ == "__main__":
    unittest.main()
