"""Can original run-count bounds certify packed token links before word discovery?"""
import csv
from pathlib import Path

base=Path(__file__).resolve().parent.parent
for src in ("f2b", "f2-word"):
    admitted=[r for r in csv.DictReader((base/src/"geometry.csv").open()) if r["took"]=="1"]
    for key in ("caps", "run_count", "root_runs", "words"):
        # x = zero-based label. Up to 32-b low bits encode a successor;
        # the high b bits have 2**b distinct values.
        held=sum(int(r[key]) <= 1 << (32-(int(r["token_rows"])-1).bit_length()) for r in admitted)
        print(src,key,f"{held}/{len(admitted)}")
    frozen=admitted[0]
    print("frozen", *(f"{k}={frozen[k]}" for k in ("caps","run_count","root_runs","words","token_rows")))
