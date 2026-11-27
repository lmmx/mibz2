# Synthesis, third pass: the fast-worker cliff is the oracle's, and the pooled scan conserves work

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), the third pass of 26 November. It follows [the first](README.md) (#954) and [the second](second-pass.md) (#965). Analytical only: no new runs, no `src/` edits, no frozen gate reopened (C2). The tables below were re-derived from the pooled-scan bundle's `runs.jsonl` (evidence `c71c4a56`, tarball SHA-256 `97e397ad…8867`, checked), 96 traced decodes, steady-phase activity sums, medians of n = 3 per cell (within 1–2 ms of #952's figures); walls are the timed medians in `pooled-scan/results/report.txt`. **(derived)** marks arithmetic done here.*

## What was read

- **Deltas since #965:** `delivery-copyout` (#966, read by neither earlier pass), `floor-and-pooled-scan-publish` (#972), `bottlenecks-floor-correction-outline` (#969) and `-publish` (#975), `smooth-striated-trajectories` (#970), `lexicon-and-cap-clarification` (#971), `tooling-review-discoverability` (#973), `event-stigmergy-shadow` (#976), `harden-frontier-tooling` (03994d6), `frontier-first-day-feedback` (#967), `blog-publishing-authority` (#964).
- **Primary:** `pooled-scan/results/{README.md,report.txt,pooled-scan.tsv}`; `ideal-parts/results/floor/README.md`; `delivery/README.md` §0–§3; `2026-11-26-explore/cold-store-and-rle1-llc.md`; `src/rle.rs:103–170`; `src/python.rs:121–139`; `surface.tsv`; `dead-ends.tsv`; every claim at `idea`.

## 1. The 8–10-worker R + D cliff in the fast arms is the C oracle's cold store

The first synthesis (#954, relation 1) read `none → PC → SC` at k = 10 as "faster workers blow up `copy_out` and `A9` together". The `S` arm is the control it did not use: every worker busy, construction real and warm.

| k | arm | wall ms | A9 (sum over workers) ms | copy_out ms | ready_bytes hwm MiB | ready_bytes time-average MiB |
|---:|---|---:|---:|---:|---:|---:|
| 8 | none | 192.9 | 102.8 | 33.5 | 6.9 | 0.60 |
| 8 | S | 184.9 | 95.4 | 33.5 | 6.0 | 0.63 |
| 8 | SC | 135.3 | 117.8 | 38.9 | 8.6 | 1.01 |
| 10 | none | 197.6 | 131.3 | 43.1 | 14.6 | 1.90 |
| 10 | C | 187.2 | 124.1 | 38.4 | 12.9 | 1.14 |
| 10 | S | 169.6 | 127.7 | 47.8 | **16.3** | 2.25 |
| 10 | SC | 147.0 | **213.4** | **76.7** | **16.3** | 4.33 |

- **S and SC share the same ready high-water mark (16.3 MiB) at 10,** yet A9 is 127.7 against 213.4 ms and copy_out 47.8 against 76.7. The C arm (cold store, workers idle 72 ms each) is not inflated either. The blow-up needs the 120.6 MiB C store (`oracle_store.C.bytes`) streamed while every worker is busy: the same mechanism #952 found in D and E_consume.
- **The S arm keeps scaling** (184.9 → 174.5 → 169.6 ms at 8, 9, 10); SC scales backwards (135.3 → 140.2 → 147.0). The backward scaling is the oracle's.
- **Real R inflation is small.** none's A9 rises 88.4 → 102.8 → 131.3 ms at 6, 8, 10. Even if all that growth were L3 and all of it were removed, it is 14.4 ms of CPU at 8 (1.8 ms of wall, 0.9% of 192.9) and 42.9 ms at 10 (4.3 ms, 2.2% of 197.6) **(derived)**. Part of the 8→10 growth is oversubscription (11–12 threads on 10 cores), not L3: the scanner's own A2+A3, a few-KiB table, rises 7% (177.5 → 189.6 ms, report.txt). In S the same bound is 42.9 ms at 10 (127.7 − 84.8), 2.5% of 169.6. copy_out is off the critical path at every count (read_wait 113–166 ms).
- **The ready queue is mostly runts.** Its time-average is 0.3–4.3 MiB, 4–17× below its high-water mark; it holds 4.4–16.8 blocks on average, so a ready block averages 70–260 KB against 879 KiB for a full block **(derived)**. A full block is drained within 0.1–0.8 ms (byte-weighted residence). At k = 8 the per-worker live set (out 0.86 MiB, Built ≈ 0.52 MB, symbols ≈ 0.12 MB, runs ≈ 0.07 MB per block, from the oracle store sizes ÷ distinct blocks) is ≈ 1.0–1.6 MiB, 8–13 MiB for 8 workers, against a 0.6–1.0 MiB ready average **(derived)**. If L3 binds, the workers' own footprint is the larger term.
- **Consequences, before data (C2 intact):** `delivery-l3-working-set`'s shipped gate (≥ 2.0% at 8–10) is at or above the whole of R's inflation (≤ 0.9% at 8, ≤ 2.2% at 10, before oversubscription is taken out), and its fast-worker gate (≥ 4.0%) was calibrated on SC; in S the bound is 2.5%. `warm-paired-oracle-k1` (b) should read the S arm, not SC, for S-paired tests. `single-pass-rle1-crc`'s 8–10W branch quotes PC/SC A9 (191/216 ms); its 1-worker branch stands.

## 2. The pooled scan conserves work: P(k) ≈ S(k) + scan/k

With the traced scan A2+A3 in P (172.9–192.7 ms), the pooled walls are the scan-ideal walls plus the scan spread over k workers, to within 1.5% at k = 6–9 on both pairs **(derived)**:

| k | S + scan/k | P | SC + scan/k | PC |
|---:|---:|---:|---:|---:|
| 6 | 267.2 | 268.5 | 193.0 | 196.0 |
| 8 | 207.1 | 207.7 | 157.5 | 158.1 |
| 9 | 194.8 | 196.7 | 160.5 | 159.9 |
| 10 | 188.9 | 187.2 | 166.3 | 161.0 |

- **So K1's capture has a closed form:** capture = 1 − (scan/k)/(C − SC) = 1 − 22.2/47.9 = 0.54 at 8, against the measured 0.52.
- **A re-test forecast needs no pooled run:** P′(8) ≈ S′(8) + 22 ms, from the S arm that `mtf-fold-build`'s S-paired registration measures anyway. With none′(8) ≈ max(scan, S′) + 8 ms (the measured coupling at 8), the built pooled scan beats none by 3% only if S′(8) ≤ ≈ 158 ms. The second pass projected 152–161 ms after the fold and forward emission: the edge.
- **The hybrid's ceiling** (scanner plus k workers, scan spent only from slack) is the work bound (k·S + scan)/(k + 1): 184.1 ms at 8 today, 4.6% under none and barely under S itself. It is ≈ 155–163 ms if S′(8) = 152–161, 9–13% under none **(derived)**. The hybrid is worth a registration only after the worker levers land, as its claim already says; this gives the number.

## 3. RLE1's four passes run on 13.7% of blocks

#952 traced R's cost to `unrle_in_place`'s four passes "whenever shift > 0". #966's census (delivery/README.md §2, block geometry) finds shift > 0 on 59 of 431 wikidata blocks (13.7%, max 193 B). The other blocks with runs take three passes and one write (`find_runs`, the compacting `copy_within`, the tiled CRC). Removing the shift-up pass (a read and a write: two of the six full-buffer traffic units on those blocks) saves at most a third of A9 on ≈ 25% of the full blocks (if those 59 are all full, 59 of 233): under 9% of A9 **(derived)**, far from `single-pass-rle1-crc`'s 35% 1-worker gate. Only fusing `find_runs`, the compaction and the CRC into one tiled pass can reach it.

## 4. An orphaned step

#966 found `src/python.rs:121–139` zeroes a fresh `Vec` of `size` bytes, copies into it, then copies again into `PyBytes`: three passes per Python `read()`. No claim owned it. Opened `python-read-single-copy`.

## Dead ends re-checked

Row 7 (memory as a third station at 8–10, not supported) is **reinforced**, not revived, for real code: the backward scaling seen since #952 lives in oracle arms. Row 6 (pooled scan as built, KILL) stands; §2 makes its revival condition a number. No other row is touched by the new results.
