"""Historically recursive trie publication failed on a realizable second-layer phrase."""
from pathlib import Path
import json
import sys
from hierarchy import decode
from phrases import compress, transform

# 1001 distinct ascending phrases in increasing lexical order. The intervening
# comparisons all descend, so a real first-layer byte transform produces the
# required 1001-symbol monotone token orbit.
pairs = [(a,b) for a in range(8) for b in range(8,256)][:1001]
text = bytes(c for pair in pairs for c in pair)
n = len(text)
order = sorted(range(n), key=lambda i: text[i:] + text[:i])
column = bytes(text[i-1] for i in order)
marks = [order.index(i) for i in [0,1,22,1321,2001]]
layers_seen = []
for pos, p in zip([0,1,22,1321,2001], marks):
    result = transform(compress(column), p, True)
    assert len(result[4]) == 1001
    out, layers, rest = decode(compress(column),p,stop=2)
    assert out == text[pos:] + text[:pos]
    assert [s["alphabet"] for s in layers] == [1001,1]
    layers_seen.append([s["carried_byte_offset"] for s in layers])
if len(sys.argv) == 2:
    Path(sys.argv[1]).write_bytes(column)
print(json.dumps({"byte_rows":n, "next_alphabet":1001, "second_layer_phrase_symbols":1001,
                  "primary_rows":marks, "carried_offsets":layers_seen}))
