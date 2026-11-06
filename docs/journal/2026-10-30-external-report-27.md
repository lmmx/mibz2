# External research report 27: research burn-down rounds 35–37, P74 at the library-call boundary (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 26 (`2026-10-30-external-report-26.md`).

**What it contains:**
- **Round 35:** a live YMM head can't be assumed to survive `memset` or `memmove`, and
  `vzeroupper` is not itself a spill.
  - The cost of preserving the head scales with the number of calls, not of tokens.
  - The net condition: Nd > N_c·p + C_other.
  - The request: count calls per 1,000 explicit tokens and per decoded MB, split by
    operation.
- **Round 36:** the head needs saving only at call boundaries, never per token.
  - R (RUNA/RUNB sequences) and F (ranks ≥ 32) can be counted exactly from
    `Symbols.syms`.
  - The condition: d > f·c + c_other.
- **Round 37:** checkpoint splitting gives R ≤ W ≤ R + K, with K = 54 for a 900,000-byte
  block. So under the save-and-restore model, splitting adds at most 3,456 B a block
  (0.00384 B/B).

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §3.2e(i).

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols lost) and is kept as received.

## Round 35, as received

````text
Burn-down round 35: P74’s hidden cost is at the library-call boundary

Claude’s disassembly finding is important, but it requires a precise interpretation.

The shipped 0.2.1 loop calls memset for RLE2 expansion and memmove for higher-rank MTF updates, with vzeroupper before those calls.

That does not prove the shipped loop spills its MTF head, nor that P74 would spill it. The shipped implementation already keeps the list in memory.

What it establishes is that P74 introduces a new requirement: preserving a live YMM register across calls to library functions that are free to overwrite vector registers.

1. The exact ABI problem

On the ordinary x86-64 calling conventions used by Linux and Windows, a full 256-bit YMM register is not callee-saved.

Consequently, a P74 implementation that keeps its 32-byte head in a YMM register cannot assume that the register survives a call to memset or memmove.

There are three basic solutions:

* Save the vector state before the call and restore it afterward.
* Avoid the call by inlining or specializing the operation.
* Reorganize the loop so the register state is not live across the call.

The first is simplest, but introduces precisely the memory transfers P74 seeks to avoid.

The second may increase code size or replace highly optimized library routines with slower ones.

The third is not always possible, since the MTF state is needed after the run or fallback completes.

This gives us a real structural cost, not merely a speculative compiler-spill concern.

2. The key distinction: RLE2 calls versus high-rank calls

The two call types occur at very different rates.

For an RLE2 run, the MTF list does not change. The decoder can preserve the current register state while emitting repeated copies of its front symbol.

For a high-rank MTF update, the list changes, so the fallback must update both the register-held head and the memory-resident tail.

The relevant frequency is therefore:

\boxed{N_{\mathrm{calls}}=N_{\mathrm{RLE2\ runs}}+N_{\mathrm{high\ ranks}}}

for a straightforward implementation retaining the calls.

That is not the same as the number of decoded bytes or the total number of Huffman symbols.

If each call forces a 32-byte save and reload, the additional logical state traffic is:

\boxed{64N_{\mathrm{calls}}\text{ bytes}.}

This is a conservative accounting for explicit save/restore operations, not a claim about actual machine-code spills.

It can be converted to bytes per output byte using the existing token census.

3. A better structural solution for RLE2

The RLE2 operation does not need to modify the YMM state.

The simplest way to avoid preserving it across a memset call may be to make the compiler recognize that the register state remains live and save it efficiently—or to keep a synchronized memory shadow of the head.

But a memory shadow has a drawback: updating it after every explicit token recreates the memory-state dependency we intended to remove.

A better distinction is between persistent shadowing and call-boundary preservation.

P74 needs at most one state save before a clobbering call and one reload afterward. It does not need to write the head back on every ordinary token.

That makes the added work proportional to the number of calls, rather than to the number of MTF updates.

For example, if calls occur once per hundred explicit tokens, the amortized cost is much smaller than an unconditional store/reload every token. The actual call frequency still needs to be measured.

4. vzeroupper is not itself the spill

vzeroupper clears the upper portions of YMM registers. It does not write their contents to memory.

If the P74 state were live in such a register, executing vzeroupper would destroy part of it unless the state had been preserved or recomputed.

But the compiler is responsible for ensuring that live values survive correctly. The presence of vzeroupper in the shipped binary is evidence about the existing call boundary, not proof that a hypothetical P74 compiler output would be incorrect or inefficient.

This is a useful distinction for the register.

5. The actual research decision

We can now sharpen the P74 performance model.

Let:

* : number of explicit MTF updates.
* : number of calls requiring head preservation.
* : average cycle saving per explicit update.
* : average additional cycle cost per preserved call.

The net advantage requires:

\boxed{Nd>N_cp+C_{\mathrm{other}}}

where covers masks, branches and other additional operations not already included in .

Equivalently,

\boxed{d>\frac{N_c}{N}p+\frac{C_{\mathrm{other}}}{N}.}

This is more useful than treating every vzeroupper as a fatal problem—or assuming that a rare high-rank fallback makes call overhead negligible.

Round 35 disposition

P74 question	Status
Can a live YMM head be assumed to survive library calls?	No
Does vzeroupper prove a memory spill?	No
Is call-boundary preservation mathematically necessary if state stays in YMM?	Yes, unless the call boundary is removed or state is otherwise preserved
Must the head be stored after every explicit token?	No
Is call-boundary overhead necessarily prohibitive?	No; depends on call frequency and cost
Is P74’s CPU benefit established?	Not yet

What I’d ask Claude to extract next

The most valuable read-only result is now the call density on the real blocks, split by operation.

Count RLE2 runs that would invoke memset, and explicit ranks at least 32 that would invoke memmove. Express each as calls per 1,000 explicit tokens and calls per decoded megabyte.

Then distinguish calls whose compiled code must preserve the YMM head from operations that can be performed inline.

This would give us a quantitative upper bound on P74’s state-preservation overhead without compiling a candidate or running the i9.

Bottom line: The disassembly has not killed P74. It has identified a real ABI constraint that earlier shuffle-only models ignored. We can now price that constraint instead of speculating about it.

The next burn-down step is to determine whether preserving the YMM head at these call boundaries consumes a meaningful fraction of the benefit available from the one-shuffle common path.
````

## Round 36, as received

````text
Burn-down round 36: can P74 avoid the YMM save/restore penalty?

The next question is whether the library-call overhead identified in round 35 is an unavoidable cost of P74, or merely a consequence of retaining the shipped loop’s implementation.

There is an important distinction:

A live YMM register must be preserved across an ordinary library call, but that does not mean it must be saved and restored on every MTF token—or even on every RLE2 run.

We can narrow the problem further by separating the two call sites.

RLE2 expansion

An RLE2 run emits copies of the current front symbol. The MTF list does not change.

If the compiler retains the memset call, P74 may require saving and restoring its register state around that call.

But the selected output byte is already available independently of the list’s later state. This creates another implementation option: use a memory-resident checkpoint of the head only at the call boundary, rather than maintaining a memory shadow after every token.

High-rank fallback

The memmove call is different: it changes the memory-resident tail and must preserve the semantic ordering of the entire MTF list.

P74’s correct fallback already specifies exactly which tail bytes move, and the head update can be computed separately. The register state must survive the call, but the tail movement doesn’t intrinsically need the register contents once the displaced head byte has been extracted.

This means the head can be saved, the tail modified, and the head restored or reconstructed. The choice affects cycles and register pressure, not correctness.

The sharper cost model

Let be the number of explicit MTF tokens, the number of nonempty RLE2 runs, and the number of explicit ranks at least 32.

If P74 retains both library calls and saves/restores its 32-byte head at each call, the additional state movement is bounded by

\boxed{64(R+F)\text{ logical bytes per block}.}

But that is not a bound on the total overhead. Calls also have instruction, register-allocation and scheduling costs, even when their data is cache-resident.

We can obtain and exactly from the existing Symbols.syms, without MTF reconstruction:

* is the number of explicit symbols with .
* is the number of maximal nonempty RUNA/RUNB sequences that the decoder expands.

This answers the call-frequency part of P74 using the same read-only source already available for the rank census. It does not require another corpus decompression pass.

A useful break-even test

The Wikidata census reports approximately 0.061 explicit MTF tokens per output byte.

If P74 saves cycles per explicit token, its gross saving is approximately

0.061d\quad\text{cycles/output byte}.

Let be the fraction of explicit-token count represented by potential state-preserving calls, and their average additional preservation cost.

Then P74’s advantage requires

\boxed{d>fc+c_{\mathrm{other}}}

where is other incremental cost per explicit token, including mask handling and dispatch.

This equation identifies the decisive quantity: not the raw number of library calls, but their cost relative to explicit MTF updates.

There is also a hard limit to our read-only analysis. We can count calls and model save/restore bytes, but we cannot establish the compiler’s actual preservation strategy for a P74 kernel that has not been compiled.

Round 36 disposition

Question	Finding
Must P74 preserve its head across external calls?	Yes, when live
Must every explicit token spill its head?	No
Can RLE2 and high-rank call frequencies be calculated exactly?	Yes, from the existing token stream
Is full-head shadowing after each token necessary?	No
Can we prove call preservation is affordable from counts alone?	No
Is more SIMD algebra necessary?	Not for this issue

Research conclusion: P74’s ABI problem has become a measurable amortization problem, rather than an unresolved correctness or representation problem.

The one remaining read-only quantity worth obtaining is , alongside the already measured . Then we should stop analyzing hypothetical compiler spills and treat actual compiled code as the evidence boundary.

That keeps the burn-down disciplined: another uncertainty is narrowed to a specific observation, without prematurely starting an implementation campaign.
````

## Round 37, as received

````text
Burn-down round 37: the checkpoint call penalty has a strict additive bound

I found a useful result in the actual mtf_checkpoints_avx2 source. It strengthens rounds 35–36 and corrects a simplification in our earlier cost model.

Checkpoint splitting can increase the number of RLE2 fill operations, but by no more than the number of checkpoint boundaries.

That lets us bound the extra register-preservation burden without running anything.

1. What the shipped code actually does

When an RLE2 run crosses a 16 KiB checkpoint, the decoder enters a loop that:

* Emits the portion ending at the checkpoint.
* Updates the symbol count.
* Records the checkpoint.
* Continues with the remainder of the run.

The relevant code is:

while n + left >= next {
    let part = next - n;
    counts[c as usize] += part as u32;
    std::ptr::write_bytes(out.add(n), c, part);
    (n, left) = (next, left - part);
    checkpoints.push(*counts);
    next += CHECKPOINT;
}

Every additional fill introduced by checkpoint splitting corresponds to a checkpoint boundary within the run’s output interval.

Distinct RLE2 runs occupy disjoint output intervals. Therefore, no checkpoint boundary can cause an additional split in two different runs.

Let:

* be the number of nonempty RLE2 runs.
* be the number of 16 KiB checkpoint boundaries encountered.
* be the number of fill operations issued by the checkpoint-aware kernel.

Then, for valid completed blocks,

\boxed{R\le W\le R+K.}

This counts source-level fill operations. A zero-length remainder can still produce a source-level write_bytes invocation; whether that becomes a library call depends on code generation.

2. The bound is small relative to block size

For a 900,000-byte block with 16,384-byte checkpoints:

K=\left\lfloor\frac{900000}{16384}\right\rfloor=54.

Therefore:

\boxed{W\le R+54.}

Whatever the RLE2 distribution, checkpoint splitting adds at most 54 fill operations per full block.

That is a surprisingly strong restriction.

The call-preservation model from round 35 should therefore distinguish ordinary RLE2 run frequency from the bounded checkpoint-splitting overhead, rather than treating every checkpoint interaction as a potentially large additional source of calls.

3. Updated P74 preservation model

Let be the number of high-rank explicit tokens requiring fallback.

If all source-level fill and fallback operations cause calls that require preserving the full YMM head, the maximum number of preservation events is:

N_c\le R+F+K.

Under a simple 32-byte save plus 32-byte restore model:

\boxed{
B_{\mathrm{preserve}}\le64(R+F+K).
}

For a 900,000-byte block:

B_{\mathrm{preserve}}\le64(R+F)+3456\text{ bytes}.

So the additional logical head traffic attributable specifically to checkpoint splitting is bounded by just 3,456 bytes per full block.

That is approximately:

0.00384\text{ bytes per output byte}.

This is a traffic bound under the stated save/restore model—not a bound on actual call cycles.

4. Two qualifications matter

First, std::ptr::write_bytes and std::ptr::copy do not necessarily produce an external library call in every compiled context. The shipped wheel provides evidence of actual calls, but the precise invocation count must come from the binary or a dynamic trace, not simply the number of Rust expressions.

Second, the checkpoint code may issue a zero-length final fill when a run ends exactly on a checkpoint boundary. That operation is semantically harmless, but it matters when comparing source-level invocations with emitted calls.

Neither qualification changes the bound on meaningful nonempty fill segments.

5. What this closes

P74 call-cost question	Disposition
Can checkpoint splitting multiply fill calls without bound?	No
Maximum additional fills on a 900 KB block	54
Maximum extra 32-byte save/restore traffic from those fills	3,456 bytes, under the model
Does a large RLE2 run necessarily cause many calls?	Only if it crosses multiple checkpoints
Is ordinary RLE2 run frequency now known?	Not yet
Does the model prove P74 wins?	No

The most important consequence is that checkpoint splitting itself is not a credible source of large P74 register-preservation traffic on 900 KB blocks. Ordinary RLE2 runs and high-rank fallback calls remain the relevant populations.

Relay to Claude

Research round 37 — P74 checkpoint call bound

Read-only inspection of mtf_checkpoints_avx2 establishes an additive limit on RLE2 fill operations introduced by checkpoint splitting.

Let R be the number of nonempty RLE2 runs, K the number of 16 KiB checkpoint boundaries, and W the number of source-level fill operations. Then R ≤ W ≤ R+K, because each additional fill corresponds to a distinct checkpoint boundary within a run.

For n=900,000 and CHECKPOINT=16,384, K=54. Under an assumed 32-byte YMM save and restore per call, checkpoint splitting adds at most 64×54=3,456 logical bytes per block, or 0.00384 B/B.

This is not a cycle bound. Intrinsic calls may be inlined or lowered differently; zero-length final writes may occur at exact boundaries. Disassembly or dynamic counts are needed to identify actual library calls.

Record this under P74’s call-boundary economics. The key unresolved frequency is ordinary RLE2 runs per explicit token, not checkpoint-splitting amplification.

Round 37 closes another structural uncertainty: checkpoint boundaries contribute only a tightly bounded additive cost. They cannot, by themselves, explain a large regression in a register-held MTF design.
````
