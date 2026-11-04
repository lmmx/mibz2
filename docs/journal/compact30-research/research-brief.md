# Actionable research brief

The highest-value authorized next experiment remains hot descriptor arrays, with
the entire call graph charged. A two-LEA saving is exposed on every scalar append;
the model must not buy it by adding uncounted resets, snapshot work or reloads.
This brief also opens a deeper direct-refill investigation. It is a PROPOSAL,
not permission to add a prototype.

## 1. Scalar address work: decision boundary

Frozen `ctor2.rs:540–551`, `one`, indexes a 40-byte descriptor. B2's four common
append paths start at `0x33a00`, `0x34350`, `0x343e0`, `0x344d0`; each is 12
instructions. Two LEAs form `40*c`. Colocated `write[256]` and `end[256]` can
instead use scale-eight indexing and a fixed 2,048-byte displacement. Splitting
the state must be a bijective change of representation, with no duplicated
authoritative pointers. The byte input proves c<256.

The gross removal is 2×38,294,883=76,589,766 instructions. The 71,589,766 estimate
requires all added work ≤5M; this is an acceptance cap. One extra instruction per
finished portion costs 1,795,482; per epoch-character 935,424; per run 610,521;
per block-character 111,104. Charge source reloads on refill, fallback and all
four scalar copies, not only the best-looking inner loop.

Safety obligations cover initialization of every slot, coherent rank and room,
stable record allocation, safe descriptor borrowing across calls, and all
open/close/run/fill/snapshot/sweep paths. Existing malformed counts must still
reject with both arrays empty. Test every character including 255, zero buckets,
crossing/preset/wide/partial records, epochs, maximum size and poisoned reuse.

Falsifiable compilation target: ≤10 instructions in every common scalar copy,
no concealed per-row spill/reload, and complete added-path frequencies. Replace
the memory `addq` with a register increment/store only if its extra instruction
and read reduction are separately reported: it does not also earn the 10-op
target. Oracle/walker, all preregistered Miri classes, and an invariant-targeted
mutation must pass before qualification.

## 2. Direct interior refill: broaden the proof before buying a fast path

Frozen `ctor2.rs:343–382` opens/closes descriptors; `one` calls `refill` at
exhaustion. On a nonempty scalar refill, `write=end`. If the next rank r is
strictly below the bucket limit, the old portion must have reached its record
boundary: otherwise its end was the bucket limit. Thus r is a multiple of 30,
even when the old portion was crossing RAW. In natural mode, if r+30≤limit,
the next record is wholly owned by this bucket and cannot have been preset RAW
by a bucket boundary or already written by another bucket. The successor can
therefore be derived directly after finishing the predecessor.

This broadens the audit's ordinary-old-record condition. The candidate must
still finish the old descriptor exactly once **before** replacing any of its
fields, preserve the record allocation's provenance, assign the current active
epoch to a new B3-derived ordinary portion, and initialize fields only on append.
Do not borrow a complete typed record before all its bytes are initialized. A
new pointer step or invariant is a new safety obligation under charter §8.2;
the exact implementation would require the user's approval in Claude's session.

The desired shape removes transient null writes and generic divide/bitmap/open
work on eligible transitions. It retains initial, exhausted, forced and partial
fallbacks. A final predecessor may be old-epoch ordinary, wide RAW or crossing
RAW; rejecting all those states unnecessarily discards opportunity. Detailed
eligibility accounting and a conservative charged net model are in progress;
no fixed savings are credited yet.

Adversaries: a one-slot crossing predecessor followed by a full ordinary group;
wide promotion before refill; forced RAW; limit=r+29 versus r+30; an epoch
boundary immediately before refill; empty and exhausted buckets; n=0/max;
failure/reuse and full-to-partial transitions. The oracle must verify canonical
RAW permutation, headers, padding/PARK and marker restoration, not only decoded
bytes. A mutation skipping or reordering the old finish must be detected.

## 3. Research priorities and rejection rules

Continue source/ASM tracing through callers and exceptional paths between
checkpoints. Refine missing fragment/refill distributions using archived
counters or a coordinator-published counted instantiation; do not invent them.
The same finish/frame instruction cannot be credited again to a refill proposal.
Descriptor splitting and refill fusion touch the same close/open sites; use a
combined listing to compute their intersection and replacement costs.

Reject the assertion that equal scratch bytes imply equal memory work. Reject
zero-cost allocator/initialization assumptions, statistical samples called hard
bounds, tiny-block ratios promoted into large-block gains, and instructions
converted to cycles. Deprioritize wide-RAW zeroing (≤216,643 candidate stores)
and bulk-only work relative to scalar/refill work on critical large blocks.

Audits of transformations first proposed here will be flagged non-independent.
Charter §6.2 then requires a fresh reviewer given only the pinned evidence.
