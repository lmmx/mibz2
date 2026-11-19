# One checked item/comma fold on held-out Wikidata

The native first word `item"` had the same protected successor `,"` on every one of 219 held-out admissions. Folding this single class removed **64,963 of 1,369,769 stock second-discovery rank queries (4.74%)**, while every matched first graph, second weighted column, marked offset and decoded byte passed. The complete inverse on the twelve prespecified timed blocks measured **1.0041 [.9630, 1.0470]** and **1.0201 [.9819, 1.0598]** of the two named controls. Direct index construction cost more than the observed discovery saving. [Held-out geometry](heldout.csv) · [Raw paired times](phase.csv) · [Phase report](summary.txt)

## Freeze and correctness

The literal and new independent performance criterion were frozen from ten training tapes at `89e4b32`. Nine of 228 admissions have exactly the same row count, primary and capped input as a training tape. The remaining 219 also have distinct cap tapes from every training tape if the primary is ignored. The twelve timed held-out ordinals are 17, 51, 87, 124, 160, 198, 237, 272, 309, 346, 380 and 411, fixed at `e42d678` before evaluating the literal. Other general fold analyses had used these admissions. The held-out question was whether this *specific literal* would retain its successor, rank savings and marked behavior. [Training discovery](TRAINING.md) · [Frozen rule](PLAN.md) · [Exact source digests and split](split.csv)

The matcher locates both complete interned byte spellings. Its single scan of the old root checks that the minimum and maximum destinations of `item"` lie in the entire `,"` source bucket, accumulates the class counts and maps any deleted mark. Requiring a third class strictly heavier than both literals keeps the same stock second anchor under all tie orderings. With old class base b and frequency f of the selected source, a removed mark at the ith incoming occurrence moves to b (adjusted by f if the target bucket precedes it) plus i and its skip grows by five bytes. Surviving marked coordinates subtract preceding deletions.

The writer drops old incoming `item"` runs, and for each old incoming `,"` occurrence reads one entry from the original comma source bucket. A deleted source there gives an incoming `item"` label, and other sources retain the comma label. Each record is emitted straight into the stock second rank index with its mapped source base and preceding occurrence count. The only changed spelling is one contiguous seven-byte dictionary slice. The original root remains available for a disjoint mark, unrecognized pair, nondeterministic successor, tie or second-level decline. For a declined attempted second, restore the old spelling before calling the original marked fallback. The constructor uses the stock count, base, run and page vectors with one cursor into the original comma window. [Applicability and restoration](pair.rs) · [Direct index](index.rs)

The gate independently walked dense old source permutations and contracted the literal arcs row-by-row. It compared the full normalized first weighted object and transferred mark, the complete normalized stock second dictionary and incoming column, the marked second offset and all output bytes. It checked 4,332 arbitrary or targeted small primaries, including 74 transferred marks, 360 nondeterministic-successor rejections, 456 missing strict heavier witnesses and eight cases where the attempted stock second fell back. The training query saving exactly reproduced 3,808. Among the 219 held-out admissions, the candidate removed 294,498 of 29,646,736 quote rows. None of their actual primaries landed in a deleted word. The twelve timed blocks saved 3,564 of 82,977 queries. [Adversarial and periodic receipt](small.csv) · [Training check](training.csv) · [Timed-panel geometry](panel.csv) · [Source and oracle](native.rs)

## Paid complete inverse

Each of 48 rounds included every twelve-block inverse and each of four arms, pinned to CPU3 at nice +12. Every input used every arm position once in each four-round time cluster. The intervals use paired log ratios of the full twelve-block time sums, grouped in twelve chronological clusters. The inputs to the calculation are integer nanoseconds. Every paid output was independently checked. Recognition-only ran the same applicability check, then the original named second writer. Each memory arm verified all 228 marked hashes in its own fresh process.

| Arm | Full / named A | Full / named B | Eligibility ms / 12 | Index + shift ms / 12 | Second discovery ms / 12 | Full ms / 12 | HWM KiB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Named A | 1 | | 0 | 1.043 | 4.293 | 26.011 | 8,228 |
| Named B | .9843 [.9413, 1.0293] | 1 | 0 | .966 | 4.051 | 25.552 | 8,224 |
| Recognize | .9847 [.9455, 1.0256] | 1.0004 [.9601, 1.0425] | .254 | 1.066 | 4.335 | 25.545 | 8,228 |
| Literal | 1.0041 [.9630, 1.0470] | 1.0201 [.9819, 1.0598] | .287 | 1.503 | 3.977 | 26.030 | 8,352 |

The literal / named A index-and-shift ratio was 1.4310 [1.2931, 1.5836]. Discovery was .9510 [.8657, 1.0448]. Peak retained bytes including output were 8,473,038 for literal vs 8,473,542 for named A/B. The complete result missed the prespecified whole-worker criterion. These numbers describe an inverse stage on the local Xeon. [Time-cluster contrasts](clusters.csv) · [228-block process memory](rss.csv)

## Reproduction

The frozen native source is `6ac9415f2b12ea9e1e12f0fe8a423385b5f215e1` and exact held-out/native receipts are `d526626c43cf73e8ee7d8d6a338c99aa524b9778`. Use the shared original-cap archives and dense hash list identified by SHA-256. [Toolchain, input and binary identities](environment.txt)

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/native/protected/closure/single
out=/tmp/astra-single-replay
python3 -B "$d/split.py" /tmp/astra-clipped/ten.bin /tmp/astra-token-all/admissions.bin "$out"
diff "$out/split.csv" "$d/split.csv"
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/fold" | diff - "$d/small.csv"
"$out/fold" check /tmp/astra-clipped/ten.bin > "$out/training.csv"
"$out/fold" check "$out/held.bin" > "$out/heldout.csv"
"$out/fold" check "$out/panel.bin" > "$out/panel.csv"
nice -n 12 taskset -c 3 "$out/fold" price "$out/panel.bin" > "$out/phase.csv"
for a in 0 1 2 3; do
  "$out/fold" rss /tmp/astra-token-all/admissions.bin "$a" /tmp/astra-clipped-final/expected.csv > "$out/rss-$a.csv"
done
head -1 "$out/rss-0.csv" > "$out/rss.csv"
for a in 0 1 2 3; do tail -n+2 "$out/rss-$a.csv" >> "$out/rss.csv"; done
python3 -B "$d/report.py" "$out"
```
