# DRAFT: a master-side layout contract between MTF's `counts` and `mtf::STEPS`

**Status: DRAFT design.** Nothing here is built, registered, frozen or timed. The Guardian's
D086 (#145 6760052047; `state/records/D086/decision.md` at 532d6342) allows it as a
**distinct experiment**:
- it is qualified on its own, against its pinned master base, under the standing CI no-harm
  test and the B5 obligations, before anything is built on it;
- only then may a materially changed #324 successor be frozen against that qualified base,
  under D082's one registered sequence.

It is not a remedy for `research/run-front-fallback` (e8a417c6), whose static screen stays
FAILED, and it reclassifies none of that screen's predicates.

## 1. What it is for, and what it is not

**The mechanism it addresses:** one named pair, found by trace in #324's head
(`2026-11-10-run-front-fallback.md` §d; corrected counts in its §"Corrected counts").
- **The store:** `mtf_avx2`'s read-modify-write of `counts[c]`, once a literal.
- **The load:** the next literal's loads of `mtf::STEPS[nn]`.
- **The candidate:** a false store-to-load dependence where the two match in the address
  bits a CPU's memory-disambiguation compares (12 bits on many x86 cores; Agner Fog reports 1
  KiB spacing on Zen 4).

**Why master is exposed too.** On master the two happen not to meet modulo 4 KiB. That rests
on two placements no one controls:
- `counts`' heap offset, which follows allocation history (it moved 0x40 with the input
  path's length);
- `STEPS`' `.rodata` offset, which follows code size.

Any change to either, by code, allocator, input or environment, can make them meet, as
#324's head did.

**What it does not address:**
- the `ll` and symbol streams (they sweep every residue);
- stack spills and the second-level `rows` table on `mtf_avx2`'s frame (224 contiguous bytes:
  every residue mod 64);
- `read_item`'s stack-relative classes;
- any other false dependence.

**A heap or static contract is not a stack remedy.** The `read_item` stack–stack 1 KiB class
that failed the candidate's screen stays an open, unattributed risk under this contract.

The claim is scoped to this pair and this mechanism. That the pair caused #324's elapsed-time
harm remains unproven: the counter rise is attributed, the runtime loss is not.

## 2. The objects, as master has them

- **`mtf::STEPS`** is `static [Step; 32]`, with `Step` `#[repr(C, align(64))]`: 64-byte rows
  `a`, `b`, `m` (16 bytes each), then `_pad` (16).
  - Extent: 2,048 bytes, 64-aligned.
  - `mtf_avx2` loads `a`, `b` and `m` of row `nn` (`_mm_load_si128` at row offsets 0, 16 and
    32), so **the accessed bytes are those at row offset 0–47.** Offsets 48–63 of every row
    are never read.
  - **To verify per kernel and build, from the listing:** `mtf_checkpoints_avx2` (the team
    path's twin) and any other user of `STEPS`.
- **`counts`** is `[u32; 256]`, 1,024 contiguous bytes, with four owners:
  - `Scratch.counts` (`decode_block`; the plain workers and the 1-thread reader);
  - `parallel::Front.counts` (the team and pipelined front: `mtf_checkpoints`);
  - the tests;
  - #324's `runs::Runs.counts` (a different kernel with its own `STEPS`; out of scope here).

  Each is placed by its owner's allocation, on the heap or the stack.
- **Every `counts[c] += …` in `mtf.rs`** (scalar, AVX2 and checkpointed) touches 4 bytes at
  `4c`. Which slots a block touches depends on its bytes, so a certificate must cover all 256.

## 3. Two candidate contracts

### Contract A: the full-extent 4 KiB certificate (D086 §3's formula)

Require `R(STEPS, 2048, 4096) ∩ R(counts, 1024, 4096) = ∅`, where
`R(a, n, m) = {(a + i) mod m : 0 ≤ i < n}`.
- **`STEPS`:** wrap it in a `#[repr(C, align(4096))]` static, so its residues are
  `[0, 2048)`. Rust and LLVM honour static alignment up to the page size on ELF; the linker
  keeps section alignment. To verify per target.
- **`counts`:** a `#[repr(C, align(4096))]` page holding `counts` at byte offset 2048, so its
  residues are `[2048, 3072)`. That is `Box`ed in every owner, since a field's offset inside
  `Scratch` is not a layout guarantee. Rust's global allocator honours the type's alignment.
  The mtf signatures stay `&mut [u32; 256]`.
- **The cost:**
  - 4 KiB a `counts` owner, against 1 KiB inline (+3 KiB padding);
  - one aligned allocation and a first touch a worker;
  - up to 4 KiB of `.rodata` padding before `STEPS`;
  - the `Scratch` and `Front` layouts change, which is a code-layout perturbation in its own
    right;
  - no change to the hot loop.
- **What it guarantees:** no residue overlap modulo 4096 between the objects' full extents,
  on every target where the two alignments hold.
- **What it can't guarantee:** anything modulo 1024. A contiguous 1,024-byte `counts` covers
  every residue modulo 1024, and so does a 2,048-byte `STEPS`. Under the full-extent formula,
  **no placement of a contiguous `counts` is disjoint from `STEPS` modulo 1 KiB.**

### Contract B: accessed bytes, separated modulo 64

This uses the bytes `STEPS` never reads.
- **The layout:** `counts` becomes a 64-aligned strided array, `StridedCounts`
  (`#[repr(C, align(64))]`, 64 lines of 64 bytes). Slot `c` sits at byte offset
  `64·(c >> 2) + 48 + 4·(c & 3)`, so every `counts` byte has `addr mod 64 ∈ [48, 64)`.
- **The disjointness:** every accessed `STEPS` byte has `addr mod 64 ∈ [0, 48)` (the rows are
  64-aligned). So the two accessed-byte residue sets are disjoint modulo 64, and therefore
  modulo every multiple of 64: **1,024 and 4,096 included.**
- **It holds wherever the two objects are,** on heap, stack or `.rodata`, from type alignment
  alone. It needs no page alignment, allocation history or particular linker placement.
- **Its preconditions, each to be proved per build from the listing:**
  - every load from `STEPS` reads row bytes 0–47 only, in every kernel that reads it
    (`mtf_avx2`, `mtf_checkpoints_avx2`);
  - every `counts` access goes through the strided index;
  - `Step`'s `align(64)` and field order (`repr(C)`) are kept.

  A future `Step` that uses its padding breaks the contract. A test can assert `size_of`,
  `align_of` and `offset_of!` for both types.
- **The interface:**
  - The MTF kernels accumulate into a worker-owned `StridedCounts`.
  - At the block's end they compact it into the ordinary `[u32; 256]` (256 loads and stores,
    once a block), so `bwt::build`, the checkpoints and every other reader are unchanged.
  - The checkpointed kernel's checkpoints read `counts` mid-block too, so they need the same
    compaction or a strided read.
- **The cost:**
  - **Footprint:** 4 KiB a worker for `StridedCounts`. The lines `counts` can touch rise from 16
    to 64. A block of B distinct bytes touches between ⌈B/4⌉ and min(B, 64) lines, against at
    most 16 contiguous; for Wikidata's few dozen bytes, up to their number.
    That is up to 3 KiB more L1 working set for a uniform-byte block. Its cost is unknown
    until timed.
  - **Per literal:** the index takes about 3 more ALU operations (a shift, an AND, an add or
    LEA). They are off the MTF list's dependence chain, so the cost is in throughput, not
    latency. Counted by callgrind before timing.
  - **Per block:** the 256-slot compaction and the strided array's zeroing (256 stores).
  - **Allocation:** none per block. One 4 KiB array a worker, inline or boxed in its owner.
- **What it guarantees:** no accessed-byte overlap between `counts` and `STEPS` modulo any
  power of two from 64 to 4,096. That covers the named pair at both the 4 KiB and the 1 KiB
  spacing.
- **What it doesn't guarantee:**
  - anything about `rows`, spills, `ll`, the symbols or the stack;
  - anything about CPUs that disambiguate on fewer than 6 low bits. None documented here do;
    to be stated per target.

### Could another layout give 1 KiB disjointness?

- **Contiguous `counts` (1,024 bytes):** no layout can, against anything of one byte or more.
- **A split or strided `counts`:** it can, but only against `STEPS`' *accessed* bytes, which
  leave 16 of every 64 residues free. That is Contract B.
- **Splitting `STEPS`** instead (pieces kept to one residue band of each 1 KiB) also works.
  But it changes the kernel's row addressing, and it can't free the residues `counts` needs
  without padding `STEPS` to twice its footprint.
- **Neither split covers the stack-side `rows` table,** whose 224 contiguous bytes span every
  residue modulo 64.

## 4. How it would be qualified (its own experiment, before any successor)

1. **The base:** master at an exact SHA, with one construction delta. The proposal is
   Contract B, since it is the only one that also covers 1 KiB. If B's footprint or per-literal
   cost is judged too high before timing, it is Contract A, chosen then. It is never picked by
   which one times better.
2. **Static proof first:**
   - from the production listing of `ab`, `bzcat` and `phases`, the accessed-byte residue
     sets of `STEPS` and `counts` in each kernel and each owner, with the corrected `agg3`
     counts showing the pair at zero;
   - the call sites' and owners' geometry recorded at several prospectively fixed allocation
     histories and input-path lengths (named in the registration before any run).
3. **The ledger,** whole cost charged:
   - instructions per block (callgrind, every corpus);
   - the footprint per worker, allocations and first touch;
   - code-size and placement changes reported, not judged.
4. **The registration:**
   - the standing whole grid, seven corpora at 1, 2 and 4 threads, with #324's rule;
   - the benefit's information objective stated: robustness, not speed. D086: robustness can
     justify a diagnostic, but it can't alone qualify production;
   - production qualification needs the standing no-harm test to pass, plus whatever benefit
     the registration names.
5. **B5:** a safety argument for any `unsafe` the strided index adds (none is needed with
   checked indexing), Miri, mutation testing, and an independent review.
6. **D083 (G3):** MTF's twins, `mtf::mtf` with `mtf_avx2` and `mtf_checkpoints` with
   `mtf_checkpoints_avx2`, plus the scalar twins and every `counts` owner, are each listed
   with their change and its evidence scope. CI's A/B executes only the plain-worker path.
7. **Only after it qualifies** may a #324 successor be frozen against it, under D082, with:
   - the original failure disclosed;
   - its exact delta;
   - the per-block `Runs` allocation and re-fault costs charged;
   - the `read_item` stack-side risk stated as unresolved, unless separately bounded before
     timing;
   - its incremental benefit compared with the contract base, not with historical master.

## 5. Open before registration

- **The listings:** prove `STEPS`' accessed offsets in every kernel and build in scope:
  x86-64 with AVX2, the default and the scalar fallbacks (the scalar kernels don't read
  `STEPS`), and aarch64, where the AVX2 path isn't compiled and the pair doesn't exist.
- **Whether the per-literal index cost is measurable at all** on high-entropy blocks, where
  MTF is about 75% of the decode.
- **Whether `counts`' line footprint (64 lines) matters** against L1 on the runners'
  hardware. That is a timing question for the registered run, not settled here.
