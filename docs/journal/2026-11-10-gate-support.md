# Gate support: #316's qualification size, #324's fallback harm

10 November 2026. The Pathfinder, under the redirect (LOG 6762859780: integration over
exploration). Arithmetic and untimed traces only. Nothing registered, frozen, built into
`src/`, timed or queued.

## 1. #316: is there a smaller defensible programme than k = 12 × R480?

**Yes, by k and R only: k = 11 × R240.** 2,368 runner-minutes against 3,546 (−33%). All-cell
lower bound at 1.005: 0.895 on the strict τ, 0.834 on the 25-link τ. Nothing else in the
request's list is smaller and still the route's claim.

**Inputs:** §2.11's own (registration draft at 8fd2c557): `plan.json`'s per-cell τ, the pilot's
mean-free `menu.json` (largest LRV a cell), the bound U_c = exp(D̄_c + t(0.95, k − 1) · sd/√k),
the cost k × (135 + 0.3344 R). The script reproduces the draft's table at k 8 and 12
(`2026-11-10-gate-support/nicalc.py.txt`, `nicalc2.py.txt`, their outputs beside them).

**The fixed design on a finer grid** (lower bound 1 − Σ misses, strict τ / 25-link τ;
runner-minutes):

| R \ k | 9 | 10 | 11 | 12 |
|---|--:|--:|--:|--:|
| 160 | 0.575 / 0.473 (1697) | 0.702 / 0.626 (1885) | 0.790 / 0.730 (2074) | 0.851 / 0.803 (2262) |
| 200 | 0.672 / 0.573 (1817) | 0.784 / 0.708 (2019) | 0.857 / 0.796 (2221) | 0.905 / 0.856 (2423) |
| **240** | 0.735 / 0.637 (1937) | 0.833 / 0.757 (2153) | **0.895 / 0.834 (2368)** | 0.934 / 0.884 (2583) |
| 320 | 0.803 / 0.705 (2178) | 0.882 / 0.805 (2420) | 0.929 / 0.867 (2662) | 0.958 / 0.907 (2904) |
| 480 | 0.848 / 0.749 (2660) | 0.910 / 0.832 (2955) | 0.947 / 0.883 (3251) | 0.969 / 0.917 (3546) |

- **Why R falls and k doesn't:** a dispatch is 135 minutes of setup plus 0.3344 R. Setup is 46%
  of it at R480, 63% at R240. And the weakest cell's variance (pattern/4/cpu, σ_D 0.0125) is
  89% τ at R480. Rounds buy little; halving them costs 3.5 points at k = 12.
- **Clearing 0.80 on both τ sets:** k 11 × R240 (2,368). The grid is flat there: k 12 × R160
  also clears it, at 2,262 (4% less), but barely on the 25-link set (0.803). k 11 × R240 keeps
  margin on both. On the strict set alone: k 10 × R240, 0.833 at 2,153 (−39%).
- **The lower bound is close to the truth here.** The product of the cells' powers
  (independent cells) is 0.899 against 0.895 at k 11 × R240. Four cells carry 91% of the
  misses: pattern/4 CPU and wall, text/4/wall, random/2/wall.
- **Elapsed:** `records` at R240 is 8.9 minutes of rounds at the pilot's pace, 15.5 at 1.75×,
  plus 15 of setup: under 31 minutes, against 49 at R480. Eleven dispatches at the draft's
  hourly spacing: 11 hours. The spacing is an independence parameter and is not shortened here.
- **R240 is 60 whole cycles.** The planning term LRV/R leans on spans up to 60 rounds, four to a
  dispatch. Validity doesn't rest on it: the SD is taken across dispatches.

**The three options named, each priced:**

1. **Fewer cells: not within the route.** STEWARDSHIP condition 2: "every cell of the standing
   CI end-to-end bench", and "any omission or change to that standing grid is a separate
   prospective bench-policy decision, not an exclusion available within this route". Such a
   decision is the team's (RULES C6) and forward-only (C2). Taken now, to shrink one
   candidate's qualification, it is the exclusion that sentence forbids; and it could not
   apply to #316 if any run of the candidate on these cells has been seen.
   - What it would buy: without the four high-τ cells, k 8 × R240 gives 0.874. Three
     dispatches saved, by removing the four cells whose history is least stable: the cells a
     no-harm claim most needs.
2. **A pooled-cell estimand: a different claim.** The 42-cell mean's σ_D is 0.0009; k = 3
   passes with power 1.000. A pooled geometric mean at 1.02 admits one cell at
   1.02^42 = 2.30× with the rest neutral. The route's claim is "no cell is worse by more than
   the margin".
3. **The τ calibration folded in: nothing to fold, and the fold that exists costs more.**
   - **The fixed design needs no A/A campaign.** U_c uses the k dispatches' own SD. τ sizes k
     and nothing else. The open calibration (t at small k under the runner-model mixture) is a
     computation on the neutral links: no runner-minutes.
   - **The fold that is exact:** Stein's two-stage procedure. k0 dispatches fix every cell's
     s0²; the total k is a registered function of the variances alone,
     k = clamp(k0, k_max, max_c ⌈s0_c² (t_α + t_β)² / Δ²⌉); the bound uses s0 and t(k0 − 1).
     No mean is read to size it, and under the draft's own model (normal iid dispatch means)
     its size is exact: 0.0502 simulated at nominal 0.05, 10⁶ draws, at the largest- and
     smallest-σ cells, strict and doubled τ.
   - **It is larger.** The maximum of 42 variances on 5–7 df overshoots, and the quantile is
     t(k0 − 1). At R480, k0 8, β 0.05:

     | cap | strict τ: all-cell, E[k], minutes | 25-link τ | τ doubled |
     |---|---|---|---|
     | 12 | 0.944, 11.5, 3407 | 0.886, 11.8, 3489 | 0.111, 12.0, 3546 |
     | 16 | 0.976, 13.7, 4037 | 0.949, 14.6, 4320 | 0.266, 16.0, 4728 |
     | 24 | 0.981, 14.9, 4407 | 0.973, 17.2, 5090 | 0.593, 24.0, 7090 |

     Fixed k 12 × R480: 0.969 at 3,546. Stein under the same cap: 0.944 at 3,407.
   - **What it buys is robustness to a wrong τ,** at a price: with τ doubled the fixed design's
     lower bound is 0 through k 14 (product 0.15 at k 12) and Stein's cap 24 reaches 0.59 for
     7,090 minutes.

**Also priced, not proposed:**
- **An early stop for success** (group-sequential): at k 6 the all-cell pass probability at the
  full α is 0.24 (R480). An interim saves under 0.24 × 6 = 1.4 dispatches in expectation and
  raises the final quantile.
- **Cells sharing a runner** (the intersection-union test needs no independence between
  cells): it would cut the 135 minutes of setup. It changes §2.2's fixed partition and each
  cell's cache and thermal history, and runner-minutes are free on this repository. Elapsed
  hours bind, and sharing lengthens them.

**What no size here covers:**
- **τ doubled:** lower bound 0 at every k ≤ 14, any R.
- **The true ratio.** All-cell lower bound at R240, strict / 25-link:

  | true ratio | k 8 | k 10 | k 11 | k 12 |
  |---|--:|--:|--:|--:|
  | 1.0000 | 0.957 / 0.916 | 0.992 / 0.975 | 0.997 / 0.986 | 0.999 / 0.992 |
  | 1.0025 | 0.853 / 0.782 | 0.959 / 0.919 | 0.979 / 0.949 | 0.989 / 0.967 |
  | 1.0050 | 0.581 / 0.445 | 0.833 / 0.757 | 0.895 / 0.834 | 0.934 / 0.884 |
  | 1.0075 | 0.000 / 0.000 | 0.463 / 0.293 | 0.608 / 0.479 | 0.714 / 0.611 |

  The planning value 1.005 is §2.4's and is not moved here.
- **Validity exposure, absent from every power above.** One shard failing after its
  measurement started makes the qualification INVALID, with no replacement. There are 9k such
  jobs: P(all valid) = (1 − q)^(9k). At q 0.2%: 0.82 (k 11), 0.81 (k 12), 0.75 (k 16). At q 1%:
  0.37, 0.34, 0.24. q is not estimated here. Shorter jobs and fewer dispatches both lower it.

**Recommendation:** k = 11 × R240 if 0.80 at 1.005 on both τ sets is the bar. k = 12 × R240
(2,583, −27%; 0.934 / 0.884) if the draft's margin above 0.80 is wanted. Either is §2.11's
design with two numbers changed: same unit, bound, admissions, schedule and claim.

## 2. #324: the fallback harm's mechanisms, ranked, each with its cheapest check

**The harm:** `random` at 1 thread +3.4% (2.2, 4.6) on an EPYC 7763 and +5.3% (4.5, 6.1) on an
EPYC 9V45; `records` +1.5% (0.2, 2.9) and +2.1% (1.0, 3.3); +1.3% (−1.1, 3.8) on a Xeon 8370C.
Instructions equal (+61 a block). It is real against the A/A record: the 19 identical-code
links' `random`/1 means have SD 1.06%; the three runs' mean, +3.3%, is 5.4 of its standard
errors.

**Sources:** agg3's corrected counts (research/run-front-fallback at 72eb8af4, CI's builds, one
890,000-byte block each, padding 0); #324's intervals; and four untimed checks made here on
existing commits, built with this container's toolchain (rustc 1.97.0, not CI's: the
mechanism-level facts transfer, a build's own class counts don't). Scripts and raw output:
`2026-11-10-gate-support/`.

### The ranking

| rank | mechanism | fixed per build natively? | dose, records ÷ random (observed 0.40–0.44; run 3 allows 0.16–0.73) | price it needs | state |
|--:|---|---|---|---|---|
| 1 | `counts` → `STEPS`, D←H at 4 KiB | **yes** | 0.30 | 71–111% of the decode per candidate a byte: 10.7–11.8 ns a pair | **open; its price is 11–67× the bound on a frame pair's** |
| 2 | code placement modulo 4 KiB, acting per redirect | **yes** | 0.46 by `mtf_avx2`'s simulated mispredicts; 1.13 by instructions (excluded) | 5.8–9.0% of the decode per mispredict a byte: about 0.9 ns, 3 cycles | **open; no event named** |
| 3 | `counts` ↔ `mtf_avx2`'s frame, S←H and H←S at 4 KiB | **no: drawn per launch** | 1.10, 1.00 | — | **closed as a mean harm** |
| 4 | `drain`'s moved spills | yes | — | — | closed at this head: no stack–stack class rose |
| 5 | `read_item`, S←S at 1 KiB | yes | — | — | closed at this head: 17,836 → 95, the wrong sign |

### 3: the frame class can't carry a mean. Measured.

- **The 1-thread arm decodes on the child's main thread** (`ab.rs` `child`, `threads == 1`),
  and the bench doesn't touch the personality.
- **Linux draws the main stack's offset per launch.** 2,000 launches of a probe: the stack's
  residue modulo 4 KiB took all 256 16-byte steps (χ² 262.5 on 255 df); the heap's residue was
  one value in all 2,000.
- **On the two builds,** a gdb probe at `mtf_avx2`'s prologue, randomisation on, 60 launches
  each:
  - `counts`' residue is one value a build: 0x0b0 master, 0x8f8 head, every launch;
  - the frame is realigned to 64 bytes and is 0x1c0 long in both builds;
  - **the frame meets `counts`' 1 KiB window in 20 of 60 launches on master, 25 of 60 on head.**
    Geometry gives 22 of 64 for either.
- **So the class is the same lottery in both arms.** It adds variance to each and nothing to
  their mean difference. This is the share the layout check could not estimate from four
  paddings (`2026-11-10-run-front-layout-check.md` §2).

### 1: `counts` → `STEPS` fits the dose and is fixed per build. Its price is the problem.

- **Fixed per build:** `counts`' heap residue is constant across launches (above); `STEPS` is
  `.rodata`. In the head, `counts` lies wholly on `STEPS`' residues; in master not at all. The
  same holds in this toolchain's builds (1,024 of 1,024 bytes against 0), so it follows from
  the inline `Runs` and not from one link.
- **Dose:** excess candidates 0.0476 a byte on `random`, 0.0142 on `records`: 0.30.
- **Price needed:** 3.4% ÷ 0.0476 = 71% of the decode per candidate a byte (run 2); 111% (run
  3). At the STEPS draft's 0.51 ns a byte: 10.7–11.8 ns a pair, 28–39 cycles.
- **The bound on a pair's price, from the frame class.** The same store (`counts`' increment),
  the same loop, the same 48-store horizon; only the load differs. A launch whose frame meets
  `counts` carries about 1.0 more candidate a byte (0.99–1.27 archived). The two arms draw
  independently, so the round ratio's variance holds at least 2p(1 − p) × (pair price × 1.0)².
  - Run 3's round SD is 1.12%, run 2's 1.68% (from their intervals, ten rounds).
  - **A frame pair costs at most 1.7% (run 3) and 2.5% (run 2) of the decode per candidate a
    byte.** Conservatively (two discordant rounds of ten, which fails 2.3% of the time): 2.7%
    and 4.0%.
- **So a `counts` → `STEPS` pair must cost 29–67× a frame pair's bound; 11–35× at the
  conservative bound and the harm's lower limit.** At a frame pair's price the class explains
  0.08–0.19% of the 3.4–5.3%.
- **What this does not show:** that a `STEPS` row load can't be that much dearer than a frame
  load. No reason is known why it would be. The STEPS placement diagnostic measures it
  directly, and this arithmetic predicts it null.

### 2: code placement fits the dose through mispredicts, and only through them.

- **All hot PCs moved modulo 4 KiB; alignment modulo 64 is kept** (`mtf_avx2` 0xca0 → 0x520,
  `read_item` 0xcc0 → 0x680).
- **Dose by instructions:** every kernel's share is level across the two corpora (`mtf_avx2`
  71% and 75%), so an instruction-proportional effect predicts 1.0–1.13. Excluded by run 3.
- **Dose by redirects:** cachegrind's simulated conditional mispredicts in `mtf_avx2`, master:
  0.590 a byte on `random`, 0.273 on `records`: **0.46.** It fits as well as rank 1's 0.30.
- **Price needed:** 5.8–9.0% of the decode per mispredict a byte, about 0.9 ns, 3 cycles a
  redirect. An ordinary front-end quantity; no event is named.
- **Against it, the record of layout re-rolls.** Six merged PRs changed only the harness (69,
  74, 219, 290, 298, 326: decoder source identical, the binary rebuilt). Built here at each
  merge and its parent, every one moved `mtf_avx2`, `read_item` and `build` modulo 4 KiB;
  two (74, 290) kept all three modulo 64, as #324's head does; two (290, 298) moved `STEPS`. Their
  `random`/1 changes on CI: −0.03, +0.32, −0.57, −0.75, +0.56, −0.06% (SD 0.50%), inside the
  A/A spread. **Six re-rolls, none past 0.75%.** A placement draw of 3.4–5.3% would be a tail
  event; six draws don't exclude a tail of one in ten.

### The cheapest checks left, on existing builds, no kernel

1. **The STEPS placement diagnostic, as drafted, with its null made decisive.** It isolates
   rank 1 in one binary. Predicted here: under 0.2% of the decode. Freeze it so that a null
   at that size closes rank 1.
2. **The four existing builds as a 2 × 2** (a diagnostic, not a qualification; the Guardian's
   call under D082 and D086, since boxed and the candidate were refused as successors):

   | build | `counts` → `STEPS` | code placement |
   |---|---|---|
   | master 27ed84df | 1× | reference |
   | head e88c0631 | 4.06× | moved, mod 64 kept |
   | boxed f72b2a3c | 4.06× | a third placement |
   | candidate e8a417c6 | 1.02× | a fourth; 0 of 6 kernels at master's mod 64 |

   Rank 1 predicts boxed slower as the head is and the candidate level with master, on
   `random` at 1 thread. Any other pattern kills it. One ordinary 10-round paired run a build
   resolves about 1% (the A/A SD).
3. **Rounds already archived:** within each arm, `random`/1 launches should fall in two modes,
   about a third in the dearer, if a frame pair costs anything visible. The modes' gap is the
   pair's price, measured instead of bounded.

### A harm on the admitted path that no corpus exercises

`takes_runs` admits by the block's **bound**, not its length (`2 * syms.len() <= max_len`,
its comment says so). A high-entropy block shorter than half its bound takes the records'
path, which costs it 18–19% more instructions by #324's own account. Whole-process
instructions of `ab child mibz2 1` on the first bytes of CI's `random`, `bzip2 -9`:

| block | master | head | change |
|--:|--:|--:|--:|
| 200,000 B | 32,923,892 | 36,310,524 | **+10.3%** |
| 440,000 B | 65,166,258 | 72,573,850 | **+11.4%** |
| 460,000 B | 67,781,158 | 67,784,508 | +0.00% |

- Every CI corpus's high-entropy blocks are full, so no cell sees it. A small incompressible
  file does: under 450 kB at `-9`, under 50 kB at `-1`.
- Instructions only; untimed. Found while tracing: the head never entered `mtf_avx2` on a
  200,000-byte block.
- A successor's admission needs the block's length, or its share of run symbols, and its
  registration a cell with a short high-entropy block.
