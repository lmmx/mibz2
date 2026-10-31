"""Freeze inputs/host without timing, or summarize an approved run's TSV. No runner dispatch."""
import argparse
import bz2
import hashlib
import json
import math
import statistics
import subprocess
from pathlib import Path


def command(*args):
    return subprocess.check_output(args, text=True).strip()


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def interval(values):
    # Registered 12 paired rounds, two-sided 95% Student t, df=11.
    assert len(values) == 12
    mean = statistics.mean(values)
    half = 2.200985 * statistics.stdev(values) / math.sqrt(12)
    return {"mean": mean, "lower95": mean-half, "upper95": mean+half}


def manifest(a):
    files = [Path(s) for s in Path(a.files).read_text().splitlines()]
    corpora = []
    meta = {}
    for line in Path(a.check).read_text().splitlines():
        tag, index, blocks, symbols, last, decoded, checkpoints = line.split("\t")
        assert tag == "META"
        meta[int(index)] = dict(blocks=int(blocks), encoded_symbols=int(symbols), last_column_bytes=int(last), decoded_bytes=int(decoded), checkpoint_records=int(checkpoints))
    audits = [[] for _ in files]
    for line in Path(a.audit).read_text().splitlines():
        if line.startswith("AUDIT\t"):
            fields = list(map(int, line.split("\t")[1:]))
            index, block, cp, symbols, explicit, high, bypass, at, n, previous, digits = fields
            audits[index].append(dict(block=block, checkpoint_interval=cp, symbols=symbols, explicit=explicit,
                high=high, bypass=bool(bypass), switch_consumed=at or None, output_before_switch=n,
                p74_high_before_switch=previous, run_digits=digits))
    for i, p in enumerate(files):
        digest = hashlib.sha256()
        size = 0
        with bz2.open(p, "rb") as f:
            while part := f.read(1 << 20):
                digest.update(part)
                size += len(part)
        assert size == meta[i]["decoded_bytes"]
        assert len(audits[i]) == 2 * meta[i]["blocks"]
        name = p.stem
        if name == "wikidata-fitted":
            spec = json.loads(Path("bench/wikidata.json").read_text())
            assert sha(p) == spec["corpus_sha256"] and size == spec["decoded_bytes"]
        corpora.append(dict(name=name, path=str(p.resolve()), compressed_bytes=p.stat().st_size,
            compressed_sha256=sha(p), decoded_sha256=digest.hexdigest(), coverage=meta[i],
            independent=name == "mixed-independent", fitted=name == "wikidata-fitted", audit=audits[i]))
    source = {str(p): sha(p) for p in [Path("examples/p74_experiment.rs"), Path("bench/p74_experiment.py"),
        Path("src/mtf/p74_c.rs"), Path("src/mtf/p74_c/tests.rs"), Path("tests/common/mod.rs")]}
    quota = Path("/sys/fs/cgroup/cpu.max")
    result = dict(schema=1, implementation_reference="1bf554db61a82ff86f832b8697b90675e6a6ba46",
        preparation_commit=command("git", "rev-parse", "HEAD"), working_tree=command("git", "status", "--porcelain"),
        sources_sha256=source, binary_sha256=sha(a.binary), compiler=command("rustc", "-Vv"),
        flags="Cargo release: fat LTO, codegen-units=1; no RUSTFLAGS; baseline target + function AVX2",
        host=dict(cpu=command("lscpu", "-J"), topology=command("lscpu", "-e=CPU,CORE,SOCKET,NODE"),
            affinity=command("taskset", "-pc", str(__import__('os').getpid())),
            quota=quota.read_text().strip() if quota.exists() else "unavailable",
            suitable_for_timing=False, approval="NOT GRANTED; preparation host only"),
        protocol=dict(rounds=12, passes_per_row=2, warmup_passes_per_arm=2,
            ordering="six permutations twice, Fisher-Yates xorshift seed 0x74c0202610310002",
            variant_order=["plain", "checkpoint"], corpus_order=[c["name"] for c in corpora],
            statistic="arithmetic mean of within-round ratios; two-sided 95% Student t df=11, critical=2.200985; no exclusions, fastest-pass selection or optional stopping"),
        assembly_gate="FAILED originally; amended for this experiment only to accept documented +1/+1 instruction discrepancy with six low-path conditional branches and no additional YMM spills; no claim of old gate passing",
        policy="S<16384 or 2*S>=max_len => standalone; else switch iff F+1>floor(consumed/32)+123, trigger included; permanent",
        corpora=corpora, independent_wikidata="not available; robustness qualification remains incomplete")
    Path(a.out).write_text(json.dumps(result, indent=2)+"\n")
    print(json.dumps({"manifest": a.out, "sha256": sha(a.out), "corpora": len(corpora)}))


def summarize(a):
    m = json.loads(Path(a.manifest).read_text())
    groups = {}
    positions = {}
    for line in Path(a.raw).read_text().splitlines():
        if not line.startswith("ROW\t"):
            continue
        f, cp, r, position, arm, ns, n = map(int, line.split("\t")[1:])
        key = f, cp
        d = groups.setdefault(key, {})
        assert (r, arm) not in d and ns > 0
        assert n == 2*m["corpora"][f]["coverage"]["last_column_bytes"]
        d[r, arm] = (ns/2, n/2)
        assert (f, cp, r, position) not in positions
        positions[f, cp, r, position] = arm
    assert set(groups) == {(i,cp) for i in range(len(m["corpora"])) for cp in [0,1]}
    result = []
    for (f,cp), d in sorted(groups.items()):
        assert set(d) == {(r,arm) for r in range(12) for arm in range(3)}
        for r in range(12):
            assert sorted(positions[f,cp,r,p] for p in range(3)) == [0,1,2]
        rows = {}
        for arm in range(3):
            rows[f"T{arm}"] = dict(latency_ns=interval([d[r,arm][0] for r in range(12)]),
                throughput_MB_s=interval([d[r,arm][1]/d[r,arm][0]*1000 for r in range(12)]))
        for arm, base in [(2,0),(1,0),(2,1)]:
            rows[f"T{arm}/T{base}"] = interval([d[r,arm][0]/d[r,base][0] for r in range(12)])
            rows[f"T{arm}-T{base}_ns"] = interval([d[r,arm][0]-d[r,base][0] for r in range(12)])
        main = rows["T2/T0"]
        tax = rows["T1/T0"]
        name = m["corpora"][f]["name"]
        verdict = "descriptive"
        if name in ["random","records","mixed-independent"]:
            verdict = "pass" if main["upper95"] <= 1.03 else "fail"
        elif name == "wikidata-fitted" and cp:
            verdict = "pass" if main["upper95"] <= .95 and 1-main["upper95"] > 2*max(abs(1-tax["lower95"]),abs(1-tax["upper95"])) else "fail"
        result.append(dict(corpus=name, variant="checkpoint" if cp else "plain", rows=rows, verdict=verdict))
    Path(a.out).write_text(json.dumps(dict(results=result, whole_decoder="unmeasured; model speedup=1/(1-f+f*r), with independently established stage fraction f and stage ratio r; do not infer f from these runs"), indent=2)+"\n")


if __name__ == "__main__":
    p=argparse.ArgumentParser(description=__doc__)
    sub=p.add_subparsers(dest="mode",required=True)
    m=sub.add_parser("manifest")
    for name in ["files","check","audit","binary","out"]:
        m.add_argument("--"+name,required=True)
    s=sub.add_parser("summarize")
    for name in ["manifest","raw","out"]:
        s.add_argument("--"+name,required=True)
    a=p.parse_args()
    (manifest if a.mode=="manifest" else summarize)(a)
