"""Faithful phrase constructor variants: separate search versus occurrence mark."""

from collections import Counter, deque

from common import Grammar, Piece, bisect_right, decode_oracle, ordered, original


def construct(column, origin, delimiter, mode):
    """Construct unchanged delimiter first returns with fully separated counters."""
    assert mode in ("search", "mark")
    c = Counter()
    pieces, _ = original.from_column(column)
    grammar = Grammar(c)
    starts = [a.s for a in pieces]
    anchors = [a for a in pieces if a.w == delimiter]
    c["input_rows"] = len(column)
    c["input_pieces"] = len(pieces)
    c["anchor_scan_records"] = len(pieces)
    c["source_index_entries"] = len(starts)
    if not anchors:
        c["no_delimiter"] = 1
        return None, c, None
    ast, acc = [a.s for a in anchors], [0]
    for a in anchors:
        acc.append(acc[-1] + a.n)
    c["anchor_index_entries"] = len(ast) + len(acc)

    def rank(x):
        k = bisect_right(ast, x, c, "rank") - 1
        assert k >= 0 and x < anchors[k].s + anchors[k].n
        return acc[k] + x - anchors[k].s

    if mode == "search":
        point, shift = origin, 0
        while True:
            c["origin_piece_visits"] += 1
            a = pieces[bisect_right(starts, point, c, "origin_search") - 1]
            if a.w == delimiter:
                break
            point = a.t + point - a.s
            shift += 1
            c["origin_lf_steps"] += 1
            if point == origin:
                c["unanchored"] = 1
                return None, c, None
        root = rank(point)
    marker = None
    queue = deque((acc[k], a.s, a.n, None) for k, a in enumerate(anchors))
    c["queue_records_created"] = len(queue)
    c["peak_queue_records"] = len(queue)
    completed = []
    while queue:
        source, x, width, word = queue.popleft()
        if mode == "mark":
            c["marker_checks"] += 1
            c["marker_comparisons"] += 1
            if x <= origin:
                c["marker_comparisons"] += 1
                if origin < x + width:
                    assert marker is None
                    c["marker_records"] = 1
                    c["marker_length_reads"] += int(word is not None)
                    marker = (source + origin - x, 0 if word is None else grammar.length[word])
        a = pieces[bisect_right(starts, x, c) - 1]
        assert x + width <= a.s + a.n
        c["constructor_steps"] += 1
        c["weighted_rows"] += width
        word = a.w if word is None else grammar.cat(word, a.w)
        y = a.t + x - a.s
        end = y + width
        j = bisect_right(starts, y, c) - 1
        while y < end:
            c["destination_overlap_visits"] += 1
            b = pieces[j]
            size = min(end, b.s + b.n) - y
            s = source + y - (a.t + x - a.s)
            if b.w == delimiter:
                completed.append(Piece(s, rank(y), size, word))
                c["completed_records_created"] += 1
            else:
                queue.append((s, y, size, word))
                c["queue_records_created"] += 1
            y += size
            j += 1
        c["peak_queue_records"] = max(c["peak_queue_records"], len(queue))
    if mode == "mark" and marker is None:
        c["unanchored"] = 1
        # Construction really happened before discovering this selected cycle is absent.
        return None, c, None
    merged = []
    for a in ordered(completed, lambda a: a.s, c):
        c["coalescing_checks"] += 1
        if merged and (
            merged[-1].s + merged[-1].n == a.s
            and merged[-1].t + merged[-1].n == a.t
            and merged[-1].w == a.w
        ):
            b = merged.pop()
            a = Piece(b.s, b.t, b.n + a.n, b.w)
            c["merged_records_created"] += 1
        merged.append(a)
    words = {a.w: grammar.expand(a.w)[::-1] for a in merged}
    c["dictionary_assignments"] += len(merged)
    tokens = ordered(list(words), words.get, c)
    counts = Counter()
    for a in merged:
        counts[a.w] += a.n
    cursors, total = {}, 0
    for word in tokens:
        cursors[word] = total
        total += counts[word]
    token_lf = True
    for a in merged:
        token_lf &= a.t == cursors[a.w]
        cursors[a.w] += a.n
    if mode == "mark":
        root, phase = marker
        shift = -phase
    starts = [a.s for a in merged]
    q, emitted = root, bytearray()
    while True:
        c["output_piece_visits"] += 1
        a = merged[bisect_right(starts, q, c, "output_search") - 1]
        emitted.extend(grammar.expand(a.w))
        q = a.t + q - a.s
        if q == root:
            break
    reverse = bytes(reversed(emitted))
    c["cycle_reversal_bytes"] += len(reverse)
    shift %= len(reverse)
    reverse = reverse[shift:] + reverse[:shift]
    c["cycle_rotation_bytes"] += len(reverse)
    repeat = (len(column) + len(reverse) - 1) // len(reverse)
    got = (reverse * repeat)[: len(column)]
    c["repeated_bytes"] += repeat * len(reverse)
    c["output_bytes"] += len(got)
    assert got == decode_oracle(column, origin)
    original_stats = dict(
        steps=c["constructor_steps"],
        weighted_rows=c["weighted_rows"],
        anchor_rows=acc[-1],
        anchor_intervals=len(anchors),
        return_pieces=len(merged),
        dictionary_words=len(words),
        dictionary_bytes=sum(map(len, words.values())),
        grammar_nodes=len(grammar.nodes) - 256,
        peak_frontier=c["peak_queue_records"],
        token_lf=bool(token_lf),
        origin_prefix_steps=shift,
    )
    # Routing identities and words are compared across modes, not just final bytes.
    expanded_map = [(a.s, a.t, a.n, original.Grammar.expand(grammar, a.w).hex()) for a in merged]
    return got, c, (original_stats, expanded_map, marker)
