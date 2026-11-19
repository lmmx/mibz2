# Sharing final endpoint spellings at width 16

The measured i9 edge writer can store its 368,867 final nonliteral band words as 186,981 byte-distinct words over the 233 Wikidata admissions. This removes 4,115,534 spelling bytes and 727,544 offset bytes. It reduces the sum of initialized edge, spelling and offset lengths by 4.06%, from 119,360,346 to 114,517,268 bytes. Each arm reached the same 1,585,440-byte high-water capacity in those four vectors across a reused worker. A native trial of full byte equality and cheaper grammar-handle equality gave no improvement over the frozen edge worker. [Measured storage and CPU](summary.json) · [Per-block allocations](checked.csv)

The private-cache result is stronger. Even with arbitrary reordering, free references, and zero construction cost, the existing packed edges plus a single contiguous byte arena containing every required word cannot fit in either 256 KiB or 192 KiB on any of these blocks. Block 47 is the only one with edges alone within 256 KiB. Its edge table uses 260,448 bytes, leaving 1,696 bytes. A proved lower bound for any such byte arena is 9,918 bytes. One verified layout attains 9,951 bytes. The lower bound applies to one contiguous slice per word with the present edges. [Proof, exhaustive checks and attainable arenas](subwords.md)

## What exact identity saves

The i9 worker at `badb2aec3624bd65606c429b2e6f79e9d9d0050f` appends the expansion of a nonliteral terminal node's grammar handle and then the terminal byte a. The constructed band supplies one such dictionary entry for a whole interval of edges. The stored tape contains its original grammar handle, marked visit count and complete final spelling for each band in construction order. The coded output matched the independent Segmented inverse. Every stored interval width equaled its actual marked visit count on these real blocks. [Compressed tape](words.bin.xz) · [Capture program](dump.inc.rs) · [Per-block census](handles.csv)

The prior Cells grammar owner schedules preserve copy volume and depend on the order of first or last use. The short-word endpoint already stores the shortest grammar joins directly and records a first expansion for each longer rule. Its append routine copies that first expansion on subsequent uses. This study keeps these choices and all joins, and considers a complete residual band expression (grammar handle, terminal byte). [Prior grammar owner review](https://github.com/lmmx/mibz2/blob/05770d721d9d294c35496baa75fe716d9c7529c1/state/reports/live-reuse-6986296845/report.md) · [Short-word endpoint](https://github.com/lmmx/mibz2/blob/badb2aec3624bd65606c429b2e6f79e9d9d0050f/research/astra-e/short-words/endpoint.rs)

| Equality or containment test | Aggregate spelling bytes avoided | Opportunities |
|---|---:|---:|
| Adjacent identical handle and terminal byte | 112,745 | 39,067 bands |
| Any earlier identical handle and terminal byte | 263,735 | 84,083 bands |
| Exact, complete byte equality | 4,115,534 | 181,886 bands |
| Remove strict substrings after equal words | 4,392,735 more | 40,318 substring-maximal words remain |
| Overlap adjacent original spellings in original order | 3,019,801 | Alternative to the above reductions |

Equal handles with the same terminal byte necessarily spell identical bytes because grammar handles are immutable within a block. Different handles can have the same expansion. For any fixed handle, terminal bytes encountered by the band constructor are nondecreasing, so storing only the last suffix seen for that handle captures every handle-and-byte duplicate in an unlimited map. The 64-entry direct map captured 81,060 of 84,083 duplicate owners using 512 stack bytes. Its hits avoided 255,240 appended bytes and 324,240 offset bytes. The comparison includes both parts of the identity, so hash collisions give an ordinary append. Emission still performs 26,567,581 edge visits. Its 4,894,668 nonliteral visits copy the same 188,024,749 bytes after either transformation. [Map-size sweep](slots.csv) · [Direct map](handle_alias.py)

The full-byte implementation runs once after the last grammar append, when no later rule expansion can consult its old memoized start. It compacts offsets and spelling in place, aliasing each band to an earlier exact match. It reuses the future edge array as a hash table, clears its used portion and overwrites that portion again during ordinary edge construction. Hashes select comparisons. Complete byte equality authorizes sharing. After 32 unsuccessful probes, or when the original edge allocation cannot host the temporary table, the code keeps an ordinary unshared entry. The original word count can exceed the actual compacted count, which is used when choosing the packed code width. Every actual corpus word fit this method, with no missed exact alias. Output is still copied into the current output buffer, preserving the original marked-cycle termination and repetition. [Implementation](byte_alias.inc.rs) · [Collisions and fallback](byte_alias_check.txt) · [Independent marked permutations](byte_alias_marked.txt)

Substring containment leaves 3,105,353 bytes before ordering the maximal words. Overlapping neighbors of that maximal set in their original first-occurrence order gives an actual arena totaling 3,102,922 bytes, within 149,113 bytes of the summed arbitrary-order lower bounds. This is an offline construction that searches the materialized strings. Endpoints and final copy lengths stay necessary. No speed claim follows from that arena size.

## Complete charged native test

The frozen comparison used all five dense route additions and the first admitted large block in each of twelve streams, seventeen blocks in all. Each block saw all four arms in every position six times over 24 rounds. Results sum the complete per-block intervals within each round before comparing paired thread CPU. Input admission, constructor work, matching, scratch setup and all marked output lie inside the interval. Both transformations matched independent libbzip2 output, block CRCs and the underlying marked bytes on every admission. The outer scanner also validated all twelve combined stream CRCs. We checked 17,064 adversarial marked comparisons for each of the three kernels, including periodic and multiple-cycle columns and narrow-to-wide endpoint transitions. [Preregistered design](pricing-plan.md) · [Real oracle receipt](checked.txt) · [Adversarial checks](adversarial.txt) · [All 1,632 paid observations](paid.csv)

| Complete inverse over frozen edge | All blocks | Twelve ordinary blocks | Five dense additions |
|---|---:|---:|---:|
| Identical-code twin | 1.0211 [1.0004, 1.0423] | 1.0015 [0.9745, 1.0292] | 1.0455 [1.0051, 1.0875] |
| Direct handle map | 1.0255 [1.0102, 1.0410] | 1.0228 [1.0032, 1.0428] | 1.0281 [1.0000, 1.0569] |
| Full byte equality | 1.0262 [1.0060, 1.0468] | 1.0229 [1.0022, 1.0440] | 1.0302 [0.9994, 1.0620] |

Against the identical-code twin, the full-panel ratios were 1.0043 [0.9818, 1.0273] for the direct map and 1.0049 [0.9773, 1.0334] for exact byte equality. The control difference limits precision for small effects. The cloud panel used a 2 MiB private L2, pinned CPU3, four CPU-equivalents and a low-priority compile on CPU1 at panel start. It provides no full-worker or physical i9 wall-time measurement. The proved footprint floor, identical retained high-water and absence of a measured stage gain leave the original i9 worker as the performance incumbent. [Replay including twin contrasts](summary.json) · [Source and machine hashes](manifest.txt)

```bash
d=research/astra-e/edge-spellings
out=/tmp/astra-edge-spellings
python3 "$d/build.py" "$out"
"$out/dump" corpora/wikidata.bz2 "$out/words.bin"
python3 "$d/handles.py" "$out/words.bin" > "$out/handles.csv"
cmp "$out/handles.csv" "$d/handles.csv"
python3 "$d/slots.py" "$out/words.bin" > "$out/slots.csv"
cmp "$out/slots.csv" "$d/slots.csv"
python3 "$d/price.py" "$out"
bzip2 -dc corpora/wikidata.bz2 > "$out/wikidata.raw"
"$out/price" adversarial
"$out/price" check corpora/wikidata.bz2 "$out/wikidata.raw"
nice -n 16 taskset -c 3 "$out/price" price corpora/wikidata.bz2 "$out/wikidata.raw" 24 > "$out/paid.csv"
python3 "$d/analyze.py"
```
