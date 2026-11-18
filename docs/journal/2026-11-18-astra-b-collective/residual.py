"""Price the actual residual after deleting clean tiles. LF labels are stored in LF order."""

import argparse
import csv
import sys
from pathlib import Path

from screen import chains, clean_links, inputs


def quotient(bands, n, primary, size):
    links, _ = clean_links(bands, size, primary)
    paths, _ = chains(links)
    count = (n + size - 1) // size
    prefix = [0]
    for tile in range(count):
        keep = tile not in links
        prefix.append(prefix[-1] + (min(size, n - tile * size) if keep else 0))
    arcs = []
    for x, y, width, byte in bands:
        while width:
            src, lane = divmod(x, size)
            dst, offset = divmod(y, size)
            take = min(width, size - lane, size - offset)
            if src not in links:
                end, rest = paths.get(dst, (dst, b""))
                assert end not in links
                append_arc(
                    arcs, prefix[src] + lane, prefix[end] + offset, take, bytes([byte]) + rest
                )
            x += take
            y += take
            width -= take
    return prefix[-1], prefix[primary // size] + primary % size, arcs


def append_arc(arcs, x, y, w, word):
    if arcs:
        a, b, before, label = arcs[-1]
        if a + before == x and b + before == y and word == label:
            arcs[-1] = (a, b, before + w, label)
            return
    arcs.append((x, y, w, word))


def marked_arcs(arcs, primary):
    for x, y, w, word in arcs:
        if x <= primary < x + w:
            off = primary - x
            if off:
                yield x, y, off, word, False
            yield primary, y + off, 1, word, True
            if off + 1 < w:
                yield primary + 1, y + off + 1, w - off - 1, word, False
        else:
            yield x, y, w, word, False


class Grammar:
    """Unmarked short words are inlined like W16. Long initial phrases are literals."""

    def __init__(self):
        self.rules = []
        self.joins = self.arena_bytes = 0

    def join(self, a, b):
        # Descriptors are (bytes or rule index, length, mark-offset or -1).
        wa, la, ma = a
        wb, lb, mb = b
        mark = ma if ma >= 0 else (la + mb if mb >= 0 else -1)
        length = la + lb
        self.joins += 1
        if mark < 0 and length <= 16:
            self.arena_bytes += length
            return wa + wb, length, mark
        index = len(self.rules)
        self.rules.append((a, b))
        return index, length, mark

    def write(self, root, n):
        # First occurrence in the root's spelling; an earlier completed rule can be copied.
        # Rotate while placing, since only subwords on the marked spine can cross the wrap.
        _, period, mark = root
        assert 0 <= mark < period <= n
        output = bytearray(period)
        owners = {}
        cursor = (-mark - 1) % period
        stack = [root]
        literal_bytes = copies = copied = first = wrap_literals = 0
        while stack:
            item, length, _ = stack.pop()
            if isinstance(item, int) and item not in owners:
                owners[item] = cursor
                a, b = self.rules[item]
                stack.extend((b, a))
                first += 1
                continue
            if isinstance(item, int):
                source = owners[item]
                assert source + length <= period and cursor + length <= period
                assert source + length <= cursor or cursor + length <= source
                chunk = output[source : source + length]
                copies += 1
                copied += length
            else:
                chunk = item
                literal_bytes += length
            take = min(length, period - cursor)
            output[cursor : cursor + take] = chunk[:take]
            if take < length:
                output[: length - take] = chunk[take:]
                wrap_literals += 1
            cursor = (cursor + length) % period
        assert cursor == (-mark - 1) % period and literal_bytes + copied == period
        return (bytes(output) * (1 + (n - 1) // period))[:n], dict(
            reached_rules=first,
            copy_commands=copies,
            copied=copied,
            literal_bytes=literal_bytes,
            repeat_bytes=n - period,
            wrap_literals=wrap_literals,
        )


def induce(arcs, support, primary, original_n, mark_lf_offset=0):
    """Diagnostic right-end induction on any weighted translation tiling."""
    top, bottom = {}, {}
    grammar = Grammar()

    def put(t, b, w, word, raw=False):
        slot = (t, b, w, word, raw)
        assert t not in top and b not in bottom
        top[t] = bottom[b] = slot

    for x, y, w, spelling, marked in marked_arcs(arcs, primary):
        forward = spelling[::-1]
        put(
            x + w,
            y + w,
            w,
            (forward, len(forward), len(forward) - 1 - mark_lf_offset if marked else -1),
            raw=True,
        )
    initial = len(top)
    frontier = support
    equal = closed = top_wins = bottom_wins = 0
    raw_top = raw_bottom = live = peak = 0
    while frontier:
        a, b = top.pop(frontier), bottom.pop(frontier)
        if a is b:
            closed += 1
            frontier -= a[2]
            if a[3][2] >= 0:
                out, writer = grammar.write(a[3], original_n)
                return out, dict(
                    support=support,
                    bands=initial,
                    cuts=grammar.joins,
                    equal=equal,
                    closures=closed,
                    period=a[3][1],
                    top_wins=top_wins,
                    bottom_wins=bottom_wins,
                    raw_top=raw_top,
                    raw_bottom=raw_bottom,
                    peak_pairs=peak,
                    rules=len(grammar.rules),
                    arena=grammar.arena_bytes,
                    **writer,
                )
            continue
        _, ab, aw, word_a, fresh_a = a
        bt, _, bw, word_b, fresh_b = b
        raw_top += fresh_a
        raw_bottom += fresh_b
        live += fresh_a + fresh_b
        peak = max(peak, live)
        live -= aw == bw
        top_wins += aw > bw
        bottom_wins += bw > aw
        del top[bt], bottom[ab]
        w = min(aw, bw)
        equal += aw == bw
        put(bt, ab, w, grammar.join(word_a, word_b))
        if aw > w:
            put(frontier - w, ab - w, aw - w, word_a)
        if bw > w:
            put(bt - w, frontier - w, bw - w, word_b)
        frontier -= w
    raise AssertionError("lost the primary")


def original(bands):
    return [(x, y, w, bytes([byte])) for x, y, w, byte in bands]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("folder", type=Path)
    parser.add_argument("--blocks", type=int, nargs="+")
    parser.add_argument("--sizes", type=int, nargs="+", default=[2, 4, 8, 16, 32, 64])
    args = parser.parse_args()
    writer = None
    for block, n, p, _, bands in inputs(args.folder, args.blocks):
        control, baseline = induce(original(bands), n, p, n)
        for size in [0, *args.sizes]:
            row = baseline
            if size:
                support, primary, arcs = quotient(bands, n, p, size)
                out, row = induce(arcs, support, primary, n)
                assert out == control
            result = dict(block=block, B=size, **row)
            if writer is None:
                writer = csv.DictWriter(sys.stdout, fieldnames=result, lineterminator="\n")
                writer.writeheader()
            writer.writerow(result)


if __name__ == "__main__":
    main()
