# Direct clipped induction inside Compact/preload

From the original run column and any nonempty rank interval U, this native constructor directly discovers the exact first-return token dictionary, source buckets and induced token BWT. It retains Compact's eight-lane contracted LF exploration, stable selector merge, second induced token decoder and final byte writer. It uses no asynchronous fronts or precomputed return arcs. The generated experiment sits outside the shipping decoder.

## Pay at the two endpoints

Split each original LF translation where its *image* crosses either endpoint of U. All LF images of the original maximal runs partition [0,n), so at most two new original-run descriptors appear. Across artificial 127-byte cap seams, merge adjacent fragments if both byte and terminal status are identical. Each resulting fragment has one LF displacement, one L byte and a uniform predicate for its image being inside U. The high bit of the packed exclusive-end/byte field is one on live fragments and zero on terminal fragments. All eligible block ends fit below the bit. A unary expansion retires exactly when its current descriptor is terminal or fails to cover the interval:

```rust
packed_end_byte < ((interval_end << 8) | LIVE)
```

If the descriptor is live and covers the whole interval, the existing translation and byte append proceed. No comparison of a mapped row with either U endpoint is performed on the unary hot path. This is one unsigned continuation/retirement comparison; the existing eight-lane scheduler retains its branches.

On retirement, selectors inspect the tagged original pieces in actual L order. For each parent and next byte, a terminal chunk emits that word. A live chunk extends that byte's child interval. An aggregated image for one byte has width at most |U|. It cannot have live ranks on both sides of a nonempty fully interior terminal U. Thus the live child stays contiguous even when terminal ranks are clipped off one end. The two selector types preserve their actual positions automatically. No mixed-case event needs to be checked on each unary step. Dynamic mixed-branch counting is enabled only in the evidence binary. The positive incoming LF images of distinct live parents are disjoint, so at most two (parent, byte) branches can have both terminal and live outcomes globally. These events differ from splits in the original run table. [Boundary proof and sharp witnesses](audit/README.md)

If U lies in one F-byte bucket, every first-return word has the same leading byte. The constructor uses Compact's single terminal owner per parent, rotates that byte to the end and rewinds to the strictly preceding section rank with skip equal to distance minus one. For sections spanning F bytes, each terminal (parent, byte) identifies its own word. A tagged selector refers directly to that terminal word ID. Raw words retain their leading byte, and the mark rewinds inclusively with full distance as skip. After merging, the general branch reuses discovery-node storage for the word bases expected by the original second stage. The mark's disjoint cycle can be read exactly by going backwards for one complete period, reversing that period and repeating/clipping to n; this covers periods not dividing n. First-token expansion remains available when the second stage declines.

## Exactness and stage timing

Native testing of every nonempty interval on binary columns through length eight and ternary columns through five passed 19,493 first objects and 132,098 marked outputs, including 34,556 disjoint marks. A dense oracle stable-sorts row indices, walks original LF to the first positive return, and separately constructs source bases, dictionary bytes, destination root runs and mark/skip. The receipt includes all three possible mixed-branch counts. [Small receipts](small.txt)

All 40 systematically selected archive geometries match the separate Python induction, including word counts, dictionary bytes, root runs and mixed branches. All 1,140 policy/block cases on the 228 registered Wikidata admissions passed independent exact first-object and final marked-byte comparisons. All 228 rotated quote objects also matched the original Compact's entire first object and mark/skip up to word-ID renaming. The other cases use raw quote, the most frequent selected quote interval, [n/5,n/5+n/11), and [n/2,n/2+n/13). All real cases proceeded through the actual second induction. Physical and aggregate split counts vary independently; for example the frequent interval produced no mixed tree branch on every block, while 220 of 228 split an original run. [All block/section receipts](admissions.csv) · [Bounds and Python comparison](geometry-summary.txt)

Thirty-six rotated paired repetitions on ten systematic tapes paid original run preparation, mark transfer, native token construction, both dictionaries/induction stages and the full output. All six arms use the same complete marked bytes. Scratch is retained per arm across repetitions on each block; each fresh block starts with fresh scratch. Each paired ratio sums ten blocks per repetition. Confidence intervals use the Student-t interval on log ratios. This is thread CPU on one local pinned core. Each arm receives the selected endpoints; selector discovery cost is additional for a policy that computes its own interval.

| Section / original Compact | First stage [95% interval] | Full marked inverse [95% interval] |
|---|---:|---:|
| Quote, single terminal byte | 1.112 [1.070, 1.156] | 1.070 [1.032, 1.110] |
| Quote forced through general raw-word path | 1.155 [1.097, 1.217] | 1.111 [1.048, 1.178] |
| Frequent selected interval | 1.411 [1.301, 1.530] | 1.088 [1.007, 1.176] |
| Fifth | 2.015 [1.903, 2.133] | 2.010 [1.905, 2.120] |
| Half | 1.596 [1.515, 1.681] | 1.393 [1.324, 1.466] |

This establishes a complete producer for arbitrary intervals and measures the price of these choices. It provides no verified speedup over the Compact control or on i9. Reported ratios compare fixed sections with substantially different word dictionaries and second-stage work. [Every paired sample](phase.csv) · [Unrounded ratios](phase-summary.txt)

The generator verifies the original Compact/preload source SHA-256 84cc39ea5322f37e5a3da7383ea342da4ed7f14ffd4957496b1de2f92a1cbe06 and second-stage SHA-256 877dd1870572c4c5237cbdd1362b8d7124c4393abdcbcebb5194e1fadcffc992. The ten-tape adapter reads only the original run bytes, lengths and primary row. The larger binary input uses the previous token-writer original-cap exporter on corpus 82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915.

```sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/clipped/native
out=/tmp/astra-clipped
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/check"
python3 -B "$d/tapes.py" research/astra-e/dense/target/wikidata "$out/ten.bin"
"$out/check" verify "$out/ten.bin"
nice -n 10 taskset -c 4 "$out/phase" time "$out/ten.bin" 36 > "$out/phase.csv"
python3 -B "$d/report.py" "$out/phase.csv"
```

For all admissions use the previous [original capped-run exporter](../../../seams/async/native/token-writer/export.rs), then pass the result to `check verify`. Full implementation, generator and native drivers are in this directory.

A second implementation tags whole-byte sections directly and clips complete maximal runs for all other cases. It restores the named quote path to an unresolved timing comparison with original Compact while preserving exact arbitrary-section construction. [Paid setup comparison and 228-block receipts](SETUP.md)
