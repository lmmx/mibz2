# The run-record front's harm: `counts` against `mtf_avx2`'s frame, a static check

10 November 2026. This is an untimed check, with no CI bench and no registration. It asks
whether #324's harm on `random` and `records` at 1 thread could be #277's layout effect,
`counts` falling under `mtf_avx2`'s stack frame modulo 4 KiB
(`2026-11-09-run-fed-fallback-codegen.md`), and whether boxing `runs::Runs` restores master's
alignment class. A Guardian ruling on a successor's admissibility under C2 is pending
(#161, 6758376066). Nothing here is a successor's registration.

## Method (#277's, adapted to the bench's own harness)

**The builds:** `examples/ab` built as CI builds it (`cargo build --release --example ab`,
rustc 1.99.1, fat LTO, `codegen-units = 1`) at three heads:
- master **5e7b9954**, #324's merge base. The only change to master since is a new example.
- #324's head **e88c0631**.
- **boxed:** e88c0631 with `Scratch::runs` as `Box<runs::Runs>`, so `Scratch` grows by one
  pointer. That is this branch's `src/lib.rs` change.

**The prologue:** `mtf_avx2`'s is the same in all three (`prologue-*.asm`): five pushes,
`and $-64,%rsp`, `sub $0x1c0,%rsp`, then `mov %rcx,%rbx`, so `rbx` holds `counts`. The frame is
0x1c0 bytes from the realigned `rsp`, and the second-level list `rows` is at rsp+0x40..0x120.

**The probe:** `stackprobe324.gdb` is #277's `stackprobe.gdb`, with its breakpoint moved to the
instruction after `mov %rcx,%rbx` (offset passed in `PROBE_OFF`).
- It runs `ab child mibz2 1 random.bz2`, the bench's 1-thread arm, through
  `BzReader::new` → `decode_block::<Segmented>` → `mtf::mtf` → `mtf_avx2`.
- It records `counts` (`rbx`), the frame base (`rsp`), and how many of the frame's 0x1c0
  bytes fall in `counts`' 1 KiB window modulo 4 KiB.
- It ran once with ASLR off and six times with ASLR on per build. All 9 blocks of a process
  share one call site.

## Result (`probes.txt`)

| build | `counts` mod 4 KiB (every run) | frame − `counts` mod 4 KiB, ASLR off | frame bytes in the window, ASLR off | ASLR on: frame − `counts` mod 4 KiB in 6 runs |
|---|--:|--:|--:|---|
| master 5e7b9954 | 0x060 | 0xc20 | 0 | 0x7a0, 0x3a0 (96 bytes overlap), 0xd20, 0x460, 0x260 (416 overlap), 0x7a0 |
| #324 head e88c0631 | 0x8a8 | 0x3d8 | 40 | 0x958, 0xb98, 0x458, 0x458, 0xd98, 0xe58 (24 overlap) |
| boxed | 0x8b8 | 0x3c8 | 56 | 0x248 (440 overlap), 0x9c8, 0xf48 (264 overlap), 0x608, 0x488, 0x608 |

1. **In the bench's harness, `counts` is on the heap.** It lives in the `BzReader` the harness
   reads from: addresses 0x55…, against the stack's 0x7ff…. In #277's harness it was on
   `main`'s stack, at a fixed distance from the frame. Here its distance from `mtf_avx2`'s
   frame modulo 4 KiB is set afresh in every process by the stack's randomisation. It took
   six different values in six runs of master.
2. **Each build's only fixed quantity is `counts` mod 4 KiB:** master 0x060, head 0x8a8,
   boxed 0x8b8. **Boxing doesn't restore master's offset.** It moves `counts` 16 bytes from
   the head's, not back to master's. The difference is the heap position of the reader's
   allocation, which the `Box<Runs>` allocation and the struct's size both move.
3. **That fixed offset doesn't change the exposure.** The frame base is 64-aligned at a
   random place in its page.
   - **In every build:** 23 of the 64 positions put frame bytes in `counts`' window (19 of
     them `rows`'), a mean of 112 bytes.
   - **So the overlap #277 found** happens in about 36% of processes, equally for base and
     head. The bench's 10 rounds are fresh processes for each arm.
   - **With ASLR off under gdb**, head and boxed overlap and master doesn't. But that layout
     is gdb's environment, not CI's.
4. **So the #277 mechanism, as a fixed per-build placement, can't explain a harm that
   reproduces across runs.** It reproduced in runs 2 and 3: `random` +3.4% and +5.3%,
   `records` +1.5% and +2.1%.
   - **What remains, unexamined:**
     - the code's layout (the column path's functions moved: `mtf_avx2` at 0xe6ca0 against
       0xea520);
     - `counts`' fixed heap offset against other heap buffers;
     - the reader's larger allocation and its first touch.
   - **Boxing addresses none of these directly.**

No timing was made, and no successor is registered.
