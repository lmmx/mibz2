# External research report 17: research burn-down rounds 13–14, the front end, fusion, whole-decoder accounting, and the fallback rule (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 16 (`2026-10-30-external-report-16.md`).

**What it contains:**
- **A valid replacement for round 11's invalid example:** the primitive 64-byte cyclic
  string (aabacaba)⁷aabacabb. The coordinator checked it: it is primitive, and both of
  its 32-row groups have internal breaks.
- **#11 / P78:** buffer reuse is distinguished from work elimination, and an alias-safety
  caveat is added for in-place construction.
- **#21 / E9:** output-buffer reuse cannot shrink the walk's hot table.
- **#19 / E7:** 20 bits suffice for every valid row index, which says nothing about
  speed.
- **The scanner:** the pipeline ceiling is distinct from its one-core Amdahl share.
- **P134:** a gap between two throughputs is not a component cost.
- **P75 and P74:** from the recorded MTF rank distribution, only 1.52% of explicit
  updates have rank ≥ 32.

**Round 14,** appended after round 13:
- the researcher's reading of Gate A's adverse traffic model;
- a necessary rejection rule, m₈ ≥ r, so r ≲ 281,250 for a 3.6 MB budget;
- early rejection between levels;
- the fallback cost N3 must carry, with options A, B and C;
- the mixed-corpus cost equation.

**The coordinator's refinement:** the fallback is a graded choice of k, not a binary
switch.
- **Level by level:** each level's m_k is exact once that level is built, and the bound
  m_2k ≤ 2m_k − 1 accepts the next level before it is built.
- **At level 1:** r is exact after MTF, and a running count rejects safely mid-block.
- **Applied to the census:**
  - `records` and `random` fall back to the T-vector at level 1, with no composition
    built;
  - `text` takes P⁴ at 2.45 MB;
  - all 233 Wikidata blocks take P⁸, 203 of them accepted from m₄'s bound before level 8
    is built.

The coordinator's ledger is in `2026-10-30-research-portfolio.md` §9.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text and is kept as received.

## Round 13, as received

````text
## Burn-down round 13: the front end, fusion, and whole-decoder accounting

Claude is right to reject my alternating-last-column example as a valid cyclic BWT. A small valid replacement exists: the primitive 64-byte cyclic string `aabacaba` repeated seven times followed by `aabacabb` gives a stable Ψ with internal translation breaks in both aligned 32-row groups. I checked that directly from its sorted cyclic rotations. This establishes that 100% impure groups are possible for a valid block, though it does not establish the worst-case exception rate of any particular FOR/EF encoding.

Now to the next part of the register. The unresolved front-end questions are more closely connected than they initially appear.

### Result 1 — H4 / #11: distinguish eliminating a buffer from eliminating the work that produced it

The register's fused-front-end proposal P78 aims to reduce memory use by combining MTF output and T-vector construction.

There are three distinct quantities here:

\\[ \text{bytes allocated},\quad \text{bytes written},\quad \text{dependent work performed}. \\]

An in-place or fused builder can improve the first two without changing the third. Conversely, a run-native representation can eliminate an entire intermediate stream, but only if its builder consumes the compressed runs directly.

This lets us separate two mechanisms previously grouped under fusion:

- P78: avoid or reuse materialization of L while constructing the existing T-vector. The benefit is primarily memory traffic and temporary RSS.
- N3: construct a different navigation representation directly from L-runs. This may eliminate both L materialization and the old T-vector's per-row scatter, but substitutes interval construction work.

They should not share an estimated speedup.

For a block of \\(n=900{,}000\\), eliminating the L array can save at most 900 KB of live L storage per simultaneously active block, before considering allocation reuse, overlaps and any larger buffers that become unnecessary. It doesn't automatically save 900 KB of peak RSS if the buffer has already been recycled or the peak occurs in another phase.

Nor can an arbitrary in-place T-vector construction safely overwrite unread L bytes: the counting/scatter destinations are not generally in source order. The implementation must prove an aliasing schedule or retain another source of the overwritten information.

Disposition of #11: the storage mechanism is clear, but the proposal remains open as an implementation and lifetime-analysis question. It is not an untested universal front-end speedup.

### Result 2 — E9 / #21: output-buffer reuse cannot improve the hot table's cache footprint

The register asks whether placing walk output in L's freed buffer could improve memory pressure.

There is a straightforward limit.

Once the T-vector has been built, the walk accesses the same T-vector entries regardless of where its output bytes are stored.

Therefore, output-buffer reuse cannot reduce the number of distinct T-vector cache lines or the number of successor transitions required.

It can reduce memory allocation or total live pages, and perhaps improve system-wide pressure when several blocks are active. It could also introduce different write/read interference.

But this mechanism does not shrink the walk's own navigation working set.

That disposes of the stronger version of P87's hypothesis. The weaker RSS/contention hypothesis remains, and it needs a measurement rather than more mathematical speculation.

### Result 3 — E7 / #19: twenty bits suffice for a 900 KB row index, but not for the decoder's complete operation

A standard maximum-size bzip2 block contains at most 900,000 rows:

\\[ 900{,}000 < 2^{20}=1{,}048{,}576. \\]

Thus a Ψ successor index needs only 20 bits. That's an exact numerical fact, independent of the corpus.

The ideal raw footprint is:

\\[ 20n/8=2.25\ \text{MB} \\]

versus 3.6 MB for a 32-bit table.

So the theoretical raw saving is 1.35 MB per block, or 37.5%.

But this is not the same as saying that the current 32-bit entry contains 12 useless bits. Some layouts use those bits to encode output-symbol or segment information, and extracting packed 20-bit successors can introduce extra instructions and cross-word accesses.

The existing #112 measurements already demonstrate why compact links can lose despite a smaller footprint.

Disposition: the bit-width question is answered. The E7 question—whether a particular 20-bit layout improves absolute throughput under high concurrency—is still an architecture-specific measurement. We should not keep rereading bit-packing literature to answer it.

### Result 4 — scanner ceilings must distinguish total CPU work from pipeline throughput

This is an important correction to how we interpret the front-end ceiling.

The portfolio lists the scanner at around 9.5% of a one-core decoder's cost in one stage table.

If a stage contributes fraction \\(f\\) of total serial work, completely eliminating it gives the Amdahl bound

\\[ S\_{\max}=\frac1{1-f}. \\]

For \\(f=0.095\\):

\\[ S\_{\max}\approx1.105. \\]

But the parallel mibz2 decoder is a pipeline with a serial scanner feeding concurrent workers. Its steady-state throughput is approximately bounded by

\\[ \boxed{ T\_{\mathrm{pipeline}} \le\min(T\_{\mathrm{scanner}},T\_{\mathrm{workers}}) } \\]

with finite-buffer and synchronization effects.

Those are different ceilings.

A scanner consuming only 9.5% of aggregate CPU work can nevertheless become the wall-clock bottleneck after the workers become sufficiently fast.

Conversely, accelerating the scanner on a four-core run may produce almost no improvement while the workers remain slower than the scanner.

That explains why #10, parallel scanning, is conditional rather than generally low-value or generally mandatory.

It is a dependency of future high-throughput designs, not a present-day CPU-work breakthrough on its own.

### Result 5 — the kernel-to-decoder gap is not an Amdahl fraction

P134 compares approximately 1,707 MB/s for a pinned kernel-plus-MTF measurement with 1,256 MB/s for the full decoder in another execution environment.

It's tempting to infer:

\\[ 1-\frac{1256}{1707}\approx26.4\\% \\]

as the cost of scanner, CRC, RLE1 and orchestration.

That conclusion is invalid.

The two figures differ in boundary and placement. They may also differ in working-set residency, scheduling, queueing and overlap.

Even for identical workloads, subtracting throughputs does not yield a component's CPU contribution.

The 26.4% figure is an observed throughput gap, not a measured overhead fraction.

That resolves the methodological part of P134. Its actual component attribution remains open and should be based on synchronized phase measurements from the same execution, rather than a subtraction of independent benchmark results.

### Result 6 — P75 and P74: the MTF distribution already rules out one broad optimization strategy

The register gives Wikidata's explicit MTF rank distribution:

- 15.2% of moves have rank at least 16.
- 1.52% have rank at least 32.
- 0.23% have rank at least 64.

The shipped implementation already handles low ranks with a short AVX2 update.

Consequently, a new method that improves only ranks at least 32 can affect no more than 1.52% of explicit MTF list updates.

That isn't a bound on its possible speedup without knowing the conditional cost of those updates. A rare operation could still dominate if it were enormously expensive.

But the current AVX2 implementation limits how much of the ordinary shifting work remains on that path.

This strongly supports the register's conditional disposition of Seward's sliding-block MTF, P75, for Wikidata.

It also explains why P74 is more relevant: the store-forwarding dependency occurs on every explicit rank update, including the overwhelmingly common low ranks.

Thus the important remaining distinction is not small-rank versus large-rank arithmetic. It's whether multiple successive state updates can be executed with less serialization.

That is an existing lead, not a new one.

## Updated question dispositions

| Existing item                  | What we can now settle                                               | What remains genuinely open                             |
| ------------------------------ | -------------------------------------------------------------------- | ------------------------------------------------------- |
| #11 / P78 — fused front end    | Buffer reuse and work elimination are distinct mechanisms            | Alias safety, RSS and total-work benefit                |
| #15 / E3 — libsais attribution | Kernel speed and decoder speed cannot be subtracted into stage costs | Same-boundary profiling                                 |
| #19 / E7 — 20-bit Ψ            | Twenty bits suffice for all valid -9 row indices                     | Layout-specific full-decoder performance                |
| #21 / E9 — L-buffer reuse      | Cannot shrink the navigation table's hot footprint                   | RSS and multicore contention benefit                    |
| P72 / #10 — scanner            | Pipeline ceiling is distinct from one-core Amdahl share              | Exact binding point under faster workers                |
| P134 — kernel/decoder gap      | Throughput subtraction does not attribute component CPU costs        | Paired phase accounting                                 |
| P75 — sliding-block MTF        | High-rank operations are rare on Wikidata                            | Whether their conditional cost justifies specialization |
| P74 — MTF dependency           | Frequent low-rank state updates are the relevant opportunity         | Actual removable cycles                                 |

## What this round changes

We have ruled out several misleading routes to a breakthrough:

1. Treating a smaller temporary buffer as if it removed decoding work.
2. Treating 20-bit sufficiency as evidence that packed successor access will be faster.
3. Treating a scanner's CPU share as its pipeline throughput share.
4. Treating differences between unrelated benchmark throughputs as additive component costs.

Those distinctions prevent us from spending experiments on incorrect premises.

The original register still has open questions, particularly the literature items #28–#33, which haven't received the primary-source scrutiny they deserve, along with several measurement- and architecture-specific questions. I would turn to those next rather than return prematurely to N1 implementation.

Research burn-down continues. The 233-block census passing the space gate is a meaningful empirical advance, but it does not change our obligation to work through the rest of the inventory.
````

## Round 14, as received

````text
This is the first substantial negative economic result for N1, and it's valuable.

The 233-block census establishes that the Ψ⁸ representation fits the storage budget on Wikidata. The new construction model reveals that a compact final table is not necessarily cheap to build.

The median construction traffic is approximately:

\\[ \frac{23}{5}=4.6 \\]

times the modeled traffic of the existing T-vector build.

That changes the interpretation of Gate A:

- Representation feasibility: passed for all 233 Wikidata blocks.
- Corpus portability: failed the storage gate on random, records and the crate's own sources.
- Construction economy: concerning; the proposed builder moves substantially more data.
- Decoding economy: still potentially attractive, but unmeasured.

It also confirms that any viable N1 decoder needs a per-block selection rule. That's not a minor edge case: the negative controls demonstrate fundamentally different structural regimes.

I would preserve the 23-versus-5 bytes-per-output-byte comparison prominently in the portfolio, while keeping it labeled as a traffic model rather than measured memory bandwidth or cycles.

## Burn-down round 14: the fallback decision can be proved before constructing Ψ⁸

There is a useful deduction from the negative controls.

Every level-eight partition refines the level-one partition. Therefore:

\\[ \boxed{m_8\ge m_1=r} \\]

where \\(r\\) is the number of L-runs.

For the proposed 12-byte Ψ⁸ descriptors and a directory of \\(D\\) bytes:

\\[ S_8\ge12r+D. \\]

To fit within the 3.6 MB T-vector budget, a necessary condition is:

\\[ \boxed{r\le\frac{3{,}600{,}000-D}{12}}. \\]

Using the approximately 225 KB directory from the portfolio gives:

\\[ \boxed{r\lesssim281{,}250.} \\]

This immediately excludes many high-run-count blocks without constructing a single composed interval.

For `records` and `random`, where \\(r\approx n\\), the necessary condition fails decisively.

For the crate's source code, \\(r=136{,}000\\) passes this necessary condition, but its measured 4.45 MB representation fails the actual budget.

That's the important distinction.

Structural fallback classification

| Condition            | Disposition                                   |
| -------------------- | --------------------------------------------- |
| \\(12r+D>4n\\)       | Definitely reject Ψ⁸ on the storage criterion |
| \\(12m_8+D\le4n\\)   | Storage criterion passes                      |
| Between those bounds | Needs additional structural information       |

This closes the simplest version of the block-selection problem mathematically. It does not yet give a sufficiently selective classifier for intermediate blocks.

### A better early rejection rule

Because the level counts are monotone,

\\[ m_1\le m_2\le m_4\le m_8, \\]

we can reject as soon as any intermediate level exceeds the final descriptor budget.

That means the builder need not complete all three doubling rounds for an obviously unsuitable block.

However, this only saves future construction work. If the decision happens after materializing L and several intermediate descriptor arrays, some of the potential savings have already been lost.

So the exact timing of the fallback decision now becomes important to the run-native pipeline.

## A second important consequence: N3 cannot assume the fallback is free

N3 proposes avoiding L materialization altogether by producing navigation records directly from the MTF/RLE2 run stream.

But if a block fails the Ψ⁸ gate, the decoder must instead build the ordinary T-vector.

That creates an unavoidable architectural choice.

Option A: retain the conventional L buffer until the block's representation is chosen. This preserves cheap fallback but forfeits some of N3's memory and traffic savings.

Option B: buffer only the compact run stream and reconstruct L for fallback. This may save memory on repetitive blocks, but high-run-count inputs can produce nearly one run per byte. The fallback then pays for run buffering and L reconstruction.

Option C: predict the representation from partial run statistics. This could reduce preparation work, but a partial prefix does not necessarily determine the final run count or the final multi-step record count. A conservative early rejection is safe; an early acceptance needs a defensible bound.

That narrows two existing questions together: #11 (front-end fusion) and #53 (direct compact construction).

A run-native decoder cannot be evaluated as if all blocks take the compact path. The negative controls establish why.

## The new cost equation

For a mixed corpus, the relevant CPU cost isn't simply the Ψ⁸ construction plus walk.

It is:

\\[ \begin{aligned} E[T]={}&p\\,T\_{\mathrm{compact}}\\\ &+(1-p)\\,T\_{\mathrm{fallback}}\\\ &+T\_{\mathrm{classification}}, \end{aligned} \\]

where \\(p\\) is the fraction of blocks that select the compact representation.

Here the fallback term must include any wasted partial compact construction.

This is a model, not a measured prediction. It clarifies why reporting a Wikidata-only pass rate of 233/233 is insufficient to establish general decoder efficiency.

For mibz2's primary Wikidata objective, that pass rate is encouraging. For the general `.bz2` decoder, the fallback cost is a first-class acceptance criterion.

## The next research gate has become sharper

The construction model now moves about 23 bytes per output byte, compared with approximately 5 for the shipped builder.

That creates a useful performance test:

Can the Ψ⁸ walk save enough expensive dependent memory operations to pay for roughly 18 additional bytes of modeled construction traffic per output byte?

This is not reducible to bandwidth alone. The current T-vector build includes irregular writes, while the new builder uses more structured passes. Twenty-three mostly cache-resident bytes may cost less than five poorly localized bytes—or more, depending on the exact operations and architecture.

Therefore the byte count is strong negative evidence against assuming a cheap builder, but not yet a proof of defeat.

### What I would update in the burn-down register

| Existing research item          | New disposition                                                                                 |
| ------------------------------- | ----------------------------------------------------------------------------------------------- |
| N1 space feasibility, Wikidata  | Measured pass: 233/233 full blocks                                                              |
| N1 space feasibility, controls  | Measured failure on random, records and source code                                             |
| N1 construction traffic         | Modeled adverse: \~23 B/B vs \~5 B/B                                                            |
| N1 storage fallback             | Necessary rejection condition derived directly from \\(r\\)                                     |
| #53 direct compact construction | Algorithm exists; performance remains open, with adverse traffic evidence                       |
| #11 / N3 front-end fusion       | Fallback costs and selection timing now explicit                                                |
| #45 per-block format selection  | Mathematically necessary for diverse corpora; optimal policy remains open                       |
| #58 distribution sensitivity    | Strong empirical negative controls now available; encoding-specific worst cases remain separate |

No numerical speedup claim is justified yet.

The result is a useful challenge to the leading hypothesis: its representation is small, but its construction is relatively heavy.

I would keep N1 scientifically alive while lowering confidence in its original 1.8–2.5× full-decoder projection until the construction model is reconciled with actual work.

And I'd continue the remaining research-question burn-down in parallel, particularly the unexamined primary literature and the already-recorded build/front-end alternatives. We shouldn't let one attractive candidate consume the entire investigation.
````
