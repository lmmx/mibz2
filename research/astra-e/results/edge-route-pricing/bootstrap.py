"""Exact mass of the zero-change component under the frozen paired block resampling."""
from collections import Counter
import json
from pathlib import Path
import random

p = Path(__file__).resolve().parent
s = json.loads((p / "manifest.json").read_text())
sample = s["sample"]
full, tiny = sample["full"], sample["tiny"]
changed = set(s["changed"])
assert changed.intersection(full) == {165} and not changed.intersection(tiny + sample["slope"])
rng = random.Random(1)  # Same seed and order as the pinned bootstrap_cells.
counts = Counter()
for draw in range(400):
    n = sum(rng.choice(full) in changed for _ in full)
    for _ in tiny:
        rng.choice(tiny)
    counts[n] += 1
print(json.dumps({"reps": 400, "seed": 1, "dense_copies_to_draw_count": dict(sorted(counts.items())),
                  "probability_dense_absent_independent_uniform_full_draws": (1 - 1 / len(full)) ** len(full),
                  "exactly_unit_edge_over_control_replicates_from_absent_block": counts[0]}, indent=2))
