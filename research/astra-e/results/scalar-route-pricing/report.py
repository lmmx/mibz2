"""Independently check tape/build/config identities and preserve distinct per-cell metrics."""
import argparse
import csv
import hashlib
import json
import lzma
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent


def csv_file(file,rows):
    with file.open("w") as f:
        w=csv.DictWriter(f,fieldnames=rows[0],lineterminator="\n")
        w.writeheader();w.writerows(rows)


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("source",type=Path,help="scorer checkout with the recorded projection")
    parser.add_argument("data",type=Path,nargs="?",default=HERE)
    a=parser.parse_args()
    sys.path.insert(0,str(a.source.resolve()/"bench/moonshot"))
    import moonshot
    manifest=json.loads((a.data/"manifest.json").read_text())
    assert hashlib.sha256(Path(moonshot.__file__).read_bytes()).hexdigest()==manifest["files"]["bench/moonshot/moonshot.py"]
    for b,datum in manifest["samples"].items():
        payload=lzma.decompress((a.data/"symbols"/f"warm1-measure{b}.sym.xz").read_bytes())
        assert hashlib.sha256(payload).hexdigest()==datum["two_record_sha256"]
    rows=[json.loads(x) for x in (a.data/"runs.jsonl").read_text().splitlines()]
    cells={(r["workers"],r["block"],r["arm"]):r for r in rows}
    assert len(cells)==len(rows)==12
    metrics=[];comparisons=[]
    for w in [7,10]:
        for b in [1,85,165]:
            arms={}
            for arm in ["edge","scalar"]:
                r=cells[w,b,arm];p=r["counters"]
                assert p["out_bytes"]=={1:900000,85:899988,165:900000}[b]
                assert r["latencies"]==moonshot.nominal_latencies(json.loads(moonshot.CALIBRATION.read_text())["model"],w)
                a1=moonshot.proxy_metrics(p)
                a1.update(instructions_raw=p["instructions"],cycles_raw=p["cycles"],past_l2_dep_raw=p["dep_past_l2_loads"],
                          dram_dep_raw=p["dep_dram_loads"],useful_overlap_n=p["dep_overlap_n"],
                          store_buffer_stall_cycles=p["store_buffer_stall_cycles"],
                          dram_read_lines=p["dram_read_lines"],dram_write_lines=p["dram_write_lines"],dram_drain_lines=p["dram_drain_lines"])
                arms[arm]=a1;metrics.append(dict(workers=w,block=b,arm=arm,**a1))
            assert cells[w,b,"edge"]["counters"]["config"]==cells[w,b,"scalar"]["counters"]["config"]
            for metric,v in arms["edge"].items():
                other=arms["scalar"][metric]
                comparisons.append(dict(workers=w,block=b,metric=metric,edge=v,scalar=other,
                                    scalar_minus_edge=other-v,scalar_over_edge=other/v if v else ""))
    csv_file(a.data/"metrics.csv",metrics);csv_file(a.data/"contrasts.csv",comparisons)


if __name__=="__main__":main()
