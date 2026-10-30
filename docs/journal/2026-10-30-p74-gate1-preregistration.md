# P74 Gate 1: pre-registration of the controlled CI comparison

Written before any P74 code exists, and committed before any measurement. The analytical
burn-down it follows is complete: every one of the register's 84 questions has a
disposition (`2026-10-30-research-portfolio.md` §9), and P74's correctness is
established (§3.2e).

**Status: not authorised.** Gate 1 starts only on the maintainer's explicit
authorisation. N1's Gate B and every i9 run are outside this document.

## The hypothesis

The shipped AVX2 MTF kernels keep the list on the stack. Each explicit token's two
32-byte list loads overlap the previous token's 32-byte and 1-byte stores, so they can't
be forwarded (the 0.2.1 wheel's machine code, §3.2e(d)). That puts about 15–17 cycles
per explicit token on the loop-carried chain (a model).

P74 keeps the list's first 32 entries in a ymm register:
- one `vpshufb` per token for ranks below 16;
- `vperm2i128` + 2 `vpshufb` + `vpblendvb` for ranks 16–31;
- a register head shift plus a blended-chunk tail shift for ranks ≥ 32.

The chain falls to 1–6 cycles, and the loop is expected to become throughput-bound at
about 5–6 cycles a token.

**Predicted** (from the model in §3.2e(h)):
- the Wikidata MTF stage (MTF + RLE2 + counts, and checkpoints where made) falls by
  20–32% for the better arm;
- the one-core decode falls by about 3–6%, and the four-worker decode by less;
- random and records: the sign is unknown, within ±10%.

## The measurement boundary

The stage measured is the whole kernel, as production calls it, not the SIMD update
alone.

**Both production kernels:**
- `mtf::mtf`: the non-cooperative inverse, and the stage table's "MTF + RLE2";
- `mtf::mtf_checkpoints`: the cooperative inverse (`decode_block` in `src/lib.rs`).
  Its 16 KiB count checkpoints come with it.

Each arm covers, for each block's `Symbols`, all of the following:
- RUNA/RUNB decoding;
- RLE2 expansion, with checkpoint splitting where it applies;
- the MTF update;
- `seq_to_unseq`;
- `counts`;
- the output bytes and the checkpoint snapshots;
- every error check (`RunTooLong`, `BlockTooLong`).

## The arms

All arms are in one binary, `examples/phases.rs`, and are timed on the same blocks in
the same passes, in an order that rotates each pass.

| arm | plain kernel | checkpoint kernel |
|---|---|---|
| **shipped** | `mtf_avx2`, unchanged | `mtf_checkpoints_avx2`, unchanged |
| **P74-branch** | rank < 16: one shuffle; 16–31: cross-lane; ≥ 32: fallback | the same, in the checkpoint loop |
| **P74-free** | rank < 32: the cross-lane sequence with a per-rank blend mask (zero for rank < 16); ≥ 32: fallback | the same |

**Common to both P74 arms:**
- RLE2 runs of ≤ 32 bytes are written with one 32-byte broadcast store (`ll` reserved
  32 bytes past `max`), and longer ones with `write_bytes`.
- The fallback's tail shift is blended 32-byte chunks in descending order, at most 7
  chunks for j ≤ 223. So no `memmove` is called on any input, including 256-symbol
  random data.
  - The 32.5 calls per 1,000 tokens (§3.2e(l)) is a Wikidata figure. Random and records
    inputs have their own run and rank distributions, so the controls measure them and
    the figure is not carried over.
- The selected symbol is the low byte of the updated head, and a run's symbol is the
  head's low byte.

**Production stays unchanged.** `mtf`, `mtf_checkpoints` and the decoder keep the
shipped kernels. The P74 kernels are reachable only from the benchmark and the tests.

## The corpora

- **Wikidata:** CI's 12 MB of dump blocks (`bench/wikidata.json`).
- **random:** `ab gen`'s random corpus. All three rank paths are exercised, with
  ~87.6% of explicit tokens on the fallback and tail shifts up to 223 bytes.
- **records:** `ab gen`'s records corpus.

The random and records runs time the MTF lines only (the head's `phases`, MTF stages
only), the same rounds as the Wikidata stage runs.

## The controls

1. **Same-binary pairing.** Shipped and the P74 arms are compared inside one binary,
   pass by pass. That removes the cross-binary variation the coop journal measured: 2.0–2.4%
   for identical kernel source.
2. **The layout control.** The head's "MTF + RLE2" line, production's unchanged
   `mtf_avx2` in the head's binary, against the same line in the base's binary. Its
   ratio estimates layout noise; it should be 1.00.
3. **Correctness, every pass:**
   - every arm's `ll`, `counts` and checkpoints are compared with the shipped arm's on
     every block, and a mismatch aborts the run;
   - the decode in `phases` is still checked against `decode`.
4. **Unit tests**, against `mtf_scalar` and `mtf_checkpoints_scalar`, errors included:
   - every alphabet size from 1 to 256 symbols;
   - streams that are all rank < 16, all 16–31, and all ≥ 32 up to 255;
   - the 15/16 and 31/32 crossings;
   - runs of 1–300 bytes, including runs crossing one or several checkpoints and runs
     ending exactly on one;
   - the shipped test corpora.
5. **Disassembly**, read before the measurement is accepted:
   - on the one-shuffle and cross-lane paths, the head register is not stored to or
     loaded from memory between explicit tokens;
   - stores and loads are allowed around a `memset` call for runs over 32 bytes, and
     around checkpoint snapshots.

   A spill on the fast path is a defect, fixed once before the measurement counts.

## The figures and the decision rules

**The figures:** each arm's c/B for each kernel on each corpus, from CI's existing stage
runs (5 rounds × 2 passes, each stage's fastest pass). The paired ratio is P74 arm over
shipped, within each round, with a 95% interval over rounds.

**Positive:**
- the better P74 arm improves the Wikidata **checkpoint** kernel by **≥ 5%**, with an
  interval excluding zero;
- it does so by more than twice the layout control's |1 − ratio|;
- and neither control corpus regresses by more than 3% (upper interval bound) on either
  kernel.

Then go to phase 2.

**Mixed:**
- Wikidata is positive, but random or records regresses beyond 3%.
- Then add one more same-binary arm: round 29's per-block selector, the alphabet guard
  plus an exact rank count over `Symbols.syms`, with the selection's cost inside the
  timing. Measure it once.
- If that arm meets the positive rule, go to phase 2 with the selector. Otherwise
  stop.

**Negative:**
- neither arm improves the Wikidata checkpoint kernel by ≥ 5%, or the gain doesn't exceed
  the layout control's noise.
- Then P74 closes as a correct but uneconomic design: a journal entry, the register, and
  the PR closed with its numbers.

**Iteration limit:** one disassembly fix before the first measurement. After the first
CI result, at most one further push, and only for a diagnosed code-generation defect
(a spill, or a call that should be inline). No tuning loop.

**Phase 2 (promotion; a separate PR, only after a positive or mixed-with-selector
result):**
- production's AVX2 kernels switch to the chosen arm, and CI's paired A/B measures the
  whole decoder at 1, 2 and 4 threads on every corpus;
- it is promoted only if all of these hold:
  - **four-worker Wikidata throughput** improves, with a 95% interval excluding zero;
  - **CPU-s/GB** is equal or lower;
  - **peak RSS** rises by no more than 1%;
  - **random and records** decoder figures regress by no more than 2%.
- A stage gain alone never promotes.

## Costs and limits

- **CI:** phases' added MTF lines take seconds a run (the MTF stage is about 10 ms a pass
  on 12 MB). The random and records stage runs add about 1–2 minutes.
- **The local machine:** a release build, for the disassembly and the tests, after
  checking free disk. Stale `target/` directories are cleared first.
- **No i9 run**, and no production change in phase 1.
- **What this can't show:**
  - the i9's or other microarchitectures' mispredict and forwarding costs (CI's EPYC
    only);
  - the cooperative decode's interaction with the other stages at four workers, which
    phase 2 measures.

## Open points for review (added before authorisation; no rule above changed)

Written for the guardian's and the maintainer's review. Settling any of them changes the
protocol, so each is for the maintainer to decide before authorising.

1. **The prediction's width.** The one-core "3–6%" above is the model's 4.6–5.8%
   (§3.2e(h)), widened on purpose. That model assumes P74's loop runs throughput-bound
   at 5–6 cycles a token, an estimate (15–20 instructions at about 4 IPC). The wider
   range is the prediction tested in phase 2.
2. **Power.** The stage runs are CI's 5 rounds × 2 passes (`STAGE_ROUNDS`,
   `STAGE_REPEATS`). A 95% interval over 5 paired rounds has 4 degrees of freedom.
   Raising the rounds for this PR alone would cost about 15 s a round per corpus.
3. **The layout control's reach.**
   - **The plain kernel is covered across binaries:** the head's unchanged `mtf_avx2`
     against the base's.
   - **The checkpoint kernel is not:** the base's `phases` doesn't time it. Its
     comparison is within one binary, where the functions still differ in placement.
   - **One option:** run the head twice, the second build with function and loop
     alignment changed (for example `-C llvm-args=-align-all-functions=6`), and require
     the result to hold in both builds.
4. **The branch model.** The 16.4% misprediction rate is a 2-bit-counter model of one
   branch in isolation, not the hardware predictor. It justifies running both arms, not
   choosing between them.
5. **One microarchitecture.** CI's runners are EPYC (Zen 3). The i9's Comet Lake has
   different forwarding and misprediction penalties (Agner: about 11 against 10 extra
   cycles for the failed forward). A Zen 3 result is not a Comet Lake result, and the
   i9 stays out of Gate 1.
6. **Attribution (a limit on interpretation, from the guardian; no arm added).** Both
   P74 arms share the inline short runs and the blended-chunk fallback. So a win
   validates the combined kernel and doesn't attribute it between register residency and
   fewer `memset` calls. Any causal split would need a later attribution experiment, run
   only if its answer would change an engineering decision.
