# One maximum-weight fold of the named quote root

The native fold removed 9,077,383 of 30,868,395 quote visits across 228 real admissions; three actual primary marks landed on deleted visits. Every reconstructed byte matched the dense oracle. The fixed 48-round complete inverse took **1.2700 [1.2427, 1.2980]** times the first named control's CPU. The stock second writer used fewer links and took less emission time, but its new anchor produced more than three times the search work and twice the dictionary volume. [Prospective rule](PLAN.md) · [Paid observations](phase.csv) · [Complete admission receipts](admissions.csv)

## Constructor and independent gates

A deterministic source class k lands entirely in the bucket of a different class g(k). Touching both endpoint owners of each maximal incoming run suffices to recognize this property, including repeated runs of the same k. The native constructor sorts K old classes by existing source base, scans the R runs with a monotonically advancing owner pointer and records the input run/offset at the start of each source bucket. It solves the maximum-weight independent set on the functional graph in linear time after sorting, peeling trees and closing each residual cycle under both possible first choices. Source base order and an explicit skip-on-equality rule make the choice repeatable. [Native algorithm](fold.rs) · [Original theorem](../PROOF.md)

The output retains source order on surviving rows. A selected k has its old contiguous byte weight replaced by the concatenation of k and g(k). We append those bytes to the retained dictionary, and track shortened or empty buckets on existing nodes. For a remaining occurrence of target ID j, an independent cursor advances over j's original source bucket and emits either the selected predecessor ID there or j itself. The cursor operates directly on the old root's RLE and the publisher coalesces every output chunk. There is no materialized target substream. For a deleted mark the old predecessor source is its k bucket base plus the marked occurrence ordinal of k; the new source uses its updated base and the same occurrence ordinal. Add the k spelling's original length to its byte skip. Other marks subtract the count of deletions preceding their original rank.

The full cost is O(K log K + R + X + B), where X is the number of lazy run-intersection chunks and B counts appended spelling bytes. The construction keeps O(K) class, graph and cursor scratch, the old spelling dictionary and the appended bytes, and an alternate vector for the published runs. Empty classes are omitted from the contracted object. Their unused word-ID slots remain reserved in the stock Rust first dictionary and second alphabet. Every allocation remains in the reported capacities.

The native weighted optimizer reached the brute-force maximum on **150,207** functional graphs with weighted classes and up to six vertices. The separate exhaustive Compact gate enumerated every ternary last column through length six and binary through nine containing a quote, with every primary. It checked 12,901 complete dictionary/column/mark/output objects, including 1,523 deleted marks, 2,159 disjoint cycles and 3,091 cases with an emptied class. On the ten real roots, the independent Python reference and native source agreed on every one of the 35,117 eligibility edges, masses and selected labels. The Python reference's original discovery-ID tie rule and the new source-ordinal rule selected precisely the same IDs on these inputs. All 912 arm/admission combinations and 1,920 paid calls produced the original marked bytes. All four real arms invoked the stock second writer on every admission. [Exhaustive receipt](small.csv) · [Original and canonical choices](reference.csv.gz) · [Native recognition](selections.csv.gz) · [Comparisons](choices.csv)

## Paid stage and memory

The two named controls use the same runtime call site with independently warmed scratch. Recognition-only prices successor detection, optimal selection and frequency accounting, then passes the original root to the writer. All 48 rounds included ten complete original-cap inverses with marked output; each arm occupied each position on each input within each four-round cluster. The intervals use the twelve chronological cluster means of paired log ratios. The named B/A null measured .9874 [.9628, 1.0127]. [Raw temporal clusters](clusters.csv) · [All phase ratios](summary.txt)

| Arm | Complete / named A | Complete / named B | Constructor ms per ten | Second ms per ten | Complete ms per ten | Peak KiB over 228 | Retained bytes with output |
|---|---:|---:|---:|---:|---:|---:|---:|
| Named A | 1 | | 0 | 8.053 | 20.188 | 8,168 | 8,473,542 |
| Named B | .9874 [.9628, 1.0127] | 1 | 0 | 7.804 | 19.944 | 8,196 | 8,473,542 |
| Recognize only | 1.1116 [1.0909, 1.1327] | 1.1258 [1.0989, 1.1533] | 2.740 | 7.752 | 22.415 | 8,632 | 9,169,542 |
| Fold | 1.2700 [1.2427, 1.2980] | 1.2862 [1.2531, 1.3202] | 4.412 | 8.948 | 25.611 | 9,220 | 9,534,958 |

On ten tapes the fold rebuilt 84,317 runs as 83,280, deleted 402,876 of 1,357,067 quote rows and appended 1,117,995 dictionary bytes. The stock second writer's link count fell from 245,445 to 95,760 and emit CPU fell from named A's 2.513 ms to 1.336 ms. Its discovery used 247,877 rank lookups against 69,999, and its dictionary grew from 1,926,457 to 3,847,233 bytes. The remaining old heaviest anchor lost 18,058–20,189 occurrences per tape. A different first-level ID became the new heaviest anchor on every tape. The next falsifier excludes contractions whose successor is the old anchor and protects its own spelling; this should keep the original second-return dictionary and order and allows a direct test of the reduction in actual rank lookups. This complete native result missed the whole-worker promotion criterion. These stage times and HWM describe the local Xeon inverse harness. [Full per-input high water](rss.csv) · [All named anchors](choices.csv)

## Reproduction

The preregistration is `cfeabbf`, the exact native source and admission gates are `00cb78bdff05ef9abd8c2d17666bf304346dba63`, and the independent reference is `728f5d8`. The original run exporter and frozen reference hashes are shared with the preceding marker studies. [Environment and SHA-256](environment.txt) · [Original-cap binary export](../../../../../native/README.md)

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/native
out=/tmp/astra-fold-replay
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/fold" | diff - "$d/small.csv"
"$out/fold" geometry /tmp/astra-clipped/ten.bin > "$out/ten.csv"
"$out/fold" admissions /tmp/astra-token-all/admissions.bin > "$out/admissions.csv"
"$out/fold" selections /tmp/astra-clipped/ten.bin > "$out/choices-native.csv"
python3 -B "$d/reference.py" research/astra-e/dense/target/wikidata > "$out/choices-independent.csv"
python3 -B "$d/verify.py" "$out/choices-native.csv" "$out/choices-independent.csv" "$out/ten.csv" "$d/../wikidata.csv" | diff - "$d/choices.csv"
nice -n 12 taskset -c 3 "$out/fold" price /tmp/astra-clipped/ten.bin > "$out/phase.csv"
for a in 0 1 2 3; do
  "$out/fold" rss /tmp/astra-token-all/admissions.bin "$a" /tmp/astra-clipped-final/expected.csv > "$out/rss-$a.csv"
done
head -1 "$out/rss-0.csv" > "$out/rss.csv"
for a in 0 1 2 3; do tail -n+2 "$out/rss-$a.csv" >> "$out/rss.csv"; done
python3 -B "$d/report.py" "$out"
```
