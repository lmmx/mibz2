"""Independent marked-output replay with physical 16-byte metadata records."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import struct
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent / "bench"))
import irec

PAIR = struct.Struct("<QQ")
COMMAND = struct.Struct("<QI")


def make_grammar(bytes_, mark, cuts, final):
    literal = [bytes([c]) for c in bytes_]
    lengths = [1] * len(literal)
    marked = [i == mark for i in range(len(literal))]
    rules = {}
    for a, b, kind in cuts:
        identity = len(lengths)
        lengths.append(lengths[a] + lengths[b])
        marked.append(marked[a] or marked[b])
        if lengths[-1] <= 16 and not marked[-1]:
            literal.append(literal[a] + literal[b])
        else:
            literal.append(None)
            rules[identity] = (a, b, kind)
    return literal, lengths, marked, rules, final


def dense(column, primary):
    inv = sorted(range(len(column)), key=column.__getitem__)
    row, data = primary, bytearray()
    while True:
        row = inv[row]
        data.append(column[row])
        if row == primary:
            return bytes(data)


def fill(grammar, full):
    literal, length, marked, rules, root = grammar
    p = length[root]
    x, mark_at = root, 0
    while x in rules:
        a, b, _ = rules[x]
        assert marked[a] != marked[b]
        if marked[a]:
            x = a
        else:
            mark_at += length[a]
            x = b
    assert marked[x]

    use = {}
    pos = {root: (-mark_at - 1) % p}
    command_list = []
    raw = bytearray(16 * len(rules))
    ids = list(rules)
    rank = {x: i for i, x in enumerate(ids)}
    for j, (a, b, _) in enumerate(rules.values()):
        PAIR.pack_into(raw, 16*j, a, b)
        use.update((c, j) for c in (a, b) if c in rules)

    out, written = bytearray(p), bytearray(p)
    stat = Counter()
    mark_physical = []

    def write(at, chunk):
        end = at + len(chunk)
        assert 0 <= at < end <= p and written.find(b"\1", at, end) < 0
        out[at:end] = chunk
        written[at:end] = b"\1" * len(chunk)

    def emit_leaf(child, at):
        write(at, literal[child])
        if marked[child]:
            mark_physical.append(at)
        stat["literal_edges"] += 1
        stat["literal_bytes"] += length[child]

    if root not in rules:
        emit_leaf(root, 0)
    elif full:
        tail = len(ids)
        assert ids[-1] == root and root not in use and len(use) == len(ids) - 1
        previous_pair = (None, None)
        for j in reversed(range(len(ids))):
            assert j < tail and ids[j] in pos
            a, b = PAIR.unpack_from(raw, 16*j)
            assert (a, b) == rules[ids[j]][:2]
            left = pos[ids[j]]
            right = left + length[a]
            if right >= p:
                right -= p
                stat["wraps"] += 1
            assert (right < left) == marked[a]
            kind = rules[ids[j]][2]
            for side, (child, at) in enumerate(((a, left), (b, right))):
                if child not in rules:
                    emit_leaf(child, at)
                    continue
                # The actual cut type and independently enumerated use list are two oracles.
                survives = previous_pair[side] == child
                assert survives == (kind == (1 if side == 0 else 2))
                assert (not survives) == (use[child] == j)
                if survives:
                    tail -= 1
                    assert j <= tail
                    COMMAND.pack_into(raw, 16*tail, child, at)
                else:
                    assert child not in pos
                    pos[child] = at
            previous_pair = (a, b)
        assert pos.keys() == rules.keys()
        command_list = [COMMAND.unpack_from(raw, 16*j) for j in range(tail, len(ids))]
    else:
        # Choose each position from reachable parents only, using overwrite order.
        for parent, (a, b, _) in reversed(rules.items()):
            if parent in pos:
                left = pos[parent]
                for child, at in ((a, left), (b, (left + length[a]) % p)):
                    if child in rules:
                        pos[child] = at
        # Preserve parent birth order for copies. Oldest chosen occurrences may be at any address.
        for parent, (a, b, _) in rules.items():
            if parent in pos:
                left = pos[parent]
                for child, at in ((a, left), (b, (left + length[a]) % p)):
                    if child not in rules:
                        emit_leaf(child, at)
                    elif at != pos[child]:
                        command_list.append((child, at))

    for child, at in command_list:
        src, count = pos[child], length[child]
        assert 0 <= src < src + count <= p and written.find(b"\0", src, src + count) < 0
        assert src + count <= at or at + count <= src
        write(at, out[src:src+count])
        stat["copies"] += 1
        stat["copy_bytes"] += count
        stat["from_later"] += src > at
    assert written.find(b"\0") < 0 and mark_physical == [p-1]
    return bytes(out), dict(stat), mark_at


def induction(column, primary):
    bytes_, widths = [], []
    for i, byte in enumerate(column):
        if i in (primary, primary+1) or not bytes_ or bytes_[-1] != byte:
            widths.append(0)
            bytes_.append(byte)
        if i == primary:
            mark = len(bytes_) - 1
        widths[-1] += 1
    marked = [i == mark for i in range(len(widths))]
    top = list(range(len(widths)))
    bottom = sorted(top, key=bytes_.__getitem__)
    cuts = []
    while top:
        a, b = top[-1], bottom[-1]
        if a == b:
            if marked[a]:
                return make_grammar(bytes_, mark, cuts, a)
            top.pop()
            bottom.pop()
            continue
        kind = 0 if widths[a] == widths[b] else 1 if widths[a] > widths[b] else 2
        t = len(widths)
        size = min(widths[a], widths[b])
        cuts.append((a, b, kind))
        widths.append(size)
        marked.append(marked[a] or marked[b])
        top.insert(top.index(b)+1, t)
        bottom.insert(bottom.index(a)+1, t)
        for i in (a, b):
            widths[i] -= size
            if widths[i] == 0:
                top.remove(i)
                bottom.remove(i)
    raise AssertionError("missing marked closure")


def witness(n, primary):
    column = bytes([1, 1] + [0]*(n-2))
    grammar = induction(column, primary)
    _, lengths, marked, rules, root = grammar
    period = lengths[root]
    out, stat, mark = fill(grammar, full=period == n)
    assert out == dense(column, primary)
    assert (out * ((n + period - 1)//period))[:n] == bytes(
        out[i % period] for i in range(n))
    if period != n:
        # The marked root's long child is exhausted in a later, unreachable rule.
        long_child = next(c for c in rules[root][:2] if c in rules)
        latest = [j for j,(a,b,_) in rules.items() if long_child in (a,b)][-1]
        assert latest > root and not marked[latest]
    else:
        long_child = latest = None
    return dict(n=n, primary=primary, period=period, root=root, rules=rules, marked=marked,
                reachable_child=long_child, actual_owner=latest, mark=mark, stats=stat,
                period_hex=out.hex())


def archive(path):
    rec = irec.load(path)
    a = rec.a
    grammar = make_grammar(a["l_byte"], rec.row_letter,
                           zip(a["s_a"], a["s_b"], a["s_type"]), rec.final)
    out, stat, mark = fill(grammar, full=rec.n == rec.period)
    column = b"".join(bytes([c])*w for c,w in zip(a["l_byte"], a["l_lam"]))
    assert out == dense(column, rec.p)
    assert len(out) == rec.period
    return dict(corpus=rec.corpus, block=rec.block, rule_count=len(grammar[3]),
                mark=mark, sha256=hashlib.sha256(out).hexdigest(), **stat)


if __name__ == "__main__":
    print(json.dumps(dict(witnesses=[witness(35, 2), witness(36, 3)])), flush=True)
    for root in sys.argv[1:]:
        for path in sorted(Path(root).glob("*.irec.xz")):
            print(json.dumps(archive(path)), flush=True)
