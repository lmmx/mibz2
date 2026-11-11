"""One run-fed delimiter probe; executed only by the frozen, limited supervisor."""

import gc
import hashlib
import json
import resource
import sys
from array import array
from collections import Counter, deque
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent / "vendor"))
from batch_induction import Grammar as OriginalGrammar
from batch_induction import Piece

C = Counter()
SAMPLES = {}
COPY_LENGTHS = Counter()
RESERVE = bytearray(2 * 1024 * 1024)  # Reporting reserve, charged to process ceiling.


def progress(stage):
    """Preserve partial counters even if the one attempt exhausts resources."""
    print(json.dumps(dict(stage=stage, counts=dict(C), metadata_live_samples=SAMPLES)), flush=True)


def allocated(obj):
    """Charge logical record storage cumulatively, conservatively including fields."""
    size = sys.getsizeof(obj)
    if isinstance(obj, tuple):
        size += sum(sys.getsizeof(x) for x in obj)
    elif isinstance(obj, Piece):
        size += sys.getsizeof(vars(obj)) + sum(sys.getsizeof(x) for x in vars(obj).values())
    C["cumulative_record_field_charge_bytes"] += size
    return obj


def copied(phase, length):
    """Record contiguous byte-copy calls, lengths and total writes by phase."""
    C[phase + "_copy_calls"] += 1
    C[phase + "_copy_bytes"] += length
    COPY_LENGTHS[length] += 1


def append_record(container, value):
    """Charge container growth as well as the separately charged record."""
    before = sys.getsizeof(container)
    container.append(value)
    after = sys.getsizeof(container)
    if after > before:
        C["container_growth_events"] += 1
        C["container_reallocation_charge_bytes"] += after


def sample(stage, roots):
    """Measure a live object-graph lower bound, excluding instrumentation counters.

    Sampling is not an exact peak. A sample above 4n proves failure; a sample
    below it cannot prove success. The temporary audit set/stack and reserve
    still count toward the enforced process ceiling, never toward oracle work.
    """
    seen, stack, total = {id(C)}, list(roots), 0
    while stack:
        obj = stack.pop()
        identity = id(obj)
        if identity in seen:
            continue
        seen.add(identity)
        total += sys.getsizeof(obj)
        if isinstance(obj, dict):
            stack.extend(obj.keys())
            stack.extend(obj.values())
        elif isinstance(obj, (list, tuple, deque)):
            stack.extend(obj)
        elif hasattr(obj, "__dict__"):
            stack.append(vars(obj))
    SAMPLES[stage] = total
    C["metadata_audit_object_visits"] += len(seen) - 1
    progress(stage)


def bisect_right(values, x, category="search"):
    """Count every actual binary-search comparison."""
    C[category + "_queries"] += 1
    lo, hi = 0, len(values)
    while lo < hi:
        mid = (lo + hi) // 2
        C[category + "_comparisons"] += 1
        if x < values[mid]:
            hi = mid
        else:
            lo = mid + 1
    return lo


class SortKey:
    """Count ordering comparisons and compared dictionary bytes."""

    __slots__ = ("value",)

    def __init__(self, value):
        """Retain the native sort value and charge its wrapper."""
        self.value = value
        C["sort_key_records"] += 1
        C["cumulative_record_field_charge_bytes"] += sys.getsizeof(self)

    def __lt__(self, other):
        """Compare in the original order, counting inspected bytes."""
        C["sort_comparisons"] += 1
        if isinstance(self.value, bytes):
            for a, b in zip(self.value, other.value):
                C["dictionary_byte_comparisons"] += 1
                if a != b:
                    return a < b
            C["dictionary_length_comparisons"] += 1
            return len(self.value) < len(other.value)
        return self.value < other.value


def ordered(items, key):
    """Charge sort items and output vector allocation."""
    C["sorts"] += 1
    C["sort_items"] += len(items)
    return allocated(sorted(items, key=lambda x: SortKey(key(x))))


class Grammar(OriginalGrammar):
    """Unchanged append/hash-cons grammar with explicit allocation/expansion counts."""

    def cat(self, a, b):
        """Intern exactly as the original constructor, without changing IDs."""
        C["word_pair_queries"] += 1
        before = len(self.nodes)
        sizes = [sys.getsizeof(x) for x in (self.nodes, self.length, self.ids)]
        word = super().cat(a, b)
        if len(self.nodes) != before:
            C["H_created"] += 1
            allocated(self.nodes[word])
            C["cumulative_record_field_charge_bytes"] += sys.getsizeof(word) + sys.getsizeof(
                self.length[word]
            )
            for old, value in zip(sizes, (self.nodes, self.length, self.ids)):
                if sys.getsizeof(value) > old:
                    C["container_reallocation_charge_bytes"] += sys.getsizeof(value)
        C["H_live"] = len(self.nodes) - 256
        return word

    def expand(self, word, phase):
        """Expand the actual word DAG, charging stack visits and byte copies."""
        C[phase + "_expand_calls"] += 1
        stack, output = [word], bytearray()
        while stack:
            C[phase + "_grammar_visits"] += 1
            value = self.nodes[stack.pop()]
            if isinstance(value, int):
                output.append(value)
                C[phase + "_leaf_appends"] += 1
            else:
                stack.extend((value[1], value[0]))
            C["grammar_expansion_stack_peak"] = max(C["grammar_expansion_stack_peak"], len(stack))
        copied(phase, len(output))
        return bytes(output)


def from_records(path, n):
    """Coalesce the actual ordered 127-capped records, without constructing L."""
    raw = array("H")
    with path.open("rb") as stream:
        raw.fromfile(stream, path.stat().st_size // 2)
    if sys.byteorder != "little":
        raw.byteswap()
    counts, logical = [0] * 256, []
    start = 0
    for record in raw:
        byte, length = record >> 8, record & 255
        assert 1 <= length <= 127
        C["run_record_visits"] += 1
        counts[byte] += length
        if logical and logical[-1][2] == byte:
            old = logical[-1]
            logical[-1] = allocated((old[0], old[1] + length, byte))
            C["input_record_coalesces"] += 1
        else:
            append_record(logical, allocated((start, length, byte)))
        start += length
    assert start == n
    cursor, total = [0] * 256, 0
    for byte, count in enumerate(counts):
        cursor[byte] = total
        total += count
        C["byte_bucket_prefix_steps"] += 1
    pieces = []
    for source, width, byte in logical:
        append_record(pieces, allocated(Piece(source, cursor[byte], width, byte)))
        cursor[byte] += width
        C["run_conversion_visits"] += 1
    C["input_records"] = len(raw)
    C["input_maximal_runs"] = len(pieces)
    C["input_record_bytes"] = len(raw) * raw.itemsize
    sample("run_conversion", (raw, counts, logical, cursor, pieces))
    return raw, pieces


def unrle_and_crc(data):
    """Count actual RLE1/output work; CRC uses bzip2's non-reflected polynomial."""
    output, i = bytearray(), 0
    while i < len(data):
        C["rle1_decisions"] += 1
        byte = data[i]
        if (
            i + 3 < len(data)
            and data[i + 1] == byte
            and data[i + 2] == byte
            and data[i + 3] == byte
        ):
            assert i + 4 < len(data)
            length = 4 + data[i + 4]
            output.extend(bytes([byte]) * length)
            copied("rle1_run", length)
            C["rle1_run_materialized_bytes"] += length
            i += 5
        else:
            output.append(byte)
            C["rle1_literal_appends"] += 1
            i += 1
    table = []
    for byte in range(256):
        value = byte << 24
        for _ in range(8):
            value = ((value << 1) ^ (0x04C11DB7 if value & 0x80000000 else 0)) & 0xFFFFFFFF
        table.append(value)
    C["crc_table_steps"] = 256 * 8
    crc = 0xFFFFFFFF
    for byte in output:
        crc = ((crc << 8) & 0xFFFFFFFF) ^ table[(crc >> 24) ^ byte]
        C["crc_byte_updates"] += 1
    return output, crc ^ 0xFFFFFFFF


def evaluate(export, meta):
    """Original phrase semantics with run-fed input and charged validation/output."""
    n, origin, delimiter = meta["n"], meta["primary"], meta["delimiter"]
    raw, pieces = from_records(export / "runs.u16le", n)
    grammar = Grammar()
    starts = [a.s for a in pieces]
    anchors = [a for a in pieces if a.w == delimiter]
    C["anchor_scan_visits"] = len(pieces)
    if not anchors:
        return dict(outcome="decline_no_anchor", complete=False)
    ast, acc = [a.s for a in anchors], [0]
    for a in anchors:
        acc.append(acc[-1] + a.n)
    C["anchor_rows"], C["anchor_intervals"] = acc[-1], len(anchors)

    def rank(x):
        k = bisect_right(ast, x, "rank") - 1
        assert k >= 0 and x < anchors[k].s + anchors[k].n
        return acc[k] + x - anchors[k].s

    point, shift = origin, 0
    while True:
        a = pieces[bisect_right(starts, point, "origin_search") - 1]
        C["origin_search_piece_visits"] += 1
        if a.w == delimiter:
            break
        point = a.t + point - a.s
        shift += 1
        C["origin_search_steps"] += 1
        if point == origin:
            return dict(outcome="decline_no_anchor", complete=False)
    root = rank(point)
    queue = deque(allocated((acc[k], a.s, a.n, None)) for k, a in enumerate(anchors))
    C["queue_records_created"] = len(queue)
    C["frontier_peak"] = len(queue)
    C["pending_completed_peak_records"] = len(queue)
    completed = []
    pending_rows, done_rows = acc[-1], 0
    while queue:
        source, x, width, word = queue.popleft()
        pending_rows -= width
        a = pieces[bisect_right(starts, x) - 1]
        assert x + width <= a.s + a.n
        C["E"] += 1
        C["weighted_rows"] += width
        assert C["E"] <= C["weighted_rows"] <= n
        word = a.w if word is None else grammar.cat(word, a.w)
        y = a.t + x - a.s
        end, j = y + width, bisect_right(starts, y) - 1
        chunks = 0
        while y < end:
            b = pieces[j]
            size = min(end, b.s + b.n) - y
            s = source + y - (a.t + x - a.s)
            C["boundary_touches"] += 1
            chunks += 1
            if b.w == delimiter:
                append_record(completed, allocated(Piece(s, rank(y), size, word)))
                C["completed_records_created"] += 1
                done_rows += size
            else:
                append_record(queue, allocated((s, y, size, word)))
                C["queue_records_created"] += 1
                pending_rows += size
            y += size
            j += 1
        C["boundary_splits"] += chunks - 1
        C["frontier_peak"] = max(C["frontier_peak"], len(queue))
        C["pending_completed_peak_records"] = max(
            C["pending_completed_peak_records"], len(queue) + len(completed)
        )
        assert pending_rows + done_rows == acc[-1]
        if C["E"] % 8192 == 0:
            progress("constructing")
    sample("constructed", (raw, pieces, starts, anchors, ast, acc, grammar, queue, completed))
    merged = []
    for a in ordered(completed, lambda a: a.s):
        C["coalescing_checks"] += 1
        if (
            merged
            and merged[-1].s + merged[-1].n == a.s
            and merged[-1].t + merged[-1].n == a.t
            and merged[-1].w == a.w
        ):
            b = merged.pop()
            a = allocated(Piece(b.s, b.t, b.n + a.n, b.w))
            C["coalescing_merges"] += 1
        append_record(merged, a)
    C["Q"] = len(merged)
    source_end = 0
    for a in merged:
        assert a.s == source_end
        source_end += a.n
    assert source_end == acc[-1]
    # Expanded dictionaries are real candidate work, not an offline oracle.
    words = {}
    for a in merged:
        expanded = grammar.expand(a.w, "dictionary")
        words[a.w] = expanded[::-1]
        copied("dictionary_reverse", len(expanded))
    tokens = ordered(list(words), words.get)
    counts = Counter()
    for a in merged:
        counts[a.w] += a.n
    cursors, z = {}, 0
    for word in tokens:
        cursors[word] = z
        z += counts[word]
    for a in merged:
        assert a.t == cursors[a.w], "token LF mismatch"
        cursors[a.w] += a.n
    C["dictionary_words"] = len(words)
    C["dictionary_bytes"] = sum(map(len, words.values()))
    assert C["dictionary_bytes"] <= C["weighted_rows"]
    lengths = Counter(len(word) for word in words.values())
    sample(
        "dictionary",
        (
            raw,
            pieces,
            starts,
            anchors,
            ast,
            acc,
            grammar,
            queue,
            completed,
            merged,
            words,
            tokens,
            counts,
            cursors,
            lengths,
        ),
    )
    st, q, emitted = [a.s for a in merged], root, bytearray()
    while True:
        a = merged[bisect_right(st, q, "residual_search") - 1]
        payload = grammar.expand(a.w, "residual")
        emitted.extend(payload)
        C["residual_token_visits"] += 1
        copied("residual", len(payload))
        q = a.t + q - a.s
        assert len(emitted) <= n
        if q == root:
            break
    assert len(emitted) == meta["oracle"]["selected_cycle_length"], "cycle mismatch"
    C["selected_cycle_bytes"] = len(emitted)
    reverse = bytes(reversed(emitted))
    copied("cycle_reverse", len(reverse))
    shift %= len(reverse)
    right, left = reverse[shift:], reverse[:shift]
    for part in (right, left):
        if part is not reverse and part:
            copied("rotation_slice", len(part))
    rotated = right + left
    if rotated is not right and rotated is not left:
        copied("rotation_concat", len(rotated))
    repeated = rotated * ((n + len(rotated) - 1) // len(rotated))
    if repeated is not rotated:
        copied("repeat", len(repeated))
    rle1 = repeated[:n]
    if rle1 is not repeated:
        copied("truncate", len(rle1))
    assert hashlib.sha256(rle1).hexdigest() == meta["artifacts"]["baseline-rle1.bin"]["sha256"], (
        "RLE1 bytes/origin mismatch"
    )
    output, crc = unrle_and_crc(rle1)
    assert len(output) == meta["output_bytes"] and crc == meta["crc"], "output length/CRC mismatch"
    assert (
        hashlib.sha256(output).hexdigest() == meta["artifacts"]["baseline-output.bin"]["sha256"]
    ), "output bytes mismatch"
    C["validation_hash_bytes"] = len(rle1) + len(output)
    C["output_bytes"] = len(output)
    sample(
        "expanded_metadata",
        (
            raw,
            pieces,
            starts,
            anchors,
            ast,
            acc,
            grammar,
            queue,
            completed,
            merged,
            words,
            tokens,
            counts,
            cursors,
            lengths,
            st,
        ),
    )
    data_buffers = {
        id(x): x for x in (emitted, payload, reverse, right, left, rotated, repeated, rle1, output)
    }
    C["live_output_buffer_bytes"] = sum(sys.getsizeof(x) for x in data_buffers.values())
    C["logical_byte_writes"] = (
        sum(v for k, v in C.items() if k.endswith("_copy_bytes") or k.endswith("_leaf_appends"))
        + C["rle1_literal_appends"]
        + C["rle1_run_materialized_bytes"]
    )
    C["excess_writes_over_one_rle1_and_final_output"] = C["logical_byte_writes"] - n - len(output)
    traversal = C["E"] + C["origin_search_steps"] + C["residual_token_visits"]
    lower_bound = max(SAMPLES.values())
    return dict(
        outcome="reject_traversal" if traversal >= n else "revise_metadata",
        complete=True,
        byte_origin_cycle_crc_match=True,
        token_lf=True,
        traversal_sum=traversal,
        n=n,
        traversal_screen_pass=traversal < n,
        metadata_threshold_bytes=4 * n,
        metadata_peak_lower_bound_bytes=lower_bound,
        metadata_screen="fail_proved_by_live_sample"
        if lower_bound >= 4 * n
        else "unresolved_peak_below_threshold_not_proved",
        metadata_peak_exact=False,
        metadata_process_address_space_upper_bound_bytes=256 * 1024 * 1024,
        phrase_length_histogram=dict(sorted(lengths.items())),
        output_sha256=hashlib.sha256(output).hexdigest(),
        rle1_sha256=hashlib.sha256(rle1).hexdigest(),
        oracle_storage_in_candidate_bytes=0,
    )


def main():
    """Preserve one outcome, including failure, under supervisor-enforced limits."""
    global RESERVE
    if not __debug__:
        raise RuntimeError("assertions required")
    assert resource.getrlimit(resource.RLIMIT_AS) == (256 * 1024 * 1024,) * 2
    assert resource.getrlimit(resource.RLIMIT_CPU) == (600, 600)
    export, identities, destination = map(Path, sys.argv[1:])
    meta = json.loads(identities.read_text())
    try:
        result = evaluate(export, meta)
    except MemoryError as error:
        error.__traceback__ = None
        RESERVE = None
        gc.collect()
        result = dict(
            outcome="decline_memory_limit",
            complete=False,
            unfinished_checks="see progress stage; no success inferred",
        )
    except BaseException as error:
        result = dict(
            outcome="stop_implementation_error",
            complete=False,
            error_type=type(error).__name__,
            error=str(error),
        )
    result.update(
        counts=dict(C),
        metadata_live_samples=SAMPLES,
        copy_length_histogram=dict(sorted(COPY_LENGTHS.items())),
        process_peak_rss_kib=resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
        reporting_reserve_bytes=2 * 1024 * 1024,
    )
    destination.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    progress("finished")


if __name__ == "__main__":
    main()
