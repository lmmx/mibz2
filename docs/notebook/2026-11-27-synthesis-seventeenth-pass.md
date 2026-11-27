# Synthesis, seventeenth pass: 0.5.17 is slower at the knee, and CI cannot see why

Read since the sixteenth pass (`fea51447`): merged #1124, #1132, #1194, #1197, #1202, #1204, #1207,
#1208, #1211–#1227 and their deltas. The pass also covers the head of the open PR #1168 and the two
0.5.17 i9 bundles on `origin/evidence`:
- `release-scaling/run-20261127T213343Z.tar.gz`, sha256 `3b3af8b1…c8309b`;
- `crossover/run-20261127T214442Z.tar.gz`, sha256 `facc37f5…456d4`.

Both digests were checked against their `.sha256` files. Nothing was re-run. Recurring claim
`synthesise`.

**Not re-scored.** Reading the release bundles and recording their verdicts in the CHANGELOG
belongs to `release-0-5-17`'s holder (RELEASING §5). The figures below are a descriptive reading of
the bundle's `summary.txt` and raw `runs.jsonl`, not that verdict. The 8- and 10-thread steps are
below §5b's 3% attribution bar, so they get no attribution row.

## 1. 0.5.17 loses about 2% CPU at 8–20 threads, and the one default-path change is the suspect

The release-scaling run is paired by round over 10 rounds, on `wikidata` (210 MB out, the bench
corpus of 233 full blocks), pinned. The ratio is 0.5.16 / 0.5.17, so a value below 1 means 0.5.17
is slower.

| threads | wall ratio [95% CI] | CPU ratio [95% CI] | median CPU, 0.5.17 − 0.5.16 |
|---:|---|---|---:|
| 1 | 1.003 [0.963, 1.045] | 1.003 [0.963, 1.045] | −1.6 ms |
| 2 | 0.981 [0.953, 1.010] | 0.979 [0.961, 0.996] | +9.6 ms |
| 4 | 1.004 [0.982, 1.025] | 1.003 [0.986, 1.020] | −1.9 ms |
| 8 | **0.974 [0.967, 0.982]** | **0.977 [0.970, 0.985]** | +9.0 ms |
| 10 | **0.985 [0.976, 0.995]** | **0.982 [0.977, 0.987]** | +8.0 ms |
| 12 | 0.980 [0.934, 1.028] | 0.988 [0.959, 1.019] | +6.5 ms |
| 16 | 0.984 [0.920, 1.053] | 0.965 [0.915, 1.017] | +26.7 ms |
| 20 | 0.959 [0.910, 1.010] | 0.965 [0.928, 1.003] | +11.6 ms |

At 8 and 10 threads, where `Plan::lazy` puts every worker on the OpenCells route, 0.5.17 is slower
beyond noise, by 2.6% and 1.5% of wall. The point estimates at 12–20 threads agree. All 240 runs
returned the same bytes.

The only default-path change between the tags is #1222 (`OpenCellsOf::cap`). #1217 and #1132 sit
behind opt-in features. Reading #1222's diff against its parent:
- **Before:** every block passing `lazy_counts_admit` (density line r/n < 0.15) got `Cap::NONE`.
  The call site skips `within_cap` when `cap.m == u32::MAX`.
- **After:** lazy-admitted blocks with 0.10 ≤ r/n < 0.15 get `Cap::dense(n)`. On Wikidata those
  are 233 − 194 − 5 = 34 blocks. Each of them now pays `within_cap`, a full scalar pass over the
  block's ≥ 90,000 records that merges the runs before the first pass. It admits all 34 anyway.

The PR describes the change as "0 within_cap overhead on 194/233". It moved 34 blocks the other way.
Its own 8-worker figure (−1.88% CPU) appears to compare against the rejected `m ≤ 0.22·n` arm, not
against the shipped rule. 34 blocks at about 0.26 ms each would give the +9 ms CPU seen at 8 and 10
threads. That scale is plausible for one sequential pass over about 100,000 `u16`s with a dependent
merge loop. **Unconfirmed:** 2 threads shows the same +9.6 ms. The route at 2 threads is not
established here. If it is not OpenCells, part of the loss is code layout (fat LTO moved #1117's
`src/mtf.rs` by 7.6%, §3), not the pre-scan.

**Implied claim, admitted at P1:** `open-cap-prescan-cost`. Its first step is a diagnostic that
separates the two causes: the time `within_cap` takes over those 34 blocks, and v0.5.17 with the
old `Cap::NONE` restored for lazy-admitted blocks below 0.15 against v0.5.17 itself. Its fix must
leave every block's route unchanged (text's m/n 0.42 blocks still refused), so it is a pure cost
change and decides no admission policy afresh. One cheap candidate checks the cap inside the
first pass, abandoning it on overflow, in place of a separate pre-scan.

The crossover bundle shows the 7-thread cliff persisting in 0.5.17: 0.104 s and 126 MB at 7
threads, against 0.081 s and 40 MB at 6 and 0.067 s and 41 MB at 8. This is #1182's finding,
filed under `plan-schedule-and-lazy-threshold`, and is not new here.

## 2. Structural cause: CI's paired bench never executes the open kernel

`bench.yml` runs `mibz2` against `mibz2@base/ab` at threads `1,2,$(nproc)`, which is 4 on its
runner. Below `LAZY_FROM_THREADS = 8` the default plan is Cells. So every OpenCells change goes
through CI with no paired measurement of the code it changes:
- the default #1222;
- the experimental #1110 and #1217 (forward lanes);
- #1132 (mtf-fold);
- #1080 (Segmented, partly).

0.5.17's own release bench said as much ("CI's 4-CPU runner runs eager Cells at 1–4T"), and
PERFORMANCE-POLICY already makes the PR's paired native run the evidence for code that runs only
at thread counts CI can't reach. But that native run is self-reported and, for #1222, compared
the wrong baseline. `examples/ab.rs` already has the arm that closes the gap: `mibz2-workers-lazy`
forces the open route at one worker (registration §5 of forward-emission uses it).

**Implied claim, idea P2:** `ci-bench-open-route-arm`. It would add a paired
`mibz2-workers-lazy` cell at 1 thread to CI's bench, diagnostic and outside the acceptance rule,
so every PR touching the open kernel gets a reproducible head-against-base number on a fixed
runner. It would have shown #1222's pre-scan cost at 1 worker on the PR.

## 3. #1117's i9 recipe must time the workers, not only S′

The jcc-aligned build was STOPped on the i9 (Comet Lake-S carries the silicon fix; the flag costs
+3.05% W), so MTF's alignment will not be pinned by a build flag. #1117's CI confound was MTF + RLE2
+7.6% from a layout shift. Its stop test's gate 2 times the scanner stage S′ alone, and S′ is
immune to that confound. But the i9 settled station balance: Wikidata is worker-bound at k = 8
(k* = 9.46) and text is scanner-bound (k* ≈ 5). An S′ pass with the MTF shift intact would gain on
text at 8 workers and lose on Wikidata at 8. The recipe written after `resume-released-pr-claim`
lands should report the worker stage table and 8-worker whole-decoder walls on Wikidata and text
beside S′. #1202's `U1_u32` rewrite of `runs::front_avx2` (142 B → 100 B hot loop) is the
alignment-robust route to removing that confound rather than masking it.

## 4. #1168's registered pin is superseded; run the improved candidate, by registration

#1168's recipe pins `cand` at `7a7a207a`, the #1110 prototype, as §2 of the registration says.
#1217 has since rewritten the same feature's walk (`#[inline(never)] run<K>`, disjoint slabs,
inline LONG). On a VM that is −7.8 to −11.5% whole-decoder CPU on top of the prototype. Every
downstream decision waits on this verdict: `low-thread-open-unification`, the Cells-route
retirement and #1197's two builds. Each of them would ship the #1217 walk, not the prototype's.
Running the old pin would decide an arm nobody will ship, which is the "obsolete candidate" case.

The registration's §4 already says the remedy: any departure from §2 needs a new registration
before data. That is a C1a addendum reusing §2–§3 verbatim with `cand` at #1217's merge
(`57678259`) and `base` at its first parent (`84e09879`; the default build is unchanged between
them), and a C1a needs no second reviewer. #1168's harness takes the pins as constants. This is
for `forward-emission-run`'s current holder to decide and record. It is stated here because the
`result-to-release` delta's "without silently substituting #1217" leaves explicit substitution
open, and the downstream claims are worth more measured on the code they would ship.

## 5. One codegen lesson in two places

#1217 removed 69 `vzeroupper` from the walk by making `run<K>` `#[inline(never)]`, which keeps the
VEX-using loop out of the caller's frame. #1132 STOPped at its gate 2 on 11 `vzeroupper` in
`QueuesOf<1008,1024>::push_slow` under `-C target-cpu=native`. That function autovectorises the
iota fill (`chunks_exact_mut(2)`) and calls `take_chunk` inside the loop. Hoisting the fill into
an `#[inline(never)]` leaf, or keeping `take_chunk` out of the vector loop, is #1217's remedy
applied to `mtf-fold-batch-measure`'s gate (a). **Unconfirmed:** that this is where the 11 come
from. It is for that claim's holder.
