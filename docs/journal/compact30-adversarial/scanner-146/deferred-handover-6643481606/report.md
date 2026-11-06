# Deferred handover: adversarial equivalence assessment

Trigger [6643481606](https://github.com/lmmx/mibz2/pull/145#issuecomment-6643481606).
Candidate `8465207a1d71af51baa1725ecf695c41b52848a2`; baseline `c4b69de363dba4485567142a0d9cec4034fba6e2`. Researcher, 2026-11-04.

**No byte or next[] counterexample found.** The transformation preserves completed segment contents and successors under the existing TVector/segment invariants. “The walk order is unchanged” should mean per-segment traversal and queued segment assignment, not chronological execution order. This is source-level equivalence reasoning plus an abstract differential check, not a fresh Rust/Miri run or production acceptance.

## Required invariant, stronger than “writes only its lane”

1. Sorted, deduplicated starts identify distinct segment IDs. Each queued ID is taken exactly once, in increasing order. Active lanes therefore own distinct buffer allocations. A new take cannot reuse a currently active lane's buffer.
2. During the full phase and tail, T and its START flags are read-only. Flags are removed only after all lanes finish. A flagged row's status cannot change because another lane hands over.
3. An ordinary step reads T at its own row, emits its byte into its own buffer, advances that lane's row and prefetches. It neither reads nor writes another lane's next[] or segment state.
4. Handover finishes the old segment using its unchanged flagged row, sets only next[old segment], and takes the next unassigned segment. take reads T and writes only that fresh segment's first byte. It does not advance another lane or modify T.

Consequently ordinary step j commutes with handover k for j != k. Reading a T row belonging to a queued or active segment does not alias its *output buffer*. This distinction also covers adjacent starts, one-byte segments, self-successor segments and multiple marked lanes.

The implementation drains marked with trailing_zeros and clear-lowest-bit, preserving increasing lane order. Thus queued IDs go to exactly the same lanes. A newly taken segment whose successor is itself flagged must wait until the next offset to finish; both implementations do that, since take emits only its initial byte.

## Queue exhaustion is the real boundary

Let k be the first marked lane with no queued replacement. That lane is finished and excluded by next_unset. For every other lane:

| Lane class | Candidate next write |
|---|---|
| ordinary, before or after k | base[j] + off + 1 |
| marked, already handed over before k | new base[j] + off + 1 = new initial p |
| marked, still pending after k | base[j] + off |

The candidate removes k's bit before computing waiting, so the remaining mask is exactly the pending marked lanes. The old implementation had not stepped ordinary j>k when it exited. Its tail performs those missing steps; commuting those independent steps to the front yields the candidate's state. No marked lane after k has emitted at off, so advancing it would introduce an uninitialised byte. The submitted expression correctly avoids that.

A lane that steps *onto* a START row is still ordinary in this offset and must advance p. It is not marked until the next read. The code tests q before updating row, as required.

## Room and pointer accounting

room is bounded by every active lane's initial spare capacity and cap−1. A replacement buffer has capacity at least cap and has already emitted one byte, so its remaining writes in this room fit. Rebased base = new_p − (off+1), formed with wrapping_sub, restores new_p on the next write; no out-of-allocation intermediate is dereferenced. This is the existing rebasing scheme, retained by the split.

At exhaustion an ordinary lane can now enter the tail exactly full because it already performed its last available write. The existing tail-entry p==end grow handles this before another write. The old tail would instead perform that write and grow afterwards. Allocation/prefetch chronology and trace-counter placement are therefore not promised invariant. Neither difference changes segment bytes or next[] in successful execution.

The mask uses u64 and asserts L<=64. The largest shift is 63 for L=64; the eight-lane loop plus remainder covers every lane once. Cases with fewer active lanes bypass the full loop and retain the old tail.

## Independent abstract check

`schedule-check.js` compares old and split offset execution, including assignment order, finished edges, write tokens, row-progress counts, pointers and exhaustion index. Before comparison it advances the old tail's still-unstepped ordinary lanes—the commutation justified above.

**40,923 cases passed:** all marked masks for L=1..10; every queue availability from zero through marked-count+1; offsets 0,1,7; plus selected empty, first, last, alternating and all-marked masks at L=31,32,63,64. This covers queue exhaustion at every marked position, non-multiple-of-eight lane counts and bit63. It is an abstract schedule check, not native pointer provenance, allocator, compiler, or arbitrary graph execution testing. Its assumptions and normalization are explicit; it is supporting evidence, not a substitute for the source proof.

## Disposition and limits

No new correctness objection to this transformation at the pinned candidate. Safety, compiler validation and end-to-end CI remain separate. The reasoning is specific to this serial T-vector room loop; a cooperative or RAW/parked walk needs its own commutation argument, since shared mutable state or waiting semantics can invalidate these premises.

The “about seven instructions” floor and performance ledger were not established by this review; semantic independence does not prove a machine-code floor. No implementation, merge, timing, gate relaxation or authority change. Prior Researcher work is disclosed; this is not a blind §6.2 certification.

Read current-branch ROLES attempt failed (404); established role record at 0ed278d722d1ef7dea0c24e5ff551dc7c0b46204 was read. The task's explicit authority limits govern.

This triggering batch prohibits an additional source-app message. Findings are published on the Researcher branch and the cursor records that no REVIEW comment was posted.
