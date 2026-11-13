# Guarded writer safety argument

This is the implementer's argument for independent Researcher review, not a B5
clearance or a claim that MONODROMY has approved the queued proposal6850491677.
It builds on corrected source d4f0e8fb3ac68fa649e1810dff58dc06e9c545c2 and the
existing slice-one safety argument. Induction and its admission/budget are exact.

1. **The predicate covers the factors actually written.** Each marked rule has
   exactly one marked child. The existing origin walk follows that child and sees
   its unmarked sibling. `all_small` is the conjunction of sibling length<=16
   along the entire walk, including both left/right marked cases. The grammar
   buffer is immutable throughout this walk and writing. Consequently the second
   and third marked walks encounter the same siblings. The guard does not inspect
   only a prefix or use an unrelated capped run count.
2. **All-small implies no position access.** The second walk's only position
   access is in its `lx>16` branch. That branch is unreachable when the guard is
   true. All original short copies and the marked byte store still execute.
   Had the old position array been initialized, no seed would change any NIL
   entry; both rule passes would therefore skip every rule. The third walk's
   only copy is also guarded by `lx>16`. Omitting those three no-op passes and
   the initialization changes no emitted byte. Constructing a pointer to an
   empty Vec is permitted; the all-small route never dereferences it.
3. **Origin and copies remain exact.** The old `j` additions, child selection,
   `(j+SHIFT)%p`, destination arithmetic, `copy_small`, marked-byte store, output
   size/slack, repetition and truncation are unchanged. No new copy is introduced.
   Existing invariants for wrapped output, fixed-width over-copy, original-row
   phase and short cycles therefore remain required and apply unchanged. The
   guard does not justify reducing output slack or omitting any copy.
4. **Any long factor retains the original writer.** When the conjunction is
   false, all `nr` position entries are initialized to NIL before the second
   walk. The seed, descending propagation, ascending expansion and final F01
   recopy execute their original operations and order. Initialization moves
   after an immutable grammar read; no operation in that read uses `pos`.
5. **Reuse and allocation bounds.** A small-only block may retain stale private
   position storage from an earlier block, but cannot read or write it. The next
   long block resets every entry used to NIL. Capacity bounds do not increase;
   fresh small-only blocks allocate no position table. No public layout, buffer
   ownership, fallback or team scheduling changes. The guard is one stack bool;
   test-only reach fields are absent from production builds.

The proof relies on the existing unique-mark, acyclic-rule and bounds invariants;
it does not establish them anew. Invalid input still follows the existing checks
and fallback. The changed control flow is inside existing unsafe code, so the
independent review must check both the omission argument and the retained path.

Tests assert exact16/17 boundaries, all-small reach with zero position capacity,
poisoned output, retained non-NIL positions across small blocks, short cycles,
return to long F01 propagation/recopy, every origin on small arbitrary columns,
real-decoder admission/refusal/error/CRC parity, and forced-route fuzz reach.
Miri runs the new witness and all prior Cells witnesses. Mutations omit a small
copy and the long F01 recopy while retaining counters: output equality must fail,
not merely reach bookkeeping. Existing invariant mutations remain required.
