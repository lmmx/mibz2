# Independent correctness and memory-safety audit — compact30 constructor v2

**Disposition: PASS for a separate construction-cost protocol/design review.** No material correctness or memory-safety defect identified in the frozen baseline x86-64 implementation. This **does not authorize timing**, imply a performance improvement, or constitute a formal Rust safety proof.

## Scope and provenance

- Source: [`examples/c30walk/ctor2.rs` at `8beffe2`](https://github.com/lmmx/mibz2/blob/8beffe2/examples/c30walk/ctor2.rs), all 1,767 source lines (1,768 with final line split). GitHub blob SHA `168c2bac9ad0d8da4d9f82788f009d32f419608e`.
- Claims checked: [`2026-11-03-compact30-constructor-v2-prototype.md` at `cd14fa9`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype.md), 464 substantive lines. `8beffe2..cd14fa9` changes only documentation/evidence; the implementation blob is unchanged.
- Cross-checked: frozen oracle `C30::build` and `Rec`/`RawG` definitions in [`examples/c30walk.rs` at `e003a32`](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk.rs); frozen constructor v1 in [`ctor.rs` at `e003a32`](https://github.com/lmmx/mibz2/blob/e003a32/examples/c30walk/ctor.rs), its canonicalization; `src/bwt.rs` for production counting scatter.
- Generated assembly inspected: [`ctor2-build-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-build-false.asm), [`ctor2-finish-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-finish-false.asm), [`ctor2-refill-false.asm`](https://github.com/lmmx/mibz2/blob/cd14fa9/docs/journal/2026-11-03-compact30-constructor-v2-prototype/ctor2-refill-false.asm), and `excerpts.asm`.
- The source was audited by GitHub connector; no repository modifications, Rust benchmark, i9 job or performance run was performed. Independent checks are a Python mathematical model, **not execution of the Rust source**.

## 1. Unsafe arena initialization and pointer validity — source-level sign-off

**Locations:** `ctor2.rs:228–249` (`Desc`/`rank`); `282–319` (`field`, `advance`, `create_raw`); `346–395` (`open`, `close`, `refill`); `690–789` (arena, failure and publication).

The independent invariant is a per-character destination cursor `rank(c)` over disjoint counting-sort buckets `[C[c],C[c]+count[c])`. An empty descriptor uses three null pointers, and `rank` returns its stored `r0`; an open descriptor's `w0`, `write` and `end` point into the **same record field array**, with `w0 <= write <= end`. The integer difference `(write as usize - w0 as usize)/2` is therefore a nonnegative byte distance divided by the size of `u16`. It is not general-purpose pointer subtraction: the proof relies on descriptors being created and advanced only through the inspected paths.

`open(c,r)` (`346–367`) executes only for `r < lim`, with `lim <= n` (checked by the bucket cursor and precomputed limits). It computes `g=r/30 < ceil(n/30)`, `w0=field(r)`, and `end=w0+min(30-(r%30),lim-r)`. This end may be the one-past field pointer (at record offset 60), but is used only for comparison/length calculation; no write occurs there. `one` (`540–551`) writes only if `write != end` or a successful `refill`; `fill` (`583–596`) writes at most the available room. `run` checks `rank+m<=lim` before any write and emits bulk groups only at aligned ranks. Thus all raw field writes and full-record writes remain in the reserved `G+1` arena even if supplied counts have the right sum but disagree with `L`.

The initialization state is safe on success: each rank below `n` is written exactly once by a character's monotonically increasing bucket cursor; non-RAW group headers are written on ordinary `finish` or direct bulk emission; premarked RAW and wide groups are installed as whole `Rec` values; unused final-group fields and PARK are explicit (`763–775`). The only `rec.set_len(G+1)` occurs at `781`, after all portion finalization and canonicalization. During rejection, `rec.len()` remains zero and `raw.clear()` discards only initialized `RawG` entries. Prior capacity is reusable; the bitmap is cleared and zero-resized, snapshots cleared and descriptors initialized afresh (`698–741`). There is no pointer into `raw` stored in a descriptor, so safe `raw.push` reallocation cannot invalidate its raw-index field. `rec` does not grow after `dst` is acquired.

**Source-level result:** no invalid pointer target, unwritten-header read or exposure via `set_len` identified in valid, rejected or reused builds. **Supporting tests:** the reported corpus, synthetic/reuse and malformed cases, plus partial Miri. **Unresolved:** no formal provenance proof or full Miri run; reviewed for the frozen 64-bit x86-64 target only.

## 2. Deferred finalization exactly once, and the bulk bypass — source-level sign-off

**Locations:** `371–381` (`close`), `387–395` (`refill`), `400–495` (`finish`), `540–681` (`one`, `tile`, `fill`, `complete`, `run`), `865–873` (end of scan).

`close(c)` calls `finish(c)` only if its descriptor has written data, then updates `r0` from the existing pointer difference and nulls the three pointers. After closing, the old portion cannot be reached again. `refill` is entered only when the descriptor is empty or `write == end`. In a run, if rows remain after `fill`, the whole current room must have been consumed (`k=min(room,m)`), and only then is it closed. At end of scan, `rank(c)==lim(c)` for every character is checked *before* closing anything; since `end` is the smaller of the group boundary and bucket limit, any still-open portion for an exhausted byte must be full. Thus the release-mode `debug_assert!(write==end)` at `403` is implied by preceding release branches.

The run emitter (`669–676`) only operates at a rank divisible by 30, after closing the prior portion, with enough remaining rows and bucket room to cover its entire group. It writes final `Rec`s and updates the empty descriptor's `r0`; none of those groups is later opened or normalized. In forced-RAW mode the bulk writer checks the RAW bitmap and writes the preallocated group's entries directly.

**Result:** no double finish, skipped pending portion, or bulk-final-record normalization path found. **Test evidence:** 43,740 bulk-to-partial synthetic cases, delayed end-of-input cases, changing block sizes and exact oracle arrays. **Unresolved:** this is a source invariant, not a mechanically verified state-machine proof.

## 3. Snapshot epochs and exact full successors — source-level sign-off and independent checks

**Locations:** `63–73`, `297–305`, `321–341`, `400–430`, `497–535`, `557–577`, `795–873`.

The identity `phi(i+61440)=phi(i)+65536` holds because 61,440 is exactly 2,048 complete 30-row groups. A source position's low 16 bits therefore determine its value within the current epoch, in `0..=65533`. Each snapshot is taken *before* the first row of that epoch (`823–826`) and stores every character's **next** destination rank, counting both already finished and currently open portions via `Desc::rank`. The scan breaks uniform runs at epoch boundaries, so no tile or run uses source labels from the wrong epoch.

For a rank `r` actually assigned to byte `c`, the epoch is `max{e: A_e[c] <= r}`. Snapshots are nondecreasing, and the first is `C[c]`, so the backward search (`335–340`) terminates even if a byte is absent across multiple epochs; it selects the greatest equal snapshot. `write_entries` (`512–530`) starts from the correct first epoch and advances at every snapshot rank boundary, including repeated values; its final epoch is bounded by the independently determined `h1`.

A source-level correctness distinction: classification uses **full recovered** labels `y0`,`yl` and `span=yl-y0`, not the truncated 16-bit difference. Only once `span<=65533` is established do ordinary fields become `(u_j-u_0) mod 65536`; as `0<=full_difference<=65533`, this equals the oracle's full offset. This handles the exact `3 -> 61440` span 65,533 (ordinary) and `2 -> 61440` span 65,534 (RAW), as well as far-wider spans with deceptively small low bits.

**Independent check:** `epoch_oracle_checks.py` tested 1,110 input configurations (1,422 epoch snapshots), including n=983,010, 15 source-boundary/large-block sizes under multiple input distributions, 65,533/65,534 boundary instances, absent-byte epoch gaps of 1/2/3/6/13 and multi-epoch wide aliasing. Every generated full successor, ordinary modular offset and RAW entry reconstructed from the snapshots matched an independent stable-count scatter. The model visited 708,726 ordinary groups, 863 wide groups and 6,483 crossing groups. This independently checks the mathematical identity and group classification, not the Rust implementation's memory safety.

## 4. RAW group ownership, promotion and canonical ordering — source-level sign-off

**Locations:** `309–319`, `346–367`, `400–495`, `497–535`, `601–641`, `795–820`, and borrowed `ctor.rs:650–702` canonicalizer.

Every internal bucket boundary `B` with `B mod 30 != 0` creates one crossing RAW group (`802–813`); monotonic bucket boundaries make deduplication by the previous group sufficient. Forced RAW only adds as-yet-unmarked groups (`815–820`). `open` receives the provisional index from the already initialized header of a premarked RAW group. One portion covers the intersection of one bucket with one group. Two characters' portions in a crossing group have disjoint rank intervals, hence disjoint field addresses and `RawG` slot ranges. At finishing, only the current portion's `[w0,w0+k)` is restored to `0xFFFF`, not any other character's fields.

A non-preset group becomes wide only once, on closing its one whole-group portion: after full-label span classification it pushes one zeroed `RawG`, writes recovered entries, installs a complete RAW record and sets the bitmap. Since a non-crossing group belongs to exactly one character and is visited in increasing rank order, no second wide promotion is reachable. A forced complete group is written directly via its existing `RawG` rather than promoted. All RAW groups are zero initialized (words 30,31 zero); word 31 is temporarily used and subsequently reset by the unchanged canonicalizer. All provisional indices arise from a fresh `raw.push`, so they are unique and in bounds, satisfying that routine's permutation precondition.

**Result:** no cross-character overwrite, double RAW creation, wrong full-span test or canonical-order defect identified. **Tests:** 1,620 crossing-boundary cases; exact span edges and far-wide cases; forced-RAW configurations; comparisons of every byte including padding; existing v1 canonicalization tests. **Unresolved:** canonicalizer's own debug-only duplicate-index check is still not a substitute for its producer invariant; no reachable duplicate source was found.

## 5. SIMD and generated machine code — source- and assembly-level sign-off

**Locations:** `ctor2.rs:423–445` and the published `ctor2-finish-false.asm` around addresses `61f4e–61fb3`. In the ordinary full-group branch `k==30`, `w0` is proved to point to slot 0 of an ordinary group. `fp.read()` has type `[u16;30]` and can read only the 60 initialized field bytes. The Rust implementation performs each `wrapping_sub(u0)` once. The compiled SSE2 normalizer has disjoint load/store regions: 16 bytes at +0x00,+0x10,+0x20; 8 bytes at +0x30; 4 bytes at +0x38. These exactly partition bytes 0–59. Header at +0x3c (60) is written separately at `61f4e`, and is **never loaded or overwritten by the vector normalization**. Later 8-byte and 4-byte loads are scheduled after earlier stores, but those byte ranges do not overlap; the source's description of a register-held copy is a semantic description, not the exact instruction schedule.

For `k<30` the Rust loop at `442–445` accesses `w0+t` only for `0<=t<k`, with k established from the portion's valid write pointer; there is no full-group `fp.read` on this path. The compiler's partial path is bounded by `k`.

The published `ctor2-build-false.asm` contains the two quoted 12-instruction normal scalar append loops (`34350–3437a` and `343e0–34406`), with a cold `refill` branch. They load no record header and do no per-row RAW/span classification. `ctor2-refill-false.asm` contains a separate cold slow edge. The complete-record emitter is preserved in form (`341d0–34233`). These findings are **static instruction-path observations only**, not a measure of execution cost. The new finalization work and descriptor stores must remain included in a future, separately registered timing comparison.

**Result:** field/header isolation and claimed compiled structure verified for the archived baseline x86-64 listing. **Unresolved:** generated code may differ on another toolchain/architecture; any later harness should verify its own compiled listing.

## 6. Correctness harness and limitations — reviewed, no material blind spot found

**Locations:** `ctor2.rs:881–1029` (two-pattern poison, two builds, oracle and walker); `1070–1165` (corpora and adversarial checks); `1205–1701` (synthetic and malformed); journal §3; archived logs.

The oracle independently builds a full successor array by stable counting scatter, then classifies and encodes the records; v2 instead appends low labels, reconstructs epochs, and finalizes groups. The two share the frozen `phi`, layout and constants—possible common-mode errors in representation definitions, **not** a hidden shared implementation of the new algorithm. `diff()` compares lengths and every byte of `Rec` and `RawG` (the parent has compile-time 64/128-byte size/layout checks). Both `build::<true>` and release-path `build::<false>` are checked, and the untouched walker is compared against production and oracle counted output and all transition counts, then restoration. Reuse is across varying sizes and `permille`, including zero-length builds through **both** variants. Malformed-count checks require rejection with both visible arrays empty. Alternating A5A5/5A5A poison patterns are better than one pattern for catching omissions.

**What the archived logs actually show:** 53,924 synthetic cases exact (99,768 array comparisons); 6,052 corpus blocks under four configurations; 528 adversarial blocks; the 11 input hashes reportedly checked. The Miri log **does not end successfully**: it shows five completed quick classes totaling **473 cases**, then an external interruption during the sixth after about 1 h 35 min. That work covered small blocks, crossing RAW, an epoch boundary and some >61,440-row wide-RAW cases. It **did not complete** the exact 65,533/65,534 tests, forced RAW, malformed counts or reused/poisoned larger cases. Do not call it a completed 524-case Miri success, and do not treat the release tests as a memory-safety sanitizer.

**Residual limitations (not blocking at this gate):**

- Poisoning only writes *existing* spare capacity; if a build grows/replaces `rec`, that old poison does not cover the new allocation. Exact oracle comparison remains independent evidence for such cases.
- Miri's recorded coverage is partial, as above. A focused Miri run on selected omitted cases could add confidence but is **not required solely for completion** by the guardian's instructions. This review did not run Rust/Miri.
- The `Desc::rank` subtraction is not defensive against arbitrary corrupted descriptor pointers; its safety follows from the source's descriptor-state invariant, not dynamic bounds checking in `build::<false>`.
- 64-bit Linux/x86-64 and the pinned rustc/LLVM listing are the accepted target. Cross-architecture optimization and allocator behavior were not examined.
- Source-level correctness does not imply the larger constructor is cheaper: 74.31% bulk coverage and a 12-instruction append loop say nothing about the cost of deferred group finalization or cache eviction.

## Decision and next step

**PASS — no material correctness or memory-safety problem found at `8beffe2`, suitable for the Guardian to consider a separately registered v2 construction-cost experiment.** Retain the frozen source and walker. The next gate, if authorized, should include the entire v2 build cost (descriptor/snapshot initialization, deferred finalization, RAW work and allocation), and independently confirm exact machine-code paths. **This report is not benchmark authorization.**
