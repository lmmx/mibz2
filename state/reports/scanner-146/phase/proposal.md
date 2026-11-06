# Phase from the label: a bounded state-elimination hypothesis

**Disposition: CONDITIONAL research target; no implementation authority and no achieved net credit.**
The reviewer originated and developed this idea, so any later formal audit of an implementation
by this reviewer would be non-independent. A fresh evidence-only reviewer is required where
independence is required. No constructor source was changed, compiled or executed.

The useful observation is `p = y & 31`, not a claim that deleting a Rust field frees a register.
The current scanner already keeps persistent `p` in memory. A plausible benefit is eliminating
its recurrent maintenance; extra extraction work and compiler consequences may consume it.

## Evidence and reproduction

Source and listings are at PR #146 pin `7f74e9f30e31af33b0d66fec3fe89af1758307d9`:

- `examples/c30walk/ctor2s.rs`: state 319–322; advance 341–350; tile 600–628;
  fill 635–648; complete 652–695; run 700–739; initialization 795–805; scanner 875–936.
- `docs/journal/2026-11-04-compact30-scanner-prototype/listings/ctor2s-build-false.asm`:
  full-tile advance `0x41a10–0x41a42`; setup `0x41ae8–0x41b27`; first refill detour
  `0x41bae–0x41bf2`; fragment update `0x41e00–0x41e1e`; bulk `0x421c0–0x42335`.
- Same directory, `v2-build-false-same-binary.asm`: equivalent B2 full-tile advance
  `0x3b1d0–0x3b200`, 11 instructions. The archived B2 listing has this block at
  `0x33a90–0x33ac0` (use address-derived instruction anchors, not line alignment).
- The primary natural `corpora.jsonl` row supplies `S=2,393,313` nonuniform full tiles.
  The independent accounting replay maps both full-tile advance entries to exactly S.

Run `python3 model.py --repo /path/to/pinned/evidence`. The script checks all 983,011
supported source-position values, 7,936 bounded advance cases, 35,360 temporary-label values,
992 bulk transitions, listing counts and conditional arithmetic. It reads only these files and
implements integer identities, not a constructor. `results.json` records file hashes. These
finite checks supplement the proof below; they do not replace it.

## Equivalence and safety obligations

At a successful operation boundary let `i` be the next source row and
`y=phi(i)=32*floor(i/30)+(i mod 30)`. Write `p=i mod 30`, so `0<=p<30`.
Because 32 is a power of two and p occupies only the low five bits, `y & 31 = p`.
This remains true after truncating y to u16, since 65536 is divisible by 32, but the
full `u32` y must remain available for headers, epochs and the finalizer.

For `0<=k<=30`, `p+k<=59`, hence there is at most one crossing. The unchanged logical
transition is

`y' = y + k + 2*[((y & 31)+k)>=30] = phi(i+k)`.

Its low bits equal `(p+k) mod 30`. Initialization has `i=y=p=0`. The source's `fill`
advances by a portion's room, at most 30. `tile` advances by 16 on its full path and
by repeated ones on the tail. `complete` advances by whole groups: adding `32*whole`
preserves the low five bits and corresponds to consuming `30*whole` rows. `run` is a
composition of these operations. Epoch boundaries preserve the invariant because 61,440
is divisible by 30 and 16. Snapshot, open, close and refill do not advance the source.

The full-tile entry phases are the 15 even residues because tiles start at multiples of
16; this restriction must **not** be applied to fill: run fragment boundaries can have odd
phase. Both the algebra and model cover all 30 phases and every `k=0..30`. For a tile
of length `len<=16`, `n1=min(len,30-(y&31))` gives the same partition and the same
temporary label `u+t+2*[t>=n1]` as the original code. The model covers 16-bit epoch wrapping.

On a malformed input, a scalar append may reject partway through a tile. The successful
operation-boundary invariant is then not a claim that y tracks each partially written row.
No further source step is taken, both published arrays are empty, and W is discarded.
Each reused build initializes its new W at zero. Preserve this rejection/publication flow.

No new unsafe operation is needed for the mathematical replacement. Existing output-pointer
obligations remain exactly as before; this note does not substitute for their complete proof.
Input validation gives `i<=MAX_N=983010`, so `phi(i)<=1048544<2^20`; legal advances stay in
range. Do not apply the single-crossing formula to a coalesced run length greater than 30.

**A mask is not a compiler proof of the valid image.** `y&31` alone is in `0..31`, and 30/31
remain syntactically possible. Safe indexing and existing checks must remain; this proposal
does not use `assert_unchecked`, `unreachable_unchecked`, unchecked indexing or any assumed
range fact. The compiler can retain phase checks. That is a target miss, not a reason to
introduce an unsafe assumption.

Counterexamples in the model reject three tempting shortcuts: `y%30` is wrong already at
i=30 (32%30=2); arbitrary y=30 gives invalid phase 30; and p=29,k=31 violates the one-crossing
formula (62 instead of phi(60)=64). The exact full-tile crossing boundary is p=14, not p>14.

## What the existing machine code supports

Scanner persistent p resides at `0x28e8(%rsp)` and y at `0x28f0(%rsp)`; B2 uses
`0x28d8` and `0x28e0`. Scanner full-tile advance is 12 instructions; B2 is 11 because
its label increment is already one LEA. The scanner's separate phase load, phase selection
and phase store are real recurrent instructions.

A falsifiable target is a y-only full-tile advance of **at most eight instructions**, one
explicit y load and one y store, computing the same carry. An illustrative cost shape is:
load y; copy for the phase; mask; clear carry register; compare phase with 14; materialize
carry; form y+16+2*carry; store y. This is an instruction-budget sketch, not compiled code.
At the tile setup, replacing independent p and y loads with y load plus copy/mask may cost
**one additional instruction** even while removing a read. All existing checks remain in the
setup target; no U3 branch-deletion credit is included.

The current first-loop refill stores/reloads r9, which holds `30-p`, at `0x41bb3/0x41be8`.
That derived value can still be live after deleting p. Reconstructing it after refill could
cost more instructions than its spill/reload. The other saved values are source pointer and
remaining length. Consequently **no refill saving is credited** and there is no established
register-pressure win. Removal of p storage can also change the W layout and every helper's
addressing; all such costs belong in the complete recount.

## Conditional budgets, overlap and falsification

Under both caps, the full-tile-only saving is:

- scanner baseline: `(12-8-1)*S = 7,179,939`, before all other deltas;
- B2 baseline: `(11-8-1)*S = 4,786,626`, before all other deltas.

Write the prospective net as `7,179,939-H_scanner` or `4,786,626-H_B2`. H is every other
changed instruction, including extraction at fill/complete/tails, phase guards, initialization,
W-layout changes, spills/reloads, calls, callees, unsuccessful/malformed paths and external
work. H is unresolved, not zero. A negative-cost component may be demonstrated later, but
none is presumed. If either compiler cap is missed, these arithmetic targets are withdrawn.

The corresponding scoped target removes one phase write and two explicit reads per full
tile (one in advance, one in setup), before all H memory effects. These are logical explicit
operations, not cache/DRAM traffic. No independent memory profile, net effect or cycle benefit
is claimed. Deriving n1 from y creates a dependence on the y load and mask, where the current
p load can be independent; fewer instructions need not shorten the critical chain.

This is **not additive** with scanner corrections or U3. The scanner's extra advance instruction
is already charged in its own-body difference. U3's claimed two-instruction phase branch
removal touches setup, and any integration must partition sites rather than add budgets.
Descriptor changes and direct-interior-refill proposals share W-layout/caller effects even
where their credited sites differ. A fresh whole-constructor B2 recount is required.

The 7,179,939 scanner-relative target exceeds the 3,804,183 own-body shortfall to 60M only
conditionally; all H and the unresolved callee/caller terms must still fit. It is not a
qualification argument. The B2 figure is deliberately smaller and is the relevant isolated
comparison for a new transformation. No critical-worker timing inference is available from
these aggregate counts.

Stop this hypothesis if the compiler merely exchanges the p load/store for extra masked
copies without meeting the scoped targets, or if other costs erase the saving. A future
authorized implementation would need all existing correctness/malformed/reuse/Miri classes,
phase/crossing mutations, unchanged row-loop bodies, and complete changed-function accounting.
This note recommends no attempt and authorizes none.

## Competing smaller hypothesis: reuse the existing segment-length discriminator

The parent review identified a more direct candidate for this particular spill pair.
At `0x41af0–0x41b02`, r9 holds `room=30-p` and rbx receives `n1=min(16,room)`.
After the clamp, the normal first-segment path uses r9 only for the later test
`cmp $0xf,%r9` at `0x41c1b`; its `ja` edge skips the second segment. In between,
the first-loop refill saves r9 at `0x41bb3` and reloads it at `0x41be8`.

For every unsigned room, not only valid phases,

`room > 15 <=> min(16,room) == 16`.

Thus the same decision can use the already-live, callee-saved rbx=n1. It is also used
for the first-loop bound and second-segment pointer arithmetic, so retaining it introduces
no additional live value in the current listing. The abstract model checks this predicate
and replays the archived exact frequency **F=996,634** at each spill/reload address.
The maximal exposed pair is **2F=1,993,268 instructions**, one explicit read and one
explicit write per F. This is a scanner-relative opportunity, not verified net saving.

A falsifiable target is that the existing predicate uses n1 and both r9 spill/reload sites
disappear, with no replacement instructions on that detour or row loop. Then the candidate
budget is `1,993,268-H`, where H contains every other compiler, caller and callee effect.
The current x86 instructions already demonstrate that register allocation does not have to
preserve r9 for the semantics. They do not demonstrate that a permitted Rust expression will
make LLVM choose that shape. A source design and any attempt remain the coordinator's work
under its actual authority. No additional range assumption or unsafe operation is proposed.

The B2 first-segment detour does not contain this exact pair, so this exposure is recovery of
scanner-added work. Do not book it as a new 1.993M B2 isolated saving. Its sites are outside
the U1/U2 eight-check set, but changing either may alter register allocation and absorb this
benefit; no sum is justified. The persistent-phase hypothesis also may make this particular
live value vanish indirectly. Their combined net must count the pair once.

This smaller hypothesis deserves priority for static compiler-target discussion: it attacks
the demonstrated live value and its two memory operations directly, whereas phase-state
elimination touches all source advancement and changes layout. The reviewed checkpoint is
unchanged and remains nonqualified. Developing either proposal makes the reviewer
non-independent for its later formal audit.
