"""Independent reduction and marked-cycle oracle for the published irec records."""
import csv
import bz2
import hashlib
import json
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "bench"))
import irec


def dense_cycle(column, primary):
    inverse = sorted(range(len(column)), key=column.__getitem__)
    row = primary
    emitted = bytearray()
    while True:
        row = inverse[row]
        emitted.append(column[row])
        if row == primary:
            return bytes(emitted)


def raw_cycle(record):
    arrays = record.a
    stack = [record.final]
    out = bytearray()
    marks = []
    while stack:
        node = stack.pop()
        if node < record.r:
            if node == record.row_letter:
                marks.append(len(out))
            out.append(arrays["l_byte"][node])
        else:
            step = node - record.r
            stack.extend((arrays["s_b"][step], arrays["s_a"][step]))
    assert len(marks) == 1
    return bytes(out), marks[0]


def undo_rle1(data):
    emitted = bytearray()
    i = 0
    while i < len(data):
        byte = data[i]
        four = bytes([byte]) * 4
        if data[i:i+4] == four:
            assert i + 4 < len(data)
            emitted += bytes([byte]) * (4 + data[i+4])
            i += 5
        else:
            emitted.append(byte)
            i += 1
    return emitted


def archived_pairs(record):
    arrays = record.a
    letters = arrays["l_byte"]
    widths = arrays["l_lam"]
    assert arrays["l_bottom"].tolist() == sorted(range(record.r), key=letters.__getitem__)
    assert widths[record.row_letter] == 1
    for k in range(record.r - 1):
        if letters[k] == letters[k + 1]:
            assert record.row_letter in (k, k + 1)
    breaks = set(arrays["c_step"])
    event = arrays["s_type"]
    count = Counter()
    for j in range(1, record.steps):
        if j in breaks:
            continue
        count["ETB"[event[j-1]] + "ETB"[event[j]]] += 1
        if event[j-1] != 1 or not arrays["s_fresh"][j] & 2:
            continue
        count["fresh"] += 1
        a = arrays["s_a"][j-1]
        b, d = arrays["s_b"][j-1:j+1]
        assert d < record.r and len({a, b, d}) == 3 and arrays["s_a"][j] == a
        residual = arrays["s_ra"][j-1] - arrays["s_rb"][j-1]
        assert residual == arrays["s_ra"][j]
        assert event[j] == (0 if residual == widths[d] else 1 if residual > widths[d] else 2)
        if residual > widths[d]:
            continue
        count["eligible"] += 1
        if residual < widths[d]:
            count["fresh_TB"] += 1
        if b >= record.r or letters[b] != letters[d] or record.row_letter in (b, d):
            count["different"] += 1
            continue
        assert d + 1 < b
        length = arrays["w_len"][a] + 1
        assert length == arrays["w_len"][record.r + j - 1] == arrays["w_len"][record.r + j]
        if length > 16 or arrays["w_kind"][a] & 4:
            count["equal_rule"] += 1
            continue
        count["short_TB" if residual < widths[d] else "short_TE"] += 1
        count["bytes_saved"] += length
    return count


def local_substitutions():
    """Enumerate nontrivial relative order and compare every final row map."""
    import itertools

    def exchange(top, bottom, width, word):
        image = {}
        pos = 0
        for name in bottom:
            image[name] = pos
            pos += width[name]
        return [(image[name] + off, word[name])
                for name in top for off in range(width[name])]

    def scalar(top, bottom, width, word, name):
        a, b = top[-1], bottom[-1]
        width[name] = min(width[a], width[b])
        word[name] = word[a] + word[b]
        top.insert(top.index(b) + 1, name)
        bottom.insert(bottom.index(a) + 1, name)
        for source in (a, b):
            width[source] -= width[name]
            if width[source] == 0:
                top.remove(source)
                bottom.remove(source)

    checked = 0
    for n in range(3, 7):
        initial_top = list(range(n))
        a = initial_top[-1]
        for initial_bottom in itertools.permutations(initial_top):
            d, b = initial_bottom[-2:]
            if a in (d, b):
                continue
            for bw, dw in itertools.product(range(1, 4), repeat=2):
                for s in range(1, dw + 1):
                    x, y = n, n + 1
                    width = {i: i % 3 + 1 for i in initial_top}
                    width.update({a: bw+s, b: bw, d: dw})
                    word = {i: (i,) for i in initial_top}
                    top, bottom = initial_top.copy(), list(initial_bottom)
                    sw, words = width.copy(), word.copy()
                    scalar(top, bottom, sw, words, x)
                    scalar(top, bottom, sw, words, y)
                    top_rep = {a: [], b: [x], d: ([d, y] if s < dw else [y])}
                    bot_rep = {a: [y, x], b: [], d: ([d] if s < dw else [])}
                    jt = [u for i in initial_top for u in top_rep.get(i, [i])]
                    jb = [u for i in initial_bottom for u in bot_rep.get(i, [i])]
                    width.update({x: bw, y: s, d: dw-s})
                    word.update({x: (a,b), y: (a,d)})
                    assert top == jt and bottom == jb
                    assert exchange(top, bottom, sw, words) == exchange(jt, jb, width, word)
                    checked += 1
    return checked


def metrics(rows):
    h = [r["short_TE"] + r["short_TB"] for r in rows]
    nets = [7*x-r["fresh"]-3*r["eligible"] for x,r in zip(h, rows)]
    steps = sum(r["steps"] for r in rows)
    return dict(blocks=len(rows), steps=steps, shared=sum(h), copies=2*sum(h),
                sites=sum(r["r"] for r in rows),
                min_net=min(nets), total_net=sum(nets),
                min_break_even_credit=max((r["fresh"]+3*r["eligible"])/x
                                          for x,r in zip(h,rows) if x))


def main():
    with (ROOT / "docs/journal/study/blocks.csv").open() as f:
        rows = [{k: (v if k == "corpus" else int(v)) for k,v in row.items()}
                for row in csv.DictReader(f)]
    index = {(r["corpus"],r["block"]):r for r in rows}
    prose_output = bytearray()
    for group in ("wikidata", "prose"):
        for path in sorted((ROOT / "data" / group).glob("*.irec.xz")):
            rec = irec.load(path)
            counts = archived_pairs(rec)
            row = index[rec.corpus, rec.block]
            for k in ("fresh","fresh_TB","eligible","different","equal_rule","short_TE",
                      "short_TB","bytes_saved", *(a+b for a in "ETB" for b in "ETB")):
                assert row[k] == counts[k], (path.name,k,row[k],counts[k])
            for k in ("index", "n", "r", "steps"):
                assert row[k] == getattr(rec, k)
            if row["admitted"] != int(rec.admitted):
                assert (group, rec.block, rec.admitted, row["admitted"]) == ("wikidata", 144, False, 1)
            col = b"".join(bytes([b])*w for b,w in zip(rec.a["l_byte"],rec.a["l_lam"]))
            p = sum(rec.a["l_lam"][:rec.row_letter])
            word, mark = raw_cycle(rec)
            emitted = dense_cycle(col, p)
            assert emitted == word[mark+1:] + word[:mark+1]
            assert len(word) == rec.period and len(col) == rec.n
            if group == "prose":
                prose_output += undo_rle1((emitted * ((rec.n + len(emitted) - 1) // len(emitted)))[:rec.n])
            print(json.dumps(dict(corpus=rec.corpus,block=rec.block,mark=mark,period=len(word),
                                  marked_sha256=hashlib.sha256(emitted).hexdigest())),flush=True)
    wiki = [r for r in rows if r["corpus"] == "wikidata.bz2"]
    prose = [r for r in rows if r["corpus"] == "prose.bz2"]
    packed = (ROOT / "data/corpora/prose.bz2").read_bytes()
    assert prose_output == bz2.decompress(packed)
    print(json.dumps(dict(local_cases=local_substitutions(), frozen=metrics(wiki[:1]),
                          wikidata=metrics(wiki), admitted=metrics([r for r in wiki if r["admitted"]]),
                          prose=metrics(prose), prose_output_bytes=len(prose_output),
                          prose_output_sha256=hashlib.sha256(prose_output).hexdigest())),flush=True)


if __name__ == "__main__":
    main()
