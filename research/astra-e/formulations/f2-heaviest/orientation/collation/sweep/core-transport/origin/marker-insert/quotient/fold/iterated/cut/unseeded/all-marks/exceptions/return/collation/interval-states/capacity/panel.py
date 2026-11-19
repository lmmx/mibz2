import csv
from pathlib import Path
import subprocess
import sys

def run(exe, wires, source, dest):
    with source.open() as f:
        rows = [r for r in csv.DictReader(f) if r["k"] == "8"]
    dest.mkdir(parents=True, exist_ok=True)
    with (dest / "dual.csv").open("w") as a, (dest / "bounds.csv").open("w") as b:
        out = {k: csv.writer(f, lineterminator="\n") for k, f in (("dual",a),("bound",b))}
        out["dual"].writerow(("block","h","vote_runs","penalty","relaxed_objective"))
        out["bound"].writerow(("block","h","policy","states","actual_all_default","relaxed_upper","winning_charge","exception_floor"))
        for row in rows:
            block = row["index"]
            data = subprocess.check_output([exe,wires / f"{block}.mask8",row["states"]],text=True)
            for line in data.splitlines():
                kind, *fields = line.split(",")
                out[kind].writerow((block,*fields))

if __name__ == "__main__":
    run(*map(Path, sys.argv[1:]))
