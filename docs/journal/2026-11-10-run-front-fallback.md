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
