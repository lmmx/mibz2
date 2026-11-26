# Paying for a variable source prefix and a fixed-depth return core

Splitting the first ten bytes from each original return reduced backward live states by 27.0% and translated LF pieces by 24.4% on the ten Wikidata blocks. It also reduced maximum fresh native process memory from 22,392 to 19,940 KiB. Complete native CPU rose to **1.051× [1.024, 1.079]** the same-code named-section control. Forward address construction, sorting and reconstructing output from two spelling pieces absorb the discovery gain in this implementation. [Full paired results](summary.json) · [Raw samples](paid.csv) · [Fresh process memory and comparison counters](resources.csv)

## Object and constructor

From each old source s in the named section U, the desired dictionary word is its positive first return W. For a fixed h no greater than the minimum return length, split W into its own first h bytes P and the remaining core C. Return the complete old incoming token column in destination order, with each word represented by the pair (P,C). Projecting each pair to C gives the incoming core column. Equal cores can have different source prefixes. The small binary last column `10`, with U covering both ranks and h=1, has two different one-byte return words and the same empty core for both. The constructor verifies the two columns independently at every destination. Both the canonical original token labels by source base and the original section mark survive, so a second induction receiving base, width, length and mark has the same input at every admitted depth.

Only original L runs, U, h and the mark enter the native constructor. The panel derives U with the bounded heaviest quote-return selector and verifies original marked bytes against the immutable input SHA-256. Depth zero uses the same index, node layout, selector merge, mark handling and writer. The common section selection cost is recorded outside all timing arms. [Input manifest and selector work](../shift-constructor/inputs.csv)

1. A source-ordered band stores consecutive old owners, its affine current ranks and a prefix identifier. Translate through the original Ψ select runs. At one depth, (old prefix, next byte) is nondecreasing in old source order. A common prefix maps its original rows increasingly and the next outgoing F byte is ordered by rank. A new distinct pair gets a new spelling node without a lookup table. Split at each original translation boundary and coalesce adjacent equal-prefix pieces with matching source displacement. Check each projected band for overlap with U during this process.
2. The first positive j with U∩Ψ^j(U) nonempty is exactly the minimum return length. A hit at j<h makes this fixed-depth split inapplicable. A hit at j=h identifies zero-core returns explicitly. All ten named sections admit depths through 11 and reject 12 at 11. A separate positive-core construction chooses depth per arc and continues past that boundary. [Variable-depth theorem and independent forward atlas](../short-core/README.md)
3. Sort the final bands by physical rank. Precolor original LF pieces at the boundaries of those bands, carrying the source-offset and prefix identifier. The table needs at most r+2B records for r original runs and B final affine tags. Original source bases are recovered directly from these annotations even when prefix groups interleave in physical rank order. For a fixed prefix P, its original owner order and endpoint order agree under Ψ^h. For a fixed core C, backward stable LF preserves the destination order while partitioning by bytes. Together they give increasing source ordinals for each completed (P,C) within its canonical contiguous old word bucket.
4. Start at old U in destination order. Publish matches in U∩Ψ^h(U) immediately for h>0. Continue backwards until the first hit of Ψ^h(U), at which the prefix annotation completes W. Each retained backward state is its old contiguous spelling interval minus F=⋃_{j=1}^h Ψ^j(U). Since h is admissible there is one selected endpoint at offset h along each old return arc, including its destination when the length equals h. On that arc the retained ranks are exactly those more than h positive steps from the previous U. Their backward discovery spelling is their original itinerary to the next U, independent of their removed source prefix. A gapped hull skips a lazily sorted union F. This implementation pays both the final ownership ordering and the union ordering. [Earlier common-byte restricted live-partition proof](../shift-constructor/proof.md)
5. Each completed core is stored once. Cache its first source prefix and allocate a separate word handle for each other distinct prefix at that terminal branch. Integer pair keys are compared exactly. Bottom-up selector merging retains those handles and produces the incoming original words. Each emitted occurrence consumes its charged prefix and core spelling slices. Transfer the original mark against U, scatter the m-entry original first-return permutation and emit exactly n bytes. Marked cycles disjoint from U use original LF. The two-piece writer pays for each reference even when its separate spelling storage is smaller.

## Frozen structural result

The named section is [21,229,45,394), with 24,165 old sources, 2,950 full word types and 5,060 incoming full-token runs. At h=10 the forward constructor forms 2,088 tagged affine bands with 1,049 distinct source prefixes. Their geometrical union has 1,273 components. Backward discovery visits 73,305 live states versus 100,644 originally. It makes 83,006 LF translations versus 110,061. The 2,333 distinct cores need 108,778 bytes. Unique prefixes need another 10,490, versus 146,183 logical full spelling bytes. A total of 157 cores need several original prefixes. Reconciliation pays 1,172 extra prefix lookups at terminal pieces. There are 4,709 incoming core runs. The joined incoming column still has all 5,060 original runs and correct source bases. At depth 11, 83 old destinations terminate at the root with an empty core. [All 80 admission and core checks](real.csv)

## Ten-block price

| h | Final affine tags | Forward translated pieces | Backward LF translations | Unique core + prefix bytes | CPU time / h=0 (95% interval) | Max VmHWM, KiB |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 10 | 0 | 1,423,936 | 1,926,457 | 1.000 | 22,392 |
| 3 | 8,140 | 10,235 | 1,308,198 | 1,810,543 | 0.984 (0.942–1.028) | 21,748 |
| 4 | 15,088 | 25,323 | 1,269,851 | 1,767,569 | 0.980 (0.947–1.015) | 21,924 |
| 6 | 19,857 | 62,725 | 1,197,401 | 1,707,745 | 1.019 (0.963–1.078) | 21,288 |
| 8 | 25,503 | 111,815 | 1,133,302 | 1,639,676 | 1.027 (0.989–1.066) | 20,672 |
| 10 | 30,448 | 169,883 | 1,076,917 | 1,583,223 | 1.051 (1.024–1.079) | 19,940 |
| 11 | 36,736 | 206,619 | 1,049,258 | 1,547,519 | untimed | 19,420 |

At h=10 forward binary searches made 2,271,106 key comparisons. Sorting final tags cost 428,155 comparisons. The live-mask sort added 2,942,722 inside the backward construction phase. Only 1,324 nodes actually required a gapped hull, with 20,418 physical holes. A possible next falsifier is to record each gap when its two adjacent live pieces are inserted, and charge a small linked record only for those actual holes. This would avoid sorting the accumulated mask. A compact forward locator or monotone run cursor would still need to pay its own indexing and query costs. No timing benefit from these changes has been established.

On the paid h=10 panel, setup used 4.325× [3.991, 4.688], core discovery plus selector assembly 0.875× [0.851, 0.900], and marked emission 1.216× [1.107, 1.335] of their respective h=0 stages. The full measurement includes constructor destruction. The initial structural receipts run independent correctness checks and give the mark table a second transfer call. The paid receipts make one transfer and exclude independent verification from timing.

Each six-depth native arm received all ten blocks and occupied each of six positions four times after warmup. Each lap sums the ten blocks before taking its paired log ratio. Intervals use Student t over 24 laps. Fresh VmHWM includes original input runs, native setup, work structures and marked output. The full bzip2 worker, entropy decoding, RLE1, CRCs, multiworker memory and target-machine throughput require additional independent testing. [Native implementation](core.hpp) · [Paired harness and oracle](main.cpp) · [Full results](summary.json)

## Verification and reproduction

The exhaustive gate examined 130,310 fixed-depth cases on binary columns through eight ranks and ternary columns through five. Of 51,340 admitted objects, 18,173 had at least one empty-core return and 5,772 had a core shared by multiple prefixes. The 353,846 marked native outputs matched independent dense LF/Ψ traversal. An additional 78,970 settings were correctly rejected at their first short return. On real inputs the checker compares every original source, mapped prefix band, true tail, original word, source base, incoming full token, incoming core token, primary transfer and emitted bytes. At the 20 real fixed-depth cases shared with the independent short-core atlas at `11cc7ee`, all nine comparable structural and dictionary statistics agreed exactly. [Small gate](small.txt) · [Independent cross-check](crosscheck.txt)

```bash
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/prefix-core
out=$(mktemp -d)
python3 -B "$d/../shift-constructor/panel.py" research/astra-e/dense/target/wikidata "$out/panel" | diff - "$d/../shift-constructor/inputs.csv"
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic "$d/main.cpp" -o "$out/core"
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic "$d/resources.cpp" -o "$out/resources"
"$out/core" | diff - "$d/small.txt"
"$out/core" check "$out/panel" 1 > "$out/real.csv"
taskset -c 3 "$out/core" time "$out/panel" 24 > "$out/paid.csv"
python3 -B "$d/report.py" "$out"
python3 -B "$d/resources.py" "$out/resources" "$out/panel" > "$out/resources.csv"
python3 -B "$d/crosscheck.py" | diff - "$d/crosscheck.txt"
```
