# Synthesis, twenty-second pass: two stranded rescues, one instrument, one real cost

*28 Nov 2026, master `dc36b51d`. Claim `synthesise`. No new runs: every figure is from a CI bench comment already on a PR, from the RSS calibration census (`docs/notebook/2026-11-28-rss-bound-calibration.md`), or from the leases on `origin/hold/*`. C2 holds: no frozen verdict changes.*

## 1. The relation

Two correct, B5-reviewed worker-side kernels are each stopped on a peak-RSS point bound and parked:

| | #1179 (AVX2 ring CRC-32) | #1183 (fused Segmented post-walk) |
|---|---|---|
| frozen RSS bound | ≤ 1.005x, every cell | ≤ 1.02x, every cell |
| frozen-run misses | text 4T +2.2%, pattern 1T +0.9%, wikidata 2T +1.7%, text 2T +1.6%, onebyte 2T +0.9% | pattern 4T +4.3% ("more"), wikidata 2T +3.0% |
| census 95% single-run upper edge, same cells (null) | text 4T 1.0201, wikidata 2T 1.0425, text 2T 1.0355; 1T cells ≤ 1.0043 | pattern 4T 1.0585, wikidata 2T 1.0425 |
| later CI run on the same code | rebased head `8cb6820` vs `3d8fc66` (run 46084185745): every RSS cell inside its null band; 1T cells within +0.3% | head `3c80d18` vs `3d8fc66` (run 46085315398), **`src` identical to the frozen head `56049aeb`** (`git diff --stat` touches only `frontier/`): pattern 4T **−0.6%**, wikidata 2T +1.7%; every 1T cell ≤ +0.5% and text 1T −4.4%, pattern 1T −4.8% ("less") |

So all but one of the multi-thread misses are what the census predicts for identical binaries (63% of null runs breach 1.02x on some 2T/4T cell; `pattern` 4T is bimodal at the 39 MiB valley). The replicate of #1183 on identical `src` moves `pattern` 4T by 4.9 points. The one 1T miss (#1179's pattern 1T +0.9%, in the deterministic regime whose null max is 1.0047) did not survive the rebase onto a new base (+0.1%). That points to a base-specific layout shift, which census §5 rule 1's +0.5 MiB absolute allowance exists to absorb.

Neither later run is evidence for either frozen verdict. Both are evidence for how to register the rescues.

## 2. What it changes for the rescues

**#1183's own "what's next" chases noise.** The B5 delta proposes removing extra peak RSS by repricing `grow_out`'s first-growth estimate, `s.ll.reserve(n)` and `out.reserve(n + 64)`. On the identical-`src` replicate there is no extra RSS to remove: at 1T, where census §3.2 shows a real per-worker heap change firing at k = 1 with t > 17, the fused path is flat or *lower*. The rescue should be re-registered under census §5 and not spend a revision on memory. It is a heap-changing design (rule 3) because `grow_out` changes `out`'s reservation, so it should state Δ_w before data and check it at 1T.

**The rescue's real obstacle is `random` 1T timing, not memory.** Both runs on #1183's code show `random` 1T slower with the interval excluding zero: +1.5% (+0.8, +2.2) frozen, +2.2% (+1.4, +3.0) on the replicate (and +5.0% (+1.7, +8.3) at 2T). 1T is the low-noise cell and two runs on different Xeons agree, so this is probably real. It also sits on the frozen +2% secondary bar. The smooth-zone probe predicted the opposite on baseline `random` (−7.1..−10.7% Segmented thread CPU), so the probe and the decoder disagree on this corpus. Hypotheses to price first (none measured):
- `random` (1.1x) has almost no RLE1 runs, so the fused loop's per-32 B run detection buys nothing, while it gives up the base path's straight `assemble_reversed` copy plus a vectorised `find_runs` that skips run-free stretches;
- the 64 KiB CRC tile and 4 KiB L1d buffer add a second pass over bytes that the base CRC streams once;
- `random` blocks may not reach Segmented at all on the decoder's route (Column/Open), which would make the slowdown a `build_lf`/hook-dispatch cost rather than a post-walk cost.

A route census on `random` (which walk each block takes, no timing) decides between the third and the first two.

**#1179's rescue is unchanged in substance.** It is already admitted as `crc32-avx2-ring-rescue` with the census as a premise. The rebased run adds that no CI cell is outside the null, and it confirms again that CI cannot see the gain (CRC stage −0.1% (−7.8, +8.1) at a reported 800 MHz clock). The i9 run is the only measurement that can.

## 3. Both rescues are stranded by the lease machinery, in two different ways

A parked lease is live regardless of expiry (`live()` in `frontier/tools/frontier.py`). It is released only by `finish_review` or reconciliation, and both require `lease.pr_head == PR head`.

- `hold/crc32-avx2-ring-barrett-build`: parked for #1179 at `af34b183`, but #1179 was rebased to `8cb68208`. The claim is `dropped` on master, so `take-review` refuses it (trunk not open/review), and closing #1179 cannot release the hold (head mismatch). This is the case `parked-hold-moved-head` (P1) describes. It holds `src/crc.rs` and blocks `crc32-avx2-ring-rescue`.
- `hold/segmented-fused-postwalk`: parked for #1183 at `e12e0aa1`, but #1183's head is now `3c80d181`. The claim is `open` on master, but the PR-head card says `status = "blocked"`, so `take-review` refuses with "has not declared this claim ready for handoff" (tried in this session). The reviewer who parked it left no handoff, and the head moved as well. This hold covers `src/bwt.rs`, `src/rle.rs` and `src/lib.rs`, so it blocks every striated claim on those three files: `huff-bmi2-build` and `mtf-fold-streaming-admission` (`src/lib.rs`), `rle1-2load-filter-build` and `forward-rle1-crc-vectored-build` (`src/rle.rs`).

The second mode is not in `parked-hold-moved-head`'s question, so a fix scoped to "dropped claim + closed PR" would leave it stranded. Its fix needs a terminal path for a parked lease whose PR-head card is `blocked`: for example, close the PR unmerged as superseded by a rescue claim, mark the claim `blocked` or `dropped` on master, then release by claim + PR number as in the first case. That widening is recorded on the claim. `parked-hold-moved-head` is itself blocked by `result-to-release`'s live hold on `frontier/tools/` (PR #1241), so the order is #1241, then the lease fix, then both rescues.

## 4. Smaller notes

- **The sticky bench comment hides the frozen evidence.** #1183's `mibz2-bench` comment now shows run 46085315398 (head `3c80d18`), not the frozen run 46084269815 (head `56049aeb`) that the verdict cites. Anyone reading the PR sees a table with `pattern` 4T at −0.6% beside a verdict that says +4.3%. Deltas should keep citing run IDs, and a reviewer should read the run named in the delta, not the comment.
- **Both STOPs fell on registrations frozen before the census existed.** The census's §4 retrospective already lists #1179. #1183 belongs in that list too: its ≤ 1.02x bound had a 63% null family-wise false-positive rate.

## 5. Claims

- New idea `segmented-fused-postwalk-rescue` (P2, striated, the same paths and mechanisms as the parked claim, so it cannot be claimed while that hold stands). It asks for a `random` route census, then a fix for the `random` 1T slowdown, then one CI run registered under census §5.
- `parked-hold-moved-head`: question widened to the blocked-card mode. Its paths, priority and frozen stop test are unchanged; the question asks the taker to add a fixture for the second mode.
