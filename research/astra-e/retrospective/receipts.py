"""Rebuild the five missing CI receipts and one measured i9 receipt using archived evidence."""
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZipFile
import argparse
import hashlib
import json
import re
import subprocess
import sys
import tarfile

ROOT = Path("research/astra-e")
STUDY = ROOT / "retrospective"
CI = ROOT / "ci/evidence"
I9 = ROOT / "workstation/evidence/combined-i9-c381da2.tar.gz"
I9_DIGEST = "70b3c3a88d330529ff2dc976ce27451a7065b8b8f5fc49e65f20c191a7d32bfa"
TARGET = "8c9640d8ab186f81a0500f48e606e0c21c2a6669"
ENTRY = "research/astra-e/narrow-short/ahead.rs"
CI_STEMS = (
    "combined-ahead-44429350230",
    "edge-budget-44431030145",
    "delimiter-refill-44433942216",
    "combined-word-actions-44436152031",
    "guarded-popcnt-44436826833",
)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def git_source(commit, path):
    return subprocess.check_output(["git", "show", f"{commit}:{path}"])


def group(pattern, text):
    return re.search(pattern, text, re.MULTILINE).group(1)


def ci_receipt(stem):
    path = CI / (stem + ".zip")
    report = path.with_suffix(".md").read_text()
    run = int(stem.rsplit("-", 1)[1])
    artifact, digest = re.search(r"published artifact `(\d+)` with SHA-256 `([a-f0-9]{64})`", report).groups()
    title = group(r"^# (.*) in CI run", report)
    head = group(r"PR head `([a-f0-9]{40})`", report)
    with ZipFile(path) as archive:
        manifest = archive.read("manifest.txt").decode()
        path_digest, selected = re.search(r"^([a-f0-9]{64})  (research/\S+)$", manifest, re.M).groups()
        pred = re.search(r"^matched predecessor (\w+) (\S+)$", manifest, re.M)
        args = [sys.executable, str(CI / "verify.py"), str(path), str(run), artifact, digest, head, title]
        if pred:
            args += ["--predecessor", *pred.groups()]
        assert subprocess.check_output(args, text=True).strip() == report.strip()
        data = json.loads(archive.read("ratios.json"))
        table = {label: {w: cell["candidate"] for w, cell in ws.items()} for label, ws in data.items()}
        return {
            "id": run,
            "artifact": str(path),
            "artifact_id": int(artifact),
            "artifact_sha256": digest,
            "head": head,
            "base": group(r"^base (\w+)", manifest),
            "selected": selected,
            "candidate_source_sha256": path_digest,
            "scorer_path_revision": group(r"^scorer path revision \w+ (\w+)", manifest),
            "printed_scorer_version": group(r"^(scorer v.*)$", manifest),
            "machine": group(r"^Model name:\s*(.+)", archive.read("lscpu.txt").decode()).strip(),
            "candidate_over_reference": table,
        }


def i9_receipt():
    assert sha(I9.read_bytes()) == I9_DIGEST
    with tarfile.open(I9) as archive:
        members = {m.name.split("/", 1)[1]: m for m in archive if m.isfile()}

        def read(name):
            return archive.extractfile(members[name]).read()

        head = read("commits.txt").decode().splitlines()[0]
        assert head == TARGET and read("mode.txt").strip() == b"measure"
        hashes = {path: digest for digest, path in (s.split() for s in read("sources.sha256").decode().splitlines())}
        for path, digest in hashes.items():
            assert sha(git_source(head, path)) == digest
        decision = json.loads(read("decision.json"))
        with TemporaryDirectory() as directory:
            # Use the original grader and scorer even when the live working tree advances.
            root = Path(directory)
            assessor = ROOT / "workstation/assess.py"
            for path in (assessor, Path("bench/moonshot/moonshot.py")):
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                (root / path).write_bytes(git_source(head, path))
            rows = root / "runs.jsonl"
            rows.write_bytes(read("runs.jsonl"))
            assert json.loads(subprocess.check_output([sys.executable, root / assessor, rows])) == decision
        checks = ["candidate-gate/check.jsonl", "gate-open.jsonl", "gate-lazy.jsonl", "gate-opend.jsonl"]
        for name in checks:
            *cases, end = map(json.loads, read(name).splitlines())
            assert end["verdict"] == "PASS" and end["failures"] == 0
            assert len(cases) == (338 if name.startswith("candidate-gate") else 98) and all(c["pass"] is True for c in cases)
        labels = {"lazy": "master LazyCells", "opend": "master opend", "open": "master open"}
        table = {}
        for arm, label in labels.items():
            table[label] = {}
            for w, cell in decision["workers"].items():
                entry = dict(cell[arm])
                table[label][w] = {**entry.pop("wall"), **entry}
        return {
            "id": "run-20261119T013135Z",
            "evidence_commit": "c381da23edd8f466bcfe8b8e6c850075d58588de",
            "artifact": str(I9),
            "artifact_sha256": I9_DIGEST,
            "head": head,
            "selected": ENTRY,
            "candidate_source_sha256": hashes[ENTRY],
            "scorer_path_revision": json.loads(read("scorer-refresh.json"))["selected_path_revision"],
            "printed_scorer_version": group(r"^(scorer v.*)$", read("scorer-report.txt").decode()),
            "machine": group(r"^Model name:\s*(.+)", read("lscpu.txt").decode()).strip(),
            "i9_criterion_met": decision["i9_criterion_met"],
            "candidate_over_reference": table,
        }


def check_older_ci():
    # Preserve the four already-published, structured records, including their original precision.
    ids = set()
    for name in ("conjugate-pilot", "lookahead-pilot", "combined-ahead-pilot"):
        path = ROOT / "results" / name / "assessment.json"
        for key, record in json.loads(path.read_text()).items():
            if not key.endswith("ci_run"):
                continue
            ident = record["id"]
            assert ident not in ids
            ids.add(ident)
            archive_path, = CI.glob(f"*-{ident}.zip")
            assert sha(archive_path.read_bytes()) == record["artifact_sha256"]
            with ZipFile(archive_path) as archive:
                assert archive.testzip() is None
                ratios = json.loads(archive.read("ratios.json"))
                for label, ws in record["candidate_over_reference"].items():
                    for w, cell in ws.items():
                        for key, value in cell.items():
                            assert ratios[label][w]["candidate"][key] == value
    assert len(ids) == 4
    return ids


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if not __debug__:
        raise SystemExit("Verifying the archive requires Python assertions.")
    prior = check_older_ci()
    ci = {stem.rsplit("-", 1)[0].replace("-", "_") + "_ci_run": ci_receipt(stem) for stem in CI_STEMS}
    assert not (prior & {r["id"] for r in ci.values()})
    assert len(prior) + len(ci) == len(list(CI.glob("*.zip")))
    outputs = {
        STUDY / "ci-receipts.json": ci,
        ROOT / "results/combined-v1-proxy/i9-receipt.json": {"combined_i9_run": i9_receipt()},
    }
    for path, obj in outputs.items():
        result = json.dumps(obj, indent=2, allow_nan=False) + "\n"
        if args.write:
            path.write_text(result)
        else:
            assert path.read_text() == result, path
    print(f"Verified {len(prior)} existing CI receipts, {len(ci)} additional CI receipts and the measured i9 receipt.")


if __name__ == "__main__":
    main()
