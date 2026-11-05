# MTF's runs read 64 symbols at a time: no branch on a symbol's kind

Research on `research/mtf-run-batch`, not registered for the CI bench. Every timing here is
local, on the shared 4-core VM (Intel Xeon, Cascade Lake, KVM), and indicative only: the CI
bench (EPYC) decides. The move itself is untouched: this is the rest of MTF's serial chain.

## What on the chain was not the move

Master's `mtf_avx2` takes one symbol an iteration. On Wikidata's block 1 (the census on
`research/mtf-census`) there are 92,055 symbols: 43,591 RUNA/RUNB (47%) and 48,464 byte
symbols, the runs 17,384 of them. Each iteration branches on the symbol's kind (`s <= 1`), and
a byte symbol then branches on a pending run (`run != 0`), checks the run against the limit,
flushes it, checks the end-of-block symbol and the limit again, then moves.

The kind branch has no pattern to learn: the census's best history predictor still misses
25-30% of symbols, and Callgrind's simulated predictor misses 41,578 conditional branches in
the kernel on block 1 (0.45 a symbol). Each miss refetches from the branch while the list's
chain (below) waits on nothing else. llvm-mca puts master's iteration bodies at 2.0 cycles
(a run symbol), 4.5-5.6 (a byte symbol) and 7.1-9.1 (a byte symbol flushing a run) on Zen 4
and Skylake-X: about 7-9 cycles a byte symbol with its run, against 31-34 TSC cycles measured
(the kernel harness's quietest rounds, at 1.9 symbols a byte symbol). The time is in the
misses and the memory chain, not in the instructions.

## The construction

`mtf_avx2` now counts a block's run symbols (16 at a time, about 1% of the kernel's instructions).
A block with at least one in 8 takes the window kernel. A block with fewer takes master's
kernel, kept verbatim as `mtf_avx2_symbols`: random data and records have about 0.4% run
symbols, and there the kind branch is predicted and a byte store a byte symbol is cheaper
than a segment (below). Both kernels are exact on every input; the choice is only of speed.

The window kernel runs master's symbol loop until the block's first byte symbol, then a
window loop:

- **A window** is 64 symbols from a run's start: four 32-byte loads, compared as 16-bit lanes
  and packed to two bit masks, the byte symbols (s >= 2) and the symbols other than RUNA
  (s != 0). The window is left to the symbol loop if it holds the end-of-block symbol or 22
  run symbols in a row (five shift-and steps on the run mask).
- **A step** is a byte symbol with the run before it: `j = tzcnt(bytes)`, `bytes = blsr(bytes)`.
  A run of m symbols is the bijective base-2 number sum (s_k + 1) 2^k = 2^m - 1 + (its RUNB
  bits), so with the byte symbol's own bit (set in the not-RUNA mask) the run plus one is
  `bzhi(notruna >> prev, j + 1 - prev)`: one shift and a bzhi, no branch on any symbol's kind.
  The next window starts after the last byte symbol (its trailing run, at most 21 symbols,
  is read again).
- **A segment** is what a step writes: the byte the step before moved to the front, then its
  run. It is `seg` bytes of one value, written as two 32-byte stores (32 at a time above 64),
  counted with one add. The step then moves its own byte to the front and leaves it to the
  next step's segment, or to the loop's exit, which writes it.
- **The bounds:** one check a step, that the segment ends at most at `max - 64`. Then the
  stores past it stay inside `ll`'s capacity, and both of `mtf_scalar`'s checks on the run
  and the byte (`n + run <= max`, `n < max` before the move) pass.
- **No call:** an index from 32 moves as the blend of the 32 bytes holding it, then each 32
  below it a byte on, from the top down so every shifted load reads the list before the move
  (LLVM unrolls it by four); a long segment is a loop of 32-byte stores. With `memmove` and
  `memset` calls in the loop (the first version, 7ad5e12d), LLVM spilled the byte mask, the
  output pointer and the counts' pointer around every step.

A step that fails its check, a window that holds the end-of-block symbol or 22 run symbols,
and the block's last 63 symbols are left to the symbol loop. It is master's loop, and it
resumes at that run's start in exactly its own state there. It finishes the block, so every
error is found where `mtf_scalar` finds it, with the same kind: `RunTooLong` at the 22nd run
symbol, `BlockTooLong` for a run past the limit or a byte at it. The window loop's exits are
all near the block's end or at an error, so the symbol loop takes about 64 symbols a block.

The AVX2 kernels now need BMI2 (`bzhi`, `shrx`), which `Kernels::detect` checks with AVX2 and
BMI1. `mtf_scalar` and `mtf_checkpoints` are unchanged.

## The chain, before and after

Per byte symbol with the run before it, in the window kernel (the listings are
`mtf-master.asm` and `mtf-branch.asm`; the step below is the common path, an index below 32
and a segment of at most 64):

| | master | window kernel |
|---|---|---|
| instructions | 12 a run symbol, 26 a byte symbol, 39 one that flushes a run: about 46 a byte symbol and its run | 38, plus about 75 a window of about 30 steps |
| branches with data-dependent outcomes | the kind, every symbol; the pending run, every byte symbol | the window's exit, once a window |
| loop-carried, besides the list | run and weight (1 cycle each), n (1); the kind branch's refetches | `bytes` (blsr, 1 cycle), n (an add, 1); `prev` comes from tzcnt and is not carried |
| the list | the move: a 32-byte load and a shifted load over the last blend store and the front's byte store, blend, store | the same |
| llvm-mca, Skylake-X / Zen 4 (no memory dependences) | about 8.7 / 7.3 cycles | 7.9 / 6.1 cycles |
| measured (TSC cycles, quietest rounds) | 31-34 | 24-26 |

```
blsr   %rdx,%rdx                  ; bytes: the next byte symbol's bit gone
mov    %cl,0xa0(%rsp)             ; (the move before: the front byte stored)
tzcnt  %rdx,%r11                  ; j
shrx   %r15,%r8,%rdi              ; notruna >> prev
lea    0x1(%r11),%r9d             ; prev' = j + 1
sub    %r15d,%r14d                ; j + 1 - prev
bzhi   %r14,%rdi,%r14             ; seg: the run's RUNB bits and the byte's 1
lea    (%r14,%r10,1),%rdi         ; n + seg
cmp    %r12,%rdi ; ja             ; past max - 64: left to the symbol loop
vmovd/vpbroadcastb; 2 x vmovdqu   ; the segment
cmp    $0x41,%r14 ; jb            ; above 64: the 32-byte fill loop
add    %r14d,(%r10,%rcx,4)        ; counts[v] += seg
movzwl (%rsi,%r11,2),%r11d        ; the byte symbol
...    vmovdqa / vpcmpgtb / vpblendvb 0x9f(%rsp) / vmovdqa  ; the move (unchanged)
movzbl 0x9f(%rsp,%r11,1),%ecx     ; v, the new front
test   %r10,%r10 ; je             ; an index below 32: done
```

What is left on the chain is the move's. Its 32-byte load and its shifted load each span the
step before's 32-byte blend store and its 1-byte store of the front, so neither can be
forwarded: each waits for both stores to commit. That is now the loop's floor, about 24-26 TSC
cycles a step against llvm-mca's 6-8. It is the move's design, for the register-front
constructions.

## Measured: instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`, the driver of
`so2cg`, rustc 1.97.0, fat LTO). The input is `wikidata-independent.bz2` (sha256
`7f5c9a0044e5a305…`) and `runs.bz2`'s block 0; Callgrind ran with `--branch-sim=yes`. The base
is master bab0d603. Profiles are in `2026-11-05-mtf-run-batch/cg/` (`base-*`; `v1-*`, the
first version; `branch-*`, this branch, whose blocks all take the window kernel), each block's
output the same (its length and CRC in the `.stdout` files).

| block | master | v1 (calls) | v2 (call-free, every block) | this branch | MTF incl., master | this branch | MTF branch misses (sim.), master / branch |
|---|--:|--:|--:|--:|--:|--:|--:|
| 1 | 19,273,484 | 18,685,716 (−3.05%) | 18,360,985 (−4.73%) | 18,388,411 (−4.59%) | 2,976,792 | 2,091,719 (−29.7%) | 41,578 / 5,640 |
| 41 | 18,647,570 | 18,141,751 (−2.71%) | 17,811,188 (−4.49%) | 17,835,598 (−4.35%) | 2,687,841 | 1,875,869 (−30.2%) | 33,684 / 5,009 |
| 233 | 19,071,853 | 18,611,525 (−2.41%) | 18,262,794 (−4.24%) | 18,288,983 (−4.10%) | 3,041,373 | 2,258,503 (−25.7%) | 35,989 / 4,840 |
| 400 | 18,651,084 | 18,202,330 (−2.41%) | 17,861,508 (−4.23%) | 17,885,635 (−4.10%) | 2,741,865 | 1,976,416 (−27.9%) | 32,869 / 5,069 |
| runs 0 | 17,394,491 | 17,131,244 (−1.51%) | 16,956,578 (−2.52%) | 16,975,184 (−2.41%) | 2,772,089 | 2,352,782 (−15.1%) | 44,093 / 37,104 |

The branch's count of run symbols costs about 25,000 Ir a block (v2 to this branch).

Every change in `decode_block` is the MTF's. Callgrind's predictor is a simple one, but its
count tells the same story as the census: master's kernel misses about 0.45 conditional
branches a symbol, the window loop about 5,000 a block, some three a window of about 60
symbols: 6-7 times fewer.

## Measured: time (local, indicative)

**The MTF stage** of `examples/phases.rs` (`bench/stages.py run`, 16 rounds, 1 pass, pinned to
CPU 1, the arms' order alternating) on `corpora/wikidata.bz2` (sha256 `82f65ace…`, as
`bench/wikidata.json`). Every round's values are in the `stages-*.jsonl` files; `pair.py`
makes the intervals (95%, t, on the mean log ratio). "Over the unchanged stages" divides each
round's MTF ratio by that round's ratio of the stages neither arm changed (scan, build, walk,
RLE1, CRC), which takes out the machine's drift between the two arms' runs.

| arm | MTF ns/B, master | arm | MTF change | unchanged stages | MTF over them | phases total |
|---|---|---|--:|--:|--:|--:|
| v1 (calls) | 0.86-1.02 | 0.64-0.84 | −24.8% (−28.5, −21.0) | −1.0% (−8.8, +7.4) | −24.1% (−26.8, −21.2) | −4.4% (−11.5, +3.4) |
| v2 (call-free, every block) | 0.90-1.25 | 0.63-0.72 | −37.5% (−41.5, −33.3) | −6.3% (−14.7, +3.0) | −33.4% (−36.2, −30.4) | −10.6% (−18.3, −2.1) |
| this branch | 0.89-1.08 | 0.62-0.75 | −31.3% (−33.1, −29.5) | −1.0% (−4.7, +2.9) | −30.6% (−32.0, −29.3) | −5.2% (−8.7, −1.7) |

Every one of the 48 rounds had the arm's MTF faster. This branch's 16 rounds, MTF ns/B master
→ branch: 0.963→0.621, 0.968→0.667, 0.898→0.635, 0.904→0.683, 1.002→0.657, 0.944→0.633,
0.958→0.650, 0.914→0.655, 0.911→0.641, 0.934→0.649, 0.899→0.643, 0.984→0.616, 0.965→0.640,
0.892→0.642, 1.077→0.755, 0.990→0.654 (ratios 0.63-0.75). In v2's run the master arm was
slower in the stages neither arm changed, which the "over them" column takes out; this
branch's run had them equal.

**The kernel alone, every corpus** (`mtfab.rs`): master's `mtf_avx2` (copied verbatim), the
window kernel alone, and this branch, on every block of each file, rdtsc around each call, the
order rotating by block and round, every output checked equal, 11 rounds
(`mtfab-final.txt`):

| corpus | the window kernel alone | this branch |
|---|--:|--:|
| wikidata | −35.6% (−37.2, −34.0) | −33.8% (−35.2, −32.4) |
| text | −49.9% (−50.5, −49.3) | −47.6% (−48.2, −46.9) |
| runs | −14.4% (−18.6, −10.1) | −6.6% (−11.4, −1.6) |
| pattern | −5.4% (−9.5, −1.1) | −6.1% (−11.5, −0.5) |
| random | +17.1% (+14.3, +19.8) | −7.5% (−9.4, −5.6) |
| records | +1.2% (−0.9, +3.3) | +0.1% (−1.7, +1.9) |
| onebyte (41 symbols) | +47.0% (+39.1, +55.4) | +2.1% (−2.7, +7.2) |

The harness's magnitudes move between runs (the window kernel was −23% to −36% on Wikidata
in four runs, +17% to +42% on random): the machine is shared. Its direction did not move.

**Whole-file 1-thread wall** (`c30cg time`, `BzReader`, 15 pairs, alternating,
`wall-1thread.txt`): master 1.45-2.37 s, this branch 1.47-2.16 s, −6.5% (−13.5, +1.1), 9
pairs faster; v2 was −2.7% (−7.5, +2.4). Neither is beyond the VM's noise. The stage figure
predicts about −4% here (MTF about 14% of the decode, at −30%), and the phases total was
−5.2% (−8.7, −1.7). On CI, MTF + RLE2 is 23% of the 1-thread stage table, so the same cut
would be about −6% to −7% of a 1-thread decode, if EPYC's store-forwarding and branch costs
scale as this machine's do.

## Negative results

- **The window kernel on every block** (no dispatch) was slower than master where runs are
  rare: random +17% to +42%, records +1% to +58% over five harness runs. A step writes two
  32-byte stores where master writes one byte, and with nearly every symbol a byte symbol,
  and most of their indices from 32 (a copy), that costs more than the predicted branches it
  saves.
- **Variants of the window kernel** (`mtfab-variants.txt`, `mtfab-hybrids.txt`, sources
  `mtfab-variant.rs`, `mtfab-hybrid.rs`), none of which fixed random without losing Wikidata:
  - byte steps for a window with no run symbol: random +34%;
  - `memmove` for the copy: records to +5%, but Wikidata from −23% to −12% (the call's
    spills);
  - one 32-byte store for a segment of at most 32: random +20%;
  - an eight-chunk branch-free copy: random +52%;
  - a per-window choice (at most 8 or 16 run symbols: a byte store when the segment is one
    byte, `memmove`): random +21% to +27%, Wikidata −16% to −21%;
  - handing run-poor windows back to the symbol loop for 1024 symbols (with `memmove` in
    the symbol loop): Wikidata only −9%, random still +24%. Even master's symbol loop,
    compiled inside the window kernel, ran slower than master's kernel on random data; only
    a separate function (`mtf_avx2_symbols`, `#[inline(never)]`) kept master's speed.
- **v1's calls**, as above: −15% to −30% on Wikidata against the call-free kernel's −23% to
  −36%.

## Correctness

- `tests/mtf_kernels.rs`: the AVX2 kernel and the checkpointed one against `mtf_scalar` on
  symbol streams made directly: random mixes of every density (bytes in use 1-256), runs of
  20-23, 64 and 65 symbols after every offset 0-139 (21 accepted, 22 and more refused), runs
  across the limit (ending at it, one past it, a byte after it), the end-of-block symbol at
  every position 0-259 and missing, indices at the top of a full list and past the bytes in
  use, every limit from 200 below a block's length to 70 above it, limits 0, 1 and 64, and
  stretches with and without runs. Each stream is run at a generous limit and at limits
  around its length. Streams with fewer than one run symbol in 8 test master's kernel.
- Mutations of the window loop all fail it (a missing byte bit in `seg`, 23 run symbols
  allowed, no end-of-block check, the pending byte not counted, resuming a symbol late, the
  limit at `max`, the lowest chunk of a copy not shifted, the top chunk's mask one short, the
  fill from 96, a fill threshold of 65), except one that is equivalent: the margin `max - 63`
  is also safe (`max - 64` is one byte more cautious).
- `mtfab` checked master's kernel and every variant equal (result, last column, counts) on
  every block of `wikidata.bz2` (431 blocks, 24,979,299 symbols, 209,706,557 bytes), random,
  records, text, runs, pattern and onebyte. `tests/mtf_checkpoints.rs` compares the
  checkpointed kernel (the old construction) with `mtf` on every kernel. `tests/decode.rs`'s
  `oversized_runs` and `block_size_levels` check `RunTooLong` and `BlockTooLong` through the
  whole decoder on both kernels.

## Not done, and risks

- `mtf_checkpoints_avx2` (the cooperative path, 4 threads and more) keeps the old loop. Its
  inner loop could take the window loop with `lim` set from its per-checkpoint limit.
- The dispatch's threshold (one run symbol in 8) is set from two clusters: about 0.4% (random,
  records) and 25-50% (Wikidata, text). A block between them takes either kernel; both are
  exact.
- A 32-byte store loop replaces `memset` for long segments. `runs` was faster
  (−7% to −14%), `onebyte`'s single 392,160-byte run within noise.
- The AVX2 path now also requires BMI2. Every CPU with AVX2 in practice has it, but a VM that
  masks BMI2 alone would fall back to the scalar MTF and RLE1.
- The gain is in mispredictions and the per-step overhead. EPYC's predictor and
  store-forwarding are not this machine's, so the CI bench may show less, or more.
