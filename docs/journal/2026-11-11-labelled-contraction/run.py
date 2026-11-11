"""Execute the fixed comparisons, preserve failures and emit deterministic evidence."""

import gzip
import hashlib
import json
import sys
import time
import traceback
from collections import Counter
from pathlib import Path

from common import bisect_right, decode_oracle, initial, inputs, oracle, original, phrase, unranges
from contraction import Stable, packed
from freeze import native_contract
from origin import construct

ROOT = Path(__file__).parent
CONTEXT = {}


def digest(path):
    """Identify exact source and frozen inputs."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def accumulate(target, counts):
    """Keep summed work separate from maximum live/transient records."""
    for key, value in counts.items():
        if key.startswith("peak_"):
            target.setdefault("maxima", {})[key] = max(target.get("maxima", {}).get(key, 0), value)
        else:
            sums = target.setdefault("sums", {})
            sums[key] = sums.get(key, 0) + value


def row_check(column, keep, pieces, grammar):
    """Check every return destination, length and byte itinerary independently."""
    f, _ = oracle(column)
    retained = set(keep)
    count = 0
    for a in pieces:
        word = original.Grammar.expand(grammar, a.w)
        for j in range(a.n):
            x = keep[a.s + j]
            y, expected = x, bytearray()
            while True:
                expected.append(column[y])
                y = f[y]
                count += 1
                if y in retained:
                    break
                assert len(expected) < len(column)
            assert keep[a.t + j] == y and word == expected, (x, a, y, word, expected)
    assert sum(a.n for a in pieces) == len(keep)
    return count


def packed_decode(pieces, grammar, marker, n, c):
    """Count final output traversal of the packed representation."""
    starts = [a.s for a in pieces]
    c["output_index_entries"] += len(starts)
    q, emitted = marker, bytearray()
    while True:
        c["output_piece_visits"] += 1
        a = pieces[bisect_right(starts, q, c, "output_search") - 1]
        emitted.extend(grammar.expand(a.w))
        q = a.t + q - a.s
        if q == marker:
            break
        assert len(emitted) <= n
    reverse = bytes(reversed(emitted))
    repeat = (n + len(reverse) - 1) // len(reverse)
    c["cycle_reversal_bytes"] += len(reverse)
    c["repeated_bytes"] += repeat * len(reverse)
    c["output_bytes"] += n
    return (reverse * repeat)[:n]


def check_origins(result, events, deadline):
    """Compare both phrase variants with the pinned original and a row oracle."""
    totals = Counter()
    for name, column, alphabet in inputs():
        for origin in range(len(column)):
            for delimiter in alphabet:
                assert time.monotonic() < deadline, "candidate execution budget exceeded"
                CONTEXT.update(
                    stage="origin",
                    family=name,
                    column=column.hex(),
                    origin=origin,
                    delimiter=delimiter,
                )
                expected = phrase.induce_on_byte(column, origin, delimiter)
                search, sc, sd = construct(column, origin, delimiter, "search")
                mark, mc, md = construct(column, origin, delimiter, "mark")
                assert search == mark
                assert (expected is None) == (mark is None)
                totals["cases"] += 1
                if expected is not None:
                    assert sd[0] == expected
                    assert {k: v for k, v in md[0].items() if k != "origin_prefix_steps"} == {
                        k: v for k, v in expected.items() if k != "origin_prefix_steps"
                    }
                    assert sd[1] == md[1] and sd[0]["token_lf"] and md[0]["token_lf"]
                    anchor_rows = [x for x, byte in enumerate(column) if byte == delimiter]
                    root, phase = md[2]
                    x = anchor_rows[root]
                    f, _ = oracle(column)
                    for _ in range(phase):
                        x = f[x]
                    assert x == origin
                    # Check actual returns, not only the final reconstructed bytes.
                    for source, target, width, hexword in md[1]:
                        for offset in range(width):
                            x, emitted = anchor_rows[source + offset], bytearray()
                            while True:
                                emitted.append(column[x])
                                x = f[x]
                                if column[x] == delimiter:
                                    break
                            assert x == anchor_rows[target + offset] and emitted.hex() == hexword
                    totals["accepted"] += 1
                else:
                    totals["declined"] += 1
                    totals["declined_extra_constructor_steps"] += (
                        mc["constructor_steps"] - sc["constructor_steps"]
                    )
                group = result.setdefault(name, dict(search={}, marked={}))
                accumulate(group["search"], sc)
                accumulate(group["marked"], mc)
                events.append(
                    dict(
                        kind="origin",
                        family=name,
                        column=column.hex(),
                        origin=origin,
                        delimiter=delimiter,
                        accepted=mark is not None,
                        search=dict(sc),
                        marked=dict(mc),
                    )
                )
    return dict(totals)


def check_contractions(result, events, deadline):
    """Use immutable baseline schedules for both representations and every round."""
    cases = json.loads((ROOT / "schedules.json").read_text())["cases"]
    totals = Counter()
    for case_id, fixture in enumerate(cases):
        assert time.monotonic() < deadline, "candidate execution budget exceeded"
        col, origin, special = (
            bytes.fromhex(fixture["column"]),
            fixture["origin"],
            fixture["special"],
        )
        CONTEXT.clear()
        CONTEXT.update(stage="contraction", case=case_id, fixture=fixture)
        pc, sc = Counter(), Counter()
        pp, pg, keep = initial(col, special, pc)
        sp, sg, keep2 = initial(col, special, sc)
        np, ng, _ = initial(col, special)
        assert keep == keep2 == unranges(fixture["initial_keep"])
        pm = nm = keep.index(origin)
        stable = Stable(sp, sg, pm, sc)
        pc["persistent_piece_records_created"] += len(pp)
        pc["coordinate_fields_written"] += 2 * len(pp)
        pc["peak_live_piece_records"] = len(pp)
        pc["input_rows"] = sc["input_rows"] = len(col)
        pc["input_pieces"] = sc["input_pieces"] = len(pp)
        for round_id, record in enumerate(fixture["rounds"]):
            CONTEXT["round"] = round_id
            seed = record["seed"]
            assert keep == unranges(record["kept_before"])
            np, nm, native_stats = native_contract(np, ng, nm, seed)
            pp, pm, pd = packed(pp, pg, pm, seed, pc)
            sd = stable.contract(seed)
            assert [list(x) for x in pd] == record["deleted_packed"]
            assert pd == sd
            gone = set(unranges(record["deleted_original"]))
            assert native_stats["removed"] == len(gone)
            keep = [x for x in keep if x not in gone]
            exported, sm = stable.export()
            assert exported == pp == np, (exported, pp, np)
            assert sm == pm == nm and keep[sm] == origin
            assert sg.nodes == pg.nodes == ng.nodes
            totals["oracle_lf_visits"] += row_check(col, keep, pp, pg)
            totals["rounds"] += 1
            totals["deleted_rows"] += len(gone)
        expected = decode_oracle(col, origin)
        assert packed_decode(pp, pg, pm, len(col), pc) == expected
        assert stable.decode(len(col)) == expected
        pc["final_piece_records"] = len(pp)
        sc["final_edge_records"] = len(stable.edges)
        sc["final_tree_nodes"] = 2 * len(stable.edges)
        pc["final_grammar_nodes"] = len(pg.nodes)
        sc["final_grammar_nodes"] = len(sg.nodes)
        totals["cases"] += 1
        totals["equal_canonical_maps"] += 1
        family = fixture["family"]
        group = result.setdefault(family, dict(packed={}, stable={}))
        accumulate(group["packed"], pc)
        accumulate(group["stable"], sc)
        events.append(
            dict(
                kind="contraction",
                case=case_id,
                family=family,
                column=col.hex(),
                origin=origin,
                packed=dict(pc),
                stable=dict(sc),
            )
        )
    return dict(totals)


def main():
    """Run once per requested output directory; never overwrite failed evidence."""
    if not __debug__:
        raise RuntimeError("Assertions must be enabled")
    dest = Path(sys.argv[1])
    dest.mkdir(parents=True, exist_ok=False)
    deadline = time.monotonic() + 14 * 60  # Two executions remain below 30 min total.
    events, result = [], dict(schema=1, origin={}, contraction={})
    try:
        result["origin_checks"] = check_origins(result["origin"], events, deadline)
        result["contraction_checks"] = check_contractions(result["contraction"], events, deadline)
        result["source_sha256"] = {
            p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(ROOT.rglob("*.py"))
        }
        result["schedules_sha256"] = digest(ROOT / "schedules.json")
        (dest / "results.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
        payload = "".join(
            json.dumps(e, separators=(",", ":"), sort_keys=True) + "\n" for e in events
        )
        (dest / "case-costs.jsonl.gz").write_bytes(gzip.compress(payload.encode(), mtime=0))
        print(
            json.dumps(
                dict(origin=result["origin_checks"], contraction=result["contraction_checks"])
            )
        )
    except BaseException:
        (dest / "failure.json").write_text(
            json.dumps(dict(context=CONTEXT, traceback=traceback.format_exc()), indent=2) + "\n"
        )
        raise


if __name__ == "__main__":
    main()
