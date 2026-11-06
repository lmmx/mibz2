# Ranked next cuts after Assembly 1

Trigger: PR #145 comment 6639496192, a re-delivery of 6637300905.  
Policy pin: `2210e4457b383e4566df99eb881b4ca0401c4cd3`.  
Baseline: B2 `8beffe28bec9b539517b442350d8a3fda62164c8`.  
Assembly-1 components audited at their pins: scanner U2 `ba261eff245f45e1c07fdfb05fa2fe6ff1ced835`, descriptor arrays `7620c0c667018772a0ba3be28eb62263b55037f8`, and v3 `dbf5139e12ae60b5f5bff40059b88d65ddccea41`.

This is a proposal-only static audit. It does not implement a constructor, compile a candidate, time anything, qualify the combined build, or authorize integration. All proposed rows are CONDITIONAL and contribute zero verified cumulative credit.

## Accounting basis

The counted Assembly-1 frequencies used here are:

- 38,294,883 scalar rows; the descriptor-relative scalar append is 10 instructions per row.
- 5,127,300 complete groups and 153,819,000 bulk rows (74.3% of rows).
- 1,647,416 fast finishes; 75,997 ordinary, 30,949 wide, and 41,120 crossing finishes.
- 496,004 run opens.
- 610,356 lookahead runs that reach a terminal tile.
- 1,084,913 is the registered lower bound for direct-interior refill eligibility.

The combined registered floor remains `F = 97,606,687`; with v3 included the stated component floor is 147,606,687, less any further demonstrated overlap. None of the numbers below are added to either floor.

Every estimate is incremental to Assembly 1. U2's uniform-tile removal, descriptor-array savings, the v3 replacement body, and registered prototype 3b are excluded. Any candidate that changes a 3b event frequency must charge that change in a fresh combined recount.

## Ranking

| Rank | ID | Cut | Counted frequency | Conditional incremental target | Why this rank |
|---:|---|---|---:|---:|---|
| 1 | C7 | Fold ordinary/full/current-epoch state into the v3 tag | 1,647,416 fast finishes; at most 1,795,482 portion opens | **6,293,532 − H** at a two-instruction/open cap; **8,089,014 − H** at one | Directly shortens the binding fast-finish path and removes its pointer-distance dependency chain |
| 2 | C8 | Natural-mode residual run-open specialization | 496,004 run opens | **8,432,068 − H** at a 16-instruction specialized-open cap | Removes generic reopen work at run boundaries, a completion-relevant transition rather than bulk-only throughput |
| 3 | C9 | Four-record natural bulk kernel | 5,127,300 complete groups | **10,254,600 − H** at an eight-instruction/group average cap | Credible exact frequency and disjoint site, but most benefit lies on the 74.3% bulk path rather than the known binding finish path |
| 4 | C10 | Extend a U2 run through the equal-byte prefix of its terminal nonuniform tile | 610,356 terminal tiles; unknown captured rows `E_f` | **10E_f − C_s·610,356 − C_b·G_f − H**; `0 ≤ E_f ≤ 9,155,340` | Potentially material scalar relief, but the decisive prefix-length and added-boundary histograms do not exist yet |

`H` is all new setup, dispatch, register-allocation, spill, tail, and caller cost not already explicit in the formula. No proposal is credible if a fresh listing cannot keep `H` bounded and separately enumerated.

## C7 — full-state v3 tag

The current v3 fast leaf first checks the active tag, then derives `write - w0` and compares it with 60 bytes. The latter sequence is six instructions on each fast finish. The portion length is already known when `open` establishes the descriptor.

Encode a spare FULL bit in the ordinary tag and compare against `active_full_tag` in the leaf. Equality then proves, in one predicate, current epoch, ordinary representation, and exactly 30 fields. The slow path remains authoritative for partial, old-epoch, crossing, and RAW portions.

Gross leaf removal is:

```
6 × 1,647,416 = 9,884,496
```

Charging at most two new tag-construction instructions at every 1,795,482 observed finish/open event gives:

```
9,884,496 − 2 × 1,795,482 = 6,293,532
```

A one-instruction/open listing raises the target to 8,089,014. The candidate is compiler-achievable only if the combined listing meets one of those explicit caps and does not spill the scalar append or refill paths.

Memory/output: the fast leaf removes the `w0` and `write` loads used only for the fullness test; it adds no logical output bytes and no new memory traffic at open. Exact explicit-read changes must be recounted.

Unsafe formulation: none is required. This is tag construction and comparison. The invariant is that the FULL bit is set iff `re - r == 30`, the ordinary bit pattern cannot collide with RAW indices, and the epoch bits come from the same active snapshot used by the slow path. A proof must enumerate the available bit range.

Overlap: this is incremental to v3/A3, never another copy of v3's 60M credit. It does not claim U2, descriptor, or 3b savings. Because it changes tag construction at open/refill, the combined listing must still check 3b register pressure and shifted instructions.

## C8 — natural residual run-open

After `run` emits its whole groups, a residual `m < 30` currently reaches the generic open machinery. In the natural-mode build, the next group is already determined by the just-completed group sequence. The generic division, bitmap test, RAW/crossing discrimination, and address reconstruction are formulation artifacts for this event class.

Specialize this residual open for natural mode. Initialize the ordinary descriptor directly from the known next group pointer and residual rank, leaving the forced-mode/generic path unchanged. At the observed 33-instruction candidate open and a 16-instruction specialized cap:

```
(33 − 16) × 496,004 = 8,432,068
```

This is compiler-achievable only if a monomorphized natural-mode path has no per-row dispatch and the 16-instruction cap includes all descriptor initialization. A once-per-block dispatch may be charged in `H`.

Memory/output: logical writes are unchanged. The specialized event should eliminate the generic bitmap/header read and its address dependency, but no read/write reduction is booked until the listing and memory replay identify it.

Unsafe formulation: the direct version may use the existing raw destination pointer. Its invariant is: natural mode; `rank + m ≤ lim`; all whole groups preceding the residual have been emitted; the next group lies within the bucket's pre-reserved allocation; the group cannot be forced/RAW/crossing; and the initialized descriptor end remains within that one 30-field record. A safe indexed formulation is acceptable if it meets the same listing cap.

Overlap: the 496,004 run-open events are distinct from 3b's scalar predecessor/interior refill events. If an implementation generalizes 3b to run opens, the event sets must be partitioned and the shared events credited once.

## C9 — four-record natural bulk kernel

The current bulk emitter core costs 51,273,000–51,883,521 instructions for 5,127,300 complete groups. Four 16-byte stores per 64-byte record are the unavoidable data movement; the candidate target is to amortize loop, address, and header recurrence across four records.

A four-record unrolled kernel, with a scalar tail, is conditional on an average cap of eight instructions per emitted group. Using the lower end of the existing core interval:

```
51,273,000 − 8 × 5,127,300 = 10,254,600
```

This is a formulation cut, not removal of the record stores. It is compiler-achievable only if the listing reaches the eight-instruction average without adding setup outside the counted core or inflating callers.

Memory/output: exactly the same four record stores and bytes per group; only instruction and dependency overhead may fall. No memory saving is claimed.

Unsafe formulation: direct writes to four consecutive uninitialized `Rec` slots are appropriate. The invariant is `g + 4 ≤ whole`, the destination allocation covers the four records, each slot is written exactly once, stores do not overlap, header/base arithmetic cannot overflow, and a checked remainder loop handles fewer than four groups. Creating references to uninitialized records is unnecessary and should be avoided.

Overlap: this is inside `complete`, disjoint from U2 classification, descriptor scalar append, v3 finish, and 3b refill. U2 determines its observed frequency, so only the Assembly-1 frequency above may be used.

## C10 — terminal-tile run fringe

U2 recognizes a run through full uniform 16-byte tiles. When lookahead stops on a nonuniform tile, a leading prefix of 1–15 bytes may still equal the held run byte; those rows currently fall back to the 10-instruction descriptor scalar append.

Before releasing the held run, compare the terminal tile with a broadcast held byte and use a bit scan to obtain only the equal leading prefix. Extend the existing run by that prefix and send the non-prefix remainder through the unchanged scalar tile path.

Let:

- `E_f` be captured prefix rows;
- `G_f` be additional group-boundary/refill events caused by extending the run;
- `C_s` be added scanner instructions per one of 610,356 terminal tiles;
- `C_b` be the fully counted cost per additional boundary event.

Then the only admissible target is:

```
10E_f − C_s × 610,356 − C_b × G_f − H
```

Because a prefix is at most 15 bytes, `E_f ≤ 15 × 610,356 = 9,155,340`. That is a bound, not an estimate. Add `E_f`, its 0–15 histogram, and `G_f` counters to the counted oracle before ranking this above C9. A reasonable prototype gate is `C_s ≤ 6`; no net is registered without the histogram.

Memory/output: captured rows leave the scalar pointer-RMW chain and join the existing run/fill stores. Logical output is unchanged. Exact read/write changes and any altered 3b eligibility must be replayed.

Unsafe formulation: a safe 16-byte slice load plus integer comparison is sufficient. If an unaligned raw load is used, the invariant is that the terminal tile has 16 available source bytes, the load is explicitly unaligned, the bit scan is defined for the all-equal and no-prefix cases, and the returned prefix is in `0..=15`. The run extension must preserve `rank + prefix ≤ lim`.

Overlap: only rows currently left scalar by U2 are eligible; U2's uniform-tile credit is untouched. The estimate is descriptor-relative, so A4 is not counted again. Any loss or gain in registered 3b transitions is charged separately in the combined replay.

## Bounded non-candidates

- **Descriptor scalar append:** the counted 10-instruction memory-RMW formulation beat the attempted register load/add/store form, which compiled to 11. Arbitrary-byte 256-way online scatter still needs character selection, descriptor lookup, end test, store, pointer advance, and source-loop progress. No further credit is credible without changing the formulation and producing a listing; this row is treated as locally irreducible.
- **Remaining-count descriptors:** replacing the end-pointer comparison with a loaded/decremented/stored count adds a write dependency per scalar row and has no demonstrated instruction reduction.
- **Pre-normalize every scalar label:** it adds work on 38,294,883 scalar rows and on fill/bulk paths to attack SIMD work in only about 1.65M fast finishes. It is dominated by C7 unless a listing proves a whole-path net after all added stores and dependencies.
- **Existing C5 SSE2 first-chunk refinement:** it remains a 6,589,500 gross, net-unknown incremental v3 proposal with a 4M minimum. It is not re-registered or added to the four rows above.

## Required evidence for any registration or reassessment

For each candidate: pin source and assembly, exact event counters, a changed-site frequency map, raw memory replay, a normalized uncounted-closure statement, and a combined Assembly-1 listing that subtracts any changed 3b eligibility. Qualification and timing remain outside this proposal.
