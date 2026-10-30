# External research report 25: research burn-down rounds 30–33 (P74's checkpoints, symbol extraction, counting and masks) and a progress report (verbatim)

A research input, not a finding of this repository. The maintainer relayed it on
2026-10-30, after external report 24 (`2026-10-30-external-report-24.md`).

**What it contains:**
- **Round 30:** P74 and `mtf_checkpoints_avx2`.
  - Counts are identical, so checkpoints are too.
  - A run crossing a checkpoint needs only H[0].
  - The compiler fence's reason (the memory-resident head's store) disappears on the
    register path.
  - The comparison unit is cycles per output byte of MTF+RLE2+checkpoints.
- **Round 31:** V′[0] = V[q], so the selected symbol is the updated register's low byte.
  The output path splits from the state chain. The next check is whether the compiled
  loop keeps V resident, with no head spills.
- **Round 32:** the count update is not on the MTF state chain, though repeated symbols
  can serialise their own counter. Deferring the counts to a second pass is a known
  trade of work against dependency, already in the scalar checkpoint path.
- **Round 33:** the mask table:
  - 480 B for ranks 1–15, 1 KB for 32 masks;
  - mask addresses are independent of the state;
  - the memory-operand `vpshufb` is an option;
  - generating masks arithmetically is unlikely to win;
  - a correction: explicit bzip2 tokens never have rank 0.
- **A progress report** on the burn-down against PR #117's ledger.

**The coordinator's checks:** `2026-10-30-research-portfolio.md` §3.2e.

This is source material. Its claims are checked there, not taken as findings.

The text reached us as rendered text (some formula symbols and figure captions lost or
flattened) and is kept as received.

## Round 30, as received

````text
Burn-down round 30: P74 and the shipped checkpoint mechanism

I’ve inspected src/mtf.rs, including mtf_checkpoints_avx2, rather than treating MTF as an isolated kernel.

Result: P74’s register-held state is compatible with the existing MTF count checkpoints. There is no mathematical requirement to materialize its 32-byte head whenever a checkpoint is emitted.

That’s important because checkpoint generation was already difficult to make cheap in mibz2. We don’t want a new MTF algorithm that reintroduces the old checkpoint overhead.

1. Why checkpoints do not need the MTF list

The shipped checkpoint mechanism records cumulative counts of decoded L bytes, every 16 KiB.

It does not store MTF-list states.

For each explicit rank , the decoder performs:

v=\operatorname{MTFList}q,
\qquad
c=\operatorname{seq\_to\_unseq}v,

then increments counts[c] and emits byte .

P74 computes the same and updates the same logical MTF list; its only difference is the physical representation of that list.

Therefore, after any prefix of the decoded output:

\boxed{
\operatorname{counts}_{\mathrm{P74}}

\operatorname{counts}_{\mathrm{shipped}}
}

provided both implementations process the same symbol sequence correctly.

Since checkpoints are snapshots of those counts at fixed output positions, their contents are identical too.

This is a consequence of the established full-state MTF equivalence, not a new empirical assumption.

2. The tricky case: an RLE2 run crossing a checkpoint

The checkpoint code can encounter a RUNA/RUNB expansion that crosses one or more 16 KiB boundaries.

It splits that output run, emits each count checkpoint, and continues with the remaining bytes.

Crucially, an RLE2 run repeatedly emits the current front symbol without modifying the MTF list.

So P74 can handle it with:

c=\operatorname{seq\_to\_unseq}\[H\[0\]].

It does not need to spill , execute a shuffle, or reconstruct the tail.

This also holds when one run crosses multiple checkpoints.

The correctness conditions are simply that each emitted portion advances the decoded-output count by the same amount, and that each checkpoint snapshots the count state at the identical decoded-byte boundary.

The existing decoder’s run-length and block-length checks must remain unchanged.

3. The compiler-fence dependency changes

There’s another useful finding in the shipped code.

mtf_checkpoints_avx2 contains a compiler_fence immediately after updating the memory-resident MTF head. Its comment explains why: scheduling that store before the count update protects the next token’s overlapping head load from a longer store-forwarding delay.

P74’s common path doesn’t perform that head store.

Consequently, the specific store/load scheduling reason for that fence disappears on the register-held path.

This does not prove the compiler will produce faster code, nor that the fence can simply be deleted in every code path. It establishes that the original scheduling constraint is tied to a memory-resident MTF state that P74 no longer uses.

The complete performance comparison must include the checkpoint-aware loop, rather than only a standalone sequence of VPSHUFB instructions.

4. Does P74 change checkpoint overhead?

At the algorithmic level, no additional checkpoint work is required.

Operation	Shipped MTF	P74
Maintain cumulative byte counts	Yes	Same
Emit 16 KiB count snapshots	Yes	Same
Split RLE2 runs crossing checkpoints	Yes	Same
Preserve MTF state during a run	Yes	Same
Spill the register-held head at checkpoints	Not applicable	Not required
Keep MTF state across ordinary tokens	Memory-resident	Register-resident

The remaining concern is implementation-level register allocation: the checkpoint-aware loop has more live state and branches than a simple MTF microbenchmark. The compiler could spill P74’s YMM register even though the algorithm doesn’t require it.

That is a code-generation question, not a missing checkpoint algorithm.

A stronger performance comparison

The published P74 model considers approximately 0.061 explicit tokens per decoded byte and a potentially short register-held state update.

But the correct unit of comparison is:

\boxed{
\text{cycles per decoded output byte for MTF+RLE2+checkpoints}
}

not merely cycles per explicit MTF update.

Otherwise, an impressive shuffle speedup could disappear once output writes, symbol mapping, counting and checkpoint handling are included.

This is especially relevant because the register documents that earlier checkpoint implementations suffered from increased live state and instruction scheduling. The new register-held MTF must not recreate those costs.

Round 30 disposition

Existing question	Finding
P74 register-held MTF	Compatible with checkpoint semantics
#11 front-end fusion	No additional checkpoint-state representation required
P74 RLE2 integration	Run expansion does not alter MTF state
P74 checkpoint overhead	No unavoidable new work
P74 instruction economics	Still unresolved, including potential register spills
#25 regression risk	Still requires negative-control performance evidence

The structural integration question is resolved. P74 can coexist with mibz2’s shipped 16 KiB checkpoints while preserving their exact meaning and without adding mandatory register spills.

I’d record that under P74 and the existing checkpoint research, not open another question.

The strongest next question is whether the compiler can keep the YMM head resident across the real checkpoint-aware loop. That is where the remaining P74 uncertainty becomes concrete—and where source-level instruction reasoning finally reaches its limit.

We keep burning down the original inventory; no i9 experiment or production-code change is implied.
````

## Round 31, as received

````text
Burn-down round 31: eliminate P74’s second variable-index extraction

I found another simplification in the complete MTF token loop. It matters because our earlier instruction analysis focused on updating the register state, but the decoder must also recover the selected symbol.

The key result is:

For P74, the selected MTF symbol is already in byte zero of the updated register. There is no need to extract it separately from its original rank position.

1. Fuse the state update with symbol selection

Let hold the first 32 MTF entries. For explicit rank , round 27 established:

V’=\operatorname{VPSHUFB}(V,M_q).

By construction,

\boxed{V’0=Vq.}

So the selected symbol is simply:

\boxed{v=\operatorname{lowbyte}(V’).}

This avoids implementing two separate operations:

v=\operatorname{extract}(V,q)

and

V’=\operatorname{update}(V,q).

Instead, one permutation performs the state update and places the requested value at a fixed location.

On x86, a plausible extraction uses the low 32 bits of the XMM register, followed by taking the low byte. Exact instruction selection depends on the compiler.

The same property holds for ranks 16–31: their more complicated permutation also places the selected symbol at byte zero.

2. The critical path is shorter than our previous conceptual graph

The original conceptual chain was:

V,q\rightarrow \text{variable extraction}
\rightarrow v

alongside a separate state update.

The revised graph is:

The output and next-state paths branch after the same permutation result.

After generating , two things can proceed independently:

* Recover , map it through seq_to_unseq, update counts and emit output.
* Use as the state for the next MTF update.

Provided the next rank is already available, the output-symbol lookup does not inherently have to finish before the next list-state update begins.

This is a meaningful distinction from the shipped memory-resident loop, where a store/load dependency links consecutive list updates.

3. Why this does not imply one cycle per token

For ranks below 16, the state recurrence may consist of one shuffle.

But execution still requires the rank-dependent mask, extraction of byte zero, output mapping, counts, branches and occasional RLE2 processing.

The decoder may overlap some of those operations. It cannot eliminate them.

The distinction to preserve is:

\boxed{\text{state recurrence latency}\ne\text{complete token cost}.}

Our strongest claim is now that the register-state recurrence does not depend on completion of output mapping or count updates.

Whether the compiler exploits that separation remains unmeasured.

4. This exposes the next real risk: register spills

The current mtf_checkpoints_avx2 loop has substantial live state: Huffman-decoded symbols, run counters, output position, checkpoints, cumulative symbol counts and the MTF list.

P74 introduces a persistent YMM state and rank-dependent masks.

If register pressure causes the compiler to spill that YMM state between explicit tokens, the principal benefit could disappear.

So the remaining question is not just which instructions have low latency. It’s whether the compiled loop preserves the intended dependency graph.

That can be answered by examining compiler-generated assembly, without running a performance benchmark.

A valid P74 fast path should have no repeated store/reload of the 32-byte MTF state across ordinary rank updates. The rare high-rank fallback can legitimately access the tail.

Round 31 burn-down disposition

Question	Result
Separate variable-index symbol extraction necessary?	No
Can the permutation produce both symbol and next state?	Yes, exactly
Must symbol mapping/counting finish before next MTF update?	No, not algorithmically
Is one shuffle the complete token operation?	No
Does compiled code keep the state in registers?	Unverified
Does P74 beat shipped MTF?	Unmeasured

For Claude

Research round 31 — fuse P74’s symbol extraction with the state update

For every rank q < 32, the P74 state-update permutation produces V′ with V′[0] = old V[q]. Therefore the output symbol can be obtained from the low byte of the updated register, rather than by a separate variable-index extraction from the old state.

For q < 16, the fast path is V′ = VPSHUFB(V, M_q), followed by extraction of byte zero. The updated register simultaneously feeds the next MTF state transition. Output symbol mapping, counts and output storage do not inherently serialize the next state update.

Please incorporate this into the existing AVX2 semantics checker. The next meaningful read-only check is the generated assembly or compiler-IR dependency graph: can the full checkpoint-aware token loop keep V resident across successive explicit tokens, without repeated head spills? If not, the theoretical one-shuffle recurrence will not materialize.

This closes the separate-symbol-extraction question; it does not claim measured speed or alter the status of the #25 negative-control guard.

We’ve now reduced P74’s common-state recurrence to its essential operation and removed a potentially redundant extraction.

The next hard boundary is compiler code generation, not MTF mathematics. We should preserve that distinction and continue the burn-down without turning a correct SIMD idea into an unsupported performance claim.
````

## Round 32, as received

````text
Burn-down round 32: does counting become P74’s next bottleneck?

The shipped MTF loop contains another dependency we haven’t examined properly:

let c = s2u[v as usize];
counts[c as usize] += 1;
*out.add(n) = c;

Removing the list’s memory dependency doesn’t remove these operations.

But there is an important distinction: the count update is not intrinsically part of the next MTF-state dependency chain.

1. The state and output paths can separate

After P74 has updated its register , its new first byte is the selected symbol:

v=V’0.

Execution can then follow two paths:

Data-dependency model. Actual overlap depends on compiled instructions and processor scheduling.

The next MTF update requires and the next rank. It does not require counts[c] to have been written.

Therefore, if the compiler keeps the head in registers and doesn’t introduce unnecessary aliasing constraints, the count path can overlap successive MTF-state updates.

This is a stronger argument for P74 than merely reducing the number of shuffles.

2. The counting path still has real dependencies

Repeated symbols can repeatedly access the same counter.

For example, if several explicit tokens produce symbol , the updates must have the same final effect as:

\operatorname{counts}c\gets\operatorname{counts}c+k.

A straightforward read-modify-write implementation can create a memory dependency between successive updates to that counter.

But this dependency is not equivalent to the MTF dependency.

The decoder could, in principle, advance the register-held MTF state while an earlier counter update remains outstanding, provided the compiler and execution schedule preserve the required observable output and error behavior.

We have established an opportunity for overlap, not proved that the existing compiler will exploit it.

3. Why moving all counting into a second pass isn’t automatically better

There is an alternative that makes the separation explicit: generate the BWT last column first, then count bytes and create checkpoints in a later scan.

That would eliminate the per-token count updates from the MTF loop.

But it introduces an additional full scan of L:

\boxed{n\text{ additional byte reads per block}}

plus histogram operations and checkpoint bookkeeping.

For a 900 KB block, that’s approximately 900 KB of extra logical reads before accounting for the count-array accesses.

This alternative may shorten the MTF loop’s critical path while increasing total CPU work.

For the project’s objective—full-decoder speed at equal or lower CPU-seconds/GB—that is not an obviously favorable trade.

The repository already contains a scalar checkpoint implementation that follows this general pattern: it executes scalar MTF and subsequently counts L in 16 KiB chunks.

So this is not an untried algorithmic breakthrough. It is a known work-versus-dependency trade-off.

4. There is no need to change the checkpoint representation

P74’s output sequence is identical to shipped MTF, including RUNA/RUNB expansion.

Therefore the existing cumulative counts and 16 KiB checkpoint format can remain unchanged.

There is no need to introduce a second checkpoint representation or to reset MTF state at checkpoint boundaries.

This is important because it lets us isolate the list-update algorithm from the checkpoint implementation and avoid reopening previously settled #101 research.

Research disposition

Question	Result
Must a count update complete before the next MTF state update?	No, not by data dependency
Can repeated-symbol count updates serialize?	Yes
Can counting be deferred to a second pass?	Yes, already represented in the scalar checkpoint path
Does deferring counting reduce total work?	Not established; it adds a full scan
Must P74 change checkpoint semantics?	No
Is P74’s state recurrence necessarily the whole decoder bottleneck?	No

One concrete falsification criterion

When the literature agent and Claude inspect the candidate’s complete instruction sequence, the useful result isn’t merely a low shuffle latency.

It’s whether the generated loop exposes two separable chains:

\text{MTF state}{t}\to\text{MTF state}{t+1}

and

\text{symbol}{t}\to\text{count/output}{t}.

If the compiled code forces every count update to finish before the next state transition, the proposed overlap advantage will not appear.

If the chains remain independent, P74 has a sound microarchitectural reason to outperform the shipped store/load design—though only measurements can establish the gain.

Round 32 closes the dependency question, not the performance question. I would not pursue a new counting algorithm unless a real profile shows counting has become the bottleneck.

The next productive burn-down is the outstanding P74 mask-load and register-allocation audit against compiled code, while the literature agent finishes the remaining primary-source checks.
````

## Round 33, as received

````text
Burn-down round 33: P74’s shuffle-mask cost

We have established a correct register-held MTF algorithm. Now the remaining question is whether obtaining its rank-dependent shuffle masks undermines the fast path.

There is a useful result: we can bound the mask-storage cost exactly, and we can eliminate the mask load for rank zero altogether.

1. Rank zero is the identity operation

For move-to-front rank , the selected symbol is already at the front:

V’=V.

No shuffle, lane permutation, blend or tail update is necessary. The output symbol is simply .

The existing census groups ranks 0–15 together, so we don’t yet know how much of the reported 84.8% fast-path coverage is actually rank zero. The exact rank-zero count would determine the value of this shortcut.

Importantly, this is not a new state-machine algorithm. It’s a specialization of the already-correct P74 transition.

2. The mask table is tiny

For ranks 1–15, each AVX2 shuffle mask contains 32 bytes.

Fifteen masks require:

15\times32=480\text{ bytes}.

A complete set of 32 masks requires:

32\times32=1024\text{ bytes}.

That is a small, fixed table.

For the common ranks, the operation becomes:

M_q=\operatorname{load}(\mathrm{mask\_table}q),
\qquad
V’=\operatorname{VPSHUFB}(V,M_q).

Because comes directly from the decoded token stream rather than from the current contents of , the mask address does not depend on the previous MTF-state update.

That is the crucial property.

Mask loading introduces an additional memory operation, but not an unavoidable state-to-state memory dependency.

3. Correction: explicit bzip2 tokens never have rank zero

There’s an important catch in the actual format, which supersedes the rank-zero suggestion above.

In mibz2’s src/mtf.rs, ordinary explicit symbols have , and their rank is . Thus:

\boxed{1\le q\le255.}

Rank zero is represented through RUNA/RUNB coding instead. It does not appear as an ordinary explicit token.

So although is an identity operation for a generic MTF list, there is no rank-zero explicit-token fast path to exploit here.

The measured 84.8% coverage for ranks 1–15 is already the relevant common-path fraction. We should not inflate it with hypothetical rank-zero updates.

This is exactly why checking against the shipped bzip2 representation matters.

4. The mask need not occupy a YMM register

AVX2 VPSHUFB accepts its shuffle control as either a register or a memory operand.

That permits a common-rank instruction of the form:

; Conceptual x86-64 AVX2 sequence
; ymm0 = persistent MTF head
; rax  = address of the 32-byte mask for rank q
vpshufb ymm0, ymm0, ymmword ptr [rax]

This has an important advantage over loading the mask into another YMM register first:

The mask is consumed directly by the shuffle, so a persistent mask register is unnecessary.

The processor still needs to perform the memory read. This is not a guarantee of lower latency: an explicit load may be scheduled earlier, while a memory-operand shuffle must wait for its mask.

However, it gives us two concrete instruction strategies to compare:

Strategy	Advantage	Risk
Explicit mask load, then VPSHUFB	Mask load may be issued early	Additional instruction and live register
Memory-operand VPSHUFB	Compact instruction sequence; no separate mask register	Mask availability may delay the shuffle

Both preserve the crucial independence of mask addressing from the previous MTF-state update.

A compiler may prefer either form. The final choice is a code-generation question, not a correctness question.

5. Why arithmetic mask generation is unlikely to beat the table

Generating the shuffle mask from rank would require constructing a vector that:

* Selects byte at position zero.
* Shifts positions 1 through .
* Leaves positions above unchanged.
* Leaves the upper 128-bit lane unchanged for .

That involves comparisons, vector constants, arithmetic or blends.

A precomputed mask already represents the complete permutation in 32 bytes.

Given that the common-rank table is only 480 bytes, arithmetic generation would have to save enough load latency to compensate for its additional instructions and register pressure.

There is no mathematical reason to expect that trade to win.

My disposition: retain the precomputed masks as the reference design. Arithmetic generation has no demonstrated advantage.

Round 33 results

P74 subquestion	Conclusion
Rank-zero explicit-token shortcut	Not applicable to bzip2
Required common-rank mask storage	480 bytes
Does mask addressing depend on current MTF state?	No
Is a separate YMM mask register mandatory?	No
Can a mask load overlap the previous state update?	Yes, architecturally possible
Is mask generation better than loading?	No established advantage
Does the complete fast path beat shipped MTF?	Still requires compiled-code and CPU evidence

The most useful next correction to the cost model

The existing P74 cost model should distinguish two recurrences:

\text{MTF state}\rightarrow\text{MTF state}

and

\text{token rank}\rightarrow\text{mask address}\rightarrow\text{shuffle}.

The effective state-update latency depends on when both inputs become available. It is not necessarily the sum of an L1 mask-load latency and a shuffle latency at every token.

We have therefore narrowed the mask question to scheduling and code generation, not table size, computational complexity or an unresolved dependency inherent in MTF.

For Claude’s report, I would preserve the explicit-rank-zero correction alongside the memory-operand shuffle option.

Round 33 conclusion: the mask table is small, independent of MTF state, and compatible with a register-resident update. The remaining question is whether the compiled checkpoint-aware loop delivers that dependency structure on the target CPUs. We have reached the point where more mask algebra is unlikely to provide a meaningful answer without actual machine code.
````

## The progress report, as received

````text
mibz2 research burn-down — progress report

30 October 2026 · PR #117

We’re approaching completion of the question-by-question research review. The strongest change since the previous report is that the speculative ideas have been reduced to correctness results, measured negative findings and narrowly defined performance uncertainties.

I checked the latest accessible PR #117 portfolio. Its reconciled ledger reports the following for the 47 originally open questions:

Burn-down of the original open inventory

47 questions

11

Answered

19

Narrowed to specific evidence

17

Handled in a separate disposition table or awaiting the literature agent

The last category is not 17 untouched questions. Ten already have individual dispositions, while seven literature items (#28–#33 and #36) remain pending reconciliation. The six originally conditional questions are tracked separately.

1. What’s been accomplished

N1 mathematics and construction

Exact record partitions, gap identities, interval doubling, correct source/image ordering, bitvector child counting and segment-cap correctness are now established and checked. The two-pass counting requirement was eliminated.

233-block structural census

Finished. Every Wikidata block fits Ψ⁸ under the resident-size rule, while random, records and source-code controls demonstrate why per-block fallback is necessary.

#63–#66 and #74

Real corpus measurements establish breakpoint clustering, symbol differences, absence of mod-32 alignment, and no material exceptional-lookup bunching across the shipped 32-lane scheduler.

MTF research

N2 and N2b were stopped by their registered evidence thresholds. P74 survived, and its 32-byte register-held MTF state machine has passed low-rank, high-rank, adversarial and real-block correctness tests.

2. The two survivors

N1 — multi-step Ψ

CPU economics unresolved

BEST MODELED BUILDER

17.5 B/B

Against 5 B/B for the T-vector build

MEDIAN PEAK LIVE MEMORY

4.23 MB

Mixed-layout bitvector builder

The final table is compact, but construction remains relatively heavy. Under the stricter 4.5 MB peak-memory limit, the modeled per-block selector distributes 233 Wikidata blocks across Ψ⁸, Ψ⁴, Ψ² and T-vector fallback rather than choosing Ψ⁸ universally.

P74 — register-held MTF

Correctness established

ONE-SHUFFLE PATH

84.8%

Explicit Wikidata tokens, ranks 1–15

HIGH-RANK FALLBACK

1.6%

Ranks 32 and above

A single AVX2 shuffle can update the register-held MTF head for the common ranks. The complete state machine is correct, including bzip2 symbol mapping and checkpoint compatibility. Remaining questions concern generated machine code, latency and avoiding #25’s random-data regression.

3. What remains to finish the burn-down

The original inventory contains 84 questions: 31 answered, 47 originally open, and six conditional. The current research ledger has now classified all but the seven literature items awaiting independent review.

The main remaining evidence gaps are:

Area	Remaining question	Nature of uncertainty
Literature #28–#33, #36	Do the primary sources contain transferable decoder methods?	Source verification
#73	What do the primary PDEP instruction tables establish?	Instruction evidence
P74	Does the complete compiled loop preserve its register-only state recurrence?	Disassembly and CPU cost
N1 / #53	Is 17.5 B/B of structured construction traffic competitive in cycles?	CPU measurement
N1 / #54, #76	Does building and walking the new table improve full-decoder throughput?	CPU measurement
#12	Would a parallel scanner become useful after worker acceleration?	Conditional on pipeline saturation
#19, #20, #43, #48	Do older compact/move structures win under their specified concurrency and hardware?	Machine-specific tests

The important distinction is that an unanswered performance experiment is not the same as an unresolved algorithm. Much of the mathematical work is now closed; the remaining uncertainties are increasingly empirical.

4. Progress against our objectives

Question inventory coverage

Nearly complete

All but seven of the 47 originally open questions have individual dispositions, including evidence requirements where unresolved.

Algorithmic correctness

Strong progress

N1 composition and capped segment handling are validated; P74’s state transitions now pass real-block checks.

CPU performance breakthrough

Not yet demonstrated

No new design has yet beaten the shipped full decoder in a matched performance experiment.

5. Where I recommend we go next

Claude is currently checking shipped code generation and the measurement behind the MTF store-forwarding observation. That’s the right next evidence for P74. We should finish that analysis rather than restart its mathematics.

In parallel, the literature agent should close its outstanding sources and PDEP review. Once those arrive, we can finish the inventory reconciliation while preserving conditional questions that genuinely require performance measurements.

I would then concentrate on the two research decisions that can still materially change mibz2: the N1 builder’s CPU economics and P74’s complete checkpoint-aware MTF dependency chain.

Overall verdict: We’re close to finishing the analytical burn-down, with no need to invent a new backlog. The remaining work is less about discovering whether a mechanism exists and more about establishing whether it actually reduces CPU cost.

Research portfolio — PR #117⁠￼
````
