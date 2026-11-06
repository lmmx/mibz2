#!/usr/bin/env python3
"""Static endpoint lemma and conservative refill ledger; no constructor executes."""
import argparse
import hashlib
import json
import re
import tarfile
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--archive", type=Path)
    args = parser.parse_args()
    checked = 0
    crossing_old = 0
    for start in range(300):
        for limit in range(start + 1, 361):
            group, slot = divmod(start, 30)
            end = min(limit, 30 * (group + 1))
            # A refill of a nonempty completed portion; no byte construction.
            if end + 30 > limit:
                continue
            assert end % 30 == 0
            assert end == 30 * (group + 1)
            assert end > start
            old_end_byte = 64 * group + 2 * (slot + end - start)
            assert old_end_byte == 64 * group + 60
            assert old_end_byte + 4 == 64 * (group + 1)
            assert old_end_byte + 64 == 64 * (group + 1) + 60
            assert end + 30 <= limit
            checked += 1
            crossing_old += slot != 0

    path = args.repo / "docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm"
    raw = path.read_bytes()
    git_blob = hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest()
    assert git_blob == "ef4a78b854385b56cee4ffef2080e4f97dc44c54", "not the pinned B2 refill listing"
    instructions = []
    for line in raw.decode().splitlines():
        m = re.match(r"\s*([0-9a-f]+):\s+(.+)", line)
        if m:
            instructions.append((int(m[1], 16), m[2]))
    # Select the successful nonempty path reopening an ordinary group; finish
    # body excluded, its call included, old portion may itself have been RAW.
    common = [x for x in instructions if x[0] <= 0x6240f and x[0] not in (0x623d4, 0x623d9, 0x623dd)]
    assert len(common) == 62
    regions = {
        "rank_division_and_multiple": (0x6237f, 0x6239c),
        "end_clamp": (0x6239f, 0x623a5),
        "bitmap_bounds": (0x623a8, 0x623b3),
        "record_address": (0x623b5, 0x623c3),
        "bitmap_test": (0x623c6, 0x623d2),
        "transient_null_reset": (0x62363, 0x6236a),
    }
    # Independently replay the same archive counters as the accompanying
    # structural replay. Read one fixed member; never extract or run archive code.
    archive = args.archive or (args.repo / "docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz")
    archive_bytes = archive.read_bytes()
    archive_sha = hashlib.sha256(archive_bytes).hexdigest()
    assert archive_sha == "ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9"
    with tarfile.open(archive) as bundle:
        member = bundle.extractfile("c30-construction-v2-20261103T222928Z/c30-construction.jsonl")
        assert member is not None
        jsonl = member.read()
    jsonl_sha = hashlib.sha256(jsonl).hexdigest()
    assert jsonl_sha == "f25325eb2339b1a1385d598e8dd5a8a6d5e37c110c207ddcff18bae53f2e172b"
    counters = []
    for line in jsonl.splitlines():
        record = json.loads(line)
        if record.get("kind") == "counters":
            counters.append(record)
    assert len(counters) == 434
    assert sorted(c["block"] for c in counters) == list(range(434))
    assert sum(c["rows"] for c in counters) == 207006515
    blockwise = []
    for c in counters:
        assert c["fin_forced"] == 0
        p = sum(c[k] for k in ("fin_fast", "fin_ordinary", "fin_wide", "fin_crossing", "fin_forced"))
        n = max(0, p - c["runs"] - 256)
        e = max(0, n - c["fin_crossing"] - int(c["rows"] % 30 != 0))
        blockwise.append((c["block"], c["rows"], p, n, e))

    # Archived audit aggregates, checked against replay and kept visibly
    # separate from the endpoint lemma and proposed compiler caps.
    finished = 1795482
    nonempty_refill_lower = 1122673
    crossing_finishes = 41120
    block_ceiling = 434
    aggregate_fallback = nonempty_refill_lower - crossing_finishes - block_ceiling
    assert aggregate_fallback == 1081119
    assert sum(b[2] for b in blockwise) == finished
    assert sum(b[3] for b in blockwise) == nonempty_refill_lower
    assert sum(c["fin_crossing"] for c in counters) == crossing_finishes
    eligible_lower = sum(b[4] for b in blockwise)
    assert eligible_lower == 1084913
    failed_guard_upper = finished - eligible_lower
    result = {
        "classification": "CONDITIONAL; no generated candidate, no net achievement",
        "finite_endpoint_cases": checked,
        "cases_with_nonzero_old_slot": crossing_old,
        "asm_sha256": hashlib.sha256(raw).hexdigest(),
        "asm_git_blob": git_blob,
        "refill_static_instructions": len(instructions),
        "baseline_nonempty_next_ordinary_wrapper": len(common),
        "baseline_wrapper_trace_addresses": [hex(pc) for pc, _ in common],
        "old_portion_class_does_not_select_wrapper_path": ["ordinary", "crossing RAW", "wide"],
        "asm_region_instruction_counts": {
            k: sum(a <= pc <= b for pc, _ in instructions)
            for k, (a, b) in regions.items()
        },
        "archived_aggregate_inputs": {
            "finished_portions": finished,
            "blockwise_nonempty_refill_lower": nonempty_refill_lower,
            "crossing_finishes": crossing_finishes,
            "all_blocks_upper_bound_for_terminal_partial": block_ceiling,
        },
        "eligible_lower": eligible_lower,
        "aggregate_only_eligible_fallback": aggregate_fallback,
        "archive_replay": {
            "archive_sha256": archive_sha,
            "primary_jsonl_sha256": jsonl_sha,
            "counter_records": len(counters),
            "nonzero_eligible_blocks": sum(b[4] > 0 for b in blockwise),
            "large_blocks": sum(b[1] >= 899000 for b in blockwise),
            "large_block_eligible_lower": sum(b[4] for b in blockwise if b[1] >= 899000),
            "tiny_block_eligible_lower": sum(b[4] for b in blockwise if b[1] < 899000),
        },
        "failed_guard_upper": failed_guard_upper,
        "old_audit_full_to_full_lower": 1040391,
        "increase_over_prior_audit_eligible_lower": eligible_lower - 1040391,
        "removed_transient_write_bytes_at_eligible_lower": 24 * eligible_lower,
        "illustrative_compiler_acceptance_caps": {
            "eligible_wrapper_instructions_including_guards": 40,
            "added_instructions_per_failed_guard": 5,
            "all_other_added_work": "H: unresolved, never zero by assumption",
            "net_lower_before_H": (62 - 40) * eligible_lower - 5 * failed_guard_upper,
        },
        "descriptor_qualification_triage": {
            "scalar_rows": 38294883,
            "c10_gross": 2 * 38294883,
            "c10_max_added_nonrow_at_40M_gate": 2 * 38294883 - 40000000,
            "c11_gross": 38294883,
            "c11_required_nonrow_saving_at_40M_gate": 40000000 - 38294883,
            "c10_net_with_5M_added": 2 * 38294883 - 5000000,
            "c11_net_with_5M_added": 38294883 - 5000000,
            "all_scalar_refills_loose_upper_including_post_run_empty": finished + 256 * 434 + 610521,
        },
    }
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
