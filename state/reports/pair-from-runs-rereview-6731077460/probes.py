"""Untimed counterexamples for #272 at 4c70dfc42536c32e9b55c6f2e95da5b1d8538b39.
Run beside the archived bench/ directory: python3 probes.py.
Uses the candidate's own fixture generator; no decoder or timing execution.
"""
import sys, tempfile, json
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent / "bench"))
import test_pair_runs_validity as t
import pair_runs_validity as v

def mutate(d, copy, field, value):
    def edit(lines):
        out=[lines[0]]
        for line in lines[1:]:
            row=line.split(",")
            if row[0] == t.CORPUS:
                row[field]=str(value)
            out.append(",".join(row))
        return out
    t.edit_raw(d,copy,edit)

results=[]
for name, changes in [
    ("control", []),
    ("different_child_paired_rows", [("2-cpu1",5,400000000)]),
    ("single_denominator_times_100", [("1",5,20000000000)]),
    ("negative_paired_blocks_all_copies", [(c,4,-233) for c in v.copies_for(2)]),
    ("different_child_output_bytes", [("2-cpu1",6,1)]),
]:
    with tempfile.TemporaryDirectory() as tmp:
        d=t.make_run(tmp)
        for args in changes:
            mutate(d,*args)
        ok,lines=v.validate(d)
        assert ok, lines
        results.append({"case":name,"valid":ok,"readings":[s.strip() for s in lines if s.startswith("  ")]})
assert "NOT MET" in results[0]["readings"][1]
assert ": MET" in results[2]["readings"][1]
print(json.dumps(results,indent=2))

