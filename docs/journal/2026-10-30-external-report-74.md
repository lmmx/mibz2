# External research report 74: P74-V: one masked boundary chunk, then unmasked copies (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-30 18:31 UTC.
- **Source:** unattributed in the relay (the researcher's register).
- **In reply to:** after report 73, on P74's high-rank fallback.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
1. P74: a better implementation candidate than H

Do not jump directly from masked chunks to libc memmove. There is a stronger intermediate design: one masked boundary chunk, followed by unmasked vector copies. Call it P74-V.

I inspected the P74 source at e57400e. Its high-rank fallback blends every 32-byte tail chunk. That is unnecessary.

For rank , define , , . The desired tail update is

U’0=H31,\qquad
U’i=Ui-1\;(1\le i\le j),\qquad
U’i=Ui\;(i>j).

Only chunk requires a partial write of bytes. Every chunk must be shifted in its entirety:

U’32c:32c+32=U32c-1:32c+31.

Execute chunks in descending order, then install at U[0]. The tail has valid prefix slack, so the first unaligned read is legal.

Cost structure:

I_{\rm old}(q)=55+12(m+1).

An implementation using one masked boundary and approximately two vector memory instructions per complete chunk removes most mask generation, blending and old-data loads:

I_{\rm V}(q)\approx I_{\rm fixed}+I_{\rm boundary}
+2m+I_{\rm loop}.

The possible saving grows with , exactly where controls are expensive. Unlike H’s memmove call, P74-V need not cross an ABI boundary or spill the YMM-resident head.

Crucial compiler constraint: LLVM previously recognized the plain vector-copy loop as memmove. The full-chunk operation must therefore be emitted as explicit AVX2 load/store assembly, or its intended instruction count is not guaranteed.

This is the first implementation candidate I would put ahead of H. It preserves the entire low-rank path, needs no selector and changes only the high-rank fallback.

2. Implementation artifacts: produced and validated

P74-V AVX2 patch

One masked boundary, explicit unmasked lower-chunk moves

P74-H memmove patch

Direct competing implementation, same starting commit

Actual AVX2 correctness harness and independent byte model

Runnable validation programs

Verified locally

* Python byte model: 76,112 correct MTF transitions, including every rank and long stateful sequences.
* Compiled AVX2 harness: 105,600 correct transitions, including every rank, randomized list states and repeated boundary crossings; run with address and undefined-behaviour sanitizers.

These validate the algorithm and an independently implemented AVX2 instruction sequence. They do not establish that the Rust patch compiles or accelerates mibz2. This environment has no Rust toolchain; no target benchmark, CI run or repository write was performed.

Both patches target research commit e57400e, not master, and are mutually exclusive alternatives.

3. P74-V performance calculation

On roughly uniform random high ranks, is uniform on , hence

\mathbb Em=3.

If removing a full chunk’s mask/blend sequence saves approximately 9 instructions, then

\Delta I_{\rm event}\approx 9(3)=27.

With ,

\boxed{\Delta I_{\rm random}\approx20.5\ \text{instructions/output byte}.}

That is potentially substantial. It is an instruction-count estimate, not a cycle saving. The compiler may already overlap much of the old work, and overlapping memory operations can introduce stalls.

The decisive comparison is therefore not H versus an imagined libc cost. It is P74-original versus H versus V, same binary, same inputs and stage boundary.

V has three structural advantages over H: no memmove call, no ABI-mandated YMM preservation across that call, and only one masked boundary chunk. Its failure modes are code-size growth, register pressure and store/load forwarding across repeated tail updates.

Required assembly audit: confirm one masked top chunk, exactly one unaligned load and one aligned store for each lower chunk, no memmove, no unexpected YMM spills, and unchanged low-rank fast-path instructions.

A patch passing semantic tests but losing its register-resident head is a failed optimization.

4. The combined breakthrough target is easier than the B2-alone target

Use one internally consistent baseline: EPYC 7763, one-copy decoder c/B, room walk c/B, MTF c/B.

For 20% higher throughput, allowable decoder cost is

T_{\rm target}=\frac{10.78}{1.20}=8.983\ \mathrm{c/B}.

The required saving is

\boxed{\Delta T_{\rm total}=1.797\ \mathrm{c/B}.}

Let the eventual measured P74-V saving be , and the walk saving . Then

\boxed{d_P+d_W\ge1.797.}

This is the right joint acceptance equation. Do not require the walk to deliver the entire breakthrough alone.

Measured P74-V saving	Maximum acceptable new room-walk cost	Walk reduction required
0.0 c/B	2.69 c/B	40.0%
0.4 c/B	3.09 c/B	31.1%
0.6 c/B	3.29 c/B	26.7%
0.8 c/B	3.49 c/B	22.2%

These are conditional budgets, not predictions. The historical P74 Zen 4/5 gains cannot be substituted for a measured Zen 3 .

Engineering consequence: a modest BWT-walk redesign becomes breakthrough-capable if P74-V ships. Requiring B2 to halve the walk independently wastes this combined opportunity.

5. BWT breakthrough: change the next implementation hypothesis

Do not make AVX2 gather the default design. Zen 3’s gather µop expansion is a serious obstacle. Preserve scalar independent loads and their existing 32-way memory-level parallelism.

The better candidate is speculative eight-step traversal, with flag handling amortized over eight lanes.

Each group owns eight independent traversal rows. Save its starting state, execute eight steps per row without branching on segment-start flags, and OR the observed flags into a group accumulator. Every link remains valid after masking the start flag.

At the end of the tile, either commit the 64 outputs or restore the group’s saved state and replay using the proven scalar handler. The speculative output writes remain within reserved buffer capacity; output lengths are committed only after validation.

This preserves 32 independent chains by running four eight-lane groups, while eliminating nearly all per-byte segment-end branches in the common case.

Exact replay-work ceiling: for a full 900,000-byte block with at most 512 segment completions, every failed eight-lane tile repeats at most 64 traversal steps:

\frac{512\times64}{900000}=3.64\%.

That is a structural bound on duplicate traversal steps, not a 3.64% runtime bound. It assumes each completion causes at most one failed tile; the replay handler must correctly process any additional segment completion within that tile.

A single 32-lane tile would instead expose duplicate-work potential. Keep replay domains at eight lanes.

The remaining cost is one flag accumulator update and one extra link mask per speculative step, against removal of the per-byte conditional and much of the segment-transition machinery. That trade needs generated-code inspection. It may lose if the added dependencies outweigh reduced control pressure.

The actual concurrency target

The shipped walk’s single-chain latency and room throughput imply effective overlap

\frac{38.55}{4.49}\approx8.6.

With c/B, the joint throughput target requires room cost c/B, corresponding to roughly

\frac{38.55}{3.29}\approx11.7

effective outstanding chains, ignoring non-load overhead.

That is an increase from approximately 8.6 to 11.7, not an impossible demand for 24 simultaneous misses. The processor has enough nominal miss capacity. The unresolved constraint is whether the implementation can expose it through the instruction window and dependency scheduler.

The experiment must distinguish a branch/control reduction from genuine memory-level parallelism improvement. Use actual uops, IPC, outstanding misses and cycles/B; source instruction counts are not interchangeable with decoded µops.

6. One controlled implementation sequence

Phase A — P74-V against H. Apply both patches independently to the frozen research source. Run the existing complete MTF/RLE2/checkpoint equivalence tests and malformed-stream tests. Inspect both assembly paths. Then use one preregistered same-binary stage comparison with the existing corpus and independent controls. No post-result threshold tuning exists because neither variant uses a selector.

Phase B — full decoder. Promote the fastest qualified candidate only if Wikidata improves materially, the upper 95% regression bound on random/records is at most 3%, and CPU-s/GB, memory and checkpoint correctness pass. Measure 1, 2 and 4 workers; the CI machine’s two physical cores with SMT cannot establish four-physical-core scaling.

Phase C — scalar-load eight-lane epochs. Keep the shipped builder and T-vector unchanged. Implement the epoch as an isolated research walk. Prove identical per-segment outputs and successor indices on arbitrary cycles, including segment completions within the same epoch. Measure its full room-walk cost including replay, not just its steady-state loop.

Phase D — joint gate. Combine qualified P74 and walk candidates, then measure both together. Instruction-cache effects, pipeline overlap and different target processors mean independent stage savings cannot simply be declared additive without testing.

Stop rules: abandon V if the intended AVX2 loop collapses into memmove or introduces fast-path spills. Abandon the epoch walk if its complete cost cannot meet c/B on the same processor. Neither failure justifies restarting N1’s expensive constructor.

Decision

P74-V is now an actual patch with independently validated AVX2 semantics, not another theoretical proposal. It is the most economical immediate performance candidate because it removes repeated fallback work without introducing classification or handoff.

The breakthrough path is joint: recover roughly 0.4–0.8 c/B in MTF and 1.0–1.4 c/B in the existing BWT room walk. Those are target budgets, not measured achievements.

The next irreplaceable evidence is generated Rust assembly and target timings. No mathematical argument can certify the required cycles/B from instruction counts alone.
````
