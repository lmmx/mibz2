#!/usr/bin/env python3
"""Static archived-evidence replay only; never builds or executes a constructor."""
import argparse
import hashlib
import json
import pathlib
import re

PINS = {
    "examples/c30walk/ctor2.rs": "168c2bac9ad0d8da4d9f82788f009d32f419608e",
    "examples/c30walk/ctor3.rs": "01f91419904a605b721dc1742ba8087fe79f74cf",
    "docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm": "aeeaca9eb28adca5619e2bb4137a55cb6e500c7d",
    "docs/journal/2026-11-04-compact30-epoch-tagged-finaliser/listings/ctor3-finish-false.asm": "44cd87259186ceebec57518ebb7f60c50cc4d7e8",
    "docs/journal/2026-11-04-compact30-epoch-tagged-finaliser/corpora.jsonl": "6fb88f1343c775cf4affad7947378cec324e4985",
}


def replay(root):
    digests = {}
    for name, expected in PINS.items():
        raw = (root / name).read_bytes()
        actual = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
        assert actual == expected, (name, actual, expected)
        digests[name] = hashlib.sha256(raw).hexdigest()
    counter_file = root / next(p for p in PINS if p.endswith("corpora.jsonl"))
    primary = [json.loads(line) for line in counter_file.read_text().splitlines()
               if line.strip()]
    primary = [r for r in primary if r["input"] == "wikidata-20261026.bz2"
               and r["permille"] == 0 and not r["extra_starts"]]
    assert len(primary) == 1
    c = primary[0]
    assert c["rows"] == c["rows_scalar"] + c["rows_fragment"] + c["rows_bulk"]
    assert c["rows_bulk"] == 30 * c["complete_groups"]
    assert c["run_rows"] == c["rows_fragment"] + c["rows_bulk"]
    assert c["fin_fast"] + c["fin_ordinary"] == c["norm_full"] + c["norm_partial"]
    assert c["leaf_fast"] == c["v2_fast_full"]
    assert c["slow_entries"] == c["slow_ordinary_old_epoch"] + c["slow_ordinary_partial_same_epoch"] + c["slow_raw_preset"]
    asm = (root / next(p for p in PINS if p.endswith("ctor2-build-false.asm"))).read_text()
    instructions = {}
    for line in asm.splitlines():
        m = re.match(r"\s*([0-9a-f]+):\s+(.+)", line)
        if m:
            instructions[int(m[1], 16)] = m[2]
    # Inclusive common append paths. Refill detours are separate, not free.
    ranges = [(0x33a00, 0x33a29), (0x34350, 0x3437a),
              (0x343e0, 0x34406), (0x344d0, 0x344f9)]
    counts = [sum(lo <= a <= hi for a in instructions) for lo, hi in ranges]
    assert counts == [12] * 4, counts
    leaf = (root / next(p for p in PINS if p.endswith("ctor3-finish-false.asm"))).read_text()
    leaf_count = sum(bool(re.match(r"\s*[0-9a-f]+:\s+\S", line)) for line in leaf.splitlines())
    assert leaf_count == 44, leaf_count
    f = c["leaf_fast"]
    slow = c["slow_entries"]
    partial = c["slow_ordinary_partial_same_epoch"]
    gross = (81 - leaf_count) * f
    historical_net = gross - 5 * (slow - partial) - 12 * partial - 4 * c["epochs"]
    # Historical model only: replaying arithmetic does not validate its completeness.
    result = {
        "scope": "archived assembly and aggregate counter arithmetic; no timing or compilation",
        "blocks": c["blocks"], "rows": c["rows"],
        "append_path_instructions": counts,
        "scalar_instructions": 12 * c["rows_scalar"],
        "scalar_explicit_reads": 4 * c["rows_scalar"],
        "scalar_explicit_writes": 2 * c["rows_scalar"],
        "scalar_pointer_rmw": c["rows_scalar"],
        "descriptor_gross_two_lea_target": 2 * c["rows_scalar"],
        "descriptor_conditional_net_at_5m_cap": 2 * c["rows_scalar"] - 5_000_000,
        "v3_leaf_instructions": leaf_count,
        "v3_gross_finish_saving": gross,
        "v3_historical_model_interval": [historical_net - slow, historical_net + slow],
        "v3_historical_read_delta_incomplete": -5*f + c["opens_ordinary"] + 2*slow + 2*partial,
        "v3_historical_write_delta_incomplete": -2*f,
        "v3_removed_push_pop_instructions_already_in_gross": 12*f,
        "sse_refinement_gross_target": 4*f,
        "input_sha256": digests,
    }
    return result


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("evidence_root", type=pathlib.Path)
    args = p.parse_args()
    print(json.dumps(replay(args.evidence_root), indent=2, sort_keys=True))
