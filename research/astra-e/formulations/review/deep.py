"""Reach a deep second-level monotone phrase from an ordinary byte block."""
import bz2
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'f4'))
from phrases import compress, transform, spelling


def cyclic_bwt(text):
    """Sort the cyclic rows by doubling equal-prefix ranks."""
    n = len(text)
    ranks = list(text)
    width = 1
    while width < n:
        keys = [(ranks[i],ranks[(i+width)%n]) for i in range(n)]
        order = sorted(range(n),key=keys.__getitem__)
        r = 0
        ranks[order[0]] = 0
        for prev,i in zip(order,order[1:]):
            r += keys[i] != keys[prev]
            ranks[i] = r
        if r + 1 == n:
            return [text[i-1] for i in order],ranks[0]
        width *= 2
    raise AssertionError("expected a primitive source cycle")


def main():
    # Distinct delimiter-terminated token words occur once, so the byte source
    # is primitive. There is no RLE1 quartet anywhere, including at boundaries.
    words = sorted((0, *(i+1 for i in range(11) if mask & (1<<i)), 255)
                   for mask in range(1 << 11))[:1001]
    text = bytes(x for w in words for x in w)
    encoded = bz2.compress(text,compresslevel=9)
    assert bz2.decompress(encoded) == text
    assert all(not (x == y == z == t) for x,y,z,t in zip(text,text[1:],text[2:],text[3:]))
    last,p = cyclic_bwt(text)
    result = transform(compress(last),p,True)
    runs,rp,skip,parents,dic,_ = result
    out = [code for code,size in runs for _ in range(size)]
    assert [tuple(spelling(x,parents)) for x in dic] == words
    assert out == [1000,*range(1000)] and rp == 0 and skip == 0
    try:
        result2 = transform(runs,rp,True)
    except RecursionError:
        second = "RecursionError in Python trie publication"
    else:
        r,p2,skip2,parents2,dic2,_ = result2
        assert r == [(0,1)] and p2 == 0 and skip2 == 0
        assert spelling(dic2[0],parents2) == list(range(1001))
        second = "correct singleton token"
    print(json.dumps(dict(bytes=len(text),primary=p,source_bzip2_sha256=hashlib.sha256(encoded).hexdigest(),
                          source_text_sha256=hashlib.sha256(text).hexdigest(),tokens=len(out),
                          python_recursion_limit=sys.getrecursionlimit(),second_level=second), indent=2))
    if len(sys.argv) > 1:
        Path(sys.argv[1]).write_bytes(encoded)


if __name__ == "__main__":
    main()
