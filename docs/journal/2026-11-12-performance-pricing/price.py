"""Price saved counters and fixed schedules; never import or execute an LF kernel."""

import argparse
import gzip
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).parent


def sha(data):
    """Bind calculations to archived input bytes."""
    return hashlib.sha256(data).hexdigest()


def inputs():
    """Verify and read original snapshots, including byte-preserved compressed cases."""
    manifest = json.loads((ROOT / "INPUTS.json").read_text())
    data = {}
    for name, item in manifest["data"].items():
        saved = (ROOT / item["snapshot"]).read_bytes()
        assert sha(saved) == item["snapshot_sha256"]
        raw = saved if item["snapshot_is_original_gzip"] else gzip.decompress(saved)
        assert len(raw) == item["bytes"] and sha(raw) == item["sha256"]
        data[name] = gzip.decompress(raw) if item["snapshot_is_original_gzip"] else raw
    assert manifest["data"]["370-results"]["sha256"] == (
        "1ba3a243ad2a04568bb0bd18ded8d72c39683a797cc066d91f8f1378a6579f76"
    )
    assert manifest["data"]["370-schedules"]["sha256"] == (
        "00ad44509c9c10519dc83d6e218153752bcf3acd7f26135171cfe17eb35f398f"
    )
    return data


def add_counts(target, values):
    """Preserve the saved distinction between cumulative counts and maxima."""
    for key, value in values.items():
        if key.startswith("peak_"):
            maxima = target.setdefault("maxima", {})
            maxima[key] = max(maxima.get(key, 0), value)
        else:
            sums = target.setdefault("sums", {})
            sums[key] = sums.get(key, 0) + value


def aggregate(cases, kind, variant):
    """Aggregate observations without mixing incomparable count categories."""
    result = {}
    for case in cases:
        if case["kind"] == kind:
            add_counts(result, case[variant])
    return result


def contractions(cases, schedules):
    """Join original schedules; infer raw record counts from instrumented identities."""
    families, all_counts, per_case = {}, Counter(), []
    records = [case for case in cases if case["kind"] == "contraction"]
    assert len(records) == len(schedules)
    for case, schedule in zip(records, schedules):
        assert case["case"] == len(per_case)
        assert (case["column"], case["origin"], case["family"]) == (
            schedule["column"],
            schedule["origin"],
            schedule["family"],
        )
        p, s, row = case["packed"], case["stable"], Counter(cases=1)
        for record in schedule["rounds"]:
            before = sum(hi - lo for lo, hi in record["kept_before"])
            removed = record["deleted_packed"]
            amount = sum(hi - lo for lo, hi in removed)
            assert amount == sum(hi - lo for lo, hi in record["deleted_original"])
            assert all(0 <= lo < hi <= before for lo, hi in removed)
            assert all(a[1] < b[0] for a, b in zip(removed, removed[1:]))
            row["rounds"] += 1
            row["rows_exposed_across_rounds"] += before
            row["deleted_rows"] += amount
            row["deletion_intervals"] += len(removed)
            if removed:
                row["active_rounds"] += 1
                row["active_rows_before"] += before
                row["active_rows_after"] += before - amount
                # Every surviving packed row after the first hole has a changed rank.
                # This is not the number of changed interval/chunk records.
                row["survivor_rows_with_changed_rank"] += before - amount - removed[0][0]
                row["unchanged_prefix_rows"] += removed[0][0]
            else:
                row["empty_rounds"] += 1
        active = row["active_rounds"]
        raw2 = p.get("rank_queries", 0) - active
        assert raw2 >= 0 and raw2 % 2 == 0
        raw = raw2 // 2
        assert raw == p.get("coalescing_checks", 0)
        assert p["persistent_piece_records_created"] == (
            p["input_pieces"] + raw + p.get("merges", 0)
        )
        assert p["coordinate_fields_written"] == 2 * p["persistent_piece_records_created"]
        assert p["selection_input_visits"] == s["selection_input_visits"]
        assert p["selection_input_visits"] % 2 == 0
        assert p["overlay_records_created"] == s["overlay_records_created"]
        assert p["output_bytes"] == s["output_bytes"]
        assert p["final_piece_records"] == s["final_edge_records"]
        assert p.get("grammar_nodes_created", 0) == s.get("grammar_nodes_created", 0)
        row["input_piece_records"] = p["input_pieces"]
        row["selection_piece_rounds"] = p["selection_input_visits"] // 2
        row["raw_records_after_holes"] = raw
        row["canonical_records_after_active_rounds"] = raw - p.get("merges", 0)
        row["packed_piece_field_slots_created"] = 4 * p["persistent_piece_records_created"]
        row["treap_field_slots_created"] = (
            5 * s["persistent_edge_records_created"] + 6 * s["persistent_tree_nodes_created"]
        )
        row["one_two_order_output_pass_entries"] = 2 * row["canonical_records_after_active_rounds"]
        row["two_coordinate_fields_per_raw_record"] = 2 * raw
        row["treap_pointer_writes"] = s.get("tree_pointer_writes", 0)
        row["treap_size_writes"] = s.get("tree_size_writes", 0)
        if active == 0:
            row["entirely_inactive_cases"] = 1
            row["inactive_case_selection_visits"] = p["selection_input_visits"]
            row["inactive_case_overlay_records"] = p["overlay_records_created"]
        families.setdefault(case["family"], Counter()).update(row)
        all_counts.update(row)
        per_case.append(dict(case=case["case"], family=case["family"], **row))
    assert all_counts["rounds"] == 12429 and all_counts["deleted_rows"] == 76751
    return dict(total=all_counts, families=families, per_case=per_case)


def origins(cases):
    """Bound a stop-after-discovery check; discovery index was not recorded."""
    groups = {}
    for case in cases:
        if case["kind"] != "origin":
            continue
        sc, mc = case["search"], case["marked"]
        column = bytes.fromhex(case["column"])
        if sc.get("no_delimiter"):
            group = "absent_delimiter"
        elif sc.get("unanchored"):
            group = "unanchored_cycle"
        elif column[case["origin"]] == case["delimiter"]:
            assert sc.get("origin_lf_steps", 0) == 0
            group = "already_anchor"
        else:
            assert case["accepted"] and sc["origin_lf_steps"] > 0
            group = "successful_nonanchor"
        c = groups.setdefault(group, Counter())
        c.update(
            cases=1,
            input_rows=len(column),
            search_steps=sc.get("origin_lf_steps", 0),
            search_queries=sc.get("origin_search_queries", 0),
            search_comparisons=sc.get("origin_search_comparisons", 0),
            baseline_constructor_steps=sc.get("constructor_steps", 0),
            marked_constructor_steps=mc.get("constructor_steps", 0),
            old_marker_checks=mc.get("marker_checks", 0),
            old_marker_comparisons=mc.get("marker_comparisons", 0),
            extra_grammar_nodes=mc.get("grammar_nodes_created", 0)
            - sc.get("grammar_nodes_created", 0),
        )
        if group == "successful_nonanchor":
            c["stop_after_discovery_checks_lower_bound"] += 1
            c["stop_after_discovery_checks_upper_bound"] += mc["constructor_steps"]
            c["maximum_checks_saved_by_early_stop"] += mc["constructor_steps"] - 1
    return groups


def real_block(result):
    """Model explicit compact layouts; never substitute them for actual Python bytes."""
    c = result["counts"]
    n, r, h = result["n"], c["input_maximal_runs"], c["H_live"]
    a, completed, frontier = (
        c["anchor_intervals"],
        c["completed_records_created"],
        c["frontier_peak"],
    )
    capacity = 1
    while capacity * 3 // 4 < h:
        capacity *= 2
    layout = dict(
        fixed_source_pieces_4xu32=16 * r,
        raw_capped_records_u16=c["input_record_bytes"],
        redundant_source_start_index_u32=4 * r,
        anchor_id_start_prefix_u32=12 * a + 4,
        grammar_child_child_length_3xu32=12 * h,
        hash_slots_child_child_id_3xu32=12 * capacity,
        completed_4xu32=16 * completed,
        frontier_4xu32_peak=16 * frontier,
    )
    conservative = sum(layout.values())
    lifetime_trim = (
        conservative - layout["raw_capped_records_u16"] - layout["redundant_source_start_index_u32"]
    )
    # Conditional lower bound at the last ordinary doubling, BEFORE inserting a new node.
    # Old/new tables coexist; fixed source map remains necessary during construction.
    old_cap = capacity // 2
    growth_h = old_cap * 3 // 4
    growth_lower = 16 * r + (12 * a + 4) + 12 * growth_h + 12 * (capacity + old_cap)
    dictionary_unique_visits = 2 * c["dictionary_bytes"] - c["dictionary_words"]
    assert (
        c["dictionary_grammar_visits"]
        == 2 * c["dictionary_leaf_appends"] - c["dictionary_expand_calls"]
    )
    return dict(
        recorded_outcome=result["outcome"],
        recorded_traversal_sum=result["traversal_sum"],
        recorded_metadata_lower_bound=result["metadata_peak_lower_bound_bytes"],
        threshold=4 * n,
        compact_layout_assumptions="u32 fields, exact-fit buffers, 75%-load open-addressed hash",
        compact_components=layout,
        hash_capacity=capacity,
        compact_maxima_sum_envelope=conservative,
        trimmed_maxima_sum_envelope=lifetime_trim,
        trimmed_envelope_headroom=4 * n - lifetime_trim,
        ordinary_hash_doubling_live_lower_bound=growth_lower,
        growth_omits="queue/completed buffers, alignment, allocator, grammar growth, all output",
        grammar_growth_node_count=growth_h,
        dictionary_unique_expansion_visits=dictionary_unique_visits,
        dictionary_redundant_grammar_visits=c["dictionary_grammar_visits"]
        - dictionary_unique_visits,
        dictionary_redundant_bytes_per_copy_phase=c["dictionary_leaf_appends"]
        - c["dictionary_bytes"],
        dictionary_membership_checks_added=c["dictionary_expand_calls"],
        residual_grammar_visits=c["residual_grammar_visits"],
        residual_word_lookups_if_dictionary_reused=c["residual_token_visits"],
        dictionary_oriented_writer_optimistic_byte_writes_removed=(
            c["residual_leaf_appends"]
            + c["residual_copy_bytes"]
            + c["cycle_reverse_copy_bytes"]
            - n
        ),
        writer_bound_conditions="one cycle n, already anchored; excludes zero-init, routing, arbitrary-origin rotation and short-cycle completion",
    )


def routes(data, expected):
    """Recheck actual row weights; no time fraction is inferred from them."""
    counts, total = {}, Counter()
    for line in data.splitlines():
        row = json.loads(line)
        if row["kind"] != "block" or row["arm"] != "cells":
            continue
        d = row["data"]
        c = counts.setdefault(d["route"], Counter())
        values = dict(blocks=1, rows=d["rows"], output_bytes=d["bytes"])
        c.update(values)
        total.update(values)
        assert d["decode_ns"] is None and d["inversion_ns"] is None
    assert counts == expected["routes"]
    assert total["rows"] == expected["rows"]
    return dict(
        counts=counts,
        total=total,
        admitted_row_share=counts["admitted"]["rows"] / total["rows"],
        budget_refusal_row_share=counts["budget"]["rows"] / total["rows"],
        critical_path_time_share=None,
        historical_writer_ns_per_row=[
            q["admitted_research_ns_per_row"]["c_write"] for q in expected["historical_repeats"]
        ],
        historical_induction_ns_per_row=[
            q["admitted_research_ns_per_row"]["c_induce"] for q in expected["historical_repeats"]
        ],
    )


def calculate():
    """Validate saved aggregation and emit derived counts and conditional bounds."""
    data = inputs()
    old = json.loads(data["370-results"])
    cases = [json.loads(line) for line in data["370-cases"].splitlines() if line]
    rebuilt = {}
    for case in cases:
        kind = case["kind"]
        variants = ("search", "marked") if kind == "origin" else ("packed", "stable")
        for variant in variants:
            group = (
                rebuilt.setdefault(kind, {}).setdefault(case["family"], {}).setdefault(variant, {})
            )
            add_counts(group, case[variant])
    assert rebuilt["origin"] == old["origin"] and rebuilt["contraction"] == old["contraction"]
    result = dict(
        kind="saved-evidence pricing; no new LF executions or timing",
        input_manifest_sha256=sha((ROOT / "INPUTS.json").read_bytes()),
        source_sha256=sha(Path(__file__).read_bytes()),
        contraction=contractions(cases, json.loads(data["370-schedules"])["cases"]),
        origin=origins(cases),
        recorded_costs={
            variant: aggregate(cases, "contraction", variant) for variant in ("packed", "stable")
        },
        real_block=real_block(json.loads(data["371-result"])),
        actual_routes=routes(data["367-census"], json.loads(data["367-analysis"])),
    )
    assert sum(g["cases"] for g in result["origin"].values()) == old["origin_checks"]["cases"]
    return result


def main():
    """Write a new evidence file or verify an existing one without overwriting it."""
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    result = calculate()
    if args.check:
        assert json.loads(args.output.read_text()) == result
        print("PASS: pinned snapshots, original aggregates, schedule joins and derived pricing")
    else:
        with args.output.open("x") as stream:
            stream.write(json.dumps(result, indent=2, sort_keys=True) + "\n")
        print(
            json.dumps(
                {k: v for k, v in result.items() if k not in ("contraction", "recorded_costs")},
                indent=2,
            )
        )


if __name__ == "__main__":
    main()
