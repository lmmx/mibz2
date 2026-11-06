# C6: a worthwhile bounded experiment, not a booked saving

FROM: researcher
TO: coordinator
RE: triggering comment 6634998636; C6 proposal and reassessment
TYPE: REVIEW

**Recommendation: ask the user to register prototype 3b as a bounded refill-only experiment. Do not implement it under the present authority.** The address argument is sound under its explicit preconditions, and the opportunity is large enough to test. The unresolved H_D is a reason to withhold qualification, not by itself a reason to reject an experiment whose purpose is to determine it. Do not make C6 a workaround for S62-F01/F02 or a new dependency that delays their repair.

This is independent analysis of the proposer's evidence, not a blind charter §6.2 review. This Researcher has prior scanner/refill discussions recorded in its cursor. No constructor was implemented, compiled, executed or timed. Only archived text listings and arithmetic were replayed. The current task-run instruction prohibits an additional source-app message: this branch report supplies the REVIEW text; no #145 comment is posted for this delivery.

## Evidence and what was actually checked

- Proposal, descriptor note and results: commit 0ac2e67e6826b9f2b1940cb876c3003e7a022144, under docs/journal/compact30-research/proposals/refill-boundary/.
- Optimiser reassessment: ea3ace12614f8d078f6167c2d8fcda64f0328799, docs/journal/compact30-research/reviews/checkpoint-6633116288/report.md.
- Descriptor source and same-binary listings: 29dd071f8787a28560a7a081fd7ac0ef13991466; examples/c30walk/ctor2d.rs and docs/journal/2026-11-04-compact30-descriptor-prototype/listings/.
- Charter and roles: 09e4ef6cb7c845781b0a2cb0d2c1a4fe6f33fe47.
- Independent findings: e62fc15f91311823ab0c48a6f6e9c7dd35a28d10, docs/journal/compact30-s62-review/report.md.

The adjacent replay.py verifies the Git blob identities of the two copied refill listings and independently counts their executed nonempty/successor-ordinary traces: descriptor 69, B2 62, including the finish call and excluding its body. It reproduces the budget and sensitivities. It does **not** replay the primary compressed counter archive: E_min=1,084,913 is an input from the pinned proposal, not a newly independently observed frequency. The counting argument was examined separately below. The generated replay output is figures.json.

## 1. Is the case sound?

For an open portion starting in group g, its end rank is b=min(lim,30(g+1)). If b+30<=lim, then b<lim and hence b=30(g+1). The successor is a complete group inside the same bucket. Natural premarking cannot mark that interior group crossing; previous writes cannot have completed a future group of this bucket. An old crossing or newly wide portion does not invalidate this argument.

The physical endpoint is record byte 60. Thus the next field begins old_end+4 bytes and its endpoint is old_end+64 bytes. Both are inside the existing record allocation when the successor-room test holds. This reasoning requires the original allocation provenance, not an integer reconstruction of an address. Empty descriptors must fail the shortcut before pointer arithmetic.

The descriptor source supports the other essential premise: finish (lines 450–545) snapshots the descriptor and writes records/RAW data/bitmap/counters, but does not advance wr, en or cold. RAW vector growth is distinct from record-arena storage. Therefore the old endpoint can be reloaded after finish; keeping an extra pointer live across that call is an option, not a mathematical requirement. This matters to the spill argument.

The frequency argument is conservative in form: subtract a ceiling on non-refill finishes from finished portions, then exclude crossing successors and a possible terminal partial group per block, clamping per block. Old crossing portions need not be excluded. Counting only norm_partial would be unsound because a partial group can finish wide. The proposed room test covers that case. Exact eligibility counters would tighten the bound but are not needed to invent a positive exposure.

Pre-implementation falsifiers are documentary/proof checks, not candidate tests:

1. Any legal scalar refill that enters with a nonempty descriptor not fully written, or whose finish mutates the descriptor or relocates its record arena, defeats the argument. No such path was found in the pinned source.
2. A natural, wholly bucket-owned successor already marked RAW would defeat bitmap bypass. Forced configurations must remain on fallback even when a particular successor happens not to be forced.
3. An initial/post-bulk empty descriptor, exhausted malformed bucket, or only 29 successor ranks must never enter the pointer shortcut. The full-successor equality case (exactly 30 ranks) must remain eligible.
4. If composed with v3, opening must assign the current epoch tag. Reusing the old tag is incorrect across an epoch boundary.
5. Finish must occur exactly once on every nonempty path, including failed guards. Falling back through close after already finishing is not equivalent.

These support the proposed experiment; they do not discharge its eventual release-mode safety review, malformed-input rejection, oracle/walker, reuse, mutation or registered Miri gates.

## 2. H_D: bound exposure, do not pretend to bound ungenerated code

The confirmed conditional expression is

    delta_D >= (69-C)*1,084,913 - g*710,569 - H_D.

At C=40 and g=5, this is 27,909,632-H_D. Relative to B2 the corresponding expression is 20,315,241-H. The difference of 7,594,391 at the eligible floor recovers descriptor-wrapper overhead. It is not another portfolio contribution. A full translation also includes the other changed paths and residuals.

The existing descriptor wrapper already has five pushes, four pops and a stack adjustment; B2 has three pushes and three pops. There is also a duplicated r0 store (0x9436f and 0x94422), and three transient null stores (0x94374, 0x9437c, 0x94385). These show real removable work, but no promise that a new compiler allocation removes the frame. Count retained prologue/epilogue, guards and endpoint reloads inside C, not again in H_D. Likewise do not double-charge failed guards already in g.

All four existing caller sites are visible at 0x3b369, 0x3bd11, 0x3bdac and 0x3be77. They already have differing spill/reload sequences. This makes a best-loop-only estimate inadequate. A new mode argument or larger live state can perturb these sequences, stack layout or unrelated loops. Existing assembly cannot furnish a finite upper bound on those *future instruction deltas*.

It can furnish useful cost multipliers:

| One additional instruction on each event | Primary-corpus charge |
|---|---:|
| scalar row | 38,294,883 |
| refill of any class, loose ceiling | 2,517,107 |
| finished portion | 1,795,482 |
| run | 610,521 |
| block-character | 111,104 |
| build/block | 434 |

The refill ceiling is P+256B+R: each nonempty call finishes a distinct portion; each character may start empty once; each run may leave an empty descriptor for a subsequent scalar refill. This is not P, and is not an exact count. Refused malformed-input calls need separate suite evidence, not a fabricated primary frequency.

One added instruction on every scalar row, absent compensating savings, changes the illustrative bound to **−10,385,251**. Keeping mode work off the common scalar path is consequently a substantive design criterion. By contrast, a *hypothetical verified cap* of three residual instructions per refill plus one setup instruction per block would charge at most 7,551,755 and leave 20,357,877. This is a sensitivity example, not an established H_D cap: it excludes nothing by assumption. Each real delta must have a site, class and multiplicity.

At g=5, budgets before H_D are 22,485,067 for C=45, 17,060,502 for C=50, 11,635,937 for C=55, and 6,211,372 for C=60. Missing the diagnostic 40 cap does not establish no value. Conversely C=40 proves no net saving until the residual ledger closes. Mode storage in build state may avoid recurrent argument preparation, and reloading unchanged wr/en after finish may avoid an extra preserved register; both remain hypotheses requiring whole-listing inspection if authorised.

## 3. Cheaper controls and competing explanations

The simplest control is transient-state cleanup: postpone empty-state reset until a path actually leaves the descriptor empty, and eliminate the duplicate r0 write where possible. This preserves the general rank/bitmap route and may recover some cost without the new successor-pointer proof. It still needs a separately scoped, attributed experiment if implementation is proposed; it is not permission to modify the registered descriptor attempt.

The three transient reset instructions alone expose 3*E_min=3,254,739 instructions at the floor, before replacement/fallback cost. Eliminating one redundant r0 store exposes another 1,084,913 at that floor. These are diagnostic site counts, not a net estimate or an exhaustive ceiling on the cleanup route. They explain part of C6's attraction but not the entire claimed 29 instructions per eligible refill. The remaining hypothesis is eliminating rank/address/bitmap reconstruction and possibly frame cost.

A useful stronger control retains the bitmap check while simplifying successor rank/address construction. That separates address work from the natural-mode assumption and mode propagation cost. It may be cheaper to prove yet slower; no generated evidence presently ranks it. Do not turn these controls into an unbounded search or silently implement them as part of C6.

S62-F02 demonstrates overlap of at least 2,393,313 instructions between scanner and descriptor contributions, at the B2 second-slice length reload (0x343b5, build offset +0x12b5). The resulting demonstrated two-prototype floor is 97,606,687, not 100 million, before any further overlap. C6's targeted refill sites are different, but its caller/register effects could touch the same setup region. Distinct core loops do not establish disjoint whole ledgers. Carry an instruction-site intersection and recalculate the combined build against B2; do not add 27.9 million to either isolated sum or treat C6 as cancelling the finding.

S62-F01's missing §5.3(b) counters must also be repaired independently. Existing Callgrind frequencies are useful corroboration, not a substitute for the charter's required counted paths. For C6, preregister mutually exclusive initial/post-run-empty, nonempty eligible, nonempty rejected and exhausted/refused classes, with caller-copy attribution where costs differ. Preserve an exhaustive partition and prove counted instrumentation disappears from uncounted code. The presently reviewed descriptor counters refills/refill_finishes alone do not identify the full C6 partition.

## 4. Scope recommendation

Recommend registration rather than dropping C6 or demanding a pre-compilation proof of H_D. It has a coherent equivalence argument and meaningful cost headroom, and the most consequential uncertainty is exactly what a bounded compiled experiment can resolve. Registration is the user's decision; this review grants no scope.

The proposed registration should name the descriptor baseline, define the incremental net minimum and combined B2 accounting treatment, retain the ordinary attempt limit, and restrict changes to the refill transition and strictly necessary mode plumbing. Keep C<=40/g<=5 diagnostic rather than confusing them with achieved qualification. Require unchanged scalar common paths as an early diagnostic, a complete residual ledger, and the existing safety/test obligations plus the successor-boundary cases. An unexplained common-row increase or unbounded material residual is a stop/HOLD, not licence to optimise unrelated code.

Finish the already-required counter/overlap repairs and preserve their HOLD status until resolved. Approval to study C6 need not be withheld merely because those repairs are unfinished; starting or integrating it must still respect the charter's sequencing and stop conditions. Do not repair accounting by changing thresholds, book unmeasured savings, change the finalizer, or infer a timing benefit from instruction counts.
