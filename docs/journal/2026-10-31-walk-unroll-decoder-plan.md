# Room walk, constant-index 32-lane round: implementation plan for the full-decoder trial (for review)

**Order:** nothing below is started until the protocol is approved and committed as
registered.

## Steps

1. **The kernel type** (`src/bwt.rs`, insertions only):
   - `walk_segments_room_unrolled` / `_parts`: `walk_segments_room` / `_parts` with
     `room_segments_unrolled::<FLAGGED_LANES, PREFETCH>`, and the same `MIN_SEGMENTED`
     fallback and `assemble`;
   - `pub struct SegmentedUnrolled`, a copy of `Segmented` (`#[doc(hidden)]`) whose
     `invert` calls it.
   - **Tests:** `SegmentedUnrolled` equals `Segmented` and `Plain` on the existing
     decoder and stream tests, as a second inverse type wherever they parameterise over
     one.
   - A `decode_with::<SegmentedUnrolled>` differential on the corpora.
2. **No other source change.** `parallel.rs`, `lib.rs`, `mtf*` and `coop.rs` are
   unchanged. The check is `git diff master -- src`: the only additions are in `bwt.rs`.
3. **The harness:** `examples/walk_decoder.rs`, `bench/walk_decoder.py` and
   `bench/walk_decoder_run.sh`, with a recipe `just bench-walk-decoder LABEL`.
   - They are a port of the P74-C decoder harness at v1.1 (`research/p74c-decoder`
     `3870c12`). Kept from it:
     - the fresh child per measurement;
     - the affinity, plan and thread-inventory checks (with v1.1's settle);
     - `wait4` CPU and RSS;
     - the range-guarded corpus fetch;
     - the frozen hashes;
     - the `text` corpus built from a pinned source tree;
     - the summariser and verdict code.
   - **Changed from it:** the arm's construction (`with_inverse::<Segmented>` /
     `<SegmentedUnrolled>`, not a `Kernels` field), and the selection counters, removed
     because the arms have none.
   - The P74-C branch isn't touched; the files are copied, with their origin recorded.
4. **Freeze:**
   - corpora, with the new walk-structure controls' hashes;
   - `frozen.json` with every source hash;
   - a pin file.

   Then a local dry run up to timing: correctness, audit, `--check` at w = 1–4.
5. **Structural audit script:** `bench/walk_decoder_audit.py` for W1–W4. It reuses
   `walk_unroll_gates.py` and `walk_asm_audit.py`; W3 and W4 come from the binary's call
   graph and normalised disassembly.
6. **Exploratory CI:** 3 jobs at w = 1, 2, on a push-triggered trigger branch pinned to
   the freeze. It needs separate authorisation.
7. **The i9:** `just bench-walk-decoder i9-10900K`, run by the maintainer after the P74-C
   i9 trial and only when authorised.

## Risks and what each step does about them

- **Separate monomorphisations, so layout differences outside the walk:** W4. If the
  instantiations differ beyond the inverse call, the run stops, and the alternative (a
  runtime switch inside one instantiation) is proposed for review rather than tried.
- **SMT sharing reduces the gain:** the primary is on the i9 with idle siblings. CI is
  exploratory only.
- **Short-segment inputs stress the 32 duplicated cold paths:** the walk-structure
  controls (`text-l1`, `zeros`, `period-3`) are gated like the other controls.
- **Interaction with P74-C:** none in this trial. Both use the shipped plain MTF.
