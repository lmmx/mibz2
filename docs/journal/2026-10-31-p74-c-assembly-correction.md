# P74-C assembly follow-up: one correction, residual +1/+1

The clean branch at `bda0d5a4c3a05d760ca11d2a0b633ecb408c5a85` was recovered
in place; no recovery or code regeneration was needed. The original commit remains
an ancestor. The remote branch was verified at that same commit before this change.

One narrowly scoped correction was attempted and retained: keep a u8 error tag in
the two combined loops, constructing `ErrorKind` only after the loops finish.
Only `RunTooLong` and `BlockTooLong` can arise there; all former assignments and the
final successful-checkpoint condition map directly to the tag. This removes the
heap-capable `ErrorKind` representation and its generated drop cleanup from loop
state. There is no new hot-path tag update or test. The policy, iterator, list,
run/weight, output/counts, checkpoints, scalar fallback and production kernels are
unchanged. No second correction was attempted.

## Reproducing the comparison

Both builds used rustc 1.90.0 (`1159e78c4`), LLVM 20.1.8, the existing release
profile (fat LTO, one codegen unit), and the same `examples/p74_c.rs` binary with
Gate 1 Free and all three research arms linked. No native CPU flags were added.

```sh
cargo build --offline --release --example p74_c
objdump -d -C -Mintel --no-show-raw-insn target/release/examples/p74_c
nm -S -C target/release/examples/p74_c
sha256sum target/release/examples/p74_c
```

The before build reproduced the previously reported binary SHA-256 exactly:
`18feb1e6365424fe9aed2843b758e553b319ecb07c5b6129dfd883807078bff7`.
After: `285674800d72637b3a0b0792260e521c4f6ffd620317b204c5b388df7b075c45`.

[Before listing](2026-10-30-p74-c-implementation/low-paths.txt) and
[after listing](2026-10-30-p74-c-implementation/followup-low-paths.txt) contain the
complete ordinary low-rank paths, with instruction addresses and operands. Each
counts one complete loop cycle, including run-accumulator reset, symbol load and
iterator increment, no-run branch, EOB/limit checks, rank dispatch, head update,
output/count updates and loop backedge. Alignment NOPs are excluded; unconditional
jumps are included. Counts are instructions, not uops. There is no loop-boundary
counting mismatch: both variants include the same semantic work and six conditional
branches. This path is rank 1..31 with another symbol remaining, no run to flush,
and no checkpoint/limit crossing. It is not a count for run digits, high ranks,
terminal EOB, checkpoint handling, or every explicit token regardless of context.

| Kernel | Free before | C before | Free after | C after |
|---|---:|---:|---:|---:|
| Plain | 37 | 38 | 37 | 38 |
| Checkpoint | 38 | 41 | 38 | 39 |

## What the original +1/+3 actually was

Plain's extra instruction was `0x2a901: lea rax,[rip+...SWAPPED]`, feeding
`0x2a908: vpshufb ymm0,ymm0,[rdx+rax]`. Free held SWAPPED in rsi, using it directly
at `0x298ea`; both recalculated BLEND. Other low-path operations correspond under
register renaming. Thus the original +1 was an address-materialization instruction,
not monitoring or a head spill. It executes on each visit to this low-rank path.

Checkpoint's +3 was a *net* register-allocation change, not three inserted adjacent
instructions. The following accounts for every difference after register renaming:

| Group | Free before | C before | Difference |
|---|---:|---:|---:|
| Run reset + symbol fetch/dispatch | 5 | 5 | 0 |
| Run test and scalar saves | 4 | 4 | 0 |
| EOB and output-limit tests | 5 | 4 | -1 |
| Rank preparation/dispatch | 5 | 5 | 0 |
| Low-rank head step and join | 8 | 10 | +2 |
| Output/count update and backedge | 11 | 13 | +2 |
| Total | 38 | 41 | +3 |

The -1 is Free's `0x29d64: mov rax,[rsp+0x30]` limit load; C compares against rdx
at `0x2b453`. The head group's +2 consists of `0x2b47c: lea rax,[rip+SWAPPED]`
and `0x2b498: mov r15,[rsp+0x40]` (return pointer reload).

At the join, C loads counts at `0x2b587` and the output pointer at `0x2b594`,
where Free has these in rbx/r10. Free instead loads s2u at `0x29e5c`, where C has
it in r9: net +1. C restores EOB with `0x2b5aa: movzx ecx,word [rsp+0x58]` and
limit with `0x2b5af: mov rdx,[rsp+0x78]`; Free restores next with only
`0x29e7d: mov rcx,[rsp+0x18]`: another net +1. The two scalar saves before the
run branch are exchanged too (Free next/EOB; C EOB/limit), without changing their
instruction count. Both originally keep n in a stack slot and load/increment/store
it each explicit token. These facts explain +3 without assigning it a cycle cost.

## After the correction

Plain still materializes SWAPPED (`0x2a8a4: lea rax,[rip+SWAPPED]`), feeding the
shuffle at `0x2a8ab`; Free now uses the equivalent shuffle at `0x298aa` after the
binary's -0x40 relocation of its function. Plain remains +1.

Checkpoint now uses n in r8 and output in rdi. Its head sequence at
`0x2b34a..0x2b383` is:

```text
2b34a: shl     edx,5
2b34d: vpermq  ymm0,ymm2,0x4e
2b353: lea     rax,[rip+SHIFT]
2b35a: vpshufb ymm1,ymm2,[rdx+rax]
2b360: lea     rax,[rip+SWAPPED]
2b367: vpshufb ymm0,ymm0,[rdx+rax]
2b36d: lea     rax,[rip+BLEND]
2b374: vmovdqa ymm2,[rdx+rax]
2b379: vpblendvb ymm2,ymm1,ymm0,ymm2
2b37f: vmovd   eax,xmm2
2b383: jmp     2b473
```

All three mask addresses are now materialized on this path. That is offset by
less scalar work: no two scalar stores at the run test; n no longer requires a
load/store each iteration; output stays in rdi; no return-pointer reload at the
head join. The exact new grouping is 5 + 2 + 4 + 7 + 11 + 10 = **39**. The rank
preparation's extra two moves are `0x2b335: mov ebx,r11d` saving EOB and
`0x2b338: mov r11,rcx` saving the limit; restores at `0x2b48e/0x2b491` precede
the backedge. Counts still reloads at `0x2b47c`. Free after is the same instruction
sequence as before with addresses relocated -0x40, including its stack-carried n.
The after listing includes every instruction, not just the head excerpt above.

No extra conditional branches or YMM-head spills/reloads occur on either ordinary
low-rank path. The head stays in ymm2. Long-run memset and checkpoint-copy calls
still require head spills, as before. The guard still exists only at high ranks;
F and the iterator origin remain local volatile slots accessed there.

## Dependencies and interpretation

The added LEAs depend on RIP, not on the previous head or token. They provide
shuffle-mask load addresses; their results can be prepared independently of the
head. They add instruction/decode/address preparation work but do not add an
arithmetic link to the head-to-head recurrence. Each head update still has the
same cross-lane shuffle/blend dependencies. Whether mask readiness or register
pressure puts any extra work on the dynamic critical path is not established by
static counts.

The old checkpoint n stack load/increment/store formed a loop-carried memory
chain. The new `inc r8` is a register chain. EOB/limit register saves/restores are
loop-carried scalar values and may be move-eliminated; the previous EOB/limit
stores/reloads used stack dependencies. No new monitor dependency reaches low
ranks. These changes can affect scheduling independently of the +1 count.

Combined function sizes change from 1699 to 1659 bytes (plain) and 4741 to 4091
bytes (checkpoint). Their addresses change from `0x2a4c0/0x2ab70` to
`0x2a480/0x2ab00`; checkpoint stack subtraction changes from 768 to 640 bytes,
plain stays 576. Original Free sizes stay 932/2641 bytes. Code/constant addresses,
branch displacements and instruction-cache placement change across the binary.
None of these observations establishes a measurable speedup or slowdown. T1's
suffix/layout tax is still unmeasured. No benchmark or performance-counter run was made.

## Verification and disposition

- `cargo fmt --all --check`: pass.
- `cargo build --offline --release --example p74_c`: pass, optimized comparison above.
- `cargo test --offline --lib mtf::p74_c`: **7 passed**, 0 failed.
- `cargo test --offline --test mtf_p74_c`: **5 passed**, 0 failed.
- `cargo clippy --offline --all-targets --all-features -- -D warnings`: pass.
- `git diff --check`: pass.

The focused tests cover both errors and exact partial outputs/checkpoints, the
invariants touched by delayed error construction. No full suite was rerun. The
prior commit's 74 tests plus doctest remain historical results, not a claim about
a newly rerun full suite.

The one correction partially succeeded: checkpoint overhead fell by two instructions,
plain stayed unchanged, and ordinary-path branches/head residency were preserved.
The strict frozen no-new-instruction gate still fails (+1/+1). Removing the remaining
allocation differences is not attempted here. This research branch is correctness
verified at the focused-test boundary; it is not ready for the frozen performance
experiment without a separately authorized resolution of that gate. Selection
robustness, the registered independent inputs and suffix/layout economics remain
unmeasured. No thresholds, policy or production dispatch were changed.
