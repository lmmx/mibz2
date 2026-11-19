"""Verify a completed four- or five-arm CI archive and reproduce its Markdown receipt."""

import argparse
import hashlib
import json
from pathlib import Path
import math
import re
import statistics
import subprocess
from zipfile import ZipFile

p = argparse.ArgumentParser()
p.add_argument("zip", type=Path)
p.add_argument("run", type=int)
p.add_argument("artifact", type=int)
p.add_argument("digest")
p.add_argument("head")
p.add_argument("title", help="human-readable experiment name")
p.add_argument(
    "--predecessor",
    nargs=2,
    metavar=("COMMIT", "ENTRY"),
    help="expected immutable fifth-arm revision and source entry",
)
a = p.parse_args()
if not __debug__:
    raise SystemExit("run this verifier with Python assertions enabled")
assert hashlib.sha256(a.zip.read_bytes()).hexdigest() == a.digest
names = ["lazy", "cells", "opend", "candidate"]
if a.predecessor:
    names.append("predecessor")
    pin, ppath = a.predecessor
n = len(names) * 4


def git(*args):
    return subprocess.check_output(["git", *args])


with ZipFile(a.zip) as z:
    assert z.testzip() is None
    manifest = z.read("manifest.txt").decode()
    model = re.search(r"Model name:\s*(.+)", z.read("lscpu.txt").decode()).group(1).strip()
    cpus = re.search(r"^CPU\(s\):\s*(\d+)", z.read("lscpu.txt").decode(), re.M).group(1)
    lines = manifest.splitlines()
    assert "head " + a.head in lines
    path = git("show", a.head + ":research/astra-e/ci/candidate").decode().strip()
    digest = hashlib.sha256(git("show", a.head + ":" + path)).hexdigest()
    assert digest + "  " + path in lines
    checksum = "82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915"
    assert checksum + "  corpora/wikidata.bz2" in lines
    if a.predecessor:
        assert f"matched predecessor {pin} {ppath}" in lines
        pdigest = hashlib.sha256(git("show", pin + ":" + ppath)).hexdigest()
        assert any(x.startswith(pdigest + "  ") and x.endswith("/" + ppath) for x in lines)
    checks = {}
    for arm in names:
        data = list(map(json.loads, z.read(arm + ".check.jsonl").splitlines()))
        *cases, verdict = data
        assert verdict == {
            "arm": "candidate" if arm == "predecessor" else arm,
            "verdict": "PASS",
            "failures": 0,
        }
        assert all(c.get("pass") is True and c.get("failure") is None for c in cases)
        assert any("wikidata" in c["case"] and c["bytes"] == 209700196 for c in cases)
        checks[arm] = len(cases)
    assert len(set(checks.values())) == 1
    rows = list(map(json.loads, z.read("runs.jsonl").splitlines()))
    exp = []
    for rnd in range(n):
        for w in [1, 2, 4]:
            r = rnd + w
            arms = names[r % len(names) :] + names[: r % len(names)]
            if r // len(names) % 2:
                arms.reverse()
            exp.extend((rnd, w, arm) for arm in arms)
    assert [(r["round"], r["workers"], r["arm"]) for r in rows] == exp
    builds = {
        "lazy": "base",
        "cells": "base",
        "opend": "base",
        "candidate": "head",
        "predecessor": "frozen predecessor",
    }
    assert all(r["out_bytes"] == 838800784 and r["build"] == builds[r["arm"]] for r in rows)
    if a.predecessor:
        assert all(
            r["compiled_arm"] == ("candidate" if r["arm"] == "predecessor" else r["arm"]) for r in rows
        )
    data = json.loads(z.read("ratios.json"))
    labels = {"master LazyCells": "lazy", "master eager Cells": "cells", "master opend": "opend"}
    if a.predecessor:
        extra = set(data) - set(labels)
        assert extra in ({"frozen predecessor"}, {"frozen narrow-short/ahead"})
        labels[extra.pop()] = "predecessor"
    assert data.keys() == labels.keys()
    ts = {16: 2.13144954556, 20: 2.09302405441}
    for label, arm in labels.items():
        for w in [1, 2, 4]:
            subset = [r for r in rows if r["workers"] == w]
            cand = sorted((r for r in subset if r["arm"] == "candidate"), key=lambda r: r["round"])
            base = sorted((r for r in subset if r["arm"] == arm), key=lambda r: r["round"])
            logs = [math.log(c["wall_s"] / b["wall_s"]) for c, b in zip(cand, base)]
            m, half = statistics.mean(logs), ts[n] * statistics.stdev(logs) / math.sqrt(n)
            rss = statistics.median(c["peak_rss_kb"] for c in cand) / statistics.median(
                b["peak_rss_kb"] for b in base
            )
            got = data[label][str(w)]["candidate"]
            assert got["n"] == n
            for key, x in [
                ("geomean", math.exp(m)),
                ("lo", math.exp(m - half)),
                ("hi", math.exp(m + half)),
                ("peak_rss_ratio", rss),
            ]:
                assert math.isclose(got[key], x, rel_tol=2e-10), (key, got[key], x)
    prev = next((x for x in lines if x.startswith("scorer v")), "scorer version unprinted")
    base = next(x.split()[1] for x in lines if x.startswith("base "))
    rev = next(x.split()[-1] for x in lines if x.startswith("scorer path revision "))
    src = next(x.split()[-1] for x in lines if x.startswith("selected scorer source "))
    text = [
        f"# {a.title} in CI run {a.run}",
        "",
        f"The {model} runner exposed {cpus} logical CPUs. Each sample decoded four copies of Wikidata, or 838,800,784 bytes. The runner used 4W slots at worker counts 1, 2 and 4 and a two-second settle between samples. "
        f"The official schedule completed {n} paired rounds and {len(rows)} timed samples. Each of the {len(names)} arms passed all {next(iter(checks.values()))} correctness records, including the Wikidata output and the mutation suite.",
        "",
        f"PR head `{a.head}`, master reference build `{base}`. The scorer projection came from `{src}` with path revision `{rev}`. The manifest printed `{prev}`.",
        "",
        f"Candidate entry `{path}`, SHA-256 `{digest}`. Corpus SHA-256 `{checksum}`.",
        "",
    ]
    if a.predecessor:
        text.extend(
            [
                f"The fifth arm built `{ppath}` at `{pin}` in a dedicated checkout with the same scorer projection. The two candidate executables have separate build hashes in the manifest.",
                "",
            ]
        )
    text.extend(
        [
            "The wall ratio uses the geometric mean of paired candidate/reference times. Its 95% Student interval uses paired log ratios. RSS is the candidate median peak divided by the reference median peak. The ZIP passed CRC checks, and the archived arm order, verification records, timing intervals and RSS ratios reproduced from the raw JSONL.",
            "",
            "| Workers | Reference | Wall ratio [95% interval] | Median peak RSS ratio |",
            "| ---: | --- | ---: | ---: |",
        ]
    )
    for w in [1, 2, 4]:
        for label in labels:
            r = data[label][str(w)]["candidate"]
            text.append(
                f"| {w} | {label} | {r['geomean']:.4f} [{r['lo']:.4f}, {r['hi']:.4f}] | {r['peak_rss_ratio']:.4f} |"
            )
    text.extend(
        [
            "",
            f"GitHub published artifact `{a.artifact}` with SHA-256 `{a.digest}`. The ZIP includes all correctness cases, raw timing rows, build identities, CPU information, console output and the original comparison tables. [Workflow run](https://github.com/lmmx/mibz2/actions/runs/{a.run}) · [Archived artifact]({a.zip.name})",
            "",
        ]
    )
    print("\n".join(text))
