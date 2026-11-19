"""Compare affine forward returns and marked output with dense Ψ, dense LF and Direct."""
import argparse
from collections import Counter, defaultdict
from hashlib import sha256
from itertools import groupby, product
import json
from pathlib import Path
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "union"))
from direct import Column, Direct


def spell(first, psi, start, length):
    word = bytearray()
    for _ in range(length):
        word.append(first[start])
        start = psi[start]
    return bytes(word)


def geometry(last):
    # Ψ by stable sorting, LF independently by C[b] plus exclusive occurrence rank.
    first = sorted(last)
    psi = sorted(range(len(last)), key=last.__getitem__)
    seen = Counter()
    lf = []
    for b in last:
        lf.append(first.index(b) + seen[b])
        seen[b] += 1
    assert all(lf[y] == x for x, y in enumerate(psi))
    return first, psi, lf


def step_to_set(row, steps, symbols, selected):
    word = bytearray()
    for _ in range(len(steps)):
        word.append(symbols[row])
        row = steps[row]
        if row in selected:
            return row, bytes(word)
    raise AssertionError("started in U but failed to return")


def sections(mask, n):
    result = []
    for chosen, rows in groupby(range(n), key=lambda x: bool(mask & (1 << x))):
        if chosen:
            group = list(rows)
            result.append((group[0], group[-1] + 1))
    return result


def cyclic(text):
    order = sorted(range(len(text)), key=lambda i: text[i:] + text[:i])
    last = bytes(text[i - 1] for i in order)
    first, psi, _ = geometry(last)
    assert spell(first, psi, order.index(0), len(text)) == text
    return last


def case_inputs():
    # One section may arise multiple ways. Deduplicate tests while preserving every label.
    by_column = defaultdict(lambda: defaultdict(set))
    for n in range(1, 7):
        for col in product(range(2), repeat=n):
            last = bytes(col)
            for mask in range(1 << n):
                by_column[last][mask].add("arbitrary_exhaustive")
    texts = {bytes(x) for n in range(1, 8) for x in product(range(2), repeat=n)}
    texts.update((b'banana', b'mississippi', b'ababa', b'aaaa', bytes([0,255,128,0,255,17])))
    cyclic_columns = {cyclic(t) for t in texts}
    additional = {bytes([0,255,0,9,9,255,1,255]), bytes([255,2,0,255,0,128,2])}
    for last in sorted(cyclic_columns | set(by_column) | additional):
        n = len(last)
        first, psi, _ = geometry(last)
        masks = by_column[last]
        for m in ((1 << n) - 1, 0, sum(1 << i for i in range(0,n,2)), 1, 1 << (n-1)):
            masks[m].add("targeted_cut")
        for k in range(1, 5):
            markers = defaultdict(set)
            for i in range(n):
                markers[spell(first, psi, i, k)].add(i)
            for selected in markers.values():
                word_rows = defaultdict(int)
                for source in selected:
                    _, word = step_to_set(source, psi, first, selected)
                    word_rows[word] |= 1 << source
                groups = list(word_rows.values())
                # All nonempty subsets of return types, including the whole cylinder.
                for choices in range(1, 1 << len(groups)):
                    mask = sum(v for i,v in enumerate(groups) if choices & (1 << i))
                    labels = masks[mask]
                    labels.add("selected_return_types")
                    if choices == (1 << len(groups)) - 1:
                        labels.add("whole_marker_cylinder")
                    if any(len(word) < k for word in word_rows):
                        labels.add("overlapping_parent_occurrences")
    return by_column, cyclic_columns


def check_bands(bands, expected):
    """Compare every translated row, including its destination tag. Splits are legal."""
    observed = {}
    prev_end = 0
    for x, y, width, word, color in bands:
        assert width > 0 and x >= prev_end and word
        prev_end = x + width
        for t in range(width):
            observed[x+t] = y+t, word, color
    assert observed == expected, (bands, expected)


def batch(binary, command, requests):
    if binary is None:
        return
    proc = subprocess.run([binary, command], input="".join(s for s, _ in requests),
                          text=True, capture_output=True)
    if proc.returncode:
        raise RuntimeError(f"forward exited {proc.returncode}: {proc.stderr[-1000:]}")
    for row, (dataset, want) in zip(proc.stdout.splitlines(), requests, strict=True):
        try:
            obj = json.loads(row)
            raw = obj["bands"]
            bands = [(x,y,w,bytes.fromhex(word),c) for x,y,w,word,c in raw]
            check_bands(bands, want[0])
            assert obj["expanded_hex"] == want[1]
            for key, value in want[2].items():
                assert obj[key] == value, (key, obj[key], value)
            component_count = int(dataset.split("\n", 1)[0].split()[-1])
            assert obj["bands_count"] == len(bands) <= obj["raw_hits"]
            assert obj["packet_inputs"] == component_count + obj["live_outputs"] - obj["live_merges"]
            assert obj["packet_inputs"] == obj.get("select_page_seeks", 0) + sum(obj[k] for k in (
                "select_binary_searches", "select_finger_retained", "select_finger_next"))
            if command == "INDEXED_BANDS":
                assert obj["select_binary_searches"] == obj["select_binary_comparisons"] == 0
                assert obj["page_capacity_bytes"] >= 4 * ((obj["n"] + 63) // 64)
                assert obj["select_page_run_increments"] <= 63 * obj["select_page_seeks"]
                assert obj["source_directory_fills"] == len(want[0])
                assert obj["source_directory_capacity_bytes"] >= 4 * (len(want[0]) + component_count)
                assert obj["sort_comparisons"] == 0 and obj["bands_count"] == obj["raw_hits"]
            assert obj["component_pieces"] == obj["live_outputs"] + obj["raw_hits"]
        except Exception as exc:
            raise AssertionError(f"binary disagrees on dataset\n{dataset}\nreported {row}") from exc


def check_page_boundaries(binary, command):
    # A singleton section has one live packet at each proper prefix. Each select
    # starts with a fresh finger. Count precisely how many run ends the page
    # lookup must cross, independently of which row was marked for expansion.
    tests = 0
    largest_page = 0
    fingerprints = sha256()
    increments = set()
    for n in (63, 64, 65, 127, 128, 129):
        for last in (bytes(range(n)), bytes(reversed(range(n))),
                     bytes((i * 37 + i // 3) % 13 for i in range(n))):
            first, psi, _ = geometry(last)
            runs = [(b, len(list(it))) for b,it in groupby(last)]
            l_run = [index for index,(_,width) in enumerate(runs) for _ in range(width)]
            select_run_rank = {r:i for i,r in enumerate(sorted(range(len(runs)), key=lambda r: runs[r][0]))}
            select_rank = [select_run_rank[l_run[y]] for y in psi]
            cuts = sorted(set(x for x in (0, 1, 62, 63, 64, 65, 126, 127, 128, n-1) if 0<=x<n))
            requests = []
            for src in cuts:
                _, word = step_to_set(src, psi, first, {src})
                row = src
                steps = 0
                for _ in word:
                    page = row // 64
                    steps += select_rank[row] - select_rank[page*64]
                    largest_page = max(page, largest_page)
                    row = psi[row]
                increments.add(steps)
                for mark in cuts:
                    pos = mark
                    cycle = set()
                    while pos not in cycle:
                        cycle.add(pos)
                        pos = psi[pos]
                    expected = spell(first, psi, mark, n).hex() if src in cycle else None
                    oracle = {"source_directory_fills": 1, "select_page_seeks": len(word),
                              "select_page_run_increments": steps, "select_translations": len(word)}
                    data = f"{n} {mark} {len(runs)} 1\n" + "".join(
                        f"{a} {b}\n" for a,b in [*runs,(src,src+1)])
                    fingerprints.update(data.encode())
                    requests.append((data, ({src:(src,word,0)}, expected, oracle)))
                    tests += 1
            batch(binary, command, requests)
    assert largest_page == 2 and 0 in increments and 63 in increments
    return dict(datasets=tests, n=[63,64,65,127,128,129], largest_zero_based_page=largest_page,
                ordered_datasets_sha256=fingerprints.hexdigest(),
                max_page_run_increments_per_singleton_orbit=max(increments))


def main(binary, command="BANDS"):

    cases, cyclic_columns = case_inputs()
    totals = Counter()
    labels = Counter()
    refusals = Counter()
    witnesses = {}
    requests = []
    fingerprint = sha256()
    for last, masks in sorted(cases.items(), key=lambda item: (len(item[0]), item[0])):
        n = len(last)
        first, psi, lf = geometry(last)
        runs = [(b, len(list(it))) for b,it in groupby(last)]
        col = Column(runs)
        cycles = {}
        for i in range(n):
            if i not in cycles:
                orbit = {i}
                j = psi[i]
                while j != i:
                    orbit.add(j)
                    j = psi[j]
                cycles.update(dict.fromkeys(orbit, frozenset(orbit)))
        totals["distinct_last_columns"] += 1
        totals["columns_with_multiple_cycles"] += len(cycles[0]) != n
        for mask, kinds in sorted(masks.items()):
            comp = sections(mask, n)
            tag = {x:c for c,(a,b) in enumerate(comp) for x in range(a,b)}
            edges = {}
            for x in tag:
                y, word = step_to_set(x, psi, first, tag)
                edges[x] = y, word, tag[y]
            backward = {}
            for y in tag:
                x, reverse_word = step_to_set(y, lf, last, tag)
                backward[x] = y, reverse_word[::-1], tag[y]
            assert backward == edges
            buckets = Counter((w,c) for _,w,c in edges.values())
            totals.update(section_tests=1, components=len(comp), tagged_buckets=len(buckets),
                          untagged_words=len({w for _,w,_ in edges.values()}), first_return_hits=len(tag),
                          fragmented_sections=len(comp)>1, empty_sections=not comp,
                          sections_on_cyclic_column=last in cyclic_columns,
                          repeated_tagged_bucket=any(v>1 for v in buckets.values()),
                          same_word_multiple_colors=len({w for w,_ in buckets})<len(buckets))
            labels.update(kinds)
            try:
                direct = Direct(col, comp)
            except AssertionError as exc:
                # Its uncut LF run width and tagged bucket contiguity assume structured U.
                # Restrict this exception to Direct construction and only allow it on arbitrary U.
                assert "selected_return_types" not in kinds, (last.hex(), comp)
                direct = None
                trace = exc.__traceback__
                while trace.tb_next:
                    trace = trace.tb_next
                where = f"{Path(trace.tb_frame.f_code.co_filename).name}:{trace.tb_lineno}"
                failed = Path(trace.tb_frame.f_code.co_filename).read_text().splitlines()[trace.tb_lineno-1].strip()
                assert failed in {
                    "assert row+width <= self.components[i][1]",
                    "assert i+1 == len(self.starts) or row+width <= self.starts[i+1]",
                }, (where, failed)
                reason = f"{where} {failed}"
                refusals[reason] += 1
                witnesses.setdefault(reason, dict(last_hex=last.hex(), section=comp))
            if direct is not None:
                check_bands(direct.bands, edges)
                totals["direct_checked_sections"] += 1
            words = {w for _,w,_ in edges.values()}
            full_trie = {w[:length] for w in words for length in range(1,len(w)+1)}
            groups = {w[:length] for w in words for length in range(len(w))}
            covered_rows = sum(len(w) for _,w,_ in edges.values())
            assert covered_rows == len(set().union(*(cycles[x] for x in tag)))
            expected_stats = dict(n=n, covered_rows=covered_rows, source_rows=len(tag), groups=len(groups),
                                  trie_nodes=1+len(full_trie), word_count=len(words),
                                  spelling_bytes=sum(map(len,words)), max_depth=max(map(len,words), default=0))
            for mark in range(n):
                hit = not cycles[mark].isdisjoint(tag)
                want = spell(first, psi, mark, n) if hit else None
                if direct is not None:
                    assert direct.expand(mark) == want
                totals.update(marked_tests=1, marked_hits=hit, disjoint_marks=not hit,
                              nondividing_period_hits=hit and n%len(cycles[mark])!=0)
                data = f"{n} {mark} {len(runs)} {len(comp)}\n" + "".join(
                    f"{a} {b}\n" for a,b in [*runs,*comp])
                fingerprint.update(data.encode())
                if binary:
                    before = 0
                    row = mark
                    if tag:
                        while row not in tag and before < len(cycles[mark]):
                            before += 1
                            row = psi[row]
                    remaining = n-before
                    lookups = 0
                    while hit and remaining > 0:
                        row, word, _ = edges[row]
                        remaining -= len(word)
                        lookups += 1
                    stats = dict(expected_stats, mark_seeks=before, mark_prefix=before if hit else 0,
                                 output_lookups=lookups)
                    requests.append((data, (edges, None if want is None else want.hex(), stats)))
                    if len(requests) == 256:
                        batch(binary, command, requests)
                        requests.clear()
    if requests:
        batch(binary, command, requests)
    assert all(totals[k] for k in ("fragmented_sections","disjoint_marks","same_word_multiple_colors",
                                    "columns_with_multiple_cycles","nondividing_period_hits"))
    assert labels["overlapping_parent_occurrences"] and refusals
    report = dict(coverage={"exhaustive_arbitrary_binary_last_columns_and_sections_through_n": 6,
                          "all_binary_cyclic_texts_through_n": 7,
                          "structured_marker_lengths": [1, 2, 3, 4],
                          "targeted_texts_hex": [x.hex() for x in (b"banana", b"mississippi", b"ababa", b"aaaa", bytes([0,255,128,0,255,17]))],
                          "all_primary_indices_tested": True},
                mode="validate forward" if binary else "dense and Direct oracles only",
                binary_sha256=sha256(binary.read_bytes()).hexdigest() if binary else None,
                native_checks=([
                    "every source row's first-return destination, word and destination color",
                    "marked expansion or disjoint-cycle fallback",
                    "exact covered mass, proper-prefix groups, full trie and distinct spellings",
                    "initial mark-to-section distance and number of output word lookups",
                    "packet conservation, select seeks and terminal/live accounting",
                ] if binary else []),
                ordered_datasets_sha256=fingerprint.hexdigest(), counts=totals,
                section_labels=labels, direct_outside_structured_domain=refusals, witnesses=witnesses)

    if command == "INDEXED_BANDS":
        report["binary_command"] = command
        report["indexed_checks"] = [
            "source table fills equal U size in every dataset, with U and component directories charged",
            "each discovered packet charged one page or finger seek; zero binary searches or order comparisons",
            "raw terminal spans retained in the source-rank directory",
            "source table allocated at setup and filled in the order phase, as recorded in forward.cpp",
            "exact page-run increments on singleton-section orbits across three pages",
        ]
        report["supplemental_page_boundaries"] = check_page_boundaries(binary, command)
    return report


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("binary", nargs="?", type=lambda x: Path(x).resolve())
    p.add_argument("command", nargs="?", choices=["BANDS", "INDEXED_BANDS"], default="BANDS")
    args = p.parse_args()
    print(json.dumps(main(args.binary, args.command), indent=2, sort_keys=True))
