# Kept-row queue assembly

**A variable-width singleton still enters the general range extension in the frozen edge decoder.** In the original scorer binary the kept-run site computes the size of the range, checks spare capacity, checks for an empty range, and compares the remaining width against the vector-loop threshold. Widths 1–7 enter a scalar counted loop. The width-one call therefore has a loop test after its only store. The primary-split singleton has a different compiled path. Its constant width eliminates the loop, though range-size arithmetic, the reserve check and a comparison of the row with `u32::MAX` remain. The current block bound is 900,000 rows. See the original at addresses 0x3707b, 0x37130 and 0x37201 in [sites.txt](sites.txt).

Two builds exercise the actual inherited source. The archived scorer from the [worker-dose panel](../../worker-dose/panel/manifest.txt) contains the unchanged edge. A separately built binary instantiates six complete `BzReader::parallel_workers_with` decoders, with the full constructor retained and separate Rust modules for each variant. It uses one replica each of edge at `badb2aec3624bd65606c429b2e6f79e9d9d0050f` and the joint scalar cap/counter arm at `50b8ecb8946a498601e3fd894ed84a2687df6cc8`, with three versions of the single kept-queue append per replica. The real scorer edge and the multi-arm edge compile the same range checks and scalar store loop at that site. Build flags, modules, toolchain and binary hashes are available in [worker.py](worker.py) and [manifest.txt](manifest.txt).

| Queue fill | Edge constructor size | Joint constructor size | Emitted capacity-hit path through the regular kept site |
|---|---:|---:|---|
| Range | 16,223 B | 16,587 B | General reserve check, range-empty and SIMD thresholds, counted scalar loop for 1–7 |
| Explicit singleton | 16,209 B | 16,545 B | Width check, `len == cap` check, one store and incremented length |
| Explicit 1–4 | 16,683 B | 17,001 B | Bounds check and indirect dispatch, two and three scalar stores for widths 2 and 3, one unaligned 16-byte store for width 4 |

For both explicit implementations, unhandled widths still take the original range-extension path. A full queue on the singleton path invokes the standard vector growth path. The fixed slices check the space for their entire width and reserve when necessary. We crossed widths 0–17 with every starting length at every requested capacity from 0 through 96. All 85,554 states produced equal queues and equal final capacities under Rust 1.99.1, including 14,025 growth cases. The six complete decoder monomorphizations accepted the frozen stream's CRCs and produced its 900,000-byte oracle length. [Capacity check](capacity.rs) · [Capacity receipt](capacity.txt) · [Decoder dry run](dry.txt)

The explicit singleton and fixed short slices both warrant a paid trial. The latter trades a loop for indirect dispatch and increases the compiled constructor by 460 bytes on edge and 414 bytes on the joint arm. A complete worker measurement is needed to establish whether either change saves time. This assembly experiment collected no timing samples.

## Reproduction

From the research worktree, using Rust 1.99.1 on x86-64 Linux with GNU binutils 2.44, run the following. The archive holds full source-mapped constructor disassembly for the six arms plus the scorer edge. The `sites.txt` offsets refer to the recorded binaries. Changing the path, toolchain or linker can change the binary hash and addresses. The standalone probe supplies a shorter reference for the Vec operations.

```sh
d=research/astra-e/queue-short-fill/asm
python3 "$d/worker.py" /workspace/work/astra-fill-asm
rustc --crate-type lib -C opt-level=3 -C codegen-units=1 --emit asm "$d/probe.rs" -o /tmp/short-fill-probe.s
rustc -C opt-level=3 -C codegen-units=1 "$d/capacity.rs" -o /tmp/short-fill-capacity
/tmp/short-fill-capacity
python3 "$d/capture.py" /workspace/work/astra-fill-asm /workspace/work/astra-dose-bins/edge /tmp/short-fill-capture
xzcat "$d/constructors.txt.xz" | less
```
