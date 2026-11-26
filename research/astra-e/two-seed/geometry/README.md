# Two forward seeds on each marked residual cycle

With a second seed half the physical rank count from the mark, the two-seed schedule can issue **6,542,615 paired hops** across all 233 real admissions before considering output placement. A perfectly balanced split on each cycle would permit 13,283,735 paired hops. The actual one-buffer half-byte reservation retains **6,496,920 paired hops**, or **48.91% of that ideal**. A seed closes the paired phase on 217 blocks. The primary byte reservation ends it on nine blocks and the secondary reservation on seven. Thus output placement preserves 99.30% of the paired opportunities available at the chosen seed. [Exact per-block arcs and words](panel.csv) · [Independent reservation calculations](bounded.csv)

The secondary arc emits 109,349,454 of the 209,697,662 coded bytes, including 97,913,694 bytes from 2,581,321 nonliteral spelling visits. The parent writer uses the existing output allocation. Its normal schedule carries 52,095,741 initialized prefix bytes across the paired phase and relocates **57,966,109 bytes**. Its boundary-directed schedule carries and relocates **52,763,621 bytes**. Prefix-length calculations on the independently restored tapes reproduce both native schedules' paired, carried, moved and boundary counts on every block. This relocation cost is smaller than copying the full logical secondary arc. [Native metered calls](../all.csv) · [Summary and alternate seeds](summary.json)

## Per-block variation

| Block | Primary / secondary hops | Primary / secondary nonliteral visits | Full secondary bytes | Actual paired hops | Normal / directed moved bytes |
|---|---:|---:|---:|---:|---:|
| 1, frozen | 14,973 / 74,901 | 2,248 / 13,030 | 724,971 | 14,973 | 189,299 / 189,299 |
| 41, dense | 192,233 / 39,367 | 36,131 / 7,861 | 150,451 | 39,367 | 150,451 / 150,451 |
| 56, dense | 2,323 / 227,711 | 300 / 37,515 | 891,451 | 2,323 | 9,212 / 9,212 |
| 165, dense | 157,625 / 63,514 | 29,335 / 11,053 | 258,619 | 63,514 | 258,619 / 258,619 |
| 270, dense | 73,853 / 125,336 | 5,320 / 8,387 | 563,898 | 73,853 | 335,404 / 335,404 |
| 278, dense | 168,446 / 32,972 | 28,158 / 4,388 | 172,640 | 32,972 | 172,640 / 172,640 |
| 277, median unrestricted balance | 146,079 / 47,104 | 31,159 / 8,563 | 208,443 | 47,104 | 208,443 / 208,443 |
| 26, worst unrestricted balance | 100,526 / 9 | 18,127 / 6 | 823 | 9 | 823 / 823 |
| 249, best unrestricted balance | 41,702 / 41,878 | 6,161 / 8,186 | 389,643 | 37,779 | 698,416 / 389,643 |

Within the half-shift cohort, the lower quartile, median and upper quartile of unrestricted paired rounds as a fraction of the per-block ideal are 25.99%, 48.77% and 74.98%. The five dense blocks together achieve 39.14% of the ideal after the byte boundary. The 228 ordinary admissions achieve 49.32%. All four prespecified seed choices landed on the marked cycle on all 233 blocks. These whole-corpus diagnostics record the effect of the seed location without doing per-block search.

| Second seed | Actual paired / ideal | Unrestricted critical hop rounds / old hops | Full secondary bytes | Directed moved bytes |
|---|---:|---:|---:|---:|
| Mark plus floor(m/2) modulo m | 48.91% | 0.75374 | 109,349,454 | 52,763,621 |
| Mark XOR 1 | 43.33% | 0.78058 | 98,031,927 | 46,282,788 |
| Rank 0 | 48.73% | 0.75457 | 105,071,226 | 51,338,603 |
| Rank floor(m/2) | 49.62% | 0.75064 | 113,317,368 | 53,238,000 |

## Precise meaning and checks

Let T be the packed successor, p the marked rank, and s a distinct second seed on the same cycle. The positive p→s and s→p arcs partition the marked cycle exactly in output order. If they use A and B hops, the unrestricted schedule pairs min(A,B) hops and completes in max(A,B) abstract issue rounds. The largest possible pair count is floor((A+B)/2), achieved by splitting halfway in **cycle order**. The baseline visits A+B ranks. For the frozen physical half-shift heuristic the aggregate unrestricted ratio is 0.75374 and the bounded counterpart is 0.75546. These ratios assume one equal-cost successor operation per lane per round. They leave dictionary load latency, output stores, boundary reads and branch cost to timed measurements.

Write P and Q for the increasing cumulative byte lengths on the two arcs. The number of complete paired rounds fitting a half-byte reservation is exactly the smaller of the number of P entries at most floor(n/2) and the number of Q entries at most ceil(n/2). A primary boundary stops before its next label. At a secondary boundary the primary label for that issue can already be written. The verifier prices the subsequent moves from the initialized byte lengths and the actual primary endpoint. The normal schedule can move the secondary prefix up to the right edge and back behind the primary. Under the directed schedule, the opposite arc completes when the first boundary proves it fits. [Independent implementation](census.rs) · [Preregistration](plan.md)

On separate cycles each seed reaches itself before the other. The primary supplies the marked bytes, including repetition to the full n-byte contract. The secondary prefix is disposable. A collision, a nonrepresentable diagnostic seed, or m=1 uses one lane. The separate-byte-vector diagnostic in the raw tables builds each arc with normal byte pushes and contiguous word copies to check those semantics. Its capacity columns and optional deterministic slot histories apply only to that external-vector comparison. The measured native one-buffer writer reserves the existing n-byte output and has no second vector. [External-vector diagnostic histories](staging.csv.xz)

The verifier joins two independently preserved tapes on block ordinal, stream, coded length, retained rows, edge bytes, word IDs and marked visit counts. It reconstructs all four schedules, matches all 209,697,662 admitted coded bytes, independently expands RLE1, and compares 209,691,304 expanded bytes with the libbzip2 corpus oracle. On every permutation through seven rows, every mark and each real or invalid second seed, a separate check exercises small periodic and off-cycle behavior. It passed 949,938 seed/mark/budget cases, including 354,012 foreign-cycle pairs and 296,341 partial final repetitions. [Real comparison](real-check.txt) · [Small exhaustive check](small.txt) · [Input and source hashes](manifest.txt)

```sh
d=research/astra-e/two-seed/geometry
root=research/astra-e
python3 "$root/edge-displacements/tape.py" restore "$root/edge-displacements/edges.select.xz" /tmp/astra-edges.bin
xz -dc "$root/edge-spellings/words.bin.xz" > /tmp/astra-words.bin
python3 -c 'import bz2,shutil; z=bz2.open("corpora/wikidata.bz2"); o=open("/tmp/astra-raw","wb"); shutil.copyfileobj(z,o)'
rustc --edition 2021 -C opt-level=3 -C overflow-checks=yes "$d/census.rs" -o /tmp/astra-two-seed
/tmp/astra-two-seed --small | cmp - "$d/small.txt"
nice -n 16 /tmp/astra-two-seed /tmp/astra-edges.bin /tmp/astra-words.bin docs/journal/2026-11-18-astra-d-memory/evidence/corpus-routes.csv /tmp/astra-raw /tmp/astra-staging.csv /tmp/astra-bounded.csv > /tmp/astra-panel.csv
cmp /tmp/astra-panel.csv "$d/panel.csv"
cmp /tmp/astra-bounded.csv "$d/bounded.csv"
python3 "$d/report.py" | cmp - "$d/summary.json"
```
