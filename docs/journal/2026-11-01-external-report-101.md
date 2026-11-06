# External research report 101: Astra's prescription against the implemented unrolled walk (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-01 01:29 UTC.
- **Source:** the maintainer's request, with Astra's machine-code prescription quoted.
- **In reply to:** the walker's fidelity question.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Astra’s contribution is intended to be machine-code-specific implementation guidance, not simply an independent recommendation to unroll the walker.

Please compare our existing research/walk-unroll implementation with Astra’s precise assembly-level prescription:

* One fully unrolled ordinary round, with all 32 lanes addressed through compile-time constants.
* No per-lane dynamic index arithmetic, comparisons or loop backedge.
* Shared scalar fallback from the first boundary lane, preserving existing traversal and prefetch semantics.
* Expected ordinary-step reduction from approximately 14.4 to 9 fused macro-ops.
* No significant additional spills, hot-path fallback overhead or LLVM loop rerolling.

Inspect the actual compiled disassembly of our existing implementation and identify which of these signatures it achieves and which it misses.

If there is a meaningful machine-code discrepancy, propose the smallest source-level change that would achieve Astra’s intended instruction sequence. Do not automatically start another benchmark or alter production code.

I want to know whether Astra has shown us how to implement the existing idea substantially better—not whether it independently invented loop unrolling.

I'm working on the performance of the lmmx/mibz2 bzip2 decoder. I want you to act as an expert x86-64 machine-code optimizer, with particular attention to memory-level parallelism, register allocation, instruction scheduling and dependency chains.

Repository: https://github.com/lmmx/mibz2

Starting point: Examine the existing BWT walker evidence pack on branch research/astra-room-walk-pack, alongside the production code on master. The primary target is src/bwt.rs::room_segments::<32, PF_L1>.

Our objective is a substantial reduction in cycles per decoded byte, without sacrificing correctness, increasing peak memory substantially or degrading performance on important input distributions.
The performance problem
The walker interleaves 32 logically independent inverse-BWT traversal chains to hide the latency of dependent T-vector loads.

Historical measurements include:
- 38.55 cycles for a single dependent-chain lookup.
- 4.49 cycles per decoded byte for the walker in a later benchmark, compared with 10.78 c/B for the complete decoder.
- A separate earlier revision measured 6.69 c/B for walking and 12.93 c/B overall.
- The lane-count curve flattened near 24 lanes.
The simplified calculation 38.55 / 4.49 ≈ 8.6 suggests substantially less effective overlap than the 32 logical lanes might imply. This is a diagnostic model, not a direct measurement of outstanding memory requests, and the latency and throughput measurements may involve different memory conditions.

The two historical performance baselines are from different commits. Do not combine their measurements as though they were simultaneous.
Your assignment
First, diagnose the compiled implementation.

Examine the exact disassembly and compiler provenance supplied in the evidence pack. Identify what limits the rate at which independent T-vector loads can be issued and retired.

Pay particular attention to:
- The Lane.row → T-vector load → next Lane.row dependency recurrence.
- Whether lane state and output cursors remain in registers or repeatedly spill to memory.
- Address-generation and load/store pressure.
- Prefetch placement, usefulness and potential contention.
- Loop-carried bookkeeping, boundary flags and branches.
- Instruction-window capacity and achievable memory-level parallelism.
Distinguish unavoidable memory-latency costs from avoidable machine-code dependencies.

Second, investigate structural superoptimization.

Consider whether processing four interleaved groups of eight scalar lanes could reduce instruction and bookkeeping overhead while preserving all 32 independent chains.

Do not assume that eight-lane batching helps. Determine whether it would actually preserve the useful overlap or inadvertently serialize the memory accesses.

Compare this with other promising transformations suggested by the machine code. Do not limit yourself to superficial instruction-count reduction or smaller function size.

Third, select the single strongest candidate.

If the analysis identifies a credible opportunity, propose one correctness-preserving transformation that is likely to produce a meaningful improvement.

Provide:
1. The specific assembly-level bottleneck, citing instructions and dependency chains from the evidence pack.
2. The proposed transformation and why it should improve actual execution.
3. An implementation sketch or minimal experimental patch, isolated from production dispatch.
4. The expected change in generated assembly.
5. A conservative, revision-specific estimate of cycles per byte saved, with assumptions.
6. The main failure modes and a falsifiable benchmark or hardware-counter test.
For context, a hypothetical 0.65 c/B saving from a separate MTF optimization would leave approximately 1.15 c/B of walk savings needed for 20% higher overall throughput on the 10.78 c/B baseline. This is a planning target, not a requirement to manufacture a predicted improvement. The MTF saving itself has not been established on that revision.

If the generated code is already close to an architectural limit, explain why and identify the evidence that would distinguish that conclusion from an avoidable compiler problem.
Boundaries
This is an independent machine-code investigation, not a broad redesign of the decoder.

Do not modify production behavior, merge anything, initiate expensive benchmarks or run a new experimental programme. You may inspect and analyze the repository and create a minimal isolated patch if you find a well-supported candidate.

Be technically ambitious, but rigorous. I value a concrete transformation supported by disassembly more than a long list of speculative optimization ideas.

Conclude with a clear recommendation: which single change should we test next, and what exact observation would falsify its expected benefit?

**Test a fully unrolled 32-lane ordinary round, with one shared scalar fallback for segment boundaries.** This is the strongest candidate supported by the supplied assembly. It preserves the existing traversal order and prefetch policy, removes substantial bookkeeping, and requires no output tile or speculative replay.

My conservative planning estimate is **0.3–0.6 c/B saved on the historical 4.49 c/B basis**, conditional on the expected code generation. Zero improvement remains plausible. The evidence does not support promising the 1.15 c/B planning target.

The audited listing is the pack’s `ae942bd` build: rustc 1.97.0, baseline x86-64, release optimization, fat LTO, one codegen unit, no `RUSTFLAGS`; symbol `room_segments::<32,1>` begins at `0x29a50`. Current master, `45b366d`, has the **same entire `src/bwt.rs` blob**. That establishes source identity, not identical code generation in historical CI binaries. The pack does not record an LLVM version. [Compiler and binary provenance](https://github.com/lmmx/mibz2/blob/c4018b54594f2ba7a6e50ad364b328f53a8980f6/docs/research/2026-10-31-astra-room-walk/README.md)

**What the machine code establishes**

The critical recurrence is:

```asm
2a534: mov    ebx,DWORD PTR [r11]         ; lane row
2a537: mov    eax,DWORD PTR [r15+rbx*4]   ; T[row]
2a53b: test   eax,eax
2a53d: js     2a570                      ; segment boundary
2a53f: mov    edi,eax
2a541: shr    edi,8                      ; next row
2a544: mov    DWORD PTR [r11],edi
2a547: prefetcht0 BYTE PTR [r15+rdi*4]
2a54c: mov    rdi,QWORD PTR [rsp+r8*8+0x70]
2a551: mov    BYTE PTR [rdi+rcx],al
```

The unavoidable dependence is `T[row] → extract successor → access successor`. The compiler adds a memory-resident row between successive visits and reloads the output base for every byte. These are addressable lane-state accesses; calling all of them register-allocation “spills” would overstate what the listing proves. The next row load can hit L1 or forward from a pending store—it does **not necessarily** use store forwarding every round. [Exact listing](https://github.com/lmmx/mibz2/blob/c4018b54594f2ba7a6e50ad364b328f53a8980f6/docs/research/2026-10-31-astra-room-walk/room_segments_32_1.nolines.asm#L450)

There is **no architectural dependence from lane k’s T-load result to lane k+1’s T-load address**:

- Reusing `eax`, `ebx` and `rdi` does not serialize independent iterations; register renaming separates them.
- The next lane address comes from the cheap `r11 += 32` recurrence.
- Predicted `js` branches allow speculative execution into subsequent lanes.
- The output address depends on the base load and `off`, independently of the byte’s arrival. An unresolved store *data* operand alone does not block subsequent independent loads.

Memory-disambiguation stalls or address-alias replays remain possible, but the listing does not demonstrate them.

The removable lane bookkeeping is precisely:

```asm
2a530: lea r10,[r8+1]
...
2a554: lea rax,[r10-1]
2a558: add r11,32
2a55c: mov r8,r10
2a55f: cmp rax,31
2a563: jb  2a530
```

That is **six instructions, five fused macro-ops**, assuming `cmp/jb` fusion. The ordinary step totals 16 instructions or 14 fused macro-ops. Counting the outer-loop instructions directly gives **14.406 fused macro-ops per ordinary step**; the pack’s approximately 14.8 includes an additional estimated allowance for other work.

| Resource | Existing ordinary step | Consequence |
|---|---:|---|
| Loads | Row, T entry, output base | Two supporting loads per useful T load |
| Stores | Row, output byte | Store queue and address-generation pressure |
| Prefetch | One | Additional address/cache lookup and potentially miss resources |
| Fused macro-ops | About 14.4 including round overhead | Dispatch and instruction-window occupancy |
| Boundary branch | One per step | Usually predictable, but still occupies resources |

Using AMD’s six-wide dispatch and three combined memory operations per cycle gives optimistic floors of approximately **2.4 c/step for dispatch** and **2.0 for memory operations**. These are overlapping bounds, not additive costs. On Comet Lake, four-wide allocation gives roughly 3.6 c/step before target-specific adjustments; the indexed output store also competes for the full address-generation units. [AMD architecture reference](https://gpuopen.com/download/GDC_AMD_Ryzen_Processor_Software_Optimization.pdf)

Thus 4.49 c/B does not establish a pure dispatch limit. Equally, the arithmetic residual `4.49 − 2.47 = 2.02` does not establish a separately additive miss penalty.

**What limits overlap remains partly unresolved**

At roughly 14.4 macro-ops per step, a 256-entry Zen 3 retirement window holds only about **18 complete step-equivalents**; Zen 4’s 320 entries hold about 22. These are occupancy illustrations, not outstanding-load counts. Scheduler and load/store queues have different allocation and release rules.

Crucially, **prefetch requests can survive retirement of their issuing instructions**. All 32 chains therefore need not simultaneously fit in the ROB to contribute useful overlap. This is why “512 instructions exceed the ROB” is insufficient to diagnose the measured throughput.

The prefetch is already placed promptly after successor extraction. It cannot start before the current T load returns. A deeper successor prefetch requires another dependent load. Approximately 144 cycles per rotation is only a rough historical model: it includes other work and does not establish the actual prefetch lead or cache-hit rate. Prefetch can help by launching misses beyond the demand-load window, while also consuming address-generation, translation and miss-handling resources.

The flat lane curve and lack of SMT throughput gain support a **shared-resource or window constraint**, but do not identify which resource. SMT partitioning can eliminate gains even when average dispatch utilization is below its maximum. Likewise, clock scaling points away from a simple fixed-time DRAM floor without proving an instruction-throughput limit.

The B2 reference itself reports that its documented-parameter simulation predicts approximately 2.47 c/B, while its additive interval model fits the observations. That unresolved disagreement prevents treating its larger predicted savings as mechanically established. [B2 analysis, §4.3](https://github.com/lmmx/mibz2/blob/c4018b54594f2ba7a6e50ad364b328f53a8980f6/docs/research/2026-10-31-astra-room-walk/refs/2026-10-30-b2-gathered-walk.md)

**Why four groups of eight are not a register-allocation solution by themselves**

There are two materially different schedules:

```text
Preserves rotation:  round → group 0, group 1, group 2, group 3
Reduces overlap:     group → many rounds of its eight lanes
```

The second can leave only eight chains progressing. `38.55/8 ≈ 4.82` illustrates its risk under the historical latency assumption; it is not a universal lower bound across cache conditions.

The first preserves all 32 chains, but state must survive between group visits. Eight rows plus eight output pointers already exceed the practical GPR budget after reserving loop and table registers. Switching groups does not make the other groups’ state disappear.

I therefore recommend using four groups of eight only as **source organization for one fully expanded round**. No group-local multi-round loop, separate calls, or requirement to finish eight loads before exposing another group.

Compared with B2’s tiled commits, this removes fewer memory operations, but has much firmer correctness and code-generation premises. Dense row storage alone does not remove the row load/store recurrence; tiling additionally introduces commit traffic, flag accumulation and replay. Neither is the strongest first test.

**Implementation sketch**

In a separately named research copy specialized to 32 lanes, replace only the inner round with this structure. Keep production dispatch unchanged:

```rust
for off in 0..room {
    macro_rules! step {
        ($exit:lifetime, $k:literal) => {{
            let q = unsafe { *t.add(lanes[$k].row as usize) };
            if q & START != 0 {
                break $exit $k;
            }
            lanes[$k].row = q >> 8;
            prefetch::<PF_L1>(t, lanes[$k].row);
            unsafe { *base[$k].wrapping_add(off) = q as u8 };
        }};
    }

    let first_boundary = 'ordinary: {
        step!('ordinary, 0);  step!('ordinary, 1);
        // Expand every literal lane index, in order, through 31.
        // Four groups of eight are formatting only.
        step!('ordinary, 31);
        32
    };

    for k in first_boundary..32 {
        // Existing scalar inner-loop body verbatim:
        // START handling, finish/take, base rebasing, ordinary step,
        // and break 'full when the queue is exhausted.
    }
}
```

This is a sketch, not compiled code; the omitted lane indices must be expanded.

At a boundary in lane `k`, lanes below `k` have committed exactly their original steps, while lane `k` and later lanes remain untouched. Resuming the original loop at `k` therefore preserves its mid-pass exit accounting. It rereads one flagged entry, but does not replay emitted bytes. Growth, partial-lane initialization and tail handling remain unchanged. No additional block-sized storage is required.

The expected assembly is **32 straight-line ordinary steps using constant lane/base offsets**, with boundary branches to cold stubs that select the failing lane and join one fallback. Per ordinary step:

| Quantity | Existing | Expected |
|---|---:|---:|
| Instructions | 16 | About 10 |
| Fused macro-ops | 14 | About 9 |
| Loads / stores / prefetches | 3 / 2 / 1 | 3 / 2 / 1 |
| Per-lane loop backedge | Present | Absent |

Do not credit register-resident rows or bases until the actual binary demonstrates them. Reject the intended signature if LLVM rerolls the loop, introduces substantial additional spills, or materializes fallback indices throughout the hot path.

The benefit is more than instruction-count reduction: the same window could cover roughly **28 rather than 18 step-equivalents**, allowing the processor to reach independent loads sooner during a long-latency stall. It still does not guarantee 32 simultaneous demand misses.

**Estimate and decisive test**

One measurement correction matters: historical `phases.rs` prints the 4.49 walk row as **walk plus segment assembly**. The separate 0.16 assembly row is already included. Also, its denominator is final decoded bytes, whereas one ordinary lane step emits one byte *before RLE1 expansion*. Per-step instruction counts must be converted using that ratio on run-heavy inputs. [Historical harness](https://github.com/lmmx/mibz2/blob/c8f9740/examples/phases.rs#L173)

For the `c8f9740`/EPYC 7763 basis, assuming ordinary full-lane steps dominate and walked bytes approximately equal decoded bytes:

- Dispatch pressure falls substantially, but the unchanged memory operations leave an approximately 2 c/step floor.
- The direct issue-floor difference is only about **0.4–0.5 c/step**.
- Allowing for unchanged tail work, cold fallback overhead and uncertain window benefit gives a provisional **0.3–0.6 c/B saving**, or a reported walk row around **3.89–4.19 c/B**.

That corresponds to approximately 3–6% higher overall throughput on the **10.78** baseline, holding other stages fixed. I would not transfer this numerical prediction to the separate **6.69/12.93** baseline or add an unestablished MTF saving.

For the next test, use the existing paired stage harness, the same binary and decode-order cache preparation, with exact CPU placement and separate walk/assembly measurements. Record retired instructions, branch misses, dispatch stalls attributable to retirement/load/store queues, and model-supported miss-occupancy events. Distinguish demand misses from prefetch activity; do not infer occupancy from the single-chain ratio.

The main failure modes are unchanged memory-system throughput, reduced prefetch lead, extra spills, instruction-cache/op-cache pressure, and boundary-heavy inputs spending too much time in fallback. Correctness checks must exercise every boundary lane, queue exhaustion, buffer growth, short/partial-lane cases and existing crafted permutations.

**Recommendation: test this one unrolled-round transformation next. Its expected meaningful benefit is falsified if the intended assembly reduction is present, but the paired walk-stage ratio’s 95% confidence interval lies entirely above 0.95**—less than a 5% saving, approximately failing to reach 4.27 c/B on the historical inclusive basis. An interval crossing that threshold is inconclusive. A reproducible regression on an important distribution also rejects adoption.

No production code was modified, and no compilation or benchmarks were run.
````
