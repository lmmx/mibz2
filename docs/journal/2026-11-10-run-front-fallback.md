# The run-record front with master's fallback path: a static gate

10 November 2026. Static only: no timing, no registration, no bench.
- **What it is:** the Coordinator's construction after #324's no-harm failure (#145, after
  D082): a candidate whose fallback path matches master's, gated statically against master
  27ed84df before any registration.
- **The four gates:**
  - (a) the fallback bodies are normalised-identical to master's, apart from one call site;
  - (b) the six hot kernels keep master's alignment modulo 64;
  - (c) the touched-slot fixed classes are within about 2% of master's;
  - (d) the buffer pair behind #324's rise in `mtf_avx2`'s heap–heap candidates is
    identified, and the candidate is shown either to remove its cause or only to restore
    its count.
- **Also checked:** the benefit path still runs the new code at #324's cost.

**Where things are:**
- **The code:** this branch, `research/run-front-fallback`. Its `src/runs.rs` and
  `src/runs/tests.rs` are #324's, and so is the `src/bwt.rs` walk. `src/lib.rs` is new.
- **The method's tools:** this entry's directory (`agg.c`, `agg2.c`, `run.sh`, `run2.sh`,
  `geom.sh`, `stackprobe324.gdb`).
- **Raw output:** `lackey/` and `callgrind/`.

## The construction

**`Scratch` is master's, byte for byte.** The records' buffers are owned by the call that
uses them (`decode_runs`'s local `runs::Runs`). Nothing is added to `Scratch`, `BzReader` or
the kernels. A process that never takes the records' path allocates and touches nothing for
them.

**`decode_block` is master's body with one call in front of it:**

```rust
if s.inverse.reads_runs() {
    if let Some(crc) = decode_runs(b, k, s, out) {
        return Ok(crc);
    }
}
```

- **`decode_runs`** is `#[inline(never)]` and returns `Option<u32>` in registers.
  - It does the selector itself (`!wants_checkpoints && runs::takes_runs`).
  - It returns `Some(crc)` only on success. On any error it leaves `out` as it was, and the
    column's path decodes the block and returns the same error. The tests check the two
    paths' errors are equal.
- **For `Cooperative<k>`, the teams' kernel,** `reads_runs` is a constant `false`, so the
  call compiles away and the team path's code is master's.
- **D083 (G3), MTF's twins:** the plain-worker path (`decode_block`) changes. The team and
  pipelined paths (`mtf_checkpoints`, `coop`) deliberately don't, because the records'
  path needs no checkpoints and has no cooperative build. CI's A/B executes only the
  plain-worker path, so any registration claims plain-worker benefit only.

**What was tried first and dropped** (each built and checked):
- an `Option<Box<Runs>>` field (changes `Scratch`, and `drain` with it);
- an `#[inline(never)]` `decode_block` wrapper that tail-calls master's body. That body
  stays identical, but master inlines `decode_block` into `ab::drain`, so the wrapper changes
  the 1-thread hot path's structure.

## Builds and method

- **The builds:** `examples/ab` built as CI builds it (rustc 1.99.1, fat LTO,
  `codegen-units = 1`) for:
  - master **27ed84df**, whose `src/` is identical to #324's base 5e7b9954;
  - #324's head **e88c0631**;
  - this candidate.
- **The bench's own 1-thread arm, `ab child mibz2 1 FILE`:** master inlines `decode_block` into
  `ab::drain::<BzReader<File>>`. That inlined copy, not the standalone `decode_block`, is the
  1-thread hot path.
- **The inputs:** one block each of CI's `random` and `records` (their first 890,000 bytes,
  `bzip2 -9`). Both take the column's path in every build.
- **SUPERSEDED (D086, #145 6760052047): the touched-slot classes below were counted by
  `agg.c`.** For each load it recorded only the class of the first matching store in ring
  order, so stack placement could move a count between "hh" and "sh". `agg2.c` filters out
  stack stores before matching, which is a different statistic (stack stores excluded), not
  an alternative count of the same classes. Corrected counts follow in
  [Corrected counts](#corrected-counts-d086-independent-classes). The original figures stay
  as they were recorded.
- **The touched-slot classes** are D082's method from `2026-11-10-run-front-layout-check.md`
  (on `research/run-front-boxed`):
  - lackey traces every data access;
  - each hot function's loads are checked against the last 48 stores for a byte match
    modulo 4 KiB and 1 KiB at a different address;
  - hh (heap–heap) and ss (stack–stack) candidates are fixed for a build; sh (stack–heap) is
    set per process by stack randomisation.
- **`agg2`** attributes `mtf_avx2`'s hh 4 KiB candidates to (store PC, load PC) pairs, with
  the address ranges on each side.
- **`geom.sh`** reads symbol placement, and `counts` from a gdb probe.

## (d) First: the cause of #324's rise in `mtf_avx2`'s heap–heap candidates

**The pair behind the rise** (`lackey/*-pairs.txt`, random):
- **master:** 27,232 candidates in all;
- **#324's head:** 69,195. The three top pairs (14,046 + 13,972 + 13,943) are one pair:
  - **the store side** is `counts`' read-modify-write store (store span 1,020 bytes, heap
    0x403b8a8–0x403bca4);
  - **the load side** is `mtf_avx2`'s loads of the rows of `mtf::STEPS`, the static 2 KiB
    first-level shuffle table in `.rodata` (load spans 960–1,024 bytes at 0x11e880–0x11eca0,
    three load PCs, one a literal).
- **Master has no such pair in its top twelve.**
- **The other pairs** are within a few percent between builds: the `ll` and symbol streams,
  and `counts` against the symbol loads.

**Why:**
- **On master:** `counts` sits at 0x0a0–0x4a0 modulo 4 KiB (0x060 in an earlier
  environment), and `STEPS` at 0x6c0–0xec0. They don't meet.
- **On #324's head:** `counts` moved to 0x8e8–0xce8 (the 2 KiB `Runs` inline in `Scratch`),
  and `STEPS` to 0x840–0x1040 (the added code moved `.rodata`). `counts`' whole window lies
  on `STEPS`' rows.
- **The effect:** every `counts` increment can then falsely precede the next literal's
  `STEPS` row loads. That is the false store-to-load dependence pattern.
- **On `records`,** the head's rise is smaller (37.2 k against 24.2 k).

**This candidate:** `counts` is back at master's heap offset (0x0a0), because `Scratch` and
`BzReader` are unchanged. `STEPS` stays where the head put it (0x840), and the windows no
longer meet.
- **Its candidates:** 27,473 on random (master 27,232), and 24,036 on records (master
  24,210).
- **Its top pairs** are master's, with the same counts.

**So it restores the count by restoring a placement. It doesn't remove the cause.**
- **Two independent placements set the overlap:**
  - `counts`' heap offset, which follows the allocation history (it differed by 0x40 between
    two input paths of different length);
  - `STEPS`' `.rodata` offset, which follows the binary's code size.
- **Nothing guarantees** they stay apart in another build, input or environment, on master
  as much as on any candidate.
- **A cause fix** would be a layout contract, for example `counts` and `STEPS` placed so that
  their 4 KiB windows can't meet. That would change master's own code, and needs its own
  justification and evidence.

## The gate table

> **The (c) rows below are SUPERSEDED (D086):** they come from `agg.c`'s first-match
> classification. See [Corrected counts](#corrected-counts-d086-independent-classes). The
> verdicts on (a), (b) and the benefit and fallback rows don't depend on that counter. This
> screen stays FAILED (D086); nothing below reclassifies it.

| gate | quantity | master 27ed84df | #324 head | candidate | verdict |
|---|---|---|---|---|---|
| (a) | `decode_block::<Segmented>` standalone, normalised | — | — | 189 against 176 lines. With registers, slots and targets masked, **23 differ:** the call, its test and register moves | close: the call site and register renaming |
| (a) | `ab::drain` (1-thread hot path, `decode_block` inlined), frame | `sub $0x5b8` | `sub $0x5c8` | **`sub $0x5b8`** | frame equal |
| (a) | `ab::drain`, body | 2,156 lines | 2,193 | 2,145. **329 differ with registers and slots masked** (1,881 unmasked): spills and moves rescheduled by register allocation around the added call | **fails** |
| (b) | alignment mod 64: `mtf_avx2`, `read_item`, `build`, `room_segments`, `unrle_avx2`, `clmul_update` | 32, 0, 0, 32, 0, 16 | 32, 0, 0, 32, 0, 16 | 48, 48, 32, 0, 16, 32 | **fails, 0 of 6** |
| (c) | `mtf_avx2` hh 4 KiB (random / records) | 26,946 / 24,105 | 61,189 / 34,017 | **27,177 / 23,952** (+0.9% / −0.6%) | **passes** |
| (c) | `mtf_avx2` hh 1 KiB (random, paddings 0 / 336) | 359,699 / 283,791 | 279,113 / 276,548 | 362,476 / 268,391 (+0.8% / −5.4%) | passes at padding 0, −5.4% at 336 |
| (c) | `read_item` ss 1 KiB (random / records) | 17,796 / 17,804 | 77 / 60 | **74,709 / 95,996** (4.2× / 5.4×) | **fails** |
| (c) | `bwt::build`, all fixed classes | — | — | within 0.3% of master | **passes** |
| (c) | `room_segments`, all fixed classes | — | — | within 0.8% of master | **passes** |
| (d) | the cause of the rise | — | `counts` store against `mtf::STEPS` loads, mod 4 KiB | the pair is absent; `counts` is back at master's heap offset; `STEPS` stays moved | **count restored; cause not removed** |
| benefit | `decode_block` Ir on blocks by records (Wikidata 1, 233, 38; text 0; pattern 0) | — | 16,000,507; 15,779,759; 20,015,061; 30,700,964; 12,580,643 | +0.01%, +0.01%, +0.01%, +0.00%, +0.01%, with the same outputs | **passes** |
| fallback | `decode_block` Ir on column blocks (random 0, records 0) | 84,474,095; 95,404,355 | +61 | **+51** (the call, the selector, the return) | as expected |

**Reading the gates:**
- **(a) and (b) can't be met by construction on stable Rust.**
  - Any code added to the path that `drain` inlines perturbs `drain`'s register allocation.
    The frame was kept equal, but the spills moved.
  - Any code added to the crate moves `.text` (it starts at 0x42830 here, against master's 0x41c30: the larger `.rodata` and tables before it). Fat LTO also
    perturbs dozens of unrelated function sizes (`drop_glue`, `fmt`, `raw_vec`), so the hot
    kernels' placement can't be pinned without function alignment, which is unstable, or a
    compiler flag that would change master too.
  - I did not search for a placement that passes. That would be padding-to-pass.
- **(c)'s one failure follows (a).** `read_item`'s 1 KiB stack–stack class is set by the
  frame-relative slots of `read_item` against `drain`'s inlined spills. `drain`'s spills
  moved, so this class moved: 4–5× master's. On this measure #324's head was 0.004× master's,
  and boxing 2.3×. This class has no attributed cost, and no harm is shown or excluded by it.
- **The benefit path is intact.** Per-block ownership of the records' buffers costs about
  1.5 k instructions a block in callgrind.
  - **Not counted by callgrind:** first-touch page faults. glibc may trim and re-fault about
    0.25 MB of buffers a Wikidata block.
  - **That cost needs timing to price,** and none was made.

## What this says for D082

The candidate restores the one fixed class that #324's head moved most (`mtf_avx2`'s
`counts`→`STEPS` candidates) to master's level, and keeps the benefit path's instructions.
It does not make the fallback path's code or placement identical to master's, and it can't:
- the inlined caller's register allocation (gate a);
- the hot kernels' alignment (gate b);
- one stack-relative class (`read_item` ss 1 KiB, gate c).

Whether those residual differences matter can't be settled statically. Its fix for the
identified pair is a restored placement, not a contract, so it is as fragile as master's own
clearance. The decision to register belongs to the Guardian, under D082. Nothing is
registered or benched here.

## Correctness (local)

- `cargo test --release`, every target, passes: 63 library tests, including #324's four
  records' tests, and every integration test.
- `cargo clippy --all-targets --release` and `cargo fmt --check` are clean.
- Miri and the fuzz smoke were not rerun for this `lib.rs`. Before any registration they are
  needed again, with the B5 items D082 lists: mutation testing and an independent
  adversarial review.

## Corrected counts (D086): independent classes

**The defect.** `agg.c` checked each load against the last 48 stores and kept only the
**first** match in ring order, recording that store's class alone. A stack store earlier in
the ring could take a load that also matched a heap store. D086's witness:
- a heap store at 0x3000 and a heap load at 0x4000, with a stack store at 0x1000002000 in the
  ring: the load counted as stack–heap;
- move the stack store to 0x1000002040: the same load counted as heap–heap.

Its "fixed" classes were therefore not independent of stack placement.

**The repair: `agg3.c`** (in `agg3/`, with the run scripts and the raw output).
- **Every class is counted independently.** For each load by a hot function, and for each
  modulus (4 KiB, 1 KiB) and each store region, it records whether any of the last 48 stores
  of that region overlaps the load's bytes modulo the spacing at a different real address.
  A load can count in several classes; none consumes another.
- **Three regions:**
  - **S,** the stack (at or above 0x1000000000);
  - **D,** the binary's static image (0x100000–0x1000000, where valgrind maps the
    executable: `.rodata` such as `mtf::STEPS`);
  - **H,** everything else (heap and mmaps).

  The old "hh" merged D and H.
- **Pairs:** for each (function, load region, store region, modulus), the top (store PC,
  load PC) pairs are kept, each counted once per load.
- **The tests** (`agg3/test_agg3.sh`, all passing, output in `test_agg3.out`):
  - D086's witness in both placements, and with the ring reordered: H←H = 1 every time;
  - a static-image store;
  - a 1 KiB-only match;
  - ring wraparound (the 49th-oldest store is evicted; the 48th is kept);
  - a true dependence and a partial real overlap (both excluded).
- **These are trace candidates, not stall counts.** A store among the last 48 observed need
  not be outstanding when a native load executes. And the stack's placement under valgrind
  (two paddings here) is one environment's, where natively it is randomised per process.

**Recomputed on the same inputs, with all four builds rebuilt as CI builds them:**
- master 27ed84df;
- #324's head e88c0631;
- boxed f72b2a3c;
- the candidate e8a417c6.

Ratios are against master; "–" means no candidates in that class.

**random, pad 0**

| function | mod | load ← store | master 27ed84df | #324 head e88c0631 | boxed f72b2a3c | candidate e8a417c6 |
|---|---|---|--:|--:|--:|--:|
| `mtf_avx2` | 4K | D<-H | 13,829 | 56,154 (4.06×) | 56,188 (4.06×) | 14,071 (1.02×) |
| `mtf_avx2` | 1K | D<-H | 341,247 | 339,795 (1.00×) | 340,067 (1.00×) | 341,414 (1.00×) |
| `mtf_avx2` | 4K | H<-H | 13,352 | 13,486 (1.01×) | 13,375 (1.00×) | 13,352 (1.00×) |
| `mtf_avx2` | 1K | H<-H | 53,025 | 53,149 (1.00×) | 53,361 (1.01×) | 53,025 (1.00×) |
| `mtf_avx2` | 4K | S<-H | 46,146 | 787,938 (17.07×) | 842,759 (18.26×) | 46,146 (1.00×) |
| `mtf_avx2` | 4K | H<-S | 48,608 | 193,967 (3.99×) | 207,880 (4.28×) | 48,608 (1.00×) |
| `mtf_avx2` | 4K | D<-S | 75,530 | 75,125 (0.99×) | 75,125 (0.99×) | 75,125 (0.99×) |
| `read_item` | 1K | S<-S | 17,836 | 95 (0.01×) | 40,972 (2.30×) | 75,067 (4.21×) |
| `read_item` | 4K | S<-S | 19 | 13 (0.68×) | 13 (0.68×) | 19 (1.00×) |
| `read_item` | 4K | H<-H | 14,007 | 14,096 (1.01×) | 14,175 (1.01×) | 14,007 (1.00×) |
| `read_item` | 1K | H<-H | 56,104 | 56,120 (1.00×) | 56,117 (1.00×) | 56,104 (1.00×) |
| `build` | 4K | H<-H | 44,461 | 44,459 (1.00×) | 44,458 (1.00×) | 44,461 (1.00×) |
| `build` | 1K | S<-S | 171,307 | 171,307 (1.00×) | 171,307 (1.00×) | 171,307 (1.00×) |
| `build` | 1K | H<-H | 169,514 | 169,514 (1.00×) | 169,513 (1.00×) | 169,514 (1.00×) |
| `room_segments` | 4K | H<-H | 6,943 | 6,933 (1.00×) | 6,931 (1.00×) | 6,943 (1.00×) |
| `room_segments` | 4K | S<-S | 16,711 | 16,711 (1.00×) | 16,711 (1.00×) | 16,711 (1.00×) |
| `room_segments` | 1K | S<-S | 19,625 | 19,625 (1.00×) | 19,625 (1.00×) | 19,625 (1.00×) |

**random, pad 336**

| function | mod | load ← store | master 27ed84df | #324 head e88c0631 | boxed f72b2a3c | candidate e8a417c6 |
|---|---|---|--:|--:|--:|--:|
| `mtf_avx2` | 4K | D<-H | 13,829 | 56,154 (4.06×) | 56,188 (4.06×) | 14,071 (1.02×) |
| `mtf_avx2` | 1K | D<-H | 341,247 | 339,795 (1.00×) | 340,067 (1.00×) | 341,414 (1.00×) |
| `mtf_avx2` | 4K | H<-H | 13,352 | 13,486 (1.01×) | 13,375 (1.00×) | 13,352 (1.00×) |
| `mtf_avx2` | 1K | H<-H | 53,025 | 53,149 (1.00×) | 53,361 (1.01×) | 53,025 (1.00×) |
| `mtf_avx2` | 4K | S<-H | 46,384 | 977,683 (21.08×) | 976,000 (21.04×) | 46,384 (1.00×) |
| `mtf_avx2` | 4K | H<-S | 48,608 | 243,197 (5.00×) | 242,988 (5.00×) | 48,608 (1.00×) |
| `mtf_avx2` | 4K | D<-S | 75,105 | 75,856 (1.01×) | 76,096 (1.01×) | 76,096 (1.01×) |
| `read_item` | 1K | S<-S | 17,836 | 95 (0.01×) | 40,972 (2.30×) | 75,067 (4.21×) |
| `read_item` | 4K | S<-S | 13 | 19 (1.46×) | 19 (1.46×) | 13 (1.00×) |
| `read_item` | 4K | H<-H | 14,007 | 14,096 (1.01×) | 14,175 (1.01×) | 14,007 (1.00×) |
| `read_item` | 1K | H<-H | 56,103 | 56,120 (1.00×) | 56,117 (1.00×) | 56,103 (1.00×) |
| `build` | 4K | H<-H | 44,461 | 44,459 (1.00×) | 44,458 (1.00×) | 44,461 (1.00×) |
| `build` | 1K | S<-S | 171,307 | 171,307 (1.00×) | 171,307 (1.00×) | 171,307 (1.00×) |
| `build` | 1K | H<-H | 169,514 | 169,514 (1.00×) | 169,513 (1.00×) | 169,514 (1.00×) |
| `room_segments` | 4K | H<-H | 6,943 | 6,933 (1.00×) | 6,931 (1.00×) | 6,943 (1.00×) |
| `room_segments` | 4K | S<-S | 16,711 | 16,711 (1.00×) | 16,711 (1.00×) | 16,711 (1.00×) |
| `room_segments` | 1K | S<-S | 19,625 | 19,625 (1.00×) | 19,625 (1.00×) | 19,625 (1.00×) |

**records, pad 0**

| function | mod | load ← store | master 27ed84df | #324 head e88c0631 | boxed f72b2a3c | candidate e8a417c6 |
|---|---|---|--:|--:|--:|--:|
| `mtf_avx2` | 4K | D<-H | 14,255 | 26,933 (1.89×) | 26,933 (1.89×) | 14,082 (0.99×) |
| `mtf_avx2` | 1K | D<-H | 334,676 | 332,311 (0.99×) | 332,465 (0.99×) | 335,075 (1.00×) |
| `mtf_avx2` | 4K | H<-H | 9,929 | 10,042 (1.01×) | 10,044 (1.01×) | 9,929 (1.00×) |
| `mtf_avx2` | 1K | H<-H | 40,010 | 39,847 (1.00×) | 39,924 (1.00×) | 40,010 (1.00×) |
| `mtf_avx2` | 4K | S<-H | 54,358 | 868,196 (15.97×) | 934,724 (17.20×) | 54,358 (1.00×) |
| `mtf_avx2` | 4K | H<-S | 48,608 | 194,399 (4.00×) | 208,291 (4.29×) | 48,608 (1.00×) |
| `mtf_avx2` | 4K | D<-S | 30,408 | 29,323 (0.96×) | 29,323 (0.96×) | 29,323 (0.96×) |
| `read_item` | 1K | S<-S | 17,835 | 74 (0.00×) | 62,863 (3.52×) | 96,372 (5.40×) |
| `read_item` | 4K | S<-S | 19 | 13 (0.68×) | 13 (0.68×) | 19 (1.00×) |
| `read_item` | 4K | H<-H | 14,118 | 14,256 (1.01×) | 14,284 (1.01×) | 14,118 (1.00×) |
| `read_item` | 1K | H<-H | 56,502 | 56,520 (1.00×) | 56,422 (1.00×) | 56,502 (1.00×) |
| `build` | 4K | H<-H | 43,799 | 43,798 (1.00×) | 43,797 (1.00×) | 43,799 (1.00×) |
| `build` | 1K | S<-S | 83,857 | 83,857 (1.00×) | 83,857 (1.00×) | 83,857 (1.00×) |
| `build` | 1K | H<-H | 170,232 | 170,232 (1.00×) | 170,231 (1.00×) | 170,232 (1.00×) |
| `room_segments` | 4K | H<-H | 10,839 | 10,742 (0.99×) | 10,740 (0.99×) | 10,839 (1.00×) |
| `room_segments` | 4K | S<-S | 16,204 | 16,204 (1.00×) | 16,204 (1.00×) | 16,204 (1.00×) |
| `room_segments` | 1K | S<-S | 19,091 | 19,091 (1.00×) | 19,091 (1.00×) | 19,091 (1.00×) |

**What the corrected counts show:**
- **The named pair is `mtf_avx2`'s D←H at 4 KiB:** `STEPS` loads (D) after `counts`'
  read-modify-write stores (H).
  - **The counts:** master 13.8 k, head 56.2 k (4.06×), boxed 56.2 k (4.06×), candidate
    14.1 k (1.02×) on random. On records: 14.3 k, 26.9 k (1.89×), 26.9 k, 14.1 k (0.99×).
  - **The pairs** (store PC → load PC): in the head, `counts`' store (…950) against
    `STEPS`' three row loads (…78c, …792, …798), about 15 k each. Master's and the
    candidate's top pairs are the `ll` stream's store against two constant-pool loads.
  - **At 1 KiB** the class is the same in every build (about 340 k), as D086 §3 predicts:
    `counts` spans every residue modulo 1 KiB.
- **New with the repair: `mtf_avx2`'s stack–heap classes.**
  - **S←H at 4 KiB:** head 16–21× master's at both paddings and on both corpora (788 k–978 k
    against 46 k), boxed the same.
  - **H←S:** 4–5× master's.
  - **The pairs:** the same `counts` read-modify-write against `mtf_avx2`'s frame (its loads
    at …800–…81e, and its load half against frame stores).
  - **The candidate equals master exactly** in these classes.
  - **But the stack side of these classes is a placement,** valgrind's here and random
    natively, so they are not build constants. The two paddings agree for these builds; that
    doesn't make the classes fixed.
- **`read_item`'s S←S at 1 KiB** stays as the original screen had it:
  - master 17.8 k;
  - head 0.1 k;
  - boxed 41.0 k (random) and 62.9 k (records);
  - candidate 75.1 k and 96.4 k (4.2× and 5.4×).

  **The (c) failure stands.**
- **`bwt::build`, `room_segments`, and `mtf_avx2`'s and `read_item`'s H←H:** within 1% in
  every build. The candidate equals master in these classes.
- **Boxing restored nothing D082 asked for.** Its D←H equals the head's.

**What changes against the original screen:**
- (c)'s heap–heap row for `mtf_avx2` is replaced by D←H and H←H. The candidate is at master's
  level on both, and the head is 4.1× and 1.9× master's on D←H.
- `read_item` fails as before.
- Nothing here reclassifies the screen, which stays FAILED (D086).
- The pair's link to #324's elapsed-time harm is unproven: the counter rise is attributed,
  the runtime loss is not.
