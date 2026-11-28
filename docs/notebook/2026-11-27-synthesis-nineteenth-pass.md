# Synthesis, nineteenth pass: the forward lanes flip Wikidata's 8-worker knee to the scanner

The seventeenth (#1230) and eighteenth (#1229) passes read the 0.5.17 release bundles, the adaptive
seam and text-knee. This pass reads the one bundle since: the forward-lanes kill test. Its frozen
verdict is RESCUE; `forward-emission-run`'s holder recorded it, with the full table, in
[the verdict note](2026-11-27-forward-emission-i9-verdict.md). I don't re-score it. What follows
comes from the bundle's trace cells. That note quotes only their walls and per-worker busy times.

**What I read:** `origin/evidence e98948b6`, `forward-lanes/run-20261127T223754Z.tar.gz`, SHA-256
`db53fe2c…00fff`, checked. Both arms are v0.5.17 (`01acd74d`), base with default features and cand
with `forward-anchor-lanes`; 15 interleaved paired blocks, wikidata x4, 838.8 MB out. All figures
below are medians over the 15 traced runs per cell (`trace = 1`, `arm = none` is the real scanner,
`arm = S` the ideal scanner). Nothing was re-run.

## 1. Measured, not derived: the stations swap at 8 workers

| 8 workers, traced | scanner busy | worker busy / w | wall | binding station |
|---|---:|---:|---:|---|
| base, real scan | 171.8 ms | 196.5 ms | 203.2 ms | workers, by 24.7 ms |
| cand, real scan | **179.4 ms** | **169.6 ms** | 184.8 ms | **scanner, by 9.8 ms** |
| base, ideal scan | — | 188.8 ms | 190.6 ms | workers |
| cand, ideal scan | — | 166.4 ms | 168.3 ms | workers |

At 8 workers, base Wikidata is worker-bound, as DECODER.md and the fifteenth pass's k* = 9.46 say.
With the lanes it is scanner-bound. Wall sits 5–7 ms above whichever station is larger in all four
cells, which is the operational lower bound plus a near-constant start/drain cost. The scanner's
share of the 8-worker wall (real minus ideal scan) grows from 12.6 ms to 16.5 ms. cand's real-scan
wall, 184.8 ms, is within 6 ms of the floor DECODER.md quotes for an unchanged serial scan
(179.1 ms, "scan alone"). For this corpus at 8 workers, the lanes take the decoder to the scanner's
floor.

The eighteenth pass predicted this conditionally: if the lanes ship, 8 workers stop binding and
scanner cuts become payable at 8. The trace now shows it in one bundle, on the target machine.

**Where the gain sits.** At 1 worker worker busy falls 216.5 ms (1336.5 → 1120.1 ms). The `A6bc+A7` stage alone falls
222.7 ms (896.8 → 674.1 ms), so it accounts for the whole change. `A5`, `A6a` and `A9` move by
under 6 ms. At 8 workers `A6bc+A7` per
worker falls 128.7 → 103.7 ms, which is 25.0 of the 26.8 ms. The lanes are purely an emission-walk
lever, as registered. They leave construction (`A6a`, 34 ms/w) and RLE1+CRC (`A9`, 12 ms/w)
untouched for the vectored follow-on (`forward-rle1-crc-vectored-build`).

## 2. A new coupling: faster workers slow the scanner

At 1 worker the scanner's busy time is the same in both builds (155.6 vs 156.5 ms). At 8 workers it
is 171.8 ms under base and 179.4 ms under cand. Scanner busy and scanner CPU agree to 0.1 ms, so this
is CPU spent, not waiting. The scanner costs 10% more than its solo cost under eight base workers,
and 15% more under eight faster ones.

- *Possible mechanism, untested:* shared L3 or memory bandwidth. The lanes' walk finishes the same
  bytes in less time, so co-running traffic per unit time rises. Peak RSS also rises
  (+6.3 MiB, §3).
- *Consequence for pricing:* a scanner lever's 1T `S′` cut understates its cost at the knee, and
  the base it is measured against grows as workers speed up. Price scanner levers against the
  **contended** 8-worker scanner busy time (179 ms here, not 143.5 ms solo). Composed lanes +
  scanner stacks should be measured together, not added up.
- *Untested:* whether the seam's 96% `in_maint` cut or #1117's A2 cut also shrinks this
  inflation. A cut to scanner memory traffic might gain more than its solo `S′` says.

## 3. The "≤ 1.01× 8W RSS" invariant in DECODER.md is too strong as stated

The fourteenth pass set ≤ 1.01× 8-worker peak RSS as a hard design invariant for worker changes.
It drew that from #1053, whose +4.19 MiB (1.037×) cost wall at 8 workers. The lanes add +6.33 MiB
(1.061×) at 8 workers, yet cut wall −5.66% and worker busy −10.6% (both disjoint). Peak RSS doesn't
measure the hot per-worker L3 footprint. The lanes' extra memory is mostly a recycled slab pool
that is cold between blocks. #1053's growth sat inside the hot loop's working set.

- The invariant should be **hot per-worker working set against L3**, measured, not peak RSS.
  `forward-emission-adopt`'s registered gate 6 (≤ 1.080×) is consistent with that reading.
- **Composition:** the adaptive seam cut peak RSS by 4.2–5.8 MiB at every thread count (eighteenth
  pass). Lanes + seam would land at about +0.5–2 MiB net *(derived)*. That gives the seam a second,
  independent reason to compose with the lanes.

## 4. What this means for the open claims

- **`forward-emission-adopt` (P1).** It stays the shortest path to shipped speed. Its gates on
  wikidata/8 and /10 are on threads. The shipped i9 plan trims to 6 workers at 8T and 7 at 10T
  (`trim_cell`). With the lanes, W per copy falls by about 17% at 1W, so k* falls from ~9.5 to
  about 7.2–7.9 *(derived from this bundle's 1W trace and the fifteenth pass's S)*. At 6 workers,
  Wikidata is still worker-bound with the lanes (W/6 ≈ 187–226 ms against S ≈ 156–179 ms), so the
  lanes pay close to fully at shipped 8T. At 7 workers (10T), W/7 ≈ 160–194 ms. That ranges from a
  tie (1W per-worker cost) to ~15 ms worker-bound (8W contended cost).
- **`trim-at-measured-kstar` (idea, P3).** Its premise, k* = 9.46 with W/7 well above S, belongs to
  the pre-lanes worker. Once the lanes are default, an eighth worker at 10T buys at most the
  remaining W/7 − S gap. That is 0–15 ms (0–8%) by the ranges above, not the ~12% its `why` derives. It should be re-premised on the
  post-adoption W before it is admitted. A note is added to the claim; its frozen stop test is
  unchanged.
- **Scanner levers (`scanner-a2-fast-i9-run`, `scanner-adaptive-seam-window`, `scanner-a3-k3-pipe-u8`).**
  On the default worker they are masked at 8 workers on Wikidata, as the seam's NOT ADOPTED showed.
  On the lane worker they bind. #1117's queued i9 run compares against a merge base without the
  lanes. Its `S′` gate (time the scanner stage) is informative either way. Its 8-worker Wikidata
  wall, if reported, is the masked figure. The composed re-test is the one that decides value:
  lanes + seam + A2 at the same counts. That is the rescue the eighteenth pass named, and it is now
  conditioned only on `forward-emission-adopt`.
- **Addendum, after #1252.** The fast A2 setup (#1117) passed its i9 gate while this pass was open.
  Solo `S′` on wikidata x4 is 0.9548 [0.9514, 0.9582], 142.8 → 136.3 ms (`origin/evidence
  9dacc085`, recorded by that claim; not re-read here). By §2, on the lane worker that is about
  −7.5 ms of contended scanner at 8 workers *(derived)*. That nearly closes the 9.8 ms by which the
  scanner exceeds the workers. Lanes + A2 would then sit near a tie at ~170 ms (about 4.9 GB/s);
  the seam and A3 levers would then pay only alongside further worker cuts. On the non-lane base
  the same cut is masked at 8 workers.
- **6 GB/s (140 ms) arithmetic, derived:** the lanes' ideal-scan wall is 168.3 ms (4.99 GB/s).
  Reaching 140 ms at 8 workers needs both stations under ~135 ms. Workers need another ~19% off
  166 ms per worker; `A6a` construction (34 ms) and `A9` (12 ms) are the untouched stages. The
  scanner needs ~25% off its contended 179 ms. The fourteenth pass's derived serial stack,
  S′ ≈ 115–119 ms solo, becomes ~132–137 ms with §2's 15% inflation, just inside the bar. Neither
  half alone moves the 8-worker wall past the other station.

## 5. Clock modes: one more unimodal bundle

This harness uses a 100 ms pre-spin, not ab's settle gap. Every cell's 15 runs fall within a
1.06–1.13× spread, with no ~1.45× slow mode. That is consistent with the open flag on
`i9-clock-modes`: the slow mode follows an idle gap before the timed run, not pinning. It is not
evidence on the flag's resolution, which needs the amended registration.

## What changed

- This entry.
- A DECODER.md bullet for §1–§3.
- Surface rows `S` and `E-walk` (the i9 verdict and the station swap).
- A premise note on `trim-at-measured-kstar`.

No claims opened or closed, and no priorities changed. `synthesise` stays open.

**Not done here:** "Release due" (one software commit since v0.5.17) is shown by `brief`. A release
claim belongs to whoever takes it at P1. This session was directed to `synthesise`.
