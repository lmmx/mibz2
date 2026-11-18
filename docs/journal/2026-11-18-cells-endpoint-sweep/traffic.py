#!/usr/bin/env python3
"""Paired source-level memory traces. No hardware counters or candidate timings."""

import argparse
import csv
import sys
from collections import Counter, OrderedDict
from pathlib import Path

from census import SIDE, irec, roots
from sweep import ARENA, LOW, MARK, RULES, Blocks, Packet, Words

# Disjoint cache-line-aligned region origins. Recipe byte offsets retain the W16 layout.
TOP, BOT, ENTRY, TBYTE, COUNTER, PKT, HEAD, LOCAL, FLAGS, DENSE, BYTE, RULE = (
    i << 32 for i in range(12)
)


class Cache:
    """Fully associative write-allocate LRU cache with a fixed capacity in lines."""

    def __init__(self, lines):
        self.limit = lines
        self.lines = OrderedDict()
        self.fills = self.dirty = 0

    def touch(self, line, write):
        """Account for one line reference, including any dirty eviction."""
        if line in self.lines:
            if write:
                self.lines[line] = True
            self.lines.move_to_end(line)
        else:
            self.fills += 1
            self.lines[line] = write
            if len(self.lines) > self.limit:
                self.dirty += self.lines.popitem(last=False)[1]


class Memory:
    """Count source field operations and feed two independent line caches."""

    def __init__(self):
        self.l1, self.l2 = Cache(512), Cache(4096)
        self.count = Counter()
        self.unique = set()

    def at(self, at, size=4, write=False, group="core"):
        """Charge a single field and all cache lines intersecting it."""
        self.count[group + ("_w" if write else "_r")] += 1
        self.count[group + "_bytes"] += size
        for line in range(at // 64, (at + size - 1) // 64 + 1):
            self.unique.add(line)
            self.l1.touch(line, write)
            self.l2.touch(line, write)

    def report(self):
        """Summarise field references and transfers including a terminal dirty drain."""
        c = self.count
        return {
            **{
                k: c[k]
                for k in (
                    "setup_r",
                    "setup_w",
                    "core_r",
                    "core_w",
                    "hint_r",
                    "hint_count",
                    "word_r",
                    "word_w",
                    "setup_bytes",
                    "core_bytes",
                )
            },
            "lines": len(self.unique),
            "l1_fills": self.l1.fills,
            "l1_dirty": self.l1.dirty + sum(self.l1.lines.values()),
            "l2_fills": self.l2.fills,
            "l2_dirty": self.l2.dirty + sum(self.l2.lines.values()),
        }


def recipe(mem, a, b, cw):
    """Account for the pinned short copy pair or two long-rule stores."""
    length = cw & LOW

    def source(w):
        return BYTE + (w >> 32) - ARENA

    if length <= 16:
        dest = source(cw)
        for w, d in ((a, dest), (b, dest + (a & LOW))):
            mem.at(source(w), 16, group="word")
            mem.at(d, 16, True, "word")
    else:
        dest = RULE + (cw >> 32) - RULES
        mem.at(dest, 8, True, "word")
        mem.at(dest + 8, 8, True, "word")


def histogram(rec, mem, extra):
    # Common capped-record reads and gates are cancelled. Retain the shipped per-run
    # letter census and scatter counters; charge a second row histogram to the sweep.
    """Charge stable-order setup and the optional row histogram over capped records."""
    byte, width = rec.a["l_byte"], rec.a["l_lam"]
    last = None
    for c, w in zip(byte, width):
        if c != last:
            mem.at(COUNTER + 4 * c, group="setup")
            mem.at(COUNTER + 4 * c, write=True, group="setup")
        last = c
    if extra:
        # Each physical record of the original coalesced byte run is charged.
        # The primary split must not create phantom capped records.
        for c, n in runs(byte, width):
            for _ in range((n + 126) // 127):
                mem.at(COUNTER + 2048 + 4 * c, group="setup")
                mem.at(COUNTER + 2048 + 4 * c, write=True, group="setup")
    for j in range(256):
        mem.at(COUNTER + (2048 if extra else 0) + 4 * j, group="setup")
        mem.at(COUNTER + (2048 if extra else 0) + 4 * j, write=True, group="setup")


def runs(byte, width):
    """Coalesce neighbouring equal-byte pieces, including fragments at the primary."""
    c0, size = None, 0
    for c, w in zip(byte, width):
        if c != c0 and size:
            yield c0, size
            size = 0
        c0 = c
        size += w
    if size:
        yield c0, size


def baseline(rec, lazy):
    """Address a pinned slot read/write using its recorded letter and the live slot promises."""
    assert rec.period == rec.n and rec.closures == 1 and rec.steps
    m, words = Memory(), Words()
    histogram(rec, m, False)
    v, r = rec.a, rec.r
    n, byte, widths = rec.n, v["l_byte"], v["l_lam"]
    ep = list(roots(widths, byte))
    index = [0] * r
    for k, i in enumerate(v["l_bottom"]):
        index[i] = k
    top_site = {t: i for i, (t, _) in enumerate(ep)}
    bot_site = {ep[i][1]: k for k, i in enumerate(v["l_bottom"])}
    ts = [i if not lazy else -1 for i in range(r)]
    bs = [k if not lazy else -1 for k in index]
    pointer = list(v["l_bottom"])
    desc = [words.literal(c, i == rec.row_letter) for i, c in enumerate(byte)]
    top = lambda i: TOP + 4096 + 20 * i
    bot = lambda i: BOT + 128 + 8 * i
    for i, k in enumerate(index):
        m.at(COUNTER + 4 * byte[i], group="setup")
        m.at(COUNTER + 4 * byte[i], write=True, group="setup")
        if lazy:
            m.at(ENTRY + 4 * k, write=True, group="setup")
            m.at(TBYTE + i, 1, True, "setup")
        else:
            m.at(top(i), 8, True, "setup")
            m.at(top(i) + 8, 8, True, "setup")
            m.at(top(i) + 16, 4, True, "setup")
            m.at(bot(k), 8, True, "setup")
    if lazy:
        for j in range(256):
            m.at(COUNTER + 4 * j, group="setup")
            m.at(COUNTER + 1024 + 4 * j, write=True, group="setup")
    else:
        m.at(top(rec.row_letter) + 8, write=True, group="setup")
        m.at(top(0), write=True, group="setup")
        m.at(bot(0), write=True, group="setup")

    def hints(t, b):
        if t is not None:
            m.at(top(t - 12) + 16, group="hint")
            m.at(top(t - 12) + 12, group="hint")
        else:
            m.at(bot(b - 12) + 4, group="hint")
            m.at(bot(b - 6) + 4, group="hint")
            addr = top(pointer[b - 6]) if b >= 6 else TOP  # zero-filled pad
            m.at(addr + 12, group="hint")
        m.count["hint_count"] += 2

    fresh_a = fresh_b = True
    free = []
    peak = 0
    clock = n
    header_reads = [0, 0]
    for i in range(rec.steps + 1):
        done = i == rec.steps
        x = rec.final if done else v["s_a"][i]
        y = rec.final if done else v["s_b"][i]
        raw = []
        modes = 0
        for side, k, go in ((0, x, fresh_a), (1, y, fresh_b)):
            if not go:
                continue
            if lazy:
                if side == 0 and clock in top_site:
                    header_reads[0] += 1
                    place = top_site[clock]
                    c = byte[place]
                    m.at(TBYTE + place, 1)
                    m.at(COUNTER + 1024 + 4 * c)
                    m.at(COUNTER + 1024 + 4 * c, write=True)
                    m.at(ENTRY + 4 * index[place])
                if side == 1 and clock in bot_site:
                    header_reads[1] += 1
                    m.at(ENTRY + 4 * bot_site[clock])
                if ts[k] < 0:
                    assert k < r
                    assert clock in (bot_site if side else top_site)
                    raw.append((side, k))
                    modes |= 1 << side
                    continue
                modes |= 4 << side
            if not lazy and not (fresh_a and fresh_b):
                hints(ts[k] if not side else None, bs[k] if side else None)
            if not side:
                m.at(top(ts[k]))
                m.at(top(ts[k]) + 4)
                m.at(top(ts[k]) + 8, 8)
                m.at(top(ts[k]) + 16)
            else:
                m.at(bot(bs[k]), 8)
                m.at(top(ts[k]) + 4)
                m.at(top(ts[k]) + 8, 8)
        if not lazy and fresh_a and fresh_b:
            hints(ts[x], None)
            hints(None, bs[y])
        if done:
            assert x == y and not raw
            break
        if lazy:
            assert modes == v["s_fresh"][i]
            for side, k in raw:
                if free:
                    t, b = free.pop()
                    m.at(top(t))
                    m.at(top(t) + 4)
                else:
                    t = b = peak
                    peak += 1
                ts[k], bs[k] = t, b
                m.at(top(t) if side else bot(b), write=True)
                m.at(ENTRY + 4 * index[k], write=True)
        a, b = v["s_ra"][i], v["s_rb"][i]
        cw = words.join(desc[x], desc[y])
        recipe(m, desc[x], desc[y], cw)
        desc.append(cw)
        fa, fb, t, k = ts[x], bs[y], ts[y], bs[x]
        if a < b:
            m.at(bot(k) + 4, write=True)
            m.at(top(t))
            m.at(top(fa), write=True)
            m.at(top(t), write=True)
        m.at(top(t) + 4, write=True)
        m.at(top(t) + 8, 8, True)
        m.at(top(t) + 16, write=True)
        if a == b:
            m.at(bot(k) + 4, write=True)
            if lazy:
                m.at(top(fa), write=True)
                m.at(top(fa) + 4, write=True)
                free.append((fa, fb))
        elif a > b:
            m.at(bot(k))
            m.at(bot(fb), write=True)
            m.at(bot(k), 8, True)
            bs[x] = fb
        else:
            ts[y] = fa
        if not lazy:
            pointer[k] = t
        ts.append(t)
        bs.append(k)
        fresh_a, fresh_b = a <= b, b <= a
        clock -= min(a, b)
    if lazy:
        assert header_reads == [r, r]
    return m.report(), peak


def projected(rec, packed):
    """Price both calendars on the same verified clock. Only this address census uses trace keys."""
    m = Memory()
    histogram(rec, m, True)
    v, r = rec.a, rec.r
    byte, widths = v["l_byte"], v["l_lam"]
    words = Words()
    npage = (rec.n + 63) // 64
    current = npage
    box = Blocks(rec.n, r) if packed else None
    for j in range(npage):
        if packed:
            m.at(HEAD + 4 * j, write=True, group="setup")
        else:
            m.at(FLAGS + 16 * j, 8, True, "setup")
            m.at(FLAGS + 16 * j + 8, 8, True, "setup")

    def post(q, side, idx, item, setup=False):
        page, offset = divmod(q - 1, 64)
        code = offset + 64 * side
        group = "setup" if setup else "core"
        if packed:
            m.at(PKT + 16 * idx, 8, True, group)
            m.at(PKT + 16 * idx + 8, 4, True, group)
            if page == current:
                m.at(LOCAL + 4 * code, write=True, group=group)
            else:
                m.at(HEAD + 4 * page, group=group)
                m.at(PKT + 16 * idx + 12, write=True, group=group)
                m.at(HEAD + 4 * page, write=True, group=group)
            box.post(q, side, idx, item)
        else:
            pos = DENSE + 12 * (2 * (q - 1) + side)
            m.at(pos, 8, True, group)
            m.at(pos + 8, 4, True, group)
            if page != current:
                m.at(FLAGS + 16 * page + 8 * side, 8, group=group)
                m.at(FLAGS + 16 * page + 8 * side, 8, True, group)

    delta, desc = [], []
    for i, (a, b) in enumerate(roots(widths, byte)):
        d = b - a
        w = Words.literal(byte[i], i == rec.row_letter)
        m.at(COUNTER + 2048 + 4 * byte[i], group="setup")
        m.at(COUNTER + 2048 + 4 * byte[i], write=True, group="setup")
        post(max(a, b), int(d > 0), i, Packet(abs(d), w, i), True)
        delta.append(d)
        desc.append(w)
    q = rec.n
    fresh = (True, True)
    for i in range(rec.steps + 1):
        page = (q - 1) // 64
        done = i == rec.steps
        x = rec.final if done else v["s_a"][i]
        y = rec.final if done else v["s_b"][i]
        while current > page:
            current -= 1
            if packed:
                m.at(HEAD + 4 * current)
                j = box.head[current]
                while j != -1:
                    m.at(PKT + 16 * j + 12)
                    route = box.route[j]
                    m.at(LOCAL + 4 * (route & 127), write=True)
                    j = (route >> 7) - 1
            else:
                m.at(FLAGS + 16 * current, 8)
                m.at(FLAGS + 16 * current + 8, 8)
        if packed:
            qp, ti, bi = box.next()
            assert qp == q
        for side, go in enumerate((fresh[0], fresh[1] and not done)):
            if go:
                if packed:
                    idx = bi if side else ti
                    assert idx >= 0 and box.packet[idx].name == (y if side else x)
                    m.at(LOCAL + 4 * ((q - 1) % 64 + 64 * side))
                    pos = PKT + 16 * idx
                else:
                    pos = DENSE + 12 * (2 * (q - 1) + side)
                m.at(pos, 8)
                m.at(pos + 8)
        if done:
            break
        d = delta[x] + delta[y]
        when = q - min(-delta[x], delta[y])
        delta.append(d)
        cw = words.join(desc[x], desc[y])
        recipe(m, desc[x], desc[y], cw)
        desc.append(cw)
        idx = (ti if ti >= 0 else bi) if packed else 0
        post(when, int(d > 0), idx, Packet(abs(d), cw, rec.r + i))
        a, b = v["s_ra"][i], v["s_rb"][i]
        fresh = (a <= b, b <= a)
        q -= min(a, b)
    return m.report(), (r if packed else 0)


def main():
    """Trace both shipped representations and both scheduling models per block."""
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("records", nargs="+")
    args = p.parse_args()
    out = None
    for arg in args.records:
        for path in irec.paths(arg):
            rec = irec.load(path)
            for mode in ("eager", "lazy", "bucket", "direct"):
                if mode in ("eager", "lazy"):
                    result, peak = baseline(rec, mode == "lazy")
                else:
                    result, peak = projected(rec, mode == "bucket")
                row = dict(block=rec.block, model=mode, pool=peak, **result)
                if out is None:
                    out = csv.DictWriter(sys.stdout, fieldnames=row, lineterminator="\n")
                    out.writeheader()
                out.writerow(row)


if __name__ == "__main__":
    main()
