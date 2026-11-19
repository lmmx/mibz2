"""Freeze newly applicable CI evidence and independently replay the existing dose and edge receipts."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path("research/astra-e")
OUT = ROOT / "retrospective/nov19-edge"
DATA = "6eaf1e8bace868fe8f8bba5f73e1f6ea9e83afca"
SCORER = "75297a1d57b2bdf262c3a19f3064848dedf2c493"


def module(name, path):
    spec = importlib.util.spec_from_file_location(name,path)
    obj = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(obj)
    return obj


def write(name, data):
    (OUT / name).write_text(json.dumps(data, indent=2, allow_nan=False)+"\n")


def main():
    assert __debug__ and subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip() == DATA
    old = module("old_receipts", ROOT / "retrospective/receipts.py")
    ci = {label + "_ci_run": old.ci_receipt(stem) for label,stem in [
        ("compact_preload", "compact-preload-44440381983"),
        ("precounted_front", "front-fusion-44445335323")]}
    # Store only the two missing receipts: the live reader recursively discovers dose and edge.
    write("ci-receipts.json",ci)
    dose = old.ci_receipt("worker-dose-44448175373")
    old_dose = next(iter(json.loads((ROOT / "results/v1p2-dose-gate/ci-receipt.json").read_text()).values()))
    fields = ["id","artifact_sha256","head","selected","candidate_source_sha256","candidate_over_reference"]
    assert all(dose[k] == old_dose[k] for k in fields)
    edge = module("edge_heldout", ROOT / "results/edge-route-pricing/heldout.py")
    receipt,_ = edge.measurement()
    saved = next(iter(json.loads((ROOT / "results/v1p2-edge-confirm/i9-receipt.json").read_text()).values()))
    assert json.loads(json.dumps(receipt)) == saved
    evidence = [(k,v["id"],v["head"],v["candidate_source_sha256"],v["artifact_sha256"]) for k,v in ci.items()]
    evidence.append(("existing dose",dose["id"],dose["head"],dose["candidate_source_sha256"],dose["artifact_sha256"]))
    evidence.append(("existing edge",saved["id"],saved["head"],saved["candidate_source_sha256"],saved["artifact_sha256"]))
    write("verified-inputs.json",evidence)


if __name__ == "__main__":
    main()
