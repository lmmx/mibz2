# Recover the cyclic position of any primary from the same grammar

At one ordinary cut let S be the surviving physical rows and let a be the incoming class on an old overlay interval in source bucket j. If the interval survives, the new rank is x minus the deleted mass before it. The byte offset stays the same. If the interval is deleted, then a is outside the delimiter set and j is inside. Its predecessor is LF(x)=C[a]+Occ(a,x). Every source row in bucket a survives. Thus the new rank is LF(x) minus the total deletion before bucket a, and the new byte offset is the old offset plus ∣Wₐ∣. Since Occ(a,x) advances one-for-one on this old interval, either branch has the form **(x+δ, s+h)** with a constant integer δ and a nonnegative h. Adjacent intervals with both actions equal coalesce. A complete old-rank transport atlas has at most R+K−1 records and costs O(R+K) time to emit from the original overlay.

Compose these maps in cut order. For a primitive input each retained permutation is a single orbit until its last row fixes. The final absolute byte offset from the terminal root then gives the cyclic position of any original primary. All ranks can use the same cut coloring and the same complete grammar. A level can be indexed by binary search in its starting coordinates. The all-primary statement here covers histories consisting entirely of ordinary cuts. The generic single-primary power transport is given in the parent study.

Frozen needed 25 levels. Their 98,484 affine descriptors occupy 1,181,808 bytes at 12 bytes each, compared with 3,599,724 bytes for one 32-bit offset per original row. This accounts only for the stored coordinates in each representation. The archived binary additionally holds the initial RLE for the independent oracle. All 899,931 original ordinals landed on row zero; their output offsets agreed with independently stable-sorted LF at *every* step around the marked cycle. The original primary 264,045 mapped to byte offset 84,941 in the terminal word. Resolving all ranks required 22,498,275 descriptor lookups and 200,524,606 binary comparisons. For one requested primary the unseeded on-the-fly method avoids retaining this atlas. The test establishes a sparse suffix-position index for this real block. It supplies no whole-decoder speed measurement. [Full compressed atlas](frozen.bin.gz) · [Every pass](passes.csv) · [Raw verification](queries.txt) · [Exhaustive per-cut oracle](small.json)

```bash
d=$(dirname "$(rg --files | rg '/cut/unseeded/all-marks/PLAN.md$')")
tmp=$(mktemp -d)
python3 -B "$d/atlas.py" | diff - "$d/small.json"
python3 -B "$d/atlas.py" research/astra-e/dense/target/wikidata/0000.irec.xz "$tmp/atlas.bin" "$tmp/passes.csv" | diff - "$d/frozen.json"
gzip -dc "$d/frozen.bin.gz" | cmp - "$tmp/atlas.bin"
g++ -std=c++20 -O2 -Wall -Wextra -Werror "$d/query.cpp" -o "$tmp/query"
"$tmp/query" "$tmp/atlas.bin" | diff - "$d/queries.txt"
```
