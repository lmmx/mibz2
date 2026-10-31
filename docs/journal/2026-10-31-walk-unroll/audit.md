# Room walk, constant-index 32-lane round: correctness and structural audit (before any timing)

**Status:** the arm is implemented and its correctness established. The structural audit
fails two registered gates **as written** (§4: S3 and S5), so under the protocol's stop
rule nothing is timed. The registered outcome is **STRUCTURAL STOP**, pending the
maintainer's decision on what the deviation below means. No timing has been run or
read.

## Build (measured)

| Item | Value |
|---|---|
| Branch, commit | `research/walk-unroll`, this commit |
| Toolchain | rustc 1.97.0 (stable), release profile (fat LTO, one codegen unit), no `RUSTFLAGS` |
| Binary | `cargo build --locked --release --example walk_arm`, sha256 `987c450b…1718` (preparation VM) |
| Arm symbol | `mibz2::bwt::room_segments_unrolled::<32, 1>`, 5,886 instructions; disassembly in `room_segments_unrolled_32_1.asm` |
| Shipped symbol, same binary | `room_segments::<32, 1>` |

## Correctness (measured; all pass)

- **`bwt::tests::unrolled_room_walk_equals_plain`:** the arm equals the plain walk and
  the safe segment oracle, with no flags left set, on:
  - 120 crafted permutations at L = 32 (512 segments) and L = 8, plus L = 5 and L = 64 on
    every fourth case;
  - every length edge (n from 1 to MIN_SEGMENTED + 1, 3 primary indices, 1 to 256
    segments, L = 32, 8 and 5, including ns < L);
  - the degenerate columns.

  The arm alone reached every handoff transition: a buffer grown, a pass ending full, a
  mid-pass take, the full phase ending mid-pass, and a lane entering the tail full. A
  mid-pass take hit **every lane index 0–31** of a round.
- **`bwt::tests::miri_room_walk_unrolled_handoffs`:** the Miri handoff blocks at L = 32
  and 8, with every transition reached by the arm.
  - `cargo +nightly miri test --lib -- miri_` passes 11 of 11 (Miri 2026-10-26, 205 s).
- The library's 34 tests pass, and clippy is clean.
- The harness checks every arm against the plain walk on every Wikidata block before
  timing. Not run: no timing.

## Structural audit (measured; `bench/walk_asm_audit.py` on the binary above)

**The audit change:**
- The audit now traces the hot path: it falls through the `START` branches and follows
  jumps to the back-edge, keeping paths with one prefetch and one `[r+r*1]` byte store per
  T load.
- The address-span reading of commit `1121038` misread the arm, because LLVM lays its round
  out in two pieces around the cold blocks.
- On the shipped walk it gives the same figures as before.

**Per lane-step:**

| Per lane-step | Shipped (inner step only) | Shipped (with its outer `off` loop, per round / 32) | Arm (whole round / 32) |
|---|--:|--:|--:|
| Instructions | 16 | ≈ 16.4 | **10.44** (334 a round) |
| Macro-ops | 14 | ≈ 14.4 | **9.53** |
| Loads | 3 | 3.0 | 3.06 |
| Stores | 2 | ≈ 2.03 (1 a round) | 2.16 (5 a round) |
| Prefetches | 1 | 1 | 1.0 |
| Conditional branches | 2 | ≈ 2.03 | 1.03 |
| Calls | 0 | 0 | 0 |

**The ordinary step is exactly the analysis's expected signature** (for example
`0x16f7f–0x16fa9`):

```
mov r14d,[rsp+ROW_k]; mov eax,[r15+r14*4]; test eax,eax; js cold_k;
mov ecx,eax; shr ecx,8; mov [rsp+ROW_k],ecx; prefetcht0 [r15+rcx*4];
mov rcx,[rsp+BASE_k]; mov [rcx+r8],al
```

That is 10 instructions and 9 macro-ops, with constant displacements. There is no lane
counter, lane pointer, bound or extra test.

**What else is in a round** (14 instructions):
- the round control: `lea` / store of `off + 1`, `cmp` against `room` on the stack, `jne`;
- two `jmp`s joining the round's two pieces;
- **register spills:** LLVM keeps three steps' `base` pointers in registers (`r11`, `r9`,
  `rcx`), and stores and reloads them around the cold blocks. That is 5 stores and about
  4 loads a round, so +0.16 stores and +0.06 loads per lane-step.
- The shipped walk's outer loop, by comparison, has 13 instructions and 1 stack store a
  round.

## Gates (§4, as registered)

| Gate | Registered | Measured | As written |
|---|---|---|---|
| S1 unrolled | 32 T loads | 32 | pass |
| S2 instructions per step | ≤ 11.0 | 10.44 | pass |
| S3 loads / stores / prefetches | 2.9–3.1 / 1.9–2.1 / 1.0 | 3.06 / **2.16** / 1.0 | **fail** (stores) |
| S4 conditional branches | ≤ 1.1 | 1.03 | pass |
| S5 no calls, stack accesses ≤ shipped's | 0; ≤ 1.0 / 0.0 | 0; **2.06 / 1.16** | **fail** |

**Reading (deduced):**
- **S3's failure is real and small:** the per-round spills above, 4 more stores a round
  than the shipped walk's round.
- **S5's bound was mis-specified at registration.** The audit counts only `rsp`/`rbp`
  operands. The shipped walk reaches its lane rows through `r11` (a pointer into the
  same stack array), so its 1.0/0.0 understates its true 2.0 loads and 1.0 store a step.
  - Counted alike (the arm's rows are constant `rsp` offsets), the arm is 2.06/1.16
    against the shipped 2.0/1.03, which is the same per-round spill difference.
- The bookkeeping (192 instructions a round) is gone. About 9 spill operations a round
  stand in its place.

Whether this counts as the intended structure, and so whether timing proceeds, is the
maintainer's decision, made before any timing exists. No alternative implementation has
been tried.
