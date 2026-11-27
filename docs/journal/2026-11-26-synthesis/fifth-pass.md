# Synthesis, fifth pass: the production point is a tie, and the scanner's plumbing was already measured

*Recurring claim [`synthesise`](../../../frontier/claims/synthesise.toml), 27 November. It follows the
[fourth pass](fourth-pass.md) (#1002). Analytical only: no new runs, no `src/` edits, no frozen gate
reopened (C2). Tables 1 and 2 were re-derived here from the pooled-scan bundle's `runs.jsonl`
(evidence `c71c4a56`, tarball SHA-256 `97e397ad…8867`, checked): walls are medians of 15 timed decodes
a cell, thread states are medians of the 3 traced decodes, summed over all phases. Table 3 is from the
two scanner-plumbing bundles (evidence `78a85fc4` and `33568fad`, both on `origin/evidence`) and the
journal's §77 and §79. **(derived)** marks arithmetic done here.*

## What was read

- **Deltas:** `huff-bmi2-dispatch` (#1012), `computational-permutation-structure` (#1016, open at
  6ad3c5de), `bwt-digram-return-map` (#1013), `mtf-fold-build` and its integration delta (#968/#1010),
  `worker-l3-footprint` (#1006), `warm-paired-oracle-k1`, `explore-scanner-dissection` (#1011),
  `mtf-fold-census`, `constructor-rescue` (#1015), `research-persistence`, `swarm-not-conversation`.
- **Primary:** the results files behind each; `pooled-scan` `runs.jsonl` (thread `states`, `gap`,
  `occupancy`); `scanner-plumbing/run-20261125T025030Z`, `-T033008Z` and `-T064054Z` (`summary.txt`,
  `lazy-summary.txt`); journal `2026-11-24-algorithm-level.md` §61, §62, §77, §79; `src/parallel.rs:1038-1062`
  (`ChannelSource::view`) and the traced build's `src/character.rs` (`InMaint`); open PRs #846, #857,
  #861, #879; `dead-ends.tsv` rows 1-23 (#1016's two included).

## 1. At production's 8 workers the two stations are tied

Production at 10 threads asked builds 8 workers (`UNPINNED_FROM_TEN`, `src/parallel.rs:457-466`).
The pooled-scan run's arms isolate the stations: `C` makes construction free, so its wall is the
scanner's; `S` makes the scan free, so its wall is the workers'.

| k | none wall | C wall (scan-bound) | S wall (worker-bound) | none − max(C, S) | scanner A2+A3 | scanner `in_maint` | scanner `slot_wait` | worker `job_wait`/w |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 6 | 242.3 | 184.9 | 238.7 | 3.6 | 171.5 | 9.8 | 55.8 | 3.4 |
| 7 | 212.0 | 184.0 | 207.1 | 4.9 | 170.7 | 10.4 | 24.6 | 4.0 |
| **8** | **193.0** | **184.4** | **185.0** | **8.0** | 170.8 | 11.2 | **4.1** | **8.5** |
| 9 | 195.3 | 186.4 | 173.1 | 8.9 | 171.0 | 12.8 | 7.9 | 20.5 |
| 10 | 198.7 | 187.5 | 169.5 | 11.2 | 189.7 | 12.8 | 4.3 | 42.5 |

(ms; `in_maint` is the input window's drain and re-copy, `ChannelSource::view`, and it grows with k as the workers contend for memory.) **(derived)**
- **The tie:** at 8 the scan-bound and worker-bound walls differ by 0.6 ms (0.3%). Both stations
  wait on each other: the scanner is blocked by the window for 4.1 ms (`slot_wait`) and each
  worker is starved for 8.5 ms (`job_wait`). The 8.0 ms over the max is that two-way loss.
- **Consequence, a cap for every single-station lever at 8:** a scanner-only lever can take the wall
  from 193.0 down to S's 185.0 at most, and a worker-only lever down to C's 184.4. Each is capped at about
  8 ms (4%) realised at 8, whatever its size. On one station, a 15% lever and a 5% lever look
  the same there.
- **Where each station binds:** the scanner binds from 9 (S falls to 173.1 while none stays near 195)
  and the workers at 7 and below. At 10 the scanner itself slows: A2+A3 rises 11% (170.8 → 189.7),
  the core-sharing mixture of §62/§78.
- **The registrations are already right:** `mtf-fold-run` judges FS/S, scan-free; `huff-bmi2-build`
  judges scan-phase time, with 8W as a no-regression guard only. Its `why` line ("~6.4-7.8% of the
  8W wall") holds only once worker wins have taken S′(8) below the scanner. Today's realised 8W
  ceiling is about 8 ms.

## 2. The law was already tested on the i9, by a fix that is now stranded

Journal §62 (Neon's course mining, verified) found two scanner plumbing costs:
- `ChannelSource::view` drains and re-copies the input window on chunk boundaries, on the scanner
  thread. This is `in_maint` in table 1: 11.2 ms at 8, 12.8 at 10, still in master at
  `src/parallel.rs:1050-1055`.
- The symbol buffers are zero-filled again at `syms.resize(max_out, 0)`.

Together they are about 15 ms, 8% of the scanner's demand. Both were built (#846: symbol buffers;
#857: a seam buffer, `src/parallel.rs` only; #861: both) and run twice on the i9:

| threads (workers) | #861 wall head/base [95%] (§79) | lazy diagnostic, explicit workers | the §61 max model's prediction **(derived)** |
|---|---|---|---|
| 8 (8) | 0.975 [0.942, 1.009] | 0.991 [0.943, 1.041] | tie: coupling recovery only, ≤ 4% |
| 9 (9), judged | 0.973 [0.936, 1.011] | **0.921 [0.884, 0.958]** | scanner-bound: ≈ 15 ms of 195, 0.92 |
| 10 (7 then; 8 now) | **0.982 [0.966, 0.998] FASTER** | **0.936 [0.889, 0.984]** (10 workers) | 0.93 at 10 workers |
| 12 (7) | **0.977 [0.961, 0.995] FASTER** | — | — |

Both runs were NOT ADOPTED, because the judged count (9 threads) was the noisiest (±4-5%). §79
decided not to rerun that registration. Instead it would re-register #861 after #860 moved
production to 8 workers, judged at production's default counts (10, 12 threads), with its
prediction from the max model. Production has since moved (§76: 8/32 from 10 threads). The
re-registration was never written: the coordinator was retired on 26 November, and #846, #857 and
#861 are among `brief`'s unclassified legacy PRs. Two cross-delta consequences:
- **The transfer coefficient is measured, not projected.** 8% of scanner demand gave 7.9% and 6.4% of
  wall where the scanner binds (explicit 9 and 10 workers), but 0.9-2.5% at the tie (8). Every scanner
  projection on row S (BMI2 "11.4-13.8 ms", coalesced "−10.8% of S") should be read through table 1's
  caps, not as wall.
- **#1011's lever 3 (no re-zeroing of `syms`) is #846.** It has two i9 runs behind it. The open
  `scanner-coalesced-u8-decode` neither cites them nor the batched-bits run (#879,
  `scanner-plumbing/run-20261125T064054Z`: 0.978-1.001, null). That run tested the same
  `live -=` line #1011 rewrites, by a different mechanism (batching, not the CSE break). Its
  registration should cite both, and treat its no-zero lever as a re-test of #846 under a new
  registration, not a fresh find.

## 3. One statistic under three names: the singleton run

Three probes counted the same object:
- **`mtf-fold-census`:** 71.67% of maximal runs have w = 1; 95.01% have w ≤ 16.
- **#1013:** 76.45% of thin runs are w = 1, so 5.548M rows, 42.54% of m. That puts the thin runs
  at ≈ 0.557 m **(derived)**.
- **#1016/#998:** `P_1(T)/m = 0.552`, the 1.8-row pieces.

They agree: 0.7167 / 0.9501 = 0.754 ≈ 0.7645, and 0.557 ≈ 0.552. **(derived)**

The singleton is paid twice: in construction, as OutQ16's 32-byte iota per run (61.3% of A6a, 8.41×
write amplification; a w = 1 branch mispredicts, −10.9%), and in emission, as Open's 82% literal
visits. It is also what voids every contraction (#1013 horn 1, #1016's obstruction).

The fold (#968/#1010) is the one shipped-path pass at run granularity. In its sink a run's LF
target interval, `[C[c] + occ[c], +w)`, is a counter, which narrows #1013's horn 1 ("bands before
A6c do not know target ranks"): the ranks exist at A6a time. That does not remove the
`K → E⁺ → K` chase: a per-run pass is ≈ 0.56 m units. On #1013's VM, at its 4-way chase floor
(37.5 ms / 13.5M ≈ 2.8 ns), that is ≈ 20 ms. **(derived)** That is most of the rescue brief's
25-30 ms incremental budget before any word assembly. Work at run granularity pays only if, like
OutQ16 in the fold (−46%), it hides behind MTF's serial chain. This is a pointer for
`bwt-contraction-constructor-rescue`, not a claim.

## 4. After everything in flight, the stations tie again

Taking row S's and the fourth pass's projections at face value, both stations move together:
- **The scanner:** BMI2, the coalesced u8 levers and the plumbing take it from ≈ 185 to ≈ 145-155 ms. **(derived)** That is −10.8% of A2+A3 for the coalesced levers, −4.5% more for BMI2 (#1011's native arm) and ≈ 10 ms of `in_maint`.
- **The workers:** the fold and 4-lane forward emission take S′(8) to 135-153 ms.

The two stay within one coupling term of each other, so the 8W wall lands near max + ≈ 8 ms ≈
153-163 ms, not the ≈ 140 ms milestone. The tie's two-way loss (table 1, 4.3% of the wall) is then
the largest unowned term, and it is a queueing property (window, block-size variance, §62's
autocorrelated sizes), not a kernel. The pooled-scan hybrid's ceiling formula already prices it
for one station; nothing prices it for two balanced ones. No claim yet: it needs the stations to
move first, and table 1 gives its size today.

## Dead ends

Rows 1-23 stand. The two rows added by #1016 are consistent with relation 3 (the 1.8-row pieces are
the singleton runs). None is revived. The scanner-plumbing NOT ADOPTED verdicts were never
dead-ended: §79 deferred them to a re-registration, which `scanner-seam-window` now carries for #857.
