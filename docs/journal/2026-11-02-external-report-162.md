# External research report 162: a ChatGPT session on memory-level parallelism in a BLAKE3 fork (johnservil/BLAKE3): independent input streams, lookahead prefetch, and load–compute pipelining (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02.
- **Source:** a new ChatGPT session, separate from the guardian, the researcher and Astra.
- **In reply to:** the maintainer's question, quoted below.

> does this code use memory level parallelism? https://github.com/johnservil/BLAKE3

**Evidence cutoff:** 2 November 2026. The reply says it checked the fork's default `servil`
branch at `09d3bbc`. Its source links are kept as relayed; this repository has not
inspected that code. The maintainer was not sure whether the repository is a fork.

**Follow-up:** report 163, a guided walkthrough. Its reply was pending when this was
recorded.

**Provenance, checked later by the guardian (report 164).** These notes sit beside the
transcript, which is unchanged:
- **The repository** is a fork of `BLAKE3-team/BLAKE3`, with `servil` as its default branch.
- **The AVX2 mechanism also appears upstream.** The eight-stream loads, and the lookahead
  prefetch at `rdx + 128` against the current block at `rdx - 64`, are established upstream
  techniques, not this fork's additions.
- **The wide AVX-512 kernel and the SME2 path** are as the reply describes. They are not
  runnable on the i9-10900K, which has AVX2 but not AVX-512.
- **The closing PMU link is for Granite Rapids.** The i9-10900K (Comet Lake, model A5) uses
  the Skylake-family event definitions.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The reply, as received

`````text
**Yes. This fork explicitly tries to exploit memory-level parallelism, including within a single thread.** Its bulk hashing kernels use independent input streams and software prefetching; its Arm SME2 implementation also loads the next block while computing the current one.

I checked the repository’s default `servil` branch at commit [`09d3bbc`](https://github.com/johnservil/BLAKE3/tree/09d3bbc6d07eedbf66545a35adfb6b5546445bc4). Here are the concrete examples.

## 1. The x86 kernels prefetch across independent input streams

The AVX2 implementation loads **eight input pointers**, then loads data from those separate inputs. Their addresses are independent: reading one input’s data is not necessary to determine where to read another. That gives an out-of-order CPU an opportunity to overlap the memory requests. See the [pointer setup and input loads](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_avx2_x86-64_unix.S#L68-L97).

It also explicitly prefetches future data from all eight streams. These are four of the eight instructions:

```asm
prefetcht0 [r8+rdx+0x80]
prefetcht0 [r12+rdx+0x80]
prefetcht0 [r9+rdx+0x80]
prefetcht0 [r13+rdx+0x80]
```

The purpose is to request future cache lines early enough that their fetches can overlap the current hashing work. Because `rdx` has already advanced by 64 bytes, these addresses are **192 bytes—three BLAKE3 blocks—ahead of the current block’s start**. [Source: block offset update](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_avx2_x86-64_unix.S#L82-L89), [prefetch instructions](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_avx2_x86-64_unix.S#L169-L176).

The wider implementations follow the same idea:

| Kernel | Independent input streams in its wide path | Explicit memory overlap mechanism |
|---|---:|---|
| AVX2 | 8 | Lookahead prefetches across the eight streams |
| Ordinary AVX-512 | 16 | Lookahead prefetches across sixteen streams |
| This fork’s wide AVX-512 kernel | 32 | Loads each input block and issues a prefetch at `current_offset + 192` |

Sources: [AVX2](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_avx2_x86-64_unix.S#L169-L176), [ordinary AVX-512](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_avx512_x86-64_unix.S#L174-L209), [32-input kernel generator](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/tools/gen_avx512_32.py#L293-L304).

The 32-input path has a specific dispatch condition: Linux, suitable AVX-512 support, AMD family `0x1A` onward—identified in the source as Zen 5—and complete groups of 32 whole chunks. Other AVX-512 CPUs use the older kernel, which already has prefetching. [CPU selection](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/src/platform.rs#L515-L558), [kernel dispatch](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/src/ffi_avx512.rs#L33-L79).

## 2. The Arm SME2 path explicitly overlaps loads with computation

This is especially clear in `SEVEN_ROUNDS_PIPELINED`.

The current block’s message vectors are held in registers, leaving the ZA tile available for the next block. The macro distributes **sixteen next-block loads among the seven compression rounds**. Those loads are independent of the current block’s arithmetic, so the hardware has an opportunity to execute them while that arithmetic proceeds. The main chunk loop actually invokes this pipeline. [Pipeline implementation](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_sme2_aarch64.S#L351-L411), [calling loop](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_sme2_aarch64.S#L623-L660).

There are also two explicit prefetch mechanisms:

- **Next group:** `PREFETCH_NEXT_GROUP` issues `prfm pldl1strm` for the next sixteen chunk pointers. [Assembly](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/c/blake3_sme2_aarch64.S#L428-L447).
- **Next subtree:** the Rust `flat_walk` wrapper issues `prfm pldl2keep` at 128-byte intervals over following input, before hashing the current subtree. The comments explicitly describe the intention: have that input arrive while the current subtree is being hashed. [Rust implementation](https://github.com/johnservil/BLAKE3/blob/09d3bbc6d07eedbf66545a35adfb6b5546445bc4/src/ffi_sme2.rs#L419-L440).

These are deliberate memory latency hiding techniques.

## What “yes” means here

The code contains several distinct forms of parallelism:

| Form | What overlaps |
|---|---|
| **SIMD** | Arithmetic on multiple chunks or messages |
| **Instruction-level parallelism** | Independent instructions and arithmetic dependency chains |
| **Memory-level parallelism** | Independent memory requests that can remain outstanding together |
| **Multithreading** | Work on multiple CPU cores |

**The independent loads and prefetches are the evidence for MLP.** Merely seeing SIMD instructions or a thread pool would be insufficient.

The qualification is that **eight, sixteen, or thirty-two input streams do not imply that many simultaneous DRAM misses**. Some accesses may hit in cache; prefetch requests are hints; and actual concurrency depends on the CPU’s execution window, miss-tracking capacity, input size, and selected kernel. Load–compute overlap also does not, by itself, establish how many memory misses overlap.

So the source establishes that the implementation deliberately exposes and encourages MLP. **How much it achieves, and how much performance that contributes on your machine, requires profiling.** Relevant measurements include outstanding-miss occupancy and memory-stall cycles; Intel, for example, documents counters for both the number of outstanding L1 misses and the duration for which misses remain outstanding. [PerfMon Events Documentation](https://perfmon-events.intel.com/platforms/graniterapids/core-events/core/?utm_source=chatgpt.com)
`````
