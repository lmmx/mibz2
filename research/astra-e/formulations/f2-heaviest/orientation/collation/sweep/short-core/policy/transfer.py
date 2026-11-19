"""Run the frozen policy against the baseline on the nine registered systematic blocks."""
import argparse
import csv
import hashlib
import json
from pathlib import Path

from model import application, get_input


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("policy_directory",type=Path)
    p.add_argument("corpus",type=Path)
    p.add_argument("output",type=Path)
    args = p.parse_args()
    literal = (args.policy_directory/"policy.json").read_bytes()
    expected = json.loads((args.policy_directory/"fit.json").read_text())
    assert hashlib.sha256(literal).hexdigest() == expected["literal_sha256"]
    policy = {bytes.fromhex(x) for x in json.loads(literal)}
    rows = []
    for ordinal in range(24,217,24):
        tape = args.corpus/f"{ordinal:04d}.irec.xz"
        col, oracle, sec, words, info = get_input(tape)
        _, ref = application(col, oracle, sec, words, 8)
        _, current = application(col, oracle, sec, words, 16, policy)
        row = dict(ordinal=ordinal,tape_sha256=hashlib.sha256(tape.read_bytes()).hexdigest(),
                   source_first=sec[0],source_end=sec[1],vocabulary=len(words),**info)
        row.update({f"cap8_{k}":v for k,v in ref.items()})
        row.update({f"literal_{k}":v for k,v in current.items()})
        row["both_budgets"] = int(all(current[k]<=ref[k] for k in ("B","F")))
        row["individual_win"] = int(current["Q"]<ref["Q"] and row["both_budgets"])
        rows.append(row)
    with args.output.open("w") as f:
        writer = csv.DictWriter(f, fieldnames=rows[0], lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)
    total = lambda mode,k: sum(row[f"{mode}_{k}"] for row in rows)
    ratios = {k:total("literal",k)/total("cap8",k) for k in ("Q","F","B")}
    individual = sum(row["individual_win"] for row in rows)
    print(json.dumps(dict(ratios=ratios, individual=individual,advance=ratios["Q"]<=0.95
                          and ratios["F"]<=1 and ratios["B"]<=1 and individual>=7)))
