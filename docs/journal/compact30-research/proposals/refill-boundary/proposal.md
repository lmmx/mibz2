# Proposal: advance a completed scalar portion directly into its interior successor

**Status: CONDITIONAL research proposal, not implementation permission.** No constructor was
implemented, compiled or run for this work. This expands the secondary refill idea in the Ultra
audit; it is outside the charter's three registered prototypes. Any implementation needs the
user's explicit scope approval in Claude's session (§12.3), including the precise new pointer
arithmetic and state invariant. A later audit by this research thread would be non-independent.

## Result

The predecessor need not have been an ordinary, 30-field portion. A completed crossing RAW
portion is also eligible. If a nonempty scalar refill has next rank `r`, natural mode is active,
and `r + 30 <= lim`, then the next portion is a complete ordinary candidate. Its field start is
the predecessor's end pointer plus **4 bytes**, and its end is that old pointer plus **64 bytes**.

The old finalizer must run exactly once and is unchanged. The opportunity is the wrapper:
avoid nulling three pointers only to overwrite them; avoid dividing the next rank by 30;
avoid reconstructing its record address and reading/testing the RAW bitmap.

Replaying the archived counters block by block gives **at least 1,084,913 eligible natural
transitions**, versus the prior audit's 1,040,391. The aggregate-only fallback is 1,081,119.
This is an exposure floor, not an instruction
saving. A falsifiable illustrative target of at most 40 instructions for the eligible wrapper
(including its guards and finish call), at most five added instructions per ineligible nonempty
refill, yields **20,315,241 − H** instructions saved on B2. `H` is every other added instruction
in the whole build, its callers/callees, setup, initial/refused refills, spills and exceptional
paths; it is unresolved and is **not** assumed zero. No achieved net saving is booked.

## Immutable evidence and replay

- Rules: `REVIEWER.md` at `b302c5d`, blob `f5941d00db4c212ef3dab61230a9a4038e4708f1`;
  active charter blob `31ec5719ca24cd3204b34595401c68ffd31bb57f`; protocol blob
  `01d5391874e756ff90bbd9a822913b201cd9dece`, read at
  `2f15e513bdb9808a2c7504e3f01b59ebce65f221`.
- B2 `examples/c30walk/ctor2.rs` at `8beffe2`, blob
  `168c2bac9ad0d8da4d9f82788f009d32f419608e`: descriptor/rank 228–248; open/close/refill
  343–395; finish 399–495; scalar append 538–551; premarking 801–821; validation/sweep 865–872.
- B3 `examples/c30walk/ctor3.rs` at `dbf5139`, blob
  `01f91419904a605b721dc1742ba8087fe79f74cf`: open/close/refill 410–471; tag and leaf 474–509.
- B2 release wrapper at `cd14fa9`,
  `docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm`,
  Git blob `ef4a78b854385b56cee4ffef2080e4f97dc44c54`; downloaded-byte SHA-256
  `e0270dc301b40c91b6bc91e87bb4d278ced83892984e5d861ae5209d72a286c1`.
- Ultra audit at `2f15e513bdb9808a2c7504e3f01b59ebce65f221`,
  `docs/journal/2026-11-04-compact30-ultra-audit-evidence/report.txt`, blob
  `0ff2f0e594d9459243de9313e7941267f8476229`: aggregates 91–106, refill floor 163,
  descriptor costs 343–368, old full-to-full bound 663.
- Primary archive at `2f15e513bdb9808a2c7504e3f01b59ebce65f221`,
  `docs/journal/2026-11-03-compact30-construction-v2-capture/c30-construction-v2-20261103T222928Z.tar.gz`,
  SHA-256 `ebdb74549b39f83f07ae1f7d1c943916bac2ecb0be85868799aa30fe5e3d8af9`;
  primary `c30-construction.jsonl` SHA-256
  `f25325eb2339b1a1385d598e8dd5a8a6d5e37c110c207ddcff18bae53f2e172b`.
- From this directory run `python3 model.py --repo /path/to/evidence-checkout`.
  From a repository checkout containing this package run
  `python3 docs/journal/compact30-research/proposals/refill-boundary/model.py --repo .`.
  The script parses the archived wrapper, checks 49,800 finite endpoint cases (48,140 with a
  nonzero predecessor start slot), and prints the conditional arithmetic. It constructs no
  compact30 tables and executes no project code. It also replays all 434 primary counter
  records, independently cross-checking the accompanying structural replay. If the archive is
  outside the evidence checkout, provide
  `--archive /path/to/c30-construction-v2-20261103T222928Z.tar.gz`.

## Equivalence proof

Let an open portion start at rank `a` in group `g=floor(a/30)`, with bucket limit `L`. Its end
rank is `b=min(L,30(g+1))`. The scalar caller invokes refill only when `write==end`; a nonempty
descriptor therefore holds exactly this complete portion. The unchanged finish classifies and
finalizes that portion, leaving the descriptor's pointers and ranks unchanged.

If `b+30<=L`, then `b<L`, hence the minimum defining `b` cannot be `L`: necessarily
`b=30(g+1)`. The next 30 ranks `[b,b+30)` lie wholly within the same character bucket. In
natural mode premarking selects only groups crossed by bucket boundaries, so this next group
is not preset RAW. Earlier streaming writes cannot have made it wide or emitted it: each
character's rank only advances; other characters own no rank of this interior group. The
predecessor itself may be crossing RAW, ordinary, or become wide during finish.

With record stride 64 and fields occupying bytes 0…59, the predecessor end pointer is at byte
`64g+60`, even when its start slot was not zero. Thus the replacement descriptor is precisely:

- `r0'=b`, `g'=g+1`, `lim'=L`;
- `w0'=write'=old_end+4 bytes`, `end'=old_end+64 bytes`;
- B2 ordinary sentinel, or B3 **current active tag** (never the predecessor's tag).

These values equal `close(c); open(c,b)` after the original finish. The following scalar append
writes the same label to the same field and advances the same pointer. Snapshot ranks are
therefore equal; RAW creation order, later normalization, header bytes, canonical permutation,
padding and PARK are unaffected. The optimization does not change which source rows are
scalar, fragment or bulk.

## Release-mode safety and exact boundaries

This is a proof obligation for a proposed operation, not a claim that tests justify unsafe code.
No debug assertion supplies a premise.

1. Keep the nonempty test before this shortcut. Empty descriptors have null pointers and can
   arise initially or after bulk emission; pointer advance is forbidden there.
2. Check natural mode and `b+30<=lim` before forming the new pointers. The validated input has
   `0<=b<=lim<=n<=MAX_N=983010`; adding 30 fits u32/usize on the archived x86-64 target.
3. `b+30<=n` proves `g+1<g_count`; the next full record is inside the stable allocation
   reserved before scatter. The proposed raw-pointer byte additions by 4 and 64 retain the
   allocation provenance of `old_end`. They do not dereference the old one-past field pointer.
   Do not substitute integer masking/address reconstruction or create references to an
   incompletely initialized `Rec`.
4. Finish the predecessor before updating any descriptor state. Finish may read all old fields,
   write its header, restore crossing slots, grow RAW storage, or set its old bitmap bit. It does
   not move the record allocation or mutate descriptor state. Its effects must occur once.
5. **Guard failure must not call finish again.** Factor the remaining reset/exhaustion/open
   fallback after the shared finish; blindly invoking `close` again would normalize finalized
   fields a second time and corrupt output.
6. No new-field read occurs on opening. The immediate scalar append initializes the first field;
   the existing progression initializes all later fields before finish reads them. The new group
   can still become wide and follows the existing full-label slow path.
7. In B3, an epoch can have changed since the predecessor opened. Assign `active_tag` at this
   new opening. The same-epoch proof then follows from immediate positive append, not from the
   predecessor's tag or the predecessor's span.
8. Forced configurations always take the original fallback, because the next wholly owned
   group may nevertheless be forced RAW. Malformed counts that exhaust a bucket fail the
   successor-room guard and retain rejection before the next write, with both output arrays
   empty on failure. Reuse never makes stale descriptor state live.

## Frequencies and conditional net budget

The archived blockwise bound is `N >= sum(max(0,P_i-R_i-256)) = 1,122,673` nonempty scalar
refills. There are `P=1,795,482` finished portions, so `N<=P`. Every refill's successor portion
is unique. On a successful primary natural build, the next rank is below the bucket limit.
An ineligible successor is therefore either a crossing portion (at most 41,120 across the
corpus), or a terminal noncrossing partial group (at most one per block, hence at most 434).

The aggregate-only fallback is `E >= 1,122,673 - 41,120 - 434 = 1,081,119`.

The accompanying structural archive replay first derived a tighter blockwise bound; this
proposal's model independently reproduces it. For each block set
`N_i=max(0,P_i-runs_i-256)` and
`E_i=max(0,N_i-fin_crossing_i-int(rows_i%30!=0))`.
The residue indicator allows a terminal partial successor whether it ultimately finishes
ordinary or wide, and is zero when all groups are full. Clamping within each block avoids
subtracting unavailable successors from tiny blocks with no positive refill floor. The replay
checks 434 unique block IDs, 207,006,515 rows, zero forced finishes, and the complete aggregate
finish/crossing totals. It yields **sum(E_i)=1,084,913**, the main `E_min` used below. Every one
of the 230 blocks with at least 899,000 rows has positive `E_i`; the remaining 204 tiny blocks
contribute zero. This improves the prior audit's exposure floor by 44,522, without creating an
achieved instruction saving.

This intentionally differs from the older audit's subtraction of both old and new crossing
portions and only 42 partial normalizations. Old crossing portions are now eligible. The 42
`norm_partial` count alone does not exclude a partial group that finishes **wide**; the 434
ceiling avoids that inference. The old report is not silently changed. Exact blockwise terminal
membership could tighten this bound; all-primary construction captures remain formally INVALID.
These structural counters are not timing evidence.

The parsed B2 nonempty/next-ordinary wrapper is 62 instructions, including its finish call but
excluding the called finish body. **This same 62-instruction wrapper applies when the old
portion is crossing RAW or becomes wide.** Before finish, the only content-dependent branch is
the nonempty test at `0x6234e`, which is not taken in all these cases. The call at `0x62353`
handles the old class inside `finish`; its class-dependent body is excluded from this count
and is retained unchanged. After it returns, the wrapper tests the new rank against the limit,
then probes the **successor** group's bitmap bit at `0x623ce`. The old RAW tag is never read by
the wrapper. The successful successor-ordinary path skips `0x623d4`, `0x623d9`, `0x623dd`
and takes `0x623df`, independently of the predecessor class. The replay publishes the full
62-address trace and verifies the listing's Git blob identity before counting.

Let `C` be the new eligible wrapper length, `g` the added
failed-guard cost, and `H` the whole-build residual added work. Then:

`net >= (62-C)*E_min - g*(P-E_min) - H`, for nonnegative saving/guard coefficients.

At the proposed diagnostic caps `C<=40` and `g<=5`, this is `20,315,241-H`. Those caps are
falsifiable compiler requirements, not predictions. If either is missed, recompute. `H` must
include any mode-field initialization, additional argument passing, moved address arithmetic,
initial/refused refill changes, prologue/epilogue changes, spills/reloads and all other changed
paths. A relevant unbounded `H` forbids claiming a verified net saving.

## Intended machine shape and memory ledger

Keep the existing finalizer call. After it, the eligible path should use the existing rank
advance, a mode guard, a room guard, constant pointer increments and direct descriptor updates.
It should contain no rank division, bitmap read/test, null-reset stores, or record-address
reconstruction from `dst` on that path. Preserve the old fallback for all other cases.

| B2 wrapper anchors | Instructions exposed | Intended replacement |
|---|---:|---|
| 0x62363–0x6236a | 3 | no transient null reset on eligible transitions |
| 0x6237f–0x6239c | 9 | use group successor, not division by 30 |
| 0x6239f–0x623a5 | 3 | guard full successor instead of min/clamp |
| 0x623a8–0x623b3 | 4 | no next-group bitmap bounds/load setup |
| 0x623b5–0x623c3 | 5 | constant advance from old end |
| 0x623c6–0x623d2 | 4 | no next-group bitmap read/test |

These exposed region counts are **not summed as net savings**: replacement instructions and
guards are real costs. The final instruction cost is the complete wrapper plus all caller and
whole-constructor deltas.

The original null reset writes 24 logical bytes through two store instructions; removing it on
`E_min` events eliminates at least **26,037,912 transient store bytes** before shifted costs.
The final descriptor writes still occur. Loading a mode flag, reloading old end after finish,
and incrementing `g` with a memory RMW can add explicit reads. A representative unspecialized
shape exchanges four old address/bitmap reads for three new reads (mode, old end, g); this is
conditional, and stack traffic or compiler reuse may change it. Report loads, stores and RMW
separately from instruction counts; do not label logical byte removal as cache/DRAM traffic
or translate any count into cycles.

## Adversarial gates and overlap

Required oracle/walker, reuse, malformed-count, full synthetic and registered Miri classes:

- old portion starts at slot 29 (one initialized field), then a full interior successor;
- old crossing portions of every start slot 1…29; several buckets sharing that old RAW group;
- old ordinary portion promoted wide; old RAW creation must not mark the successor;
- old portion opened in an earlier epoch; successor begins on 61,440 and 122,880 boundaries;
- exactly 29 versus 30 successor rows available; nonzero terminal padding; empty input;
- forced-next-group after ordinary predecessor; all forced configurations stay on fallback;
- initially empty and post-bulk empty descriptors; exhausted malformed bucket; reuse after
  success and failure, both growing and shrinking output capacities;
- all four scalar caller copies, including source tail and label-hole split copies.

Mutations must separately expose stale tag propagation, skipping natural-mode guard, using
`old_end+0`/`+2` instead of `+4` bytes, accepting only 29 rows of room, and double finishing on
guard failure. Every targeted Miri class must complete. Assembly gates cover uncounted release
`build::<false>`, refill, finish, all callers/fallbacks and the pinned toolchain, with unchanged
frozen files. Add counted-only exact eligibility/guard/initial/refusal counters; verify the
uncounted listing unchanged by their introduction.

Do not add this estimate directly to descriptor-array or v3 estimates. It touches the refill
helper, whose changed costs already belong to descriptor-array integration accounting. The
v3 finish leaf is unchanged and earns no second credit; current-tag opening work is charged
once. Scanner scalar address LEAs and scanner control are distinct old instruction sites, but
register allocation/caller costs can interact. Combined savings require a fresh B2 recount.

For priority, the entire demonstrated eligible floor occurs on the 230 large blocks, rather
than the 204 tiny blocks. This strengthens the earlier scalar-work exposure argument. It does
not establish how many eligible refills execute on each four-worker critical path, nor a
completion-time benefit: those require the separate archived worker-assignment/exposure
analysis and eventual authorized valid measurement. No scheduling or timing change is proposed.

## Rejected shortcuts

- Inferring `next_raw=false` from the old RAW tag alone fails for forced successors; natural
  mode and full-bucket containment are essential.
- Reusing the old epoch tag fails across source epoch boundaries.
- Removing the end comparison on every scalar row using pointer low bits misses bucket ends
  inside crossing groups; a replacement discriminator adds recurrent work and has no proved
  net advantage.
- Splitting the scalar memory RMW into register increment plus store removes an explicit read
  but adds one instruction per scalar row (38,294,883); that is not instruction minimization.
- This is not permission to route runs through a new helper or change the constructor's
  format, finalizer, oracle, walker, protocol or production sources.
