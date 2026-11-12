# B2, the gathered walk: what binds the room walk, and what a gathered tile can buy

**Status:** read-only analysis. Nothing was built, benchmarked or dispatched. Inputs are
recorded CI figures, the shipped walker's machine code (`scratchpad/phases-sel.asm`), and
published instruction tables. The models are `ks_model.py`, `predict.py` and
`window_sim.py` beside this note.

**Boundaries:** the walk alone (`room_segments`, assembly excluded), c/B of output at the
run's reported clock unless marked ns/B. One copy, unless marked "SMT pair" (two copies on
the siblings of one core, CI's four-copy runs).

**Labels:**
- **measured:** a recorded figure, cited;
- **deduced:** follows from measured figures, the code or a published table;
- **model:** an estimate, its assumptions listed.

## 1. Summary

1. **The target, on one consistent run** (deduced; CI run 40579732655, EPYC 7763: room walk
   4.49, stage total 10.78):
   - −20% decoder time needs the walk at ≤ 4.49 − 0.2·10.78 = **2.33 c/B** (0.52× today);
   - +20% throughput needs ≤ 4.49 − 10.78/6 = **2.69 c/B** (0.60×).

   Both hold the other stages and the assembly fixed.
2. **What binds the shipped walk** (§4):
   - No single bound fits.
   - With documented Zen 3 sizes, a pure miss-buffer cap, a pure window cap and pure issue
     predict 1.61, 2.23 and 2.47 c/B, against 4.49 measured.
   - A step-level out-of-order simulation with those sizes also gives 2.47.
   - What fits is the first-order interval model, issue time plus a window-limited miss
     penalty, w = issue + ℓ·U/R, with ℓ from the plain chain and nothing fitted. It gives:

     | Figure | Model | Measured | Error |
     |---|--:|--:|--:|
     | room walk, one copy | 4.70 | 4.49 | +5% |
     | room walk, SMT pair | 9.39 | 9.40 | 0% |
     | flagged walk, one copy | 5.84 | 6.10 | −4% |
     | flagged walk, SMT pair | 11.7 | 13.03 | −10% |

     It also reproduces the Zen 4 lane curve and its SMT-pair convergence within 2–16%.
   - The "per-core miss cap at the walk's own rate" reading fails the flagged walk and #42.
     The "fitted issue rate" reading fails the hosts' spread and the prefetch gain.
3. **Kernels (µops per byte on Zen 3, from the asm and uops.info):**

   | Kernel | µops/B |
   |---|--:|
   | A, shipped | 14.8 |
   | B, the same 32 scalar chains with deferred flags and tiled commits | 8.0–9.0 |
   | C, AVX2-gathered tiles | 6.1–6.2 |

   The gather alone is 4.88 µops/B, and its 8-cycle throughput floors C's issue at
   1.0 c/B.
4. **Prediction on the consistent run** (model, §6):

   | Hypothesis | B (c/B) | C (c/B) |
   |---|--:|--:|
   | interval model with 24 MABs (best supported) | 3.30–3.42 | 2.53–3.17 |
   | interval model, no MAB cap | 3.13–3.24 | 2.17–3.17 |
   | fitted issue rate | 2.48–2.90 | 1.90–2.04 |
   | per-core miss cap | ≈ 4.5 | ≈ 4.5 |
5. **Verdict under the kill rule:**
   - Under the best-supported hypothesis, C does not reach 2.33. Its floor there is 2.51
     (deduced within the model: gather throughput 1.0, plus ℓ/24 = 1.46, plus replay 0.05).
   - **B2 is not a breakthrough candidate** on the 7763's consistent run.
   - What it plausibly buys:
     - C: a 0.29–0.44 cut in the walk, −12% to −18% decoder time;
     - B, without any gather, about two thirds of that: −10% to −13%.
   - C reaches 2.33 only if the fitted issue reading holds, or at the optimistic corner of
     the no-MAB-cap reading.
6. **The cheapest falsifier** (§8, described, not proposed to run): A, B and C at k = 1, 2, 4
   and 6 on the same built T-vectors in decode order.
   - B/A alone separates the hypotheses: ≥ 0.95 for the per-core miss cap, 0.70–0.76 for
     the interval model, 0.55–0.65 for the fitted issue rate.

## 2. The target, corrected (deduced)

| Run | Walk A | Stage total T | −20% time: A − 0.2T | +20% throughput: A − T/6 | As a ratio to A |
|---|--:|--:|--:|--:|--:|
| 7763, run 40579732655 (used) | 4.49 | 10.78 | **2.33** | **2.69** | 0.52 / 0.60 |
| 7763, PR #114 head | 6.64 | 12.98 | 4.04 | 4.48 | 0.61 / 0.68 |
| 9V74, PR #115 | 4.77 | 10.33 | 2.70 | 3.05 | 0.57 / 0.64 |
| 9V45, PR #118 | 4.18 | 9.17 | 2.35 | 2.65 | 0.56 / 0.63 |

- Run 40579732655's total is the sum of its stages without the assembly row (0.16):
  1.22 + 2.27 + 2.27 + 4.49 + 0.28 + 0.26.
- The economics note's 12.93 / 6.69 for PR #114 differ from the bench comment's 12.98 /
  6.64 (`n1-fourcopy/bench_runs.json`). That is immaterial here.
- The earlier 4.10 bar (from mixing runs) buys about 3.6% on the consistent run, and is
  dropped.

## 3. The shipped kernel, counted (deduced from the asm)

**Source:** `scratchpad/phases-sel.asm`, `room_segments::<32, 1>`. The hot step is at
0x2d240–0x2d273; the per-offset loop is at 0x2d200–0x2d22f. The asm is copied to
`room_segments_32_1.asm` here.

**One step** is 16 instructions and 14 Zen macro-ops:

| Role | Instructions | Macro-ops |
|---|---|--:|
| loads | `mov ebx,[r11]` (lane row), `mov eax,[r15+rbx*4]` (T), `mov rdi,[rsp+r8*8+0x70]` (base) | 3 |
| prefetch | `prefetcht0 [r15+rdi*4]` | 1 |
| stores | `mov [r11],edi` (row), `mov [rdi+rcx],al` (byte) | 2 |
| flag test | `test eax,eax; js` (fused) | 1 |
| link and loop | `mov edi,eax` (eliminated, still a slot), `shr`, 2×`lea`, `add`, `mov r8,r10`, `cmp; jb` (fused) | 7 |

**Per byte:**
- the step, 14 macro-ops;
- the per-offset loop, 14 macro-ops per 32 steps, so 0.44 a byte;
- the residual of the i9's 16.9 instructions/B over the loop's 16.47, about 0.4: passes,
  takes and the tail.

That gives **U_A ≈ 14.8 macro-ops/B**, with 4 load-pipe and 2.03 store operations a byte.

**Zen 3's issue floors** for A:
- dispatch: 14.8/6 = 2.47 c/B;
- AGUs: 6.03/3 = 2.0;
- ALUs: 6/4 = 1.5.

**The flagged walk** (0x3652c–0x3652a, `walk_segments_flagged.asm`):
- 21 instructions and 18 macro-ops a step, assuming `cmp r14,[mem]; jne` fuses;
- 6 load-pipe and 3 store operations a step;
- U_F ≈ 18.4 a byte.

**Assumptions:**
- the CI binary has the same code generation (the same source and baseline x86-64 target);
- eliminated moves take dispatch slots and retire-queue entries;
- the i9's residual stands for Zen's.

## 4. What binds the shipped walk per core

### 4.1 The evidence (measured)

| # | Figure | Source |
|---|---|---|
| E1 | one copy: room 4.49, flagged 6.10, bitmap (12 lanes) 13.25, plain chain 38.55 c/B (ℓ) | run 40579732655, `n1-fourcopy/measured_fourcopy.json` |
| E2 | SMT pair: room 9.40 (×2.09), flagged 13.03 (×2.14), bitmap 25.15 (×1.90), plain 40.27 (×1.045). Per core: 0.955, 0.94, 1.05 and 1.91× one copy | same |
| E3 | the 7763's walk is 1.49–2.33 ns/B over 15 bench runs, while build and MTF stay at 0.65–0.73 ns/B | `n1-fourcopy/bench_runs.json` |
| E4 | the 9V74's walk in c/B is the same in its 2.87 and 3.7 GHz groups (4.51–5.06 against 4.62–5.15), and its build scales with the clock | same |
| E5 | the Zen 4 sweep runner, 32 lanes, L1 prefetch: lanes 8 → 64 give 8.01, 6.36, 5.35, 4.87, 5.00, 5.15, 4.96; flat from about 24 with and without prefetch; L1 prefetch saves 4–7% at 8–12 lanes and 13% at 16 or more. SMT pair: 11.97, 10.60, 10.23, 10.28, 10.37, 10.46, 10.83 | run 40573190665, `2026-10-27-walk-sweep.md` |
| E6 | #42's move walk (41 against 16 instructions a step, a 0.6 MB table): 60–63% faster as one chain, 90–108% slower under the 32-lane room walk (local Intel VM) | `2026-10-27-output-side.md` §4 |
| E7 | #29's LF² (`LF[LF[i]]`, 8 B pairs): +9.3% / +20.7%; its gather pass alone about 10 c/B locally | `2026-10-27-literature-reading.md` |
| E8 | the effective misses in flight are ℓ/w = 38.55/4.49 = **8.6** | deduced from E1 |

**Why #29 lost** (deduced from E7): it added a pass of n random reads, a walk's worth of
misses, to save half the walk's dependent steps, and doubled the table to 7.2 MB. KKP's
published construction is a counting sort. B2 adds no pass and no table, so it does not
share #29's failure. #29 does show that random reads at full MLP still cost several c/B.

### 4.2 The published core parameters

| | Zen 3 (7763) | Zen 4 (9V74) | Zen 5 (9V45) | Source |
|---|--:|--:|--:|---|
| ROB (retire queue) | 256 | 320 | 448 | vendor figures, widely reported; not re-read here |
| SMT: retire queue, store queue | statically partitioned | statically partitioned | statically partitioned (reported) | AMD GDC 2024 deck, "Zen 4 core resource sharing"; hwcooling Zen 5 analysis |
| SMT: load queue, memory request buffers | — | watermarked | — | AMD GDC 2024 deck |
| L1D MABs | 24 | 24 | not found | AMD 56665 rev 3.00 and 57647 rev 1.00, via `2026-10-27-mlp-scheduling-and-run-structures.md` §4 |
| L3 average latency (cycles) | 46 | 50 | — | same |
| dispatch (macro-ops/cycle) | 6 | 6 | 8 | AMD GDC 2024 (Zen 4) |
| loads / stores per cycle | 3 / 2 | 3 / 2 | 4 / 2 | AMD GDC 2024 (Zen 4) |

### 4.3 The bounds tested

ℓ = 38.55 and U = 14.8 throughout (model).

| Bound | Formula | Predicts A | E1 level | E2 SMT 0.955× | E3 spread | E5 knee near 24 | E5 prefetch gain | E6 #42 | Flagged/room = 1.36 |
|---|---|--:|---|---|---|---|---|---|---|
| (i) MABs, documented | ℓ/24 | 1.61 | ✗ (2.8× low) | ✓ shared | ✓ | ✓ (24) | ✗ | ✗ | ✗ (equal) |
| (i′) per-core miss cap at A's rate | w_A | 4.49 (fitted) | fitted | ✓ | ✓ | — | — | ✗ (smaller table should win) | ✗: flagged at one copy is below the cap, yet gains 0.94× under SMT |
| (ii) window | ℓ·U/R | 2.23 | ✗ (2× low) | ✓ partitioned | ✓ | ✓ (R/U = 17 on Zen 3, 22 on Zen 4) | ✗ in principle | ✓ | ✓ (1.24) |
| (iii) issue, documented | U/6 | 2.47 | ✗ (1.8× low) | ✗ (headroom to 1.8×) | ✗ | — | ✗ | ✓ | ✓ |
| (iii′) issue rate fitted to A | U/3.3 | 4.49 (fitted) | fitted | ✓ only if the cap is shared | ✗ | ✗ (would flatten near 10 lanes) | ✗ (prefetch adds a µop) | ✓ | ✓ (1.24) |
| (iv) interval model: (ii) + (iii) | U/D + ℓ·U/R | **4.70** | ✓ (+5%) | ✓: both terms halve per thread, 9.39 against 9.40 | ✓ through ℓ | ✓ | partly (an ℓ reduction) | ✓ | ✓ (5.84 against 6.10) |

**(iv) on the other figures** (model; `ks_fit.json`):
- **The flagged walk's SMT pair:** 11.7 against 13.03 measured.
- **The Zen 4 sweep runner,** with ℓ fitted at 32 lanes (54.8 reported cycles; AMD gives
  Zen 4's L3 as 50 on average):

  | | 8 lanes | 12 | 16 | 24–64 |
  |---|--:|--:|--:|--:|
  | one copy, model | 9.31 | 7.03 | 5.89 | 5.00 |
  | one copy, measured | 8.01 | 6.36 | 5.35 | 4.87–5.15 |
  | SMT pair, model | 11.78 | 10.00 | 10.00 | 10.00 |
  | SMT pair, measured | 11.97 | 10.60 | 10.23 | 10.28–10.83 |

  - The model's in-flight limit per thread under SMT is R/2U = 10.8. It reproduces "fewer
    lanes inflate less": 8 lanes ×1.27 (×1.50 measured), 32 lanes ×2.00 (×2.07).
  - It is pessimistic below the knee, where dispatch overlaps the stall. That is a known
    property of first-order interval models.
- **The average misses in flight:** (R/U)·(miss-term share) = 17.3 · 2.02/4.49 = 7.8,
  against Little's 8.6 (E8).
- **Fitted ℓ per runner** (the miss term solved from A):
  - 35.0 on the fast 7763 instance;
  - 73 on PR #114's 7763;
  - 49.7 on the 9V74 (AMD's L3 average is 50);
  - 41–69 on the 9V45 at an assumed true clock of 3.5–4.5 GHz. That reproduces its
    0.92 ns/B, but the reported clock is unreliable there (four-copy note §1.7).

**An apparent fit for pure (ii), rejected** (deduced): ℓ·U/128 = 38.55 · 14.8/128 = 4.46, as
if one copy already ran with a halved ROB (an idle but unhalted sibling under the
hypervisor). But then a second copy would cost little, against the measured ×2.09.

**Negative, from the simulation** (model, `window_sim.py`, `sim_calib.json`,
`fit_window.json`):
- **The set-up:** a step-level out-of-order model with in-order retire, R = 256, 6-wide
  dispatch, 24 MABs, prefetches that retire, and the plain chain's latency.
- **The shipped walk comes out issue-bound at 2.47** for any window of 5 or more steps,
  with or without prefetch. At 32 lanes a lane's next load is 79 cycles of issue away, more
  than ℓ.
- **It reproduces 4.49 only with an effective 8–9 outstanding misses** (m = 8 gives 4.70).
  That is the miss-cap reading, which the flagged walk rejects.
- **So the interval model's additive form fits the measurements but is not derived from
  documented mechanics.** Something in the hardware keeps dispatch from overlapping the
  miss service. That is consistent with mostly ineffective prefetches (13% saved), but not
  shown.

**Conclusion** (deduced from the table; the mechanism is unconfirmed):
- The shipped walk behaves as **per-thread window-and-issue bound**, its cost proportional
  to µops per byte times (ℓ/R + 1/D).
- The cap is per core in the sense that matters: ROB and dispatch are split between
  siblings, so a second copy adds nothing.
- A pure miss-buffer cap and pure issue are rejected as the binding constraint. A miss cap
  at 24 still floors any kernel at ℓ/24, and this matters for B and C.

**What each bound predicts for B2:**
- (i′): nothing;
- (iii′): B2's full µop ratio;
- (iv): a µop-proportional gain on the miss term and the issue term, floored by ℓ/24 and
  by the gather's throughput.

## 5. The three kernels per byte (Zen 3 macro-ops; deduced from tables, estimated layout)

| Item | A (shipped) | B (scalar chains, tiled) | C (AVX2 gather, k = 6 groups of 8) |
|---|--:|--:|--:|
| random T loads | 1 | 1 | 1 (gather elements) |
| gather | — | — | 39/8 = 4.88 (Zen 4: 5.25; Zen 5: 4.00) |
| link extract | `shr` 1 (flag tested first) | `bextr` 1 (uops.info: 1 µop on Zen 3–5) | `vpand` + `vpsrld` 2/8 |
| flag | `test; js` 1 | `or` into an accumulator 1; one test per tile | `vpor` 1/8; `vmovmskps` + test per tile |
| lane row | load + store 2 | load + store 2 | in the vector: 0 |
| prefetch | 1 | 1 | none (the gather is the load) |
| byte out | base load + store 2 | tile store 1 | `vpshufb` + `vpor` into two accumulators 1.75/8 |
| commit | — | 8-byte load, base load, store per lane per tile: 0.375 | 2 unpack, 2 extract, 8 stores (`vmovhps` is 2 µops), 8 base loads: 3.1–3.6/8 |
| mask, destination, loop | `lea`, `add`, `mov`, `cmp; jb`, round 4.4 | lanes unrolled ×4: about 0.5 | `vpcmpeqd` + `vpxor` 2/8; loop 0.4/8 |
| **macro-ops/B** | **14.8** | **8.0–9.0** | **(39 + 9.9–10.4)/8 = 6.1–6.2** (Zen 4: 6.5; Zen 5: 5.2) |
| loads / stores per byte | 4 / 2.03 | 3.25 / 2.125 | 1.125 / 0.125 |
| issue floor, Zen 3 | 2.47 (dispatch) | 1.80 (AGU: 5.375/3) | 1.02–1.21 (gather throughput 8/8 = 1.0, plus the rest if microcode blocks dispatch) |
| in flight at R = 256 | 17 steps | 28–32 steps | 5.2 gathers = 42 elements (capped at 24 by the MABs: 3 gathers) |
| per-chain latency | ℓ + L1 hit | ℓ + L1 hit | ℓ_C = ℓ + Δg (8–24 cycles on Zen 3: index → destination ≤ 28, against 4 for a load) + Δmax (3–15: the slowest of 8) |

**Replay** (estimate):
- One segment end per 1,758 bytes on average (900k/512), so 0.46% of lane-tiles and 3.6%
  of 8-lane group tiles fail.
- Replaying only the failed lane costs a finish and take (about 100 µops, mostly
  `seg_of`'s binary search), a mispredict, and on average 4 wasted lane-steps. That is
  0.05 c/B.
- Replaying whole group tiles at scalar cost (the brief's bound) is about 0.17 c/B.
- Range used: 0.05–0.17 c/B, for B and C.

**The segment-start flag** (deduced from `src/bwt.rs`, origin/master):
- **The encoding:**
  - `START = 1 << 31` is ORed into each start row's own entry before the walk and cleared
    after. Links are below 2^20, so the bit is free.
  - A lane that reads a flagged entry has reached the next segment's start. That entry's
    byte belongs to the next segment, written by `take`.
- **What follows for B2:**
  - `(q & LINK) >> 8` past a flag is still a valid row (the start's successor). So walking
    up to 7 steps beyond a segment's end is in bounds: deferred commit is memory-safe, and
    only the bytes are wrong.
  - The failed lane commits bytes 0..j−1. `finish` takes the row from step j−1's link,
    which the scalar replay recomputes from the tile's saved start row.
  - `vmovmskps` on the tile's OR gives the failing lanes' mask directly: the flag is the
    sign bit.

## 6. Predictions (model; `predict.py` → `predictions.json`)

**The method:** each hypothesis is fitted to the measured A on each runner, then predicts B
and C over the ranges in §5. Units are c/B on the 7763's consistent run, and the ratio to A
elsewhere.

**The hypotheses:**

| Label | Formula |
|---|---|
| per-core miss cap (i′) | w ≥ w_A |
| window (ii) | w = ℓ·max(U/R, 1/m), ℓ fitted |
| fitted issue rate (iii′) | w = U/(U_A/w_A); C floored by its gather throughput |
| interval model (iv) | w = issue + ℓ·max(U/R, 1/m, 1/L); ℓ fitted; m = 24 |
| interval model, no MAB cap (iv′) | as (iv) with m unbounded |

| Run (target ratio, −20% time / +20% throughput) | Kernel | (i′) | (ii) | (iii′) | **(iv)** | (iv′) |
|---|---|--:|--:|--:|--:|--:|
| 7763, run 40579732655 (0.52 / 0.60), c/B | B | 4.54–4.66 | 3.29–3.41 | 2.48–2.90 | **3.30–3.42** | 3.13–3.24 |
| | C | 4.54–4.66 | 3.29–3.41 | 1.90–2.04 | **2.53–3.17** | 2.17–3.17 |
| 7763, PR #114 (0.61 / 0.68), ratio | B | 1.01–1.02 | 0.73–0.75 | 0.55–0.63 | **0.73–0.75** | 0.65–0.68 |
| | C | 1.01–1.02 | 0.73–0.75 | 0.42–0.44 | **0.61–0.66** | 0.46–0.61 |
| 9V74, PR #115 (0.57 / 0.64), ratio | B | 1.01–1.04 | 0.91–0.94 | 0.55–0.64 | **0.82–0.85** | 0.75–0.78 |
| | C | 1.01–1.04 | 0.91–0.94 | 0.45–0.48 | **0.67–0.73** | 0.50–0.65 |
| 9V45, PR #118 (0.56 / 0.63), ratio, m = 24 / 48 | B | 1.01–1.05 | 1.00–1.06 | 0.55–0.66 | **0.86–0.93** | 0.89–0.93 |
| | C | 1.01–1.05 | 0.69–1.05 | 0.37–0.41 | **0.56–0.81** | 0.56–0.81 |

**The reading (model):**
- **The 7763's consistent run, under the best-supported hypothesis (iv):**
  - B gives 3.30–3.42 c/B, −24% to −27% walk, −10% to −11% decoder time;
  - C gives 2.53–3.17, −12% to −18% decoder time;
  - **2.33 is not reached.** The +20%-throughput bar (2.69) is reached only at C's
    optimistic end.
  - Within (iv), C's floor is 1.0 (gather throughput) + 35/24 (the MABs) + 0.05 = **2.51**.
- **The split between tiling and gathers (7763, (iv)):**
  - B − A ≈ −1.1 to −1.2 c/B, from issue (2.47 → 1.80) and the miss term (2.02 → 1.46,
    now the MAB floor);
  - C − B ≈ −0.13 to −0.89, from issue only (1.80 → 1.0–1.2). The gather's longer chain
    and the MAB floor return part of that.
- **2.33 is reachable** only under (iii′) (C at 1.90–2.04), or under (iv′) if
  Δg + Δmax ≤ about 17 cycles.
- **Slow instances gain more.** On PR #114's 7763 (fitted ℓ = 73), C reaches the −20% bar
  under (iv′) (0.46–0.61) and touches it under (iv) (0.61 against 0.613). The miss term
  dominates there, and B2 shrinks it.
- **The 9V74 gains least under (iv).** Its fitted ℓ = 49.7 puts the MAB floor (ℓ/24 =
  2.07) near A's own miss term (2.30), so cutting µops buys little there.
- **On the 9V45, both the clock and the MAB count are unknown.** With m = 24, A itself would
  be MAB-bound (ℓ/24 > ℓ·U/R), so Zen 5 most likely has more than 24 (assumption).
  - B gains 7–14%: 32 lanes' latency floor (ℓ + 6)/32 binds on an 8-wide core.
- **At four copies (deduced within (iv)):** both terms halve per thread for A, B and C.
  The gather unit (1 per core) and the MABs (shared, watermarked on Zen 4) are per core,
  so each copy's floors double too. B2's ratio to A is therefore about the same at four
  copies as at one. Under (i′) it buys nothing at either.

## 7. Risks

1. **The mechanism is a fit.** The interval form reproduces seven measured figures, but a
   simulation of documented Zen 3 mechanics says the walk should already run at 2.47.
   - If the true cause is a per-core miss-throughput limit near A's own rate (i′), B and C
     both land at about 4.5.
   - Only counters or the falsifier (§8) settle it.
2. **Gather microcode on Zen 3/4:** 39 and 42 µops, throughput 8 cycles (uops.info).
   - The table measures L1 hits. Whether a missing element stalls the issue of the
     sequence's later elements is not documented.
   - If it does, a gather costs the sum of its misses' issue gaps, and C falls behind B.
3. **Gathers in the load queue and the MABs:** 8 entries each.
   - Zen 3's load queue holds 44 or 72 entries (sources disagree; Chips and Cheese reports
     136 loads in flight on Zen 4).
   - So 5–9 gathers fit, against 5.2 in the ROB. Only 3 gathers' elements fit in 24 MABs.
   - The MAB term is in (iv). A smaller effective queue moves C towards (ii)'s 3.3.
4. **Replay on real geometry:**
   - Random starts make segment lengths about exponential (mean 1,758 B), so 0.45% of
     segments are under 8 bytes. Their tiles fail at once, so the replay cost is linear in
     segment ends.
   - **Not modelled: the tail.** After the last take, A drains 32 lanes, about
     32 × 1,758 ≈ 56 KB (6% of a block), with falling MLP. C with 48 lanes drains about
     84 KB, with masked gathers wasting slots.
   - Only a whole-block measurement prices this.
5. **Higher MLP may raise ℓ.** B and C put about 24 misses in flight against A's 8.6.
   - The fitted ℓ on the fast 7763 (35) sits at the unloaded chain's (38.55), so A doesn't
     show load-dependent latency.
   - At 24 in flight it is unmeasured. Other tenants' L3 traffic (E3) also enters here.
6. **Four copies:** the gather unit, the FP pipes and the MABs are per core.
   - Two siblings each gathering halve each other's floor: 2.0 c/B per copy on Zen 3/4.
   - (iv) already doubles, so B2's four-copy ratio holds only if no new shared bottleneck
     appears.
7. **µop counts for B and C are estimates.** A's 14.8 is from the asm.
   - For B, register pressure: 32 lanes cannot live in 16 GPRs, so rows stay in memory
     (2 µops a step). B with 12 register-resident lanes would be 5–6 µops/B, but its
     latency floor (ℓ + 6)/12 is about 3.4 c/B.
   - For C, the transposed accumulators and 8 tile vectors fit in 16 ymm only with two
     accumulators. That is the design counted here.

## 8. The cheapest falsifier (described; not proposed to run here)

**What:** a microbenchmark of the walk kernel only, on the same built T-vectors (Wikidata
blocks, fresh T-vector per block, flags set as shipped, decode order, the assembly
excluded).

**The arms:**
- A: the shipped `room_segments::<32, 1>`;
- B: the same 32 scalar chains, flags ORed and tested per 8-step tile, bytes into a tile,
  8-byte commits, failed-lane scalar replay;
- C at k = 1, 2, 4 and 6 groups of 8, AVX2 gathers with B's tiling and replay;
- optionally, A plus 4 independent ALU µops a step (a filler arm).

**What to record:**
- cycles and retired instructions, and on a Zen host with a PMU:
  - retired macro-ops;
  - dispatch-stall reasons: retire-token (ROB full), load-queue and store-queue token
    stalls;
  - MAB allocations and in-flight L1 misses (with MergeEvent);
  - L2/L3 misses;
- the replay count, and the tail's share of cycles.

**What separates the hypotheses** (model):

| Observable | (i′) miss cap | (iv) interval model | (iii′) issue rate |
|---|--:|--:|--:|
| B/A | ≥ 0.95 | 0.70–0.76 | 0.55–0.65 |
| filler arm − A | ≈ 0 | +1.2 c/B | +1.2 c/B |
| C/A | ≥ 0.95 | 0.56–0.71 | 0.42–0.46 |
| C, k = 1 → 6 | flat | improving to k ≈ 3 (the MABs) | improving to k ≈ 5–6 |
| A's ROB-full stall share | low | about 45% of cycles | low |

**Kill thresholds** (on the 7763, or the run's own A and T):
- **Not a breakthrough candidate** if the complete kernel (replay and tail included) has
  C/A > 0.52, or generally C > A − 0.2T. Record the gain as what it is.
- **Drop C** if C ≥ B − 0.05·A: the gather adds nothing over tiling.
- **Close the fewer-µops family** if B/A ≥ 0.95: the per-core miss cap binds. That covers
  B2, and N1-style lookups judged by µop count.

## 9. Assumptions whose failure would reverse the projection

| Assumption | If it fails |
|---|---|
| (iv) is the binding mechanism, not (i′) | B and C gain nothing |
| (iv), not (iii′) | the projection is too pessimistic: C reaches 2.33 |
| m = 24 MABs, held for ℓ per element | without the cap, C's optimistic end reaches 2.17 |
| gather elements issue without serialising on misses | C ≥ B |
| ℓ does not rise from 8.6 to 24 misses in flight | B and C's gains shrink in proportion |
| U_B = 8–9 and U_C = 6.1–6.2 µops/B, U_A = 14.8 | each ±1 µop/B moves (iv)'s B by ±0.31 c/B, and C's issue term by ±0.17 |
| replay ≤ 0.17 c/B, and a tail no worse than A's | C's 48 lanes lengthen the tail: unpriced |
| the CI binary's code generation matches the scratchpad asm | U_A changes |

## 10. Negative findings and limits

- **No documented-parameter model explains the shipped walk's level.** Pure MAB, window
  and issue bounds, and a step-level simulation, all predict 1.6–2.5 c/B against 4.49.
  The interval form fits, but it is not derived.
- **The "8.6 misses in flight" isn't a hardware count.** It is (R/U) × the stall share,
  7.8 in (iv).
- **No Zen 5 figure is in true cycles,** and its MAB count wasn't found. AMD's
  "320 in-flight misses" is a secondary report.
- **Zen 3's load queue** is reported as 44 (AnandTech) and as 72 elsewhere; it is not
  resolved here.
- **The AMD optimisation guides' zips** (56665, 58455) returned an HTML page, not the
  guide. The MAB counts are as cited in mibz2's own journal.
- **No counters on CI,** so every mechanism statement here is a fit to timings.

## 11. Files and sources

**Files, in `scratchpad/b2-walk/`:**
- `inputs.json`: every measured and published input, with its source;
- `room_segments_32_1.asm`, `walk_segments_flagged.asm`: the counted machine code (from
  `scratchpad/phases-sel.asm`);
- `ks_model.py` → `ks_fit.json`: the interval model against the measured figures;
- `window_sim.py` → `sim_calib.json`, `fit_window.py` → `fit_window.json`: the step-level
  simulation (the negative result);
- `predict.py` → `predictions.json`: §6;
- `uops_parse.py`: uops.info extraction.

**Sources:**
- uops.info: VPGATHERDD ymm and xmm, VPSHUFB, VPUNPCKLDQ, BEXTR, VMOVHPS, VEXTRACTI128
  (https://uops.info/html-instr/VPGATHERDD_YMM_VSIB_YMM_YMM.html and siblings).
- AMD, "AMD Ryzen Processor Software Optimization", GDC 2024: Zen 4 core resource sharing,
  dispatch and load/store widths, software prefetch fill levels.
- AMD 56665 rev 3.00 and 57647 rev 1.00 (MABs, L3 latency), as quoted in mibz2
  `docs/journal/2026-10-27-mlp-scheduling-and-run-structures.md` §4.
- Chips and Cheese, "AMD's Zen 4, Part 2: Memory Subsystem": DTLB sizes, the 136 loads in
  flight, L3 latency at 4 KB pages.
- hwcooling.net, "Zen 5: AMD's biggest innovation since first Zen", p. 5: SMT partitioning
  of the ROB, and the 320 in-flight misses (as reported).
- T. S. Karkhanis and J. E. Smith, "A First-Order Superscalar Processor Model", ISCA 2004:
  the interval form.
- mibz2 origin/master: `src/bwt.rs`, `docs/journal/2026-10-27-walk-sweep.md`,
  `2026-10-27-output-side.md`, `2026-10-27-literature-reading.md`.
- `scratchpad/wt-portfolio/docs/journal/2026-10-30-n1-four-copy-sensitivity.md` §2–4;
  `scratchpad/n1-fourcopy/measured_fourcopy.json`, `bench_runs.json`.
