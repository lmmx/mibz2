"""Check all raw bands, spelling multiplicities and marked bytes on the two real section families."""
import argparse
from collections import Counter
import csv
from hashlib import sha256
from io import BytesIO
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[5]
sys.path.insert(0, str(HERE.parents[1] / "union"))
from direct import Column, Direct, heavy_set
sys.path.insert(0, str(ROOT / "docs/journal/2026-11-18-astra-c-output-first"))
from inputs import paths, read_input


def native(binary, runs, mark, section):
    size = sum(w for _,w in runs)
    data = f"{size} {mark} {len(runs)} 1\n" + "".join(f"{a} {b}\n" for a,b in [*runs,section])
    stream = BytesIO(subprocess.check_output([binary], input=data.encode()))
    keys = ("bands", "lf_probes", "page_increments", "spelling_bytes", "scatter_writes",
            "mark_probes", "output_lookups", "out_bytes")
    counts = dict(zip(keys, map(int, stream.readline().split()), strict=True))
    bands = []
    for _ in range(counts["bands"]):
        src, dest, width, length = map(int, stream.readline().split())
        bands.append((src, dest, width, stream.read(length), 0))
    output = stream.read()
    assert len(output) == counts["out_bytes"]
    return counts, sorted(bands), output


def verify(binary, runs, mark, section):
    col = Column(runs)
    expected = Direct(col,[section])
    counts, bands, out = native(binary, runs, mark, section)
    assert bands == expected.bands
    mult = Counter()
    for _,_,w,word,_ in bands:
        mult[word] += w
    assert dict(mult) == expected.word_counts
    assert counts["lf_probes"] == counts["spelling_bytes"] == expected.visits
    assert counts["scatter_writes"] == section[1]-section[0] == sum(mult.values())
    assert (out or None) == expected.expand(mark)
    return dict(**counts, unique_words=len(mult), unique_spelling_bytes=sum(map(len,mult)),
                output_sha256=sha256(out).hexdigest())


def main(tapes):
    keys = {}
    with (HERE/"real.csv").open() as f:
        for row in csv.DictReader(f):
            keys[int(row["ordinal"]),row["section"]] = row
    assert len(keys) == 20
    with TemporaryDirectory() as tmp:
        binary = Path(tmp)/"check"
        compiler = ["c++", "-O3", "-std=c++17", "-Wall", "-Wextra"]
        src = HERE/"direct_native_check.cpp"
        subprocess.run([*compiler, src, "-o", binary], check=True)
        small = []
        for runs, section in [([(0,2),(1,3)],(0,2)), ([(1,2),(0,1)], (1,2)),
                              ([(1,1),(0,2)], (0,1))]:
            for mark in range(sum(w for _,w in runs)):
                small.append(verify(binary,runs,mark,section))
        records = []
        for tape in paths(tapes):
            ordinal = int(tape.name.split(".")[0])
            runs, mark = read_input(tape)
            col = Column(runs)
            quote = col.interval(b'"')
            (frequent,),_,_ = heavy_set(col,quote,1)
            for kind,section in (("quote",quote),("frequent",frequent)):
                old = keys.pop((ordinal,kind))
                assert section == (int(old["section_lo"]),int(old["section_hi"]))
                assert sha256(tape.read_bytes()).hexdigest() == old["tape_sha256"]
                checked = verify(binary,runs,mark,section)
                assert checked["output_sha256"] == old["output_sha256"]
                assert checked["bands"] == int(old["direct_raw_bands"])
                assert checked["lf_probes"] == int(old["direct_hops"])
                records.append(dict(ordinal=ordinal,section=kind,**checked))
        assert not keys
    return dict(scope="Single nonempty quote cylinder or the single most frequent parent-return interval",
                comparisons="All raw descriptors and their spellings, weighted multiplicities, canonical mark output and source coverage",
                command=compiler, header_sha256=sha256((HERE/"direct_native.hpp").read_bytes()).hexdigest(),
                wrapper_sha256=sha256(src.read_bytes()).hexdigest(), real=records,
                small_marks=len(small), small_disjoint_marks=sum(not c["out_bytes"] for c in small))


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("tapes",type=Path)
    print(json.dumps(main(p.parse_args().tapes),indent=2))
