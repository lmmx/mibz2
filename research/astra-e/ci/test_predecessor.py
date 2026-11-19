"""Run both CI rotors without elapsed-time samples; verify arm identity, pairing and reuse."""
from collections import Counter
from contextlib import redirect_stderr
import io
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from unittest.mock import patch

import inputs
import paired


def check(count):
    expected = json.loads((paired.ROOT / "bench/wikidata.json").read_text())["decoded_bytes"] * paired.REPEAT
    calls = Counter()
    costs = {("base", "lazy"): (1.0, 100), ("base", "opend"): (0.875, 80),
             ("base", "cells"): (0.625, 130), ("head", "candidate"): (0.75, 75),
             ("match", "candidate"): (0.9375, 85)}
    def synthetic(self, arm, workers, corpus, perf=None):
        binary = str(self.bin)
        lat, rss = costs[binary, arm]
        key = binary, arm, workers
        rnd = calls[key]
        calls[key] += 1
        return dict(arm=arm, workers=workers, out_bytes=expected, wall_s=lat*(rnd+1),
                    cpu_s=1.0, peak_rss_kb=rss*(rnd+1))
    with TemporaryDirectory() as tmp:
        path = Path(tmp)
        args = ["paired.py","base","head","unused-corpus",tmp]
        arms = ["lazy","cells","opend","candidate"]
        if count == 5:
            args.extend(["--predecessor","match"])
            arms.append("predecessor")
        rounds = 4*len(arms)
        with (patch.object(sys,"argv",args), patch.object(paired.moonshot.Scorer,"run_once",synthetic),
              patch.object(paired.moonshot.time,"sleep",lambda seconds: None), redirect_stderr(io.StringIO())):
            paired.main()
        rows = [json.loads(line) for line in (path/"runs.jsonl").read_text().splitlines()]
        roster = [(rnd,w,arm) for rnd in range(rounds) for w in paired.WORKERS
                  for arm in paired.moonshot.order(arms,rnd+w)]
        assert [(r["round"],r["workers"],r["arm"]) for r in rows] == roster
        assert all(v == rounds for v in calls.values())
        assert len(rows) == rounds*len(arms)*len(paired.WORKERS)
        assert len(calls) == len(arms)*len(paired.WORKERS)
        tables = json.loads((path/"ratios.json").read_text())
        ratios = {"master LazyCells": .75, "master eager Cells": 1.2, "master opend": .75/.875}
        if count == 5:
            ratios["frozen predecessor"] = .8
        assert set(tables) == set(ratios)
        for w in paired.WORKERS:
            for label, desired in ratios.items():
                cell = tables[label][str(w)]["candidate"]
                assert cell["n"] == rounds and abs(cell["geomean"]-desired) < 1e-10
                assert abs(cell["hi"]-desired) < 1e-10 and abs(cell["lo"]-desired) < 1e-10
            if count == 5:
                cell = tables["frozen predecessor"][str(w)]["candidate"]
                assert abs(cell["peak_rss_ratio"] - 75/85) < 1e-10
        assert all(r["build"] == "frozen predecessor" and r["compiled_arm"] == "candidate"
                   for r in rows if r["arm"] == "predecessor")
        print(f"{count} arms: {rounds} complete rounds, {len(rows)} verified binary/arm samples; all paired wall and predecessor RSS checks passed")


def receipts():
    head = inputs.git("rev-parse","HEAD")
    base = "fixture-base"
    receipt = dict(head=head, base=base, scorer=inputs.scorer_revision(),
                   inputs=["research/astra-e/ci/candidate"], generated={})
    selections = {
        "research/astra-e/worker-dose/candidate.rs": inputs.MEASURED_EDGE,
        "research/astra-e/front-fusion/precount/worker/candidate.rs": inputs.MEASURED_COMBINED,
        "research/astra-e/formulations/f2-pages/payload/slots/preload.rs": inputs.GUARDED_POPCNT,
        "research/astra-e/formulations/f2-pages/instructions/candidate.rs": inputs.COUNTED,
        "research/astra-e/edge-budget/candidate.rs": inputs.COMBINED,
        "research/astra-e/final-slot/candidate.rs": inputs.COMBINED,
        "research/astra-e/formulations/f2-rle/combined/candidate.rs": inputs.COMBINED,
        "research/astra-e/formulations/f2-scheduler/candidate.rs": inputs.STATIC_FRONTIER,
    }
    for candidate, pin in selections.items():
        with patch.object(Path, "read_text", return_value=candidate+"\n"):
            assert inputs.predecessor() == pin
            assert inputs.git("cat-file", "-t", pin["commit"]+":"+pin["path"]) == "blob"
            assert not inputs.reusable(receipt,base)
            assert not inputs.reusable(dict(receipt, predecessor=dict(pin,commit="f"*40)),base)
            assert inputs.reusable(dict(receipt, predecessor=pin),base)
    with patch.object(Path, "read_text", return_value="unmatched-candidate\n"):
        assert inputs.predecessor() is None
        assert inputs.reusable(receipt,base)
        assert not inputs.reusable(dict(receipt, predecessor=inputs.STATIC_FRONTIER),base)
    print("Completed-input check rejected absent, changed, and extraneous predecessors. Edge-budget, final-slot and word actions matched combined-ahead; refill matched its static eight-path frontier; guarded POPCNT matched counted; compact preload matched guarded POPCNT; the fused front matched the exact measured i9 combined commit. The mixed worker dose matched the i9 edge leader.")


if __name__ == "__main__":
    check(4)
    check(5)
    receipts()
