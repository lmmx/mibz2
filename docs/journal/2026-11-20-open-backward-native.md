# Queue-Free Backward Open, native three-arm test (negative result)

**Question.** Neon's cycle 2 (`docs/journal/2026-11-20-neon-cycle2-backward-open.md`, branch
`research/neon-bipolar-contraction` at 5cdd0cb7) proposes **Queue-Free Backward Open**. It
stores g = P ∘ LF_v directly, `edges[j] = code | g(j)`, written sequentially in j, instead of
shipped Open's inverse arrows `edges[g(j)] = code | j`, which need the per-symbol queues. It
then walks g from the marked row and writes the output right to left. Its C cache model
predicted that the run pass plus the wiring would take half the L2 misses. The audit of cycles 1
and 2 (`docs/journal/2026-11-20-neon-cycles-1-2-audit.md`, branch `audit/neon-cycles-1-2`) found
that the same model mis-ranks `qi16` against `i16` (#653), so only a native measurement counts.
Is Backward Open faster natively than the shipped `qi16`?

**Answer. No: FAIL.**
- **Against qi16:** Backward Open's median total is **1.10–1.15× qi16** on all six blocks,
  with every block's p10 above 0.99×. The rule needed at most 0.98× on at least 4 of 6.
- **Against i16:** it is **0.94–0.97× i16**, so the reversed orientation recovers a quarter to
  two fifths of i16's loss (i16 is 1.16–1.23× qi16 here) and no more.
- **Where it loses:** the queue-free wiring. The second scan of the records, with a per-run
  expansion into `edges`, costs more than the queue fill and the band-span copy it replaces:
  the run pass plus the wiring is 27–36% slower than qi16's.
- **Emission:** the backward walk takes the same time as the forward one (0.99–1.05×), though
  it executes 15% fewer instructions.

## The variant

The variant is research-only safe Rust: `PhasedOpen::run_backward::<ZERO>` in
`src/open/endpoint/phases.rs`, behind the existing `open-phases` feature. Production code is
unchanged; nothing outside the feature is compiled differently.

- **Run pass:** `stage_runs::<false>`, the unqueued pass i16 uses (no queues).
- **Elimination, bands and spelling:** shipped Open's, as the timed clone runs them (#653).
- **Wiring (`stage_wire_backward`):** a second `each_run` scan. Each byte's current band is held
  in a 256-entry table (`rank, end, delta, code, pack, band`), the reference's
  `ActiveBand[256]` (g5_backward_open.c). A thin run inside the current band (the fast path) is
  one table read and one table write, and its rows go to `edges[j..j + w]` in increasing j.
  The wide case (codes in `extra`) is handled as in shipped Open.
- **Emission (`stage_emit_backward`):** from `mark`, `edge = edges[j]`, `j = edge & mask`, and
  each word is copied into `out[w - len..w]`, moving `w` down from `n`. The walk is bounded by
  `edges.len()` steps and clamped at 0, as `emit` is.
- **Short cycles:** if the walk returns to the mark with `w > 0`, the cycle's text sits in
  `out[w..n]` in forward order. It is moved to the front and repeated to `n` bytes, the
  semantics of shipped `emit` plus `repeat`, also when the period doesn't divide n.
- **The output buffer:** `ZERO = false` reuses the worker's buffer and only `resize`s it to n,
  a no-op from block to block at equal n. `ZERO = true` zero-fills the whole n bytes every block
  (`clear`, then `resize`), the cost of initialising a fresh buffer. The verdict uses the reused
  buffer.

**A change made before the measured run.** A 3-round smoke run on block 0 used a first wiring
that read `bands[cursors[a]]` on every run. It showed the wiring at about i16's cost. The wiring
was then rewritten to the reference's `ActiveBand` fast path, so as to measure the candidate as
specified. The rewrite moved the wiring from 0.57 to 0.53 ms on block 0, and its correctness
tests were rerun. No other change followed any timing.

## Correctness

- **Exhaustive (`backward_arrows_and_output_match_shipped`):** 26,135 cases, every binary
  column to length 8 and ternary to length 5, at every mark and every limit in {0, 1, 2, 4, 16}
  (the enumeration of checks.rs's arrow audit). In every case:
  - each Backward Open arrow is a shipped arrow reversed, with the same code. That is,
    `edges_bwd[j] = code | i` wherever shipped (`<true, true>`, the table checks.rs audits
    arrow by arrow) has `edges[i] = code | j`, `extra` included;
  - both buffer variants' output equals shipped Open's.
- **Short cycles (`backward_short_cycles_match_plain`):** random columns with n up to 4099,
  alphabets of 1, 2, 11 and 256 symbols, three marks, limits 0–1025, both variants, against the
  plain inverse.
- **Real blocks:** every arm's output, in every round of every run (61 single-thread rounds,
  41 four-worker rounds), was byte-exact against a dense-LF walk on blocks 0, 2, 85, 143, 207
  and 210, checked outside the timed span.
- **The full suite:** `cargo test --release --features open-phases --lib` passes (154 tests).

## Method

- **Harness:** `examples/open_phases.rs`, extended to six arms rotated within each round:
  - `qi16` (production, and the timed clone);
  - `i16` (production, and the clone);
  - `bwd` (reused buffer);
  - `bwdz` (zero-filled buffer).
- **Rounds:** 61 rounds after 2 warm-ups, `taskset -c 2`, single thread, on the shared 4-vCPU
  cloud VM (Xeon @ 2.1 GHz). Each arm keeps its buffers from round to round, as a worker does.
- **Per-round ratios:** the paired ratio of the arms' wall times in the same round. The tables
  give the median and p10–p90 of those ratios.
- **The verdict's basis:** `bwd` against the qi16 clone (`qi16_phased`), like for like, since
  both carry the same stage timers and the same deferred spelling. This was fixed after the
  smoke run and before the measured run. Against production `qi16` the verdict is the same,
  and the ratios are 1–2% worse for BWD.
- **Instructions:** cachegrind, `--cache-sim=no`. Each arm's clone was run twice per block
  (`PHASES_ONLY`), the load-only run (`PHASES_ONLY=none`) subtracted, and the result halved,
  with each `#[inline(never)]` stage function's count taken separately.
- **Four workers:** `PHASES_MT=4` runs four threads, each with its own kernel and block replica,
  under `taskset -c 0-3`. It times the production qi16 and i16 arms and `bwd`, rotated each
  round. After a barrier, each worker times a batch of 20 decodes, and the round's figure is the
  slowest worker's, per decode. There is no per-thread pinning, since std has no safe affinity
  call.
- **Indicative only:** these are local timings on a shared VM.

## Results

**Total time, the verdict** (medians of the per-round ratios, p10–p90 in brackets):

| block | BWD/qi16 (clone) | BWD/qi16 (prod) | BWD/i16 (clone) | BWD/i16 (prod) | i16/qi16 (prod) | bwdz/bwd |
|---|---|---|---|---|---|---|
| 0 | **1.115** [1.056, 1.176] | 1.131 | 0.953 [0.901, 0.989] | 0.944 | 1.198 | 1.032 |
| 2 | **1.098** [1.034, 1.155] | 1.105 | 0.965 [0.930, 1.026] | 0.959 | 1.157 | 1.026 |
| 85 | **1.105** [0.992, 1.259] | 1.127 | 0.936 [0.837, 1.045] | 0.913 | 1.228 | 1.054 |
| 143 | **1.112** [1.050, 1.200] | 1.126 | 0.956 [0.885, 1.031] | 0.947 | 1.193 | 1.085 |
| 207 | **1.118** [1.040, 1.190] | 1.128 | 0.939 [0.834, 0.996] | 0.933 | 1.219 | 1.058 |
| 210 | **1.147** [1.085, 1.197] | 1.152 | 0.964 [0.921, 0.996] | 0.961 | 1.204 | 1.054 |

The rule needed BWD/qi16 ≤ 0.98 on at least 4 of 6 blocks and none above 1.01. **0 of 6
qualify, and all 6 are above 1.01: FAIL.**

**Stage split** (median ms per block). Construction is the run pass, the elimination with its
setup and spelling (stages b–d), and the wiring; emission is separate:

| block | arm | run pass | elim + spell | wiring | run + wiring | emission | zero-filled emission |
|---|---|---|---|---|---|---|---|
| 0 | qi16 | 0.428 | 0.428 | 0.075 | 0.503 | 0.539 | |
| 0 | i16 | 0.152 | 0.427 | 0.598 | 0.750 | 0.542 | |
| 0 | BWD | 0.152 | 0.418 | 0.533 | 0.685 | 0.546 | 0.601 |
| 2 | qi16 | 0.525 | 0.556 | 0.101 | 0.626 | 0.751 | |
| 2 | i16 | 0.167 | 0.558 | 0.724 | 0.891 | 0.739 | |
| 2 | BWD | 0.168 | 0.550 | 0.646 | 0.814 | 0.757 | 0.832 |
| 85 | qi16 | 0.711 | 0.266 | 0.091 | 0.802 | 0.971 | |
| 85 | i16 | 0.202 | 0.253 | 0.985 | 1.187 | 0.955 | |
| 85 | BWD | 0.199 | 0.269 | 0.838 | 1.037 | 0.961 | 1.109 |
| 143 | qi16 | 1.003 | 0.808 | 0.186 | 1.189 | 1.562 | |
| 143 | i16 | 0.298 | 0.818 | 1.453 | 1.751 | 1.557 | |
| 143 | BWD | 0.309 | 0.807 | 1.198 | 1.507 | 1.646 | 1.965 |
| 207 | qi16 | 0.673 | 0.515 | 0.115 | 0.788 | 0.898 | |
| 207 | i16 | 0.247 | 0.515 | 0.981 | 1.228 | 0.899 | |
| 207 | BWD | 0.237 | 0.521 | 0.832 | 1.069 | 0.901 | 1.099 |
| 210 | qi16 | 0.947 | 0.442 | 0.141 | 1.088 | 1.442 | |
| 210 | i16 | 0.314 | 0.448 | 1.350 | 1.664 | 1.373 | |
| 210 | BWD | 0.315 | 0.442 | 1.121 | 1.436 | 1.476 | 1.676 |

- **The run pass:** dropping the queues saves 0.28–0.69 ms.
- **The wiring:** the second scan and its per-run expansion cost 0.53–1.20 ms, against qi16's
  0.08–0.19 ms. The net is +0.18–0.35 ms a block, run plus wiring 1.27–1.36× qi16's.
- **Against i16's wiring:** BWD's sequential writes save 11–18%. That is what the orientation
  buys over the scattered direct wiring.
- **Elimination and spelling:** unchanged, as they should be.
- **Emission:** 0.99–1.05× the forward walk. The walk is a chain of dependent loads of
  `edges`, and the direction of the output stores doesn't change it.
- **The zero-fill:** costs 0.055–0.32 ms a block (+3–9% of the total). On top of the verdict
  arm, it makes BWD about 1.13–1.21× qi16 (the product of the two medians).

**Instructions** per decode (cachegrind, baseline subtracted, in millions; "other" is mostly
libc `memcpy`/`memset` called from the stages):

| block | qi16 | i16 | BWD | BWD/qi16 | BWD/i16 | qi16 run / wiring / emit | BWD run / wiring / emit |
|---|---|---|---|---|---|---|---|
| 0 | 11.20 | 15.01 | 14.15 | 1.264 | 0.942 | 4.45 / 0.35 / 2.83 | 2.46 / 5.60 / 2.41 |
| 2 | 13.23 | 17.19 | 16.18 | 1.223 | 0.941 | 4.80 / 0.45 / 3.63 | 2.62 / 6.05 / 3.09 |
| 85 | 15.02 | 21.33 | 19.51 | 1.299 | 0.915 | 7.02 / 0.37 / 4.76 | 3.73 / 8.86 / 4.05 |
| 143 | 23.53 | 31.66 | 29.19 | 1.240 | 0.922 | 9.43 / 0.73 / 6.55 | 5.10 / 11.96 / 5.58 |
| 207 | 16.84 | 23.65 | 21.85 | 1.298 | 0.924 | 7.51 / 0.49 / 4.36 | 4.10 / 9.54 / 3.72 |
| 210 | 22.59 | 32.93 | 29.91 | 1.324 | 0.908 | 11.14 / 0.55 / 6.45 | 6.01 / 14.14 / 5.49 |

- **Where the instructions go:** qi16 builds its arrows with 2.0–5.1 M instructions for the
  queue fill (its run pass minus the unqueued one) plus 0.35–0.73 M for the band-span copy
  into `edges`, a tight loop over contiguous spans.
- **BWD's wiring:** 5.6–14.1 M. About 2.5–6.0 M of that is the second scan of the records
  itself, as much as the whole unqueued run pass; the rest is the per-run expansion.
- **The trade:** the queues replace a second scan of the records, and that scan is
  instruction- and branch-bound, not memory-bound.

**Four workers** (`PHASES_MT=4`, 41 rounds, per-decode time of the slowest worker; medians of
per-round ratios, p10–p90 wide on the shared VM):

| block | BWD/qi16 | BWD/i16 | i16/qi16 |
|---|---|---|---|
| 0 | 1.155 [1.035, 1.444] | 0.957 | 1.182 |
| 2 | 1.121 [1.050, 1.398] | 0.959 | 1.177 |
| 85 | 1.130 [0.942, 1.291] | 0.914 | 1.221 |
| 143 | 1.065 [0.792, 1.503] | 1.021 | 1.138 |
| 207 | 1.122 [0.957, 1.400] | 0.966 | 1.172 |
| 210 | 1.132 [0.888, 1.404] | 0.920 | 1.241 |

This is consistent with the single thread: no sign that contention for memory traffic, the
cost the model said BWD cuts, turns the ranking around on four workers.

## Reading

- **The model counted the wrong cost.** The C model's "work events" and L2-miss counts treated
  the second scan of the records as nearly free, since they are L2-hot. Natively, the
  coalescing loop and the per-thin-run branching cost as much as the first scan. Removing the
  queues' memory traffic doesn't pay for that scan.
- **It is the second model ranking the audit warned of.** This is the same failure as qi16
  against i16 (#653), where the model again preferred the arm with fewer misses.
- **The backward walk is a wash.** It is no slower, so the descending-store and prefetcher
  question the write-up raised is answered: neutral on this Xeon. It is also no faster: the
  15% fewer instructions don't matter to a walk bound by load latency.
- **Not tried, and outside this test:** a branch-free fixed-width store in the expansion
  (write 16 rows, advance by w), or fusing the second scan into the first. Either would change
  qi16's queue fill too, and the rule allows no rescue rounds.

## Files

- `src/open/endpoint/phases.rs`: `run_backward`, `stage_wire_backward`, `stage_emit_backward`
  and the two tests.
- `examples/open_phases.rs`: the `bwd` and `bwdz` arms, `-rounds.csv`, `PHASES_ONLY=bwd|none`
  and `PHASES_MT`.
- `2026-11-20-open-backward-native.csv`: per-block ratio medians and p10–p90, the arms' walls
  and the stage groups.
- `-rounds.csv`: every round, arm and stage.
- `-instructions.csv`: the cachegrind counts by stage group.
- `-mt.csv`: the four-worker rounds.

Reproduce:

```text
cargo test --release --features open-phases --lib phases
cargo build --release --features open-phases --example open_phases
taskset -c 2 target/release/examples/open_phases DIR 61 OUT
PHASES_MT=4 taskset -c 0-3 target/release/examples/open_phases DIR 41 OUT
PHASES_ONLY=bwd valgrind --tool=cachegrind --cache-sim=no target/release/examples/open_phases DIR 1 OUT 0
```
