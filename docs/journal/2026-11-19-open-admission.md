# Where the open kernel wins, block by block: the admission for the B5 port

*19 November 2026. The Pathfinder.*

- **The occasion:** CHECKPOINT 7004759994 on #314, which dispatches the B5 port of ungated
  open for 0.5.7. Measured before any i9 timing, to set the port's admission.
- **What it decides:** nothing in production. It is the evidence behind the port's admission,
  which the C6 registration fixes and the i9 run tests.

**The finding:** the open kernel is faster than production only on the blocks LazyCells admits.
On every block LazyCells refuses, it is slower, by up to 5×. Both of LazyCells' refusals that
matter here are its first two gates, which read only the record count before any allocation:
- **the density line,** r/n ≥ 0.15, which refuses text;
- **the sparse gate,** r/n < 1/64, which refuses pattern and onebyte.

So the port should take the open kernel where those two gates admit, and production's path
everywhere else. The constants are the ones LazyCells ships; there is no new line to tune.

## Method

- **The census:** `ab census-timed`, on `bench/open-admission`, with the research feature
  `open-return`.
- **What it times:** every block, one thread, in the cloud container (Intel Xeon, 2.1 GHz),
  twice:
  - by production's route, LazyCells as shipped, its own fallbacks included;
  - by the open kernel, `Open<16, true, true>`, the moonshot `open` arm and bench-open-return's
    ungated arm.
- **How:** the fastest of 3, interleaved, with LazyCells' decision recorded for the block.
- **Correctness:** the two outputs are compared on every block. All 650 agree.
- **The inputs:** the pinned Wikidata sample (82f65ace…), and `ab gen`'s corpora from the
  recalibration (`inputs.sha256`).
- **The data:** `2026-11-19-open-admission/blocks.jsonl`, one line a block.

## Results

| corpus | LazyCells' decision | blocks | rows | production | open | open ÷ production |
|---|---|--:|--:|--:|--:|--:|
| wikidata | admitted | 228 | 205.2 M | 365.5 ms | 355.2 ms | **0.972** |
| wikidata | density | 155 | 4.5 M | 14.3 ms | 14.4 ms | 1.005 |
| text | density | 48 | 42.9 M | 133.0 ms | 679.7 ms | **5.112** |
| runs | density | 2 | 1.0 M | 3.3 ms | 5.3 ms | **1.615** |
| pattern | sparse | 88 | 78.4 M | 117.8 ms | 182.2 ms | **1.546** |
| onebyte | sparse | 2 | 1.6 M | 1.1 ms | 2.1 ms | **1.870** |
| random | column route | 36 | 32.0 M | 123.6 ms | 123.1 ms | 0.996 |
| records | column route | 43 | 38.4 M | 172.5 ms | 172.2 ms | 0.998 |

- **The column route is the same code in both:** production's fallback and the open kernel's are
  both `Segmented`, so random and records are level.
- **Earlier runs agree:**
  - the i9 measured text 1.73× slower under ungated open at 8 and 10 threads
    (bench-open-return, LOG 6992226855);
  - MONODROMY's record 76 §3 says the same of dense, text-like blocks.
- **Never decoded by an arm on the i9:** pattern, onebyte and runs (record 71's B5 checklist).
  Here they lose under ungated open too.

**What this leaves open:**
- *On the i9,* ungated open measured about 2.5% faster than opend, with about 18% less peak
  memory, in the moonshot harness's runs.
- *One thread here,* the two differ only on Wikidata's 155 small over-line blocks (2% of its
  rows), where they are level.
- *So the i9's difference* is not the compute on those blocks. The registration's confirmatory
  run carries ungated open as a context arm, to measure it in the release build.

## The admission, for the port

The open kernel inverts a run-route block when it passes both gates; otherwise production's
path does, unchanged:

    recs.len() * 64 >= n  &&  recs.len() * 20 < n * 3

These are LazyCells' `LAZY_DENSITY` and Cells' sparse gate, as shipped.

## Reproducing

```sh
cargo build --release --features open-return --example ab
target/release/examples/ab census-timed WIKIDATA.bz2 GEN/text.bz2 GEN/random.bz2 \
  GEN/records.bz2 GEN/runs.bz2 GEN/pattern.bz2 GEN/onebyte.bz2 > blocks.jsonl
```
