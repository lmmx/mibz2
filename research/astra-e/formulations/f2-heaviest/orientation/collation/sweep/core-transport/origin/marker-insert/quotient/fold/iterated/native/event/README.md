# A linear event scheduler for deterministic grammar contractions

The event scheduler produced terminal marked grammars on all 228 Wikidata admissions, with one successor edit per eliminated row. It reduced complete inverse CPU time by 25.2% relative to repeated column rebuilding. The unchanged named writer remained 2.67 times as fast on the local ten-block panel and used less peak memory. [Event implementation](event.rs) · [All paid observations](phase.csv) · [Paired analysis](summary.txt)

## Local update

Write P for the current successor permutation on live original token rows. For each source class k, retain the ordered list of its live original rows and one spelling handle. Initialize P by stable occurrence matching the native incoming run column to the contiguous source buckets. Within each class, P is strictly increasing in original row rank. The surviving source classes occupy consecutive intervals of the live rank order.

If the images of the first and last live rows of k both belong to another class j, every image of k belongs to j. Fold k by redirecting each edge x→y to P(y), removing y from j, and replacing k's spelling by the immutable concatenation of the old k and j handles. For k the new image order is increasing because its two component maps are increasing on k and j. For remaining j sources it is a subsequence of the old order. All untouched classes keep their old images. The contracted permutation is first return to the surviving rows, and all surviving row owners retain their identity. Thus no other class changes eligibility, even when the last member of j disappears. A marked deleted y moves to its surviving predecessor x, adding the old k word length to its byte offset. The closed self-loop word, rotated at that offset and repeated to n, emits the original marked bytes.

The initial pass considers K classes. A FIFO holds at most one entry per class. When a popped k remains eligible, the linked list supplies exactly its surviving x rows. P is injective, so each bypassed y is distinct and the edit removes a row forever. Then retest only k and j. With m initial rows, the sum of all class scans is at most m−1 per nonempty block, even when constructed word lengths grow. There are at most K+2(m−1) queue insertions, including stale entries. Initialization and natural folding cost O(R+m+K) after the original named stage, with R input runs. Marked grammar expansion is separately charged and visits every emitted original leaf occurrence. Saturation without a marked fixed point uses the preserved named fallback. The natural FIFO schedule has no universal terminal guarantee. [Parent grammar and fallback](../README.md)

This prototype uses four u32 arrays indexed by original row, for successor, owner and the two membership links. It retains endpoint, handle and queue state per class, the immutable grammar, the original named object and the final byte output. The direct indexed control uses the same native publication and dense owner/successor construction, skips link and grammar construction, and walks original named words for one marked period. The released stage disables per-leaf audit counters.

## Checks and paid result

On 13,567 small marked column cases, all four arms equalled the independent stable-sort LF byte output. After each of 10,240 actual small contractions the audit checked the entire live permutation, membership order, strict stability of destinations, eligible target by full enumeration, and unchanged eligibility outside k and j. There were 3,326 mark transfers, 6,373 terminal arms with another live row and 5,241 partial final period copies. The original named fallback handled 712 natural stalls and 2,159 uncovered marked cycles. The harness used the exact Segmented byte fallback on 666 quote-absent inputs. [Small checks](small.csv)

Both the audit and paid specializations of the event and one-level indexed writers then passed full byte-vector equality on 205,197,794 bytes across all 228 retained production blocks. The natural event writer produced 228 terminal grammars with 30,868,167 edits from 30,868,395 old rows. Each graph reached a sole live row. There were 9,463,884 class contractions, 10,184,302 queue insertions and 10,183,929 pops in total. [Dense byte checks](dense.txt) · [Per-block events and high-water memory](rss-3.csv)

| Native inverse | CPU μs/block | First named + mark | Extra index | Contractions | Output or named second stage | Max total retained bytes | Fresh process peak KiB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Original named | 1,833 | 1,126 | 0 | 0 | 707 | 8,473,542 | 8,184 |
| Repeated-scan terminal | 6,514 | 1,086 | 119 | 3,779 | 1,529 | 8,179,526 | 8,036 |
| One-level indexed walk | 2,470 | 1,116 | 202 | 0 | 1,150 | 7,648,262 | 7,300 |
| Event terminal | 4,882 | 1,124 | 313 | 1,783 | 1,660 | 10,920,006 | 9,436 |

Event/repeated CPU was 0.7481 [0.7312, 0.7653], event/named 2.6680 [2.5791, 2.7600], and event/indexed 1.9746 [1.9238, 2.0267]. The one-level indexed walk used the least measured space among these four, with a time ratio of 1.3511 [1.3065, 1.3974] against named. Both the event contraction and leaf-order DAG expansion independently exceed the unchanged named writer's 707 μs second stage. Cutting dense table memory can change the tradeoff, while both time terms remain relevant. No arm passed the preregistered whole-worker promotion rule.

The CPU sample consists of 32 rounds of ten systematic inputs, four arms rotating into each position within eight disjoint four-round chronological clusters. It ran on CPU0 at nice +16. A separate dry rehearsal used CPU1–2. The original capped symbols and byte frequencies are supplied to all arms. The measured unit ends after reconstructing all marked bytes and excludes MTF, RLE1, CRCs and worker coordination. The production memory figures come from four separate single-arm sequence processes, with no dense oracle resident. These results establish a formulation-level speedup over the terminal predecessor in this native process. Target-machine throughput remains unverified. [Preregistration](PLAN.md) · [Rebuild/input hashes](manifest.json)

## Reproduction

Use the Rust 1.99.1 release library and the two cap tapes from the preceding native experiment. The builder reconstructs its frozen control before adding the event implementation. Arm indices in order are named, repeated, indexed and event. With '-' as the expected-file argument, the harness independently constructs and checks every output byte in both build modes. Supplying the corpus hash manifest performs the separate memory measurement. [Parent input recipe](../README.md)

```sh
source /workspace/work/env.sh
d=research/astra-e/formulations/f2-heaviest/orientation/collation/sweep/core-transport/origin/marker-insert/quotient/fold/iterated/native/event
out=/tmp/astra-terminal-event
python3 -B "$d/build.py" "$out" "$PWD/target/release/deps"
"$out/event"
for a in 2 3; do "$out/event" census /tmp/astra-token-all/admissions.bin - "$a"; done
for a in 0 1 2 3; do
    "$out/event" census /tmp/astra-token-all/admissions.bin /tmp/astra-clipped-final/expected.csv "$a"
done
taskset -c 0 nice -n16 "$out/event" price /tmp/astra-clipped/ten.bin
python3 "$d/report.py" "$d"
```
