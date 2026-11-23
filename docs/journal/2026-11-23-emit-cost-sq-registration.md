# bench-emit-cost-sq: registration

*23 November 2026. Written and frozen before any data, on `bench/emit-cost-sq`. The recipe is
`just bench-emit-cost-sq` (`.just/bench.just`); the verdict is `bench/emit_cost_sq.py`'s, tested
by `bench/test_emit_cost_sq.py`. The workstation conventions (one new timestamped directory per
attempt, its console log, the machine's records, `bench/bundle.sh`, nothing in the home
directory) are bench-emit-cost's (#725).*

**Diagnostic only (C1a).** This run decides no production default and no merge. A
PORT-CANDIDATE goes to ordinary adoption under PERFORMANCE-POLICY.md, with the memory gate
(8m bytes of extra scratch), and its own registration.

**The spec and the rule are MONODROMY's record 93,**
`docs/journal/monodromy/2026-11-23-squared-trace-arm.md`, at `c3dab1ef` (#497), with its
addendum at `b36ec51129e7f0c79de770ece7388777f0f7df2f`. They were accepted by the
Coordinator's LOG 7097052169 on #145, which builds both arms in one run.

## The arms

One binary, `examples/emit_cost_sq.rs` with the `squared-trace` research feature. The arm is
switched at run time between decodes (`mibz2::emit_cost::set_arm`), and is read once a block at
its start. The arms differ only on the blocks the open kernel already emits from both ends
(`QUEUE && !wide && m ≥ 65,536`).

- **`base`:** the kernel as shipped, `emit_bidirectional`.
- **`sq`** (record 93 §2):
  - `succ[u] = T(u)` is recorded in `wire_xor`, charged to `construct_ns`;
  - the serial square pass has its own stamp, `square_ns`. It zeroes `x2`, then for each j:
    v₂ = succ[succ[j]], x2[j] ^= v₂, x2[v₂] ^= j. That gives x2 = T² ⊕ T⁻²;
  - `emit_bidirectional_sq` then walks T² forward and T⁻² back, two codes an end a round from
    `edges`, into the same `out` with the same lo/hi layout.
  - On any coincidence among a round's four rows, or near the meeting point, it hands the
    exact state (prev, front) = (T⁻¹f, f), (back, next) = (b, T b) to the single-step loop.
- **`sq-shadow`** (record 93 §5, the addendum):
  - `succ` is recorded as in `sq`;
  - the base two-ended walk runs, and each of its steps (one row from each end) is followed by
    **4** iterations of the square pass, in increasing j: first the zero-fill, then the scatter.
    The batch is fixed at 4 and is not tuned after the data;
  - when the pass completes, the walk switches to `emit_bidirectional_sq` from its current
    exact state. `x2` is read only once complete. If the walk ends first, the pass is
    abandoned and nothing changes;
  - all of its cost lands in `emit_ns`.

**One implementation choice, for MONODROMY's review before the ready tag:** the addendum counts
the zero-fill in iterations. Here a zero-fill iteration clears one 64-byte line of `x2`
(16 entries), and a scatter iteration is one row j. The pass then completes after
m/64 + m/4 base steps, about 53% of the rows, and the T² walk emits the rest. A zero-fill of one
entry an iteration would spend the walk's first m/4 steps (half its rows) on sequential stores.
With the scatter's m iterations after it, the pass would complete only at the walk's end, so no
T² round would ever run.

## The run

- **The workload:** `corpora/wikidata.bz2` (checked by its SHA-256), four copies end to end
  (838,800,784 decoded bytes), read into memory once.
- **The decodes:** production's lazy workers (`BzReader::parallel_workers`, `OpenCells`), unpinned.
- **The grid:** {base, sq, sq-shadow} × {1, 6, 10} workers × 20 rounds. The nine cells rotate by
  one a round, with 2 s of sleep before each decode.
- **Per block:** n, m, `construct_ns`, `square_ns`, `emit_ns` (four clock reads a block in every
  arm), the arm, the two-ended emitter, whether the T² walk ran, the arm's scratch.

## The rule (frozen now, before any data)

E and K are taken over the open blocks of a decode (every block the open kernel took), each arm
paired with `base` in the same round at the same worker count:

- **E** = emit_arm / emit_base, each sum(emit_ns) / sum(m);
- **K** = (construct + square + emit)_arm − (construct + square + emit)_base, over sum(m), in ns
  per retained row (base's `square_ns` is its two adjacent clock reads' gap);
- each is the mean over the 20 rounds, with its 95% Student's t interval.

Judged at **10 workers**. 1 and 6 workers are reported and decide nothing. Quoting LOG 7097052169:

> - **`sq` is judged by record 93's rule, unchanged:**
>   - KILL if E > 0.80;
>   - PORT-CANDIDATE if E ≤ 0.80, K ≤ −0.3 ns a row and K's 95% interval lies wholly below 0;
>   - SHADOW otherwise.
> - **`sq-shadow` is judged by the same PORT-CANDIDATE condition,** taken on its own E and K. If it
>   isn't PORT-CANDIDATE, the line ends, per record 93. Its prediction is record 93's, conjectured:
>   E 0.80 [0.70, 0.92], K −0.9 [−1.7, 0.0].
> - **If both are PORT-CANDIDATE:** the lower K goes to adoption.
> - **If `sq` is KILL:** `sq-shadow`'s result is reported and decides nothing, because KILL ends the
>   line.

And the addendum at b36ec511:

> - **Each arm is judged on its own E and K,** against the same base decodes in the same rounds,
>   with the rule above. The frozen predictions stand:
>   - `sq`: E 0.65 [0.55, 0.78], K +0.7 [−0.4, +1.8];
>   - `sq-shadow`: E 0.80 [0.70, 0.92], K −0.9 [−1.7, 0.0];
>   - both at 10 workers, in ns per retained row.
> - **If both are PORT-CANDIDATE,** the one with the lower K goes forward. If neither is, the
>   line ends. `sq`'s SHADOW outcome no longer calls for a further arm: `sq-shadow` is that arm.

The verdict script prints each arm's class and one `DECISION:` line:

- KILL (`sq`'s E > 0.80; `sq-shadow` reported, deciding nothing);
- PORT-CANDIDATE naming the arm (the lower K when both are);
- END (neither is).

**VOID** (no verdict), in addition to bench-emit-cost's conditions (the machine not the
i9-10900K, a decode missing or duplicated, bytes not 838,800,784, a decode with no open block):

- the decodes' (n, m, refused) block multisets differ, between arms or worker counts;
- the two-ended block count differs between decodes;
- a block recorded under another arm than its decode's;
- a base block with the T² walk or the arms' scratch.

**Reported, not judged:**
- base's emit at 10 workers against bench-emit-cost's 5.786 ns a row (within 10%, else the
  machine's state has moved);
- the square pass and the construction's change per row;
- the peak scratch growth: `succ` and `x2`, a worker's kernel, against 8m bytes.

**Not changed after the data (C2):** the arms, the batch of 4, the zero-fill's iteration, the
grid, the block set, the lines 0.80 and −0.3, and the judged count.
