"""Cross-check the complete output with libbz2, then identify every native trace."""
import bz2
import csv
import hashlib
import sys
from pathlib import Path

def digest(path):
    """The SHA-256 of a file."""
    with path.open("rb") as f: return hashlib.file_digest(f,"sha256").hexdigest()
corpus,verify,evidence=map(Path,sys.argv[1:4])
roots=list(map(Path,sys.argv[4:]))
modes=["stock","direct","tiles"]
expected=bz2.decompress(corpus.read_bytes())
sha=hashlib.sha256(expected).hexdigest()
pool=[]; phases=[]; identities=[]; shapes=[]; floors=[]
for mode,root in zip(modes,roots,strict=True):
    d=verify/mode
    assert (d/"output").read_bytes()==expected
    for x in csv.DictReader((d/"pool.csv").open()):
        assert int(x["blocks"])==431 and int(x["output_bytes"])==len(expected)
        pool.append(dict(arm=mode,**x))
    if mode!="stock":
        s=(d/"tests").read_text()
        needle={"direct":"astra_overlaps_and_refusals","tiles":"astra_all_rows_and_cycles"}[mode]
        assert needle+" ... ok" in s
    for run in sorted(p for p in root.iterdir() if p.is_dir()):
        data=(run/"output").read_bytes()
        assert data==bz2.decompress((run/"input.bz2").read_bytes())
        ins=list(csv.reader((run/"instructions").open()))
        for p,n in ins: phases.append(dict(arm=mode,ordinal=int(run.name),phase=p,guest_I=n))
        hashes={k:digest(run/k) for k in ("events","ranges","instructions")}
        identities.append(dict(arm=mode,ordinal=int(run.name),trace_bytes=(run/"events").stat().st_size,
                                  output_sha256=hashlib.sha256(data).hexdigest(),**hashes))
        ranges = {}
        for line in (run / "ranges").read_text().splitlines():
            parts = line.split()
            if parts[:1] == ["range"] and parts[1] in {"compressed", "final"}:
                ranges[parts[1]] = int(parts[2], 16)
        input_size = (run / "input.bz2").stat().st_size
        output_size = len(data)
        input_offset = ranges["compressed"] % 64
        final_offset = ranges["final"] % 64
        input_lines = (input_offset + input_size + 63) // 64
        final_lines = (final_offset + output_size + 63) // 64
        input_first = ranges["compressed"] // 64
        final_first = ranges["final"] // 64
        assert (input_first + input_lines <= final_first or
                final_first + final_lines <= input_first)
        floors.append(dict(arm=mode, ordinal=int(run.name), compressed=input_size, output=output_size,
                           input_line_offset=input_offset, output_line_offset=final_offset,
                           input_lines=input_lines, output_lines=final_lines,
                           input_plus_output_B_per_B=f"{1+input_size/output_size:.7f}",
                           ordinary_cold_B_per_B=f"{64*(input_lines+2*final_lines)/output_size:.7f}"))
        shape={}
        for line in (run/"ranges").read_text().splitlines():
            a=line.split()
            if a[0]=="info": shape.update(dict(zip(a[1::2],a[2::2])))
        if mode=="stock": shapes.append(dict(ordinal=int(run.name),**shape))
for name,rows in [("pool",pool),("instructions",phases),("trace-digests",identities),("stock-shapes",shapes),("compulsory",floors)]:
    with (evidence/(name+".csv")).open("w") as f:
        w=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n")
        w.writeheader();w.writerows(rows)
s=(verify/"stock/rows.csv").read_text()
(evidence/"corpus-routes.csv").write_text(s)
(evidence/"checks.txt").write_text(
    f"libbz2 and all three whole-stream outputs: {len(expected)} bytes SHA-256 {sha}\n"+
    f"Single-block libbz2 comparisons: {len(identities)}\n"+
    "Native small checks passed: astra_overlaps_and_refusals, astra_all_rows_and_cycles\n")
