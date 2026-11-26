"""Route the official scorer's paired schedule through the true stock-scanner references."""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import sys

def main(build,corpus,out):
    sys.path.insert(0,str(build/"stock/bench/moonshot"))
    import moonshot
    workers=[1,4];arms=["lazy","opend","candidate"]
    binaries={a:build/(a+"-target/release/moonshot") for a in ("stock","phrase")}
    class Routed(moonshot.Scorer):
        def run_once(self,arm,w,corpus,perf=None):
            source="phrase" if arm=="candidate" else "stock"
            self.bin=binaries[source]
            r=super().run_once(arm,w,corpus,perf)
            assert (r["arm"],r["workers"],r["out_bytes"])==(arm,w,419400392)
            return dict(r,source=source)
    opts=argparse.Namespace(workers=workers,rounds=12,repeat=2,settle=2,target_dir=str(build/"stock-target"))
    out.mkdir(exist_ok=False,parents=True)
    s=Routed(opts)
    meta={"printed":moonshot.VERSION+" at "+moonshot.scorer_commit(),
          "module_sha256":hashlib.sha256(Path(moonshot.__file__).read_bytes()).hexdigest(),
          "binary_sha256":{a:hashlib.sha256(f.read_bytes()).hexdigest()for a,f in binaries.items()}}
    for a in ("lazy","opend"):
        path=out/f"gate-{a}.jsonl"
        with path.open("w") as f:
            subprocess.run([binaries["stock"],"check","--quick","--arm",a,"--wikidata",corpus],stdout=f,check=True)
        *cases,verdict=map(json.loads,path.read_text().splitlines())
        assert verdict==dict(arm=a,verdict="PASS",failures=0) and len(cases)>=98
        assert all(c["pass"] and c["failure"] is None for c in cases)
    (out/"metadata.json").write_text(json.dumps(meta,indent=2)+"\n")
    official=s.timing(corpus,out,arms)
    (out/"scorer-summary.json").write_text(json.dumps(official,indent=2)+"\n")
    rows=list(map(json.loads,(out/"runs.jsonl").read_text().splitlines()))
    roster=[(r,w,a)for r in range(12)for w in workers for a in moonshot.order(arms,r+w)]
    assert [(r["round"],r["workers"],r["arm"]) for r in rows]==roster
    assert all(r["source"]==("phrase" if r["arm"]=="candidate" else "stock")for r in rows)
    result={}
    for w in workers:
        result[w]={}
        for ref in ("lazy","opend"):
            cell={}
            for metric in ("wall_s","cpu_s","peak_rss_kb"):
                x=[r[metric]for r in rows if r["workers"]==w and r["arm"]=="candidate"]
                y=[r[metric]for r in rows if r["workers"]==w and r["arm"]==ref]
                cell[metric]=dict(paired=moonshot.paired_ratio(x,y),median_phrase=statistics.median(x),
                                  median_base=statistics.median(y),maximum_phrase=max(x),maximum_base=max(y))
            result[w][ref]=cell
    (out/"summary.json").write_text(json.dumps(result,indent=2)+"\n")
if __name__=="__main__":main(*[Path(x).resolve()for x in sys.argv[1:]])
