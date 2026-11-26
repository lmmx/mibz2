# Sparse marker insertion on the original Compact quote root

The published root has m+s rows, where m is the original quote count and s is the number of eligible old visits. Compact can supply every byte slice directly from its original dictionary. All 1,824 native arm/block combinations entered the stock second writer and reproduced the independent marked output on 228 Wikidata admissions. Sparse insertion beat matched two-phase construction on all three depths, but the preregistered complete-inverse comparison against ordinary named quote missed its threshold. [Frozen plan](PLAN.md) · [Native implementation](publish.rs) · [All admissions](admissions.csv)

## Exact orientation and marker transfer

Write an original positive quote-return spelling as W = qA for quote byte q. Stock Compact stores B = Aq and uses one phase-shifted source per old occurrence, in the same order as original quote sources. For the fixed trimmed core C = comma at depth h, the old word is eligible exactly when |W| ≤ h+1 and W ends in comma. Compact can test |B| ≤ h+1, |B| ≥ 2 and B's penultimate byte being comma. Split a selected B into B without its final two bytes and a marker spelling comma followed by quote. Keep each unselected B intact. The marker starts at the intended physical comma cut and carries the following quote to the next original phase-shifted source. Selected prefixes can be empty. Every augmented cycle still emits positive bytes. Moreover m+s ≤ n: each selected return on the original disjoint U-return partition has at least two bytes and every return has at least one.

The old source buckets shift by s. The new marker bucket occupies [0,s) in old selected-landing order. Emit the selected portion of the original incoming RLE, preserving original token IDs; this describes incoming arcs at marker ranks. Append the full original incoming RLE with selected IDs replaced by the marker ID, describing incoming arcs at old ranks. Coalesce adjacent equal labels. Stable occurrence order holds for every old token because membership in the selected set is constant per ID. The marker's outgoing ranks enter the corresponding old destinations in order. The two passes touch O(K+R) words/runs, including a scratch flag per original word. No per-occurrence array is needed to insert the marker. Appending the single marker node to the retained first-level discovery vector can occasionally reallocate and copy its Q existing nodes if its spare capacity is zero; the linear publication bounds exclude that allocation event. Stock second induction may subsequently allocate links per selected occurrence. The stock original merge/selector vectors exchange roles and the dictionary keeps all original bytes. A third mass-count pass is paid only by the explicit full-word control.

The original marked pair is an old **rotated** source ordinal and the byte offset into its B. If it is outside a selected suffix, shift the source by s. Otherwise find the occurrence number within its old source bucket; in the original incoming RLE, count selected visits preceding that occurrence's destination. That count is the new marker rank. Subtract the old selected prefix length from the offset. The new mark is valid also for an empty selected prefix. A cycle disjoint from the original quote section keeps its original run-based fallback. A fixed comma absent from the block leaves the ordinary original object in place. For a fixed comma that occurs less often than some full-word ID, the column remains exact and ordinary second induction may instead choose that most frequent ID. On the real panel the stock link count confirmed that the inserted comma marker always won. [Dense reference](oracle.rs) · [Native mark, cold case and timed harness](main.rs)

The exhaustive gate checked 10,600 augmented columns, every represented terminal spelling and source base, the marked offset and all output bytes over binary columns through length seven and ternary columns through five that contained a quote. It encountered 2,581 absent fixed markers, 3,685 marks inside moved suffixes, 2,084 disjoint marked cycles and 5,902 cases with an empty old prefix. The ten-block geometry also matched the complete independent sparse objects. [Small receipt](small.txt) · [Ten-block geometry](ten.csv)

## Paid eight-arm screen

The eight arms were the fast original named quote root, an explicit full-word insertion control selecting that root's usual most frequent full word, sparse comma insertion at h4/h8/h12, and matched forced-core two-phase construction on quote at the same depths. The existing two-phase h4/h8 producer was retained. Its h12 arm uses an isolated twelve-byte prefix array in place of the existing eight-byte array. All 32 complete ten-block rounds contained every arm in each of the eight order positions equally often on every input. The measured full path begins on original cap records, and includes first discovery/merge, publication, the second index and mark, second discovery/merge and final output. The exact dense oracle is excluded from interval timing. Thread CPU was observed on the shared Xeon, pinned to CPU3 at nice +12.

| Depth | Sparse / named, complete CPU | Sparse / two-phase, complete CPU | Sparse / explicit full-word marker, complete CPU |
|--:|--:|--:|--:|
| 4 | .9318 [.7890, 1.1003] | .5637 [.5198, .6112] | .9951 [.9110, 1.0871] |
| 8 | .9478 [.7497, 1.1981] | .4545 [.3872, .5334] | 1.0122 [.8574, 1.1950] |
| 12 | .9665 [.7846, 1.1906] | .4004 [.3566, .4496] | 1.0323 [.9136, 1.1664] |

The explicit full-word control was .9363 [.7652, 1.1457] against named quote. The registered intervals retain all observations. Individual timings have large excursions: the median named call was 1.822 ms with a maximum of 208.570 ms, and sparse8 reached 327.233 ms. The median complete-round sparse4/named ratio was 1.021. Relative location of outliers changes the apparent sub-ten-percent ranking. The separate 144-round source-identical replication subsequently measured sparse4/named-A at 1.0213 [.9966, 1.0465], sparse4/named-B at 1.0205 [.9875, 1.0545] and named-B/named-A at 1.0008 [.9705, 1.0321]. [Replication and temporal control checks](twins/README.md) [Raw chronological calls](phase.csv) · [All ratios and phase means](summary.txt) · [Twin protocol](twins/PLAN.md)

Across the ten blocks, the sparse h4/h8/h12 second dictionaries occupied 1,688,586 / 1,663,435 / 1,658,397 bytes; all three equal their matched two-phase dictionaries. The sparse arms paid 125,946 / 124,516 / 124,514 second rank lookups, versus 248,998 / 238,908 / 231,751 in two-phase. Named quote used 69,999 lookups and compiled 1,926,457 bytes. Moving only eligible visits reduces phase transitions against two-phase, but still increases lookups against named quote.

| Arm | Fresh-process peak over 228 admissions (KiB) | Maximum retained decoder capacity including output (bytes) | Second dictionary bytes, all admissions |
|---|--:|--:|--:|
| Named quote | 8,320 | 8,473,542 | 41,433,719 |
| Explicit full word | 8,432 | 8,526,150 | 41,433,719 |
| Sparse h4 | 8,452 | 7,943,286 | 36,230,837 |
| Sparse h8 | 8,356 | 7,948,358 | 35,833,727 |
| Sparse h12 | 8,428 | 7,957,286 | 35,709,582 |
| Two-phase h4 | 11,072 | 8,513,006 | 36,230,837 |
| Two-phase h8 | 11,548 | 8,703,754 | 35,833,727 |
| Two-phase h12 | 12,668 | 8,908,494 | 35,709,582 |

Each memory arm ran in a fresh process and checked output against independently frozen marked hashes without allocating dense reference arrays in that process. Separately each of the eight arms checked every marked output byte against dense LF on all 228 admissions. At depths four and eight, three of the actual stream primaries transferred to the inserted suffix; at twelve, four did. These are inverse-stage measurements, separate from full-worker, libbzip2 stream and i9 verification. [Per-input RSS](rss.csv) · [Complete correctness receipt](admissions.log)

## Reproduction

The eight-arm preregistration is `18b8dc1`; the publisher, checker and initial geometry are `842c79b`; raw complete-rate, memory and admission receipts are `f37c557`. The script below uses the archived run-geometry files and whole-corpus hashes identified by their SHA-256s. The upstream export recipe reconstructs them from canonical Wikidata. [Toolchain and input digests](environment.txt) · [Original-cap exports](../../../native/README.md)

```bash
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/native
out=/tmp/astra-marker-replay
bash "$d/build.sh" "$out" "$PWD/target/release/deps"
"$out/marker" | diff - "$d/small.txt"
"$out/marker" geometry /tmp/astra-clipped/ten.bin
nice -n 12 taskset -c 3 "$out/marker" price /tmp/astra-clipped/ten.bin 32 > "$out/phase.csv"
python3 -B "$d/report.py" "$out/phase.csv"
"$out/marker" admissions /tmp/astra-token-all/admissions.bin
for arm in 0 1 2 3 4 5 6 7; do
  "$out/marker" rss /tmp/astra-token-all/admissions.bin "$arm" /tmp/astra-clipped-final/expected.csv > "$out/rss-$arm.csv"
done
```
