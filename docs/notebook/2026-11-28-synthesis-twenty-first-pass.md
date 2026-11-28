# Synthesis, twenty-first pass: two clocks, the seam back in the ledger, and the worker side as the long pole

*28 Nov 2026, master `4bb6591`. Smooth zone, no run made. Every figure not quoted from a source is
derived and marked so. The twentieth pass (#1261) landed at ~00:50 UTC; this pass reads what came
after it and prices it against `frontier/DECODER.md`.*

**Read:** `2026-11-28-i9-clock-modes-verdict` (notebook and delta, bundle `d5658cfb`, #1283);
`2026-11-28-explore-scanner-inflation-is-clock` (#1262); `delivery-l3-working-set` (delta and
`docs/journal/2026-11-26-synthesis/delivery-l3-working-set.md`, #1265); `scanner-a2-fast-build-integrate`
(#1117 merged as `15c289b5`); `forward-emission-adopt` deltas (queued, VOID `f170c5c4`, fix #1279);
`scanner-adaptive-seam-window-verdict` (#1259); `rss-bound-calibration` (#1273);
`runs-front-and-fill-build-stop` (#1266); `forward-emission-adopt.toml`'s frozen stop test;
`bench/workstation-queue.txt`; `git ls-remote` for `ready/bench-forward-adopt`; `src/parallel.rs`
(`StreamDecoder::new`, `plan_with`).

## 1. Two different clock effects, and what each does to a comparison

The two results are about different things and should not be merged into one "clock mode":

| effect | source | size | avoidable? | what it corrupts |
|---|---|---|---|---|
| **turbo bin** (active-core count) | #1262: 5.28 GHz at k = 1, 4.91 GHz from k = 4 | ×~1.075 time at k ≥ 3 vs k = 1 | no (Comet Lake cores share one clock) | any comparison *across* thread counts: a 1T S′ against an 8W W/8 |
| **HWP settle hysteresis** | `i9-clock-modes`: pinned & 0.2 s settle explains 13/13 6T slow runs | −12.1% achieved clock, +18..35% wall | yes: 2 s settle (0/48 slow) or 0 s | any comparison *within* a cell whose runs are mixed between modes |

**An independent cross-check from the clock-modes bundle (derived):** its fast-mode cycles per
ref-cycle are 1.907 at 1T and 1.764 at 6T, a ratio of **1.081**, against #1262's 5.28/4.91 = 1.075
from the character and forward-lanes bundles. Two bundles, two harnesses, the same turbo bin.

Consequences:
- Every 1T-measured scanner saving is worth ~1.075× in ms at 8W (#1262), and every 1T S′ must be
  scaled before it is set against a W/8 measured at 8W. That applies to §2 below.
- The settle effect is solved at collection time. `bench-forward-adopt` (the only live queued run)
  freezes 2 s settle and runs unpinned above 1, so its multi-thread cells should be unimodal fast.
- **A third noise source the clock-modes bundle also isolates:** 26 of 144 1T runs are full-turbo
  (`cpr` 1.910 vs 1.907) with identical instructions but CPI +6.9% (`cpb_ratio` 1.03..1.13). They are
  front-loaded: 7 in round 0, none in rounds 9–11. `rss-bound-calibration` (#1273) found round 0 is
  also the CI RSS outlier (1.054x on strict nulls). Two independent censuses point at the same
  object: **round 0 of a fresh-process paired design is a warm-up, in both CPI and RSS.** Future
  registrations should declare a discarded warm-up round before their data. No frozen recipe changes
  (C2); for forward-adopt the 1T gate (≤ 0.900 upper) has wide margin against the lanes' measured
  −222.7 ms of ~1.38 s (~0.84, derived), so the warm-up noise is unlikely to decide it.
- The clock-modes secondary finding (6T pinned to 6 CPUs 0.298 s vs unpinned 0.247 s, −17%: 6
  workers + scanner + reader time-share 6 cores) does not touch the shipped trims, whose pinned plans
  already leave the scanner and reader a CPU each (8 threads → 6 workers). It does mean any recipe
  that pins N workers to N CPUs measures oversubscription, not the shipped state.

## 2. Repricing the 8-worker Wikidata knee with the clock correction and the seam window

Start where the nineteenth and twentieth passes started: the lanes trace cell (`e98948b6`, 8
workers, which is exactly the shipped unpinned 10-thread plan, 8/32): scanner busy 179.4 ms,
workers 169.6 ms/w, wall 184.8 ms ≈ max(S, W) + 5.4 ms. Decompose the scanner by #1262: clock-scaled
1W 156.5 × 1.075 = 168.3 ms plus an **11.1 ms contention residual** (lanes-specific, mechanism
unknown).

Two changes against the twentieth pass's table:
1. Scanner levers measured at 1T are scaled by ×1.075 rather than applied proportionally to the
   contended 179.4 ms. This is ~1 ms more conservative per lever; the residual is held fixed.
2. **The adaptive seam window was missing.** `station_balance`'s S (which #1235's stack ratios use)
   excludes window maintenance; the traced scanner busy time includes it (character's busy
   breakdown lists it as a component, and the nineteenth pass found scanner busy and scanner-thread
   CPU agree to 0.1 ms, so work on the scanner thread is counted). The seam bundle (`d6bc8ca2`, #1259) measured in_maint at 8
   workers 10.36 → 0.41 ms. Maintenance is also the one scanner component #1262 found *growing* with
   k (+3.6 ms by 8). The seam STOPped because 8-worker Wikidata was worker-bound on the non-lane
   base; on the lanes it is a ~10 ms cut on the binding station.

| step (cumulative, derived) | S, 8W | W, ms/w | binds | wall |
|---|---:|---:|---|---:|
| lanes (measured) | 179.4 | 169.6 | S by 9.8 | 184.8 |
| + A2 (#1117, merged: −6.5 ms 1T × 1.075) | ~172.4 | 169.6 | S by ~2.8 | ~177.8 |
| + adaptive seam, composed (−9.95 ms in_maint, i9 8W) | ~162.4 | 169.6 | W by ~7 | ~175.0 |
| + mtf-fold A5+A6a, portable (−12 ms/w, VM) | ~162.4 | ~157.6 | S by ~5 | ~167.8 |
| + k3-pipe (#1235 S13/S1 −6.3% of ~136 ms 1T × 1.075, VM) | ~153.2 | ~157.6 | W by ~4 | ~163.0 |

So the reachable stack is ~163 ms (~5.15 GB/s on 839 MB, derived), and the two sides then stand very
differently against the 140 ms milestone (max(S, W) ≈ 134.6 ms):
- **Scanner:** from 179.4 it needs −44.8 ms. Listed levers give −26.2 ms (seam 10, A2 7, k3-pipe
  9.2). The 11.1 ms residual would bring it to −37.3 ms (83%), leaving ~7.5 ms for the two-lane scan
  or BMI2 work. `scanner-contention-residual` (P3 idea) is therefore the largest unattacked scanner
  term after the lanes: bigger than A2's whole 8W gain.
- **Workers:** from 169.6 they need −35 ms/w. mtf-fold covers 12 (34%). A6a (34 ms/w) and A9
  (12 ms/w) are untouched by the lanes and only partly by the fold. **The worker side is the long
  pole to 140 ms**, the reverse of the nineteenth pass's emphasis, which followed the lanes flipping
  the binding station. P2 worker research beyond the fold has the most headroom per ms.

**The residual's suspects narrow.** `delivery-l3-working-set` (#1265) priced the ready queue's L3
residency out at 8–10W (ready hwm ≤ 8.6 MiB at 8, the cap never binds, bound ~0% on `none`). The
residual's remaining candidates are the lanes' own traffic: the +6.33 MiB cold recycled slab pool
and the workers' DRAM stalls (character: 84% of the whole process's extra cycles at 8W are memory
stalls). The seam's −4.2..−5.8 MiB peak RSS is in the same pool of shared L3, so the composed seam
run (`scanner-adaptive-seam-composed`) is also the cheapest test of whether the residual moves:
trace the scanner at 8W on lanes + seam and compare the clock-adjusted residual with 11.1 ms.

## 3. What forward-adopt will and will not show

`ready/bench-forward-adopt` is at `9875c712` (#1279's fix), **before** #1117 (`15c289b5`). Both arms
build that commit, so the run measures the lanes on a base without A2, as registered. Derived reading
before the bundle returns (not a gate, no C2 change):
- At the 8-worker cell (shipped unpinned `wikidata/10`) the RESCUE trace's own ratio is
  184.8/203.2 = **0.909**; gate 4 is ≤ 0.985 upper. The 8-thread cell's worker count is set by
  `plan_with` and the trims, which this pass did not resolve; read it from the bundle's trace.
- Once the lanes land on master **with** A2, the non-lane base stays worker-bound (S ~164.8 vs W
  196.5), while lanes + A2 reach the ~177.8 ms tie: the 8-worker gain becomes ~0.875 (derived). The
  adoption run understates the merged gain; that is the right direction for a gate.
- On ADOPT, the shortest path to the §2 table is: seam composed rebase (#1144 onto lanes master, a
  fresh stop test, the existing harness), then the fold's streaming admission. Both wait on this one
  run, which makes reading its bundle promptly the binding shared step.

## 4. Smaller relations

- **0.5.18's pre-release gate** now holds #1117 on master. On the non-lane base, the 10T (8 workers)
  cell is worker-bound by ~32 ms after A2 (derived), so A2 should be invisible there and visible only
  at 1–2T; #1246's worker-side −2.9% 8T CPU is what the 8/10T cells should show. That repeats the
  twentieth pass's prediction, now with A2 actually in the candidate.
- **0.5.15's restatement** (`i9-clock-modes` §5): the within-mode 1T change is ~0–2% and 12T ~4.5%
  (or ~1–2% within the fast sub-cluster), against the recorded pooled +5.7% / +9.5%. This is consistent
  with the fifteenth pass's flat 8T/10T scaling (1.002, 0.996), and the changelog edit is in hand in
  another session.
- **`runs-front-i9-rescue` and `crc32-avx2-ring-rescue`** can now freeze RSS rules from #1273's
  calibrated table (1T ≤ 1.010x or ≤ +0.5 MiB; i9 k × Δ_w linearity). Both are worker-side (`M`, `R`)
  and, per §2, on the long pole; the runs front's Intel-only advantage (−2.9..−7.8% VM) is worth its
  i9 run more than its CI STOP suggests, because the i9 is the target and Zen's two shuffle ports were
  the failing mechanism.

## Caveats

- §2 mixes i9 measurements (lanes cell, A2, seam in_maint) with VM ratios (fold, k3-pipe). The seam's
  −9.95 ms was measured on the non-lane base; on lanes the window's work could differ.
- The ×1.075 factor holds compute-bound work; the residual is held fixed through every step, which is
  an assumption (it could shrink as the workers get faster or grow).
- Traced scanner busy is taken to include window maintenance (it runs on the scanner thread, whose
  CPU matches busy to 0.1 ms); if some of it ran elsewhere, the seam row overstates its cut.
- Wall ≈ max + 5.4 ms is the four traced cells' fit, not a law.
