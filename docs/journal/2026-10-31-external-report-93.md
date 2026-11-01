# External research report 93: the 32 lanes don't collapse into 8 chains; constant-index expansion of the round (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-10-31 16:32 UTC.
- **Source:** Astra.
- **In reply to:** the walker evidence pack.

It is kept exactly as received (rendered text, cards flattened). Paths to uploaded
files are the maintainer's.

## The message, as received

````text
Astra says

The evidence does not show 32 lanes collapsing into eight serialized chains. It shows independent chains advanced through a relatively expensive instruction stream, with additional limits that the available timings cannot uniquely identify.

My one candidate is constant-index expansion of the complete 32-lane round, retaining the existing memory-backed state and immediate segment-boundary handling. Confidence is high that this can remove bookkeeping; confidence in the resulting speedup is lower. A conservative conditional estimate is about 4.0–4.3 c/B, versus the historical 4.49 c/B. The evidence does not justify promising the requested 1.15 c/B saving.

I read the README, both disassembly listings, source extract and recorded measurement/model notes. The listings contain identical instructions. Their binary is from ae942bd; source identity with the historical measured revision is established, but identity with that CI binary is not.

What the assembly establishes

The central sequence is:

2a534: mov    ebx,DWORD PTR [r11]           ; lane row
2a537: mov    eax,DWORD PTR [r15+rbx4]     ; q = T[row]
2a53b: test   eax,eax
2a53d: js     2a570                        ; segment boundary
2a53f: mov    edi,eax
2a541: shr    edi,0x8
2a544: mov    DWORD PTR [r11],edi          ; save next row
2a547: prefetcht0 BYTE PTR [r15+rdi4]     ; request next entry
2a54c: mov    rdi,QWORD PTR [rsp+r8*8+0x70] ; output base
2a551: mov    BYTE PTR [rdi+rcx],al

This supports four specific deductions:

- Each lane has a genuine dependent chain: row-state load → T load → shift → row-state store → next round’s row-state load. The stack adds a dependent load and store traffic, although these locations should usually be cache-resident. Next round’s load might forward from the store buffer or read L1; the listing does not establish which.
- Successive lanes are independent on the ordinary path. Their addresses advance through r11 and the r8/r10 induction variables independently of q. Register renaming removes the apparent dependencies caused by reusing eax, edi and rdi. In particular, loading the output base into rdi need not wait for the preceding prefetch to complete.
- The output-base load is real per-byte work. It supplies the output-store address. Conversely, seg, p and end are not all repeatedly maintained in this hot step: the shared-offset design has already moved much of that work to pass boundaries.
- There is no visible serialization instruction or hot-path call. Memory-disambiguation stalls or address-alias replays remain possible, but nothing in the evidence demonstrates them.

One detail matters when reading the listing: the compiled Lane.row is at offset +0x18, not the README’s illustrative +0. The first row is at [rsp+0x248], while its lane begins at [rsp+0x230].

The hot loop contains 16 instructions, or 14 fused-domain operations assuming the two compare/test-and-branch fusions. Including the executed outer-loop instructions and alignment NOP gives 16.46875 instructions and 14.4375 fused operations per full-phase byte. The pack’s approximately 14.8 operations/B additionally estimates pass, boundary and tail overhead.

The lane-loop bookkeeping itself is six instructions, five fused operations:

2a530: lea r10,[r8+1]
2a554: lea rax,[r10-1]
2a558: add r11,0x20
2a55c: mov r8,r10
2a55f: cmp rax,0x1f
2a563: jb  2a530

Those are directly removable without changing the chain algorithm.

Why the result is 8.6, rather than 32

The measured ratio is:

\[
38.55/4.49=8.586.  
\]

It compares the throughput of two different execution streams. The single-chain cost includes its dependent execution and cache behavior; it is not the measured latency distribution of requests in the prefetched walker. Therefore the ratio cannot identify actual outstanding requests.

At 4.49 c/B, revisiting a lane takes approximately 144 cycles. That is ample average lead relative to the pack’s roughly 39-cycle serial-chain cost and 46–50-cycle vendor L3 figures. It does not guarantee that prefetches arrive in time: issue delays, translation, eviction, contention and longer-latency misses can consume that lead.

Crucially, a prefetch request can remain outstanding after its instruction retires. All 32 lanes’ instructions need not coexist in the reorder buffer. The pack’s suggestion that every group must remain simultaneously inside the instruction window is too strong for this software-prefetched loop.

Using the pack’s operation counts and hardware capacities gives these approximate limits:

Core	Dispatch floor	Memory-operation floor	ROB capacity expressed as current lane-steps

Zen 3	2.47 c/B	≈2.0 c/B	≈17
Zen 4	2.47 c/B	≈2.0 c/B	≈22
Comet Lake	3.70 c/B	≈2.0–2.5 c/B	≈15

These are resource models, not predicted running times. Stores split into backend work; prefetch consumes address-generation resources; Intel’s indexed output store also competes for a suitable AGU. Scheduler occupancy cannot be inferred simply by dividing its capacity by 14: issued operations leave scheduler queues while remaining unretired, and Zen uses separate queues. The pack also leaves Zen 3 load-queue capacity unresolved.

The strongest interpretation is consequently mixed instruction/window and memory-service pressure:

- The lane sweep flattens around 24 lanes: adding logical chains alone stops helping.
- Prefetch still saves approximately 13% in that sweep: memory latency remains relevant.
- The historical SMT result gives only 2/2.09 = 0.955× throughput per core: some shared or partitioned resource prevents useful sibling throughput.
- None of this identifies the resource specifically as MABs, the load queue, the scheduler or the ROB.

The clock-scaling observation points away from a simple fixed-nanosecond DRAM floor. It does not uniquely establish dispatch limitation; cache timing and inaccurate reported clocks also matter.

The B2 note explicitly records the central uncertainty: its documented-parameter simulation predicts 2.47 c/B, while its additive interval model fits the observations. The fit is useful, but does not prove that removing instructions will reduce the fitted miss penalty proportionally.

The single transformation I would nominate

Expand k = 0..32 into 32 constant-index scalar steps inside the existing off loop. Keep rows and output bases memory-backed, retain one-hop prefetching, and share the cold START handling.

Four groups of eight are acceptable as source organization, provided every off processes all four groups before advancing any lane again. There must be no inner loop that runs several traversal depths of one group before visiting the others.

This preserves the existing scheduling opportunity across 32 chains. It does not promise 32 simultaneous requests or register residency for 32 rows and 32 pointers. Trying to retain that state in GPRs would introduce a different register-pressure problem.

The expected ordinary-step signature is:

mov    ebx,[rsp+ROW_k]          ; constant displacement
mov    eax,[r15+rbx4]
test   eax,eax
js     cold_k
mov    edi,eax
shr    edi,8
mov    [rsp+ROW_k],edi
prefetcht0 [r15+rdi4]
mov    rdi,[rsp+BASE_k]         ; constant displacement
mov    [rdi+rcx],al

Register names and stack offsets may differ, but the structural requirement is exact:

Ordinary lane-step	Existing	Expected

Instructions	16	10
Fused-domain operations	14	9
Loads	3	3
Stores	2	2
Prefetches	1	1
Dynamic lane-loop control	Per lane	Absent

Only shared outer-round control remains. No additional hot calls, spills, per-lane increments or status tests should replace the removed work.

This reduces instruction-window occupancy and induction/address bookkeeping. It does not eliminate either the row round-trip or the output-base reload. That restraint makes its mechanism more defensible than assuming scalar grouping magically makes all lane state register-resident.

Conservative estimate and failure modes

Using the historical baseline only:

\[
4.49 = 2.47 + 2.02  
\]

is a model decomposition, not a measured stall breakdown. After expansion, dispatch demand falls below the approximately 2.0 c/B memory-operation floor. Holding the unexplained residual unchanged yields approximately:

\[
2.01+2.02=4.03\ \mathrm{c/B}.  
\]

Allowing incomplete realization and unchanged setup/tail costs, 4.0–4.3 c/B is a conditional planning estimate: roughly 0.2–0.5 c/B saved. No gain, or a regression, remains credible.

The proposed change could fail because:

- Removed instructions currently overlap otherwise unavoidable stalls.
- Miss handling or translation already determines throughput.
- Faster rounds shorten effective prefetch lead.
- Expanded code and 32 branch sites hurt instruction delivery or prediction.
- LLVM duplicates cold code, adds spills or reconstructs dynamic bookkeeping.

The cold handler must preserve the existing distinction between lanes already processed at off and those not yet processed when the final segment ends. A future implementation would need differential and Miri coverage of that boundary, replacement, buffer growth and fewer-than-32-lane inputs. No transpose, deferred commit or replay is part of this proposal.

The B2 estimate of 3.30–3.42 c/B additionally assumes reduced memory traffic and a miss-term reduction from 2.02 to 1.46 c/B. The present assembly does not establish that second saving. I would not use it to promise the conditional 1.15 c/B target or any decoder-level gain.

Decisive future falsification

After separate authorization:

1. Audit the same-binary candidate against the exact signature above. If bookkeeping has merely become spills, calls or extra tests, the proposed transformation was not realized.
2. Compare shipped versus candidate in a paired, single-physical-core stage experiment on an EPYC 7763, with identical decode-order inputs and complete room-walk costs, including setup and tail. Preserve output checks and report ns/B alongside cycles.
3. Pre-register a 5% minimum useful saving: candidate/shipped upper 95% confidence endpoint below 0.95. On the historical scale, that is below 4.266 c/B. An interval entirely at or above 0.95 falsifies that useful-saving prediction; one crossing it is inconclusive.

If instructions fall as expected but time does not, that rejects this candidate’s practical benefit—not every possible scalar transformation.

The implementation appears near its useful lane-count limit. The evidence does not establish that it is near the best achievable scalar implementation limit. This one bookkeeping reduction is defensible to test; a 1.15 c/B improvement is not yet defensible to expect.

No repository files were changed, and no builds, benchmarks, CI actions or publication operations were performed.
````
