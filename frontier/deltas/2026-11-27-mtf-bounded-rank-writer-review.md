claim: none
by: codex
outcome: done
evidence: PR #1132 head 39d25dfbd054add55f0dd3a8a85d790577fb32f5; src/open.rs; src/open/outq.rs; native executable SHA-256 bf56e98bde5424724244bcd9fd5bab9325cc9b76180ced8da8ac9e4a0d1b4694

**What was asked:** Identify the source of the native code-generation failure in the L1-batched MTF fold and a bounded experiment for its successor.

**What I found:** On an AMD EPYC 9V74, Rust 1.90.0 emitted 11 static `vzeroupper` sites in the library's folded `QueuesOf<1008, 1024>::push_slow`. One precedes its normal return, five precede memory or allocation calls, and five precede bounds or slicing panic calls. These counts describe static sites. Their execution frequency depends on the input.

The rank-copy loop at executable addresses `0x868b0..0x8695e` writes four ZMM vectors and increments its rank index by 128. `push_thin_fast` calls the slow path with at most 16 ranks in production. Splitting a push at a chunk or epoch boundary only reduces that count. Although the 128-rank body cannot execute for these calls, the function sets up wide-vector constants before it knows the copy length. The generic slow path also serves larger test limits.

**Reproduce:** Build the head above with `RUSTFLAGS='-C target-cpu=native' cargo build --release --locked --offline --features mtf-fold,trace --example mtf_a6a_dissection`. In `objdump -d` of the executable, the folded library symbol ends in `he7d51ff25adb8019E`; its chunk bound is 0x800 bytes. Symbols under the example's own `Queues` have different bodies.

**What changed:** This review delta records the rank bound and the assembly at the tested revision.

**What's next:** For the proposed `mtf-fold-batch-measure` experiment, expose the bound to code generation with a dedicated small-rank writer while retaining the generic path for larger inputs. Inspect the complete folded slow path and any helper, including the wide-vector setup and normal return, then test exactness at chunk and epoch boundaries.

**Caveats:** This independent build confirms the native disassembly failure on this host. It supplies no timing result. The original local STOP and i9 queue decision still apply.
