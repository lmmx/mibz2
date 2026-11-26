# Recover a selected physical section by its missing ranks

For a fixed positive core C and cut depth h, the exact physical section H can be constructed as the full first-hit interval J=Z(C) with a set of missing ranks. On the ten frequent h8 tapes, recovering all nine exclusions took 104 original backward run intersections and 72 one-row forward select steps after the endpoints of J were known. The earlier full prefix atlas used 111,815 forward intersections to classify every original arc. These counts compare section recognition for a supplied C and an atlas containing much more information. They exclude the index and inversion of the new section. [Constructor](holes.py) · [All geometry and second-word sizes](holes-wikidata.csv)

## A membership certificate using only original byte runs

Start from all of J and trace its previous original U visit for at most h backward steps. At depth d, active ranks correspond to a d-step preimage of an unclassified member of J. When a rank falls in U, that rank is the original source and d is the distance from that source to the member of J. Accept its physical member exactly if d=h or C is a single byte. A rank still outside U after h steps is excluded. This is the bounded-prefix identity for old word length d+|C|. Positive first encounter and delayed clipping matter when a rank is in both U and J at depth zero.

Every live rank has the same depth and needs only a distance test, so coalesce adjacent child intervals across different preceding spellings. Parents arrive in increasing rank order. Stable LF sends a symbol's children into increasing intervals, which can be appended to its bucket and coalesced before concatenating the buckets in byte order. This removes the prefix state from the recognition problem. The Python version sorts only touched symbol keys and retains at most 256 such keys per depth. At any one depth, distinct active ranks give distinct members of J under the forward permutation. Thus the total number of original backwards run intersections is at most h|J|. Each visited interval also pays its run location, first rank for each distinct byte and its emitted children.

Store classified predecessor intervals with their respective depths. Their forward images are disjoint and partition J. Count both sides first. Re-advance only the smaller side through the original per-byte select index and sort its ranks. Complement it inside J if the chosen side was the holes. With s selected rows and e excluded rows, this costs at most h·min(s,e) individual forward selects, plus rank storage and sorting. It outputs at most e+1 selected components. Returning early when an input violates a storage/time budget can use the original exact decoder. No full prefix atlas, full first-level dictionary, or forward permutation is needed by this constructor. The bounded word endpoints use at most two symbol-rank queries per byte of C after the original byte index is available.

## Every core has a common landing shift

Move every selected physical row forward through its common C. This maps H bijectively and in rank order onto a possibly fragmented landing section inside U. The true byte return between two H members starts with C, so the paired return between their landing images moves that initial C to the end. The itinerary can visit some other H in the first |C|−1 steps only if two chosen cut rows lie inside one original arc. The construction has one cut per old arc, and H is a subset of these cuts. Consequently these rotations are exact even when thousands of distinct original prefixes yield the same core. The partners, distinct second-word count and total dictionary spelling bytes are preserved, with the corresponding byte shift of the mark. The result is specific to this positive first-hit core construction; an arbitrary set of points sharing a byte prefix can have intervening visits to the set.

The constructor transforms the sparse H intervals to landing intervals by stable per-byte select-run intersections along C. Their union remains sorted at every symbol. The predecessor and successor tables and both independently marked outputs were checked against dense original navigation in the verifier. On 31,728 small configurations that covered 177,192 marks, including empty selections, absent cores, nonconvex landings and multiple cycles. All sixty real pairs matched individually, including every selected second partner and every complete physical and landing word. For fixed names on the frequent sections h4/h8/h12 the landing used only seventeen components across ten blocks. At quote h12 the same 299,675 partner rows occupied 24 physical components and 3,322 landing components; the second alphabet had the same 1,658,397 total distinct spelling bytes in both coordinates. [Small receipts](holes-small.json) · [Full origin and fixed-name conventions](README.md)

| Section and depth | Selected rows | Holes | Original backward pieces | Forward selects to recover holes | Physical components | Landing components |
|---|---:|---:|---:|---:|---:|---:|
| Frequent 4 | 31,358 | 8 | 56 | 32 | 15 | 17 |
| Frequent 8 | 31,358 | 9 | 104 | 72 | 16 | 17 |
| Frequent 12 | 31,358 | 9 | 144 | 108 | 16 | 17 |
| Quote 4 | 282,172 | 17,526 | 24,671 | 70,104 | 778 | 3,479 |
| Quote 8 | 288,648 | 11,050 | 42,488 | 88,400 | 427 | 3,687 |
| Quote 12 | 299,675 | 23 | 47,309 | 276 | 24 | 3,322 |

C was specified as the appropriate suffix of the frozen nineteen-byte word for frequent, or a comma for quote. These names were already examined on these ten tapes; the comparison is exact structural evidence, not a held-out frequency-selection experiment. A new decoder is free to use J directly as an independently marked section; the native comparison of those fully paid policies is separate. Exact reproduction of the former moving-cut H requires the excluded ranks or a mathematically equivalent correction. [Independent direct-J experiment](direct-native/REGISTER.md)

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin
python3 "$d/holes.py" | diff - "$d/holes-small.json"
python3 "$d/holes.py" research/astra-e/dense/target/wikidata | diff - "$d/holes-wikidata.csv"
```
