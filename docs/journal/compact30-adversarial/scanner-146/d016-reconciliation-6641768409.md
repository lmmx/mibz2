# D016 reconciliation and phase-diagnostic mismatch

Trigger: [6641768409](https://github.com/lmmx/mibz2/pull/145#issuecomment-6641768409).
Researcher, 2026-11-04.

**Concur with D016's scoped rejection and causal limits. One additional finding: the production port's phase harness does not measure the production compact30 kernel.** Do not use its near-flat walk timing to explain or rebut the whole-decoder regression.

## Independent checks

Read D016 at `2b58b12acc57e270a667923e7edf81808a41c39d`, the evidence README at `c493a45d3f24e9c07657b3183cd2ab59fd66f5b2`, and the completed [CI report](https://github.com/lmmx/mibz2/pull/155#issuecomment-6641501139).

From the published scope totals:
- decode delta = 29,530,459 − 25,751,794 = **3,778,665**.
- Holding candidate build **and remainder** fixed, allowable walk = 25,751,794 − 4,369,204 − 4,953,555 = **16,429,035**, or **18.25507155 instructions/row** for 899,971 rows.
- Net build credit after remainder charge = 1,059,858 − 57,715 = **1,002,143**, or **1.11352810 instructions/row**.

The old 17.1 walk-parity target is unnecessarily strict for this conditional whole-decoder arithmetic screen. Conversely 18.255 is not a universal lower bound or a performance guarantee. Recomputed arithmetic from published totals, not a new raw-profile replay.

The CI report confirms Wikidata wall intervals above zero at all three topologies and the one-thread records interval crossing zero. Its resource table explicitly says no hardware counters and unknown shared L3 size. D016 is right: net regression does not prove zero cache benefit; matching a one-block instruction percentage to a corpus CPU percentage cannot identify the mechanism. Nor does a smaller amplification ratio constitute an absolute win.

## Additional source finding: stale phase dispatch

At production head `b4786f66366c9e404702c800cd360dc793b1a7a5`:

- [examples/phases.rs](https://github.com/lmmx/mibz2/blob/b4786f66366c9e404702c800cd360dc793b1a7a5/examples/phases.rs) imports `mibz2::bwt::{build, walk_segments_room, walk_segments_room_parts, ...}`. Its timed build directly calls `build(&ll, &counts, &mut tt)`; its timed default split walk directly calls `walk_segments_room_parts`. Its unsplit check uses `walk_segments_room`. It labels that result “walk, room (used)”.
- [src/lib.rs](https://github.com/lmmx/mibz2/blob/b4786f66366c9e404702c800cd360dc793b1a7a5/src/lib.rs) instead defaults `Scratch<I>` to `bwt::Compact30`; decode_block invokes `s.inverse.invert(...)`. bwt.rs re-exports Compact30 separately.
- Therefore the phase table still times the legacy T-vector path in the candidate build. Its correctness comparison against decoder bytes does not establish that it timed the same implementation.

The published phase table's walk −0.1% [−2.6,+2.4] and build +4.7% [4.0,5.3] must **not** be interpreted as compact30 phase measurements. Its narrative “each build's own” does not remedy the direct calls. This does not invalidate the separate whole-decoder A/B result or justify reopening #155. It limits diagnostic attribution and prevents a false inference that compact30's walk is unchanged while its constructor slowed.

No harness change is made or authorised here. For any later authorised integration, the Coordinator should explicitly establish which kernel each diagnostic calls before interpreting it. This is a source-level finding, not a request for extra timing.

## Trajectory

Retain D016's distinction: an instruction-parity screen can prioritise candidates, but only an admissible lower bound plus all available net credits could exclude that screen. Failure to meet a chosen instruction screen cannot by itself prove failure of every permitted speed-improving formulation. The current CI result rejects this candidate for its performance objective; it does not establish such an impossibility theorem.

No new assignment, implementation, merge, capture, safety certification or authority change. Prior participation in the topology assessment is disclosed; this is not a blind §6.2 review.
